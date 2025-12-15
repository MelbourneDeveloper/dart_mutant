//! dart_mutant - A blazingly fast mutation testing tool for Dart
//!
//! Uses tree-sitter for AST-based mutations, ensuring precise and valid code modifications.

mod ai;
mod cli;
mod mutation;
mod parser;
mod report;
mod runner;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use report::MutationResult;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let args = Args::parse();

    print_banner();

    let start = Instant::now();

    // Run the mutation testing pipeline
    let result = run_mutation_testing(&args).await?;

    let duration = start.elapsed();
    print_summary(&result, duration);

    // Exit with appropriate code
    if result.mutation_score >= args.threshold {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn print_banner() {
    const BANNER: &str = r"
    DART MUTANT - Mutation Testing for Dart
    ========================================
";
    println!("{}", BANNER.bright_cyan());
    println!(
        "    {} {}\n",
        "Mutation Testing for Dart".bright_white(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
}

async fn run_mutation_testing(args: &Args) -> Result<MutationResult> {
    let multi_progress = MultiProgress::new();

    // Step 1: Discover Dart files
    let discover_pb = create_spinner(&multi_progress, "Discovering Dart files...");
    let dart_files = parser::discover_dart_files(&args.path, &args.exclude)?;
    discover_pb.finish_with_message(format!(
        "{} Found {} Dart files",
        "✓".green(),
        dart_files.len().to_string().cyan()
    ));

    if dart_files.is_empty() {
        anyhow::bail!("No Dart files found in {}", args.path.display());
    }

    // Step 2: Parse files and generate mutations
    let parse_pb = create_progress_bar(&multi_progress, dart_files.len() as u64, "Parsing files");
    let mut all_mutations = Vec::new();

    for file in &dart_files {
        let mutations = parser::parse_and_find_mutations(file)?;
        all_mutations.extend(mutations);
        parse_pb.inc(1);
    }
    parse_pb.finish_with_message(format!(
        "{} Generated {} mutations",
        "✓".green(),
        all_mutations.len().to_string().cyan()
    ));

    // Add AI-suggested mutations if enabled
    if args.is_ai_enabled() {
        let ai_pb = create_spinner(&multi_progress, "Getting AI mutation suggestions...");
        let ai_result = ai::suggest_mutations_for_files(
            &dart_files,
            args.ai,
            args.get_ai_api_key(),
            &args.ollama_url,
            &args.ollama_model,
            args.ai_max_per_file,
        )
        .await;
        match ai_result {
            Ok(ai_mutations) => {
                ai_pb.finish_with_message(format!(
                    "{} AI suggested {} additional mutations",
                    "✓".green(),
                    ai_mutations.len()
                ));
                all_mutations.extend(ai_mutations);
            }
            Err(e) => {
                ai_pb.finish_with_message(format!(
                    "{} AI suggestions failed: {e}",
                    "✗".red()
                ));
            }
        }
    }

    if all_mutations.is_empty() {
        println!(
            "\n{}",
            "No mutations generated. Your code might be too simple or already well-tested!"
                .yellow()
        );
        return Ok(MutationResult::default());
    }

    // Apply sampling if requested
    let mutations_to_test = if let Some(sample_size) = args.sample {
        mutation::sample_mutations(&all_mutations, sample_size)
    } else {
        all_mutations.clone()
    };

    // Step 3: Run mutation tests (or skip in dry-run mode)
    let results = if args.dry_run {
        println!(
            "\n{} Dry run mode - skipping test execution",
            "ℹ".cyan()
        );
        println!("  {} mutations would be tested\n", mutations_to_test.len());

        // Print first few mutations as preview
        for (i, m) in mutations_to_test.iter().take(10).enumerate() {
            println!(
                "  {}. [{}:{}] {} → {}",
                i + 1,
                m.location.file.file_name().unwrap_or_default().to_string_lossy(),
                m.location.start_line,
                m.original,
                m.mutated
            );
        }
        if mutations_to_test.len() > 10 {
            println!("  ... and {} more", mutations_to_test.len() - 10);
        }

        // Return empty results for dry run
        vec![]
    } else {
        let test_pb = create_progress_bar(
            &multi_progress,
            mutations_to_test.len() as u64,
            "Testing mutations",
        );

        let results = runner::run_mutation_tests(
            &args.path,
            &mutations_to_test,
            args.parallel,
            args.timeout,
            test_pb.clone(),
        )
        .await?;

        test_pb.finish_with_message(format!(
            "{} Tested {} mutations",
            "✓".green(),
            mutations_to_test.len().to_string().cyan()
        ));

        results
    };

    // Step 4: Generate reports
    let report_pb = create_spinner(&multi_progress, "Generating reports...");

    let mutation_result = MutationResult::from_results(&results);

    if args.html {
        let html_path = args.output.join("mutation-report.html");
        report::generate_html_report(&mutation_result, &results, &dart_files, &html_path)?;
        report_pb.set_message(format!(
            "{} HTML report: {}",
            "✓".green(),
            html_path.display().to_string().cyan()
        ));
    }

    if args.json {
        let json_path = args.output.join("mutation-report.json");
        report::generate_json_report(&mutation_result, &results, &json_path)?;
    }

    if args.ai_report {
        let ai_path = args.output.join("mutation-report-ai.md");
        report::generate_ai_report(&mutation_result, &results, &ai_path)?;
    }

    report_pb.finish_with_message(format!("{} Reports generated", "✓".green()));

    Ok(mutation_result)
}

fn create_spinner(mp: &MultiProgress, message: &str) -> ProgressBar {
    let pb = mp.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

fn create_progress_bar(mp: &MultiProgress, len: u64, message: &str) -> ProgressBar {
    let pb = mp.add(ProgressBar::new(len));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} {msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█▓▒░  "),
    );
    pb.set_message(message.to_string());
    pb
}

fn print_summary(result: &MutationResult, duration: std::time::Duration) {
    println!("\n{}", "═".repeat(70).bright_cyan());
    println!(
        "{}",
        "                        MUTATION TESTING RESULTS                        "
            .bright_white()
            .bold()
    );
    println!("{}\n", "═".repeat(70).bright_cyan());

    // Score display with color based on threshold
    let score_color = if result.mutation_score >= 80.0 {
        "green"
    } else if result.mutation_score >= 60.0 {
        "yellow"
    } else {
        "red"
    };

    let score_bar = create_score_bar(result.mutation_score);
    println!("  Mutation Score: {}", score_bar);
    println!(
        "  {:.1}%\n",
        match score_color {
            "green" => format!("{:.1}%", result.mutation_score).green(),
            "yellow" => format!("{:.1}%", result.mutation_score).yellow(),
            _ => format!("{:.1}%", result.mutation_score).red(),
        }
    );

    println!("  {} Killed:      {}", "●".green(), result.killed);
    println!("  {} Survived:    {}", "●".red(), result.survived);
    println!("  {} Timeout:     {}", "●".yellow(), result.timeout);
    println!("  {} No Coverage: {}", "●".dimmed(), result.no_coverage);
    println!("  {} Errors:      {}\n", "●".magenta(), result.errors);

    println!(
        "  Total Mutants: {}",
        result.total.to_string().bright_white()
    );
    println!(
        "  Time Elapsed:  {}\n",
        format!("{:.2}s", duration.as_secs_f64()).bright_white()
    );

    println!("{}", "═".repeat(70).bright_cyan());
}

fn create_score_bar(score: f64) -> String {
    let width = 40;
    let filled = ((score / 100.0) * width as f64) as usize;
    let empty = width - filled;

    let bar = format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(empty)
    );

    if score >= 80.0 {
        bar.green().to_string()
    } else if score >= 60.0 {
        bar.yellow().to_string()
    } else {
        bar.red().to_string()
    }
}
