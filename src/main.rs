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
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // deploy-toolkit --version contract: must run BEFORE clap parsing so the
    // plain form emits exactly `<bin> <semver>\n` with no banner/logging.
    // See deployment-toolkit.json and `deployment_toolkit/schemas/version-manifest.schema.json`.
    if handle_version_contract() {
        return Ok(());
    }

    let args = Args::parse();

    init_logging(args.verbose, args.quiet);

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

/// Initialize `tracing` output, mapping CLI flags to a default log level.
///
/// `--verbose` enables `dart_mutant` debug/trace output; `--quiet` drops to
/// warnings only. An explicit `RUST_LOG` always overrides these defaults so
/// power users can tune per-module verbosity. Logs go to stderr, leaving stdout
/// for the report banner and progress bars.
fn init_logging(verbose: bool, quiet: bool) {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let directives = if verbose {
            "info,dart_mutant=trace"
        } else if quiet {
            "warn"
        } else {
            "info"
        };
        EnvFilter::new(directives)
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(verbose)
        .with_writer(std::io::stderr)
        .init();
}

/// deploy-toolkit `--version` contract.
///
/// Returns `true` when a version flag was handled; the caller should exit 0.
/// TODO: once `deploy-toolkit-cli` is published on crates.io, replace this
/// body with `deploy_toolkit_cli::dispatch(...)`.
fn handle_version_contract() -> bool {
    let mut has_version = false;
    let mut has_json = false;
    for a in std::env::args().skip(1) {
        match a.as_str() {
            "--version" | "-V" => has_version = true,
            "--json" => has_json = true,
            _ => {}
        }
    }
    if !has_version {
        return false;
    }
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    if has_json {
        let json = format!(
            "{{\"manifestVersion\":1,\"name\":\"{name}\",\"version\":\"{version}\",\"kind\":\"cli\",\"language\":\"rust\",\"product\":\"dart-mutant\"}}"
        );
        println!("{json}");
    } else {
        println!("{name} {version}");
    }
    true
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
    info!(
        path = %args.path.display(),
        parallel = args.parallel,
        timeout_secs = args.timeout,
        "starting mutation testing run"
    );

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
        match parser::parse_and_find_mutations(file) {
            Ok(mutations) => all_mutations.extend(mutations),
            Err(error) => {
                warn!(file = %file.display(), %error, "failed to parse file, skipping");
            }
        }
        parse_pb.inc(1);
    }
    info!(
        files = dart_files.len(),
        mutations = all_mutations.len(),
        "parsing complete"
    );
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
                ai_pb.finish_with_message(format!("{} AI suggestions failed: {e}", "✗".red()));
            }
        }
    }

    if all_mutations.is_empty() {
        warn!("no mutations generated across all discovered files");
        println!(
            "\n{}",
            "No mutations generated. Your code might be too simple or already well-tested!"
                .yellow()
        );
        return Ok(MutationResult::default());
    }

    // Apply sampling if requested
    let mutations_to_test = if let Some(sample_size) = args.sample {
        let sampled = mutation::sample_mutations(&all_mutations, sample_size);
        info!(
            requested = sample_size,
            sampled = sampled.len(),
            total = all_mutations.len(),
            "sampled mutations"
        );
        sampled
    } else {
        all_mutations.clone()
    };
    info!(
        count = mutations_to_test.len(),
        "mutations selected for testing"
    );

    // Step 3: Run mutation tests (or skip in dry-run mode)
    let results = if args.dry_run {
        println!("\n{} Dry run mode - skipping test execution", "ℹ".cyan());
        println!("  {} mutations would be tested\n", mutations_to_test.len());

        // Print first few mutations as preview
        for (i, m) in mutations_to_test.iter().take(10).enumerate() {
            println!(
                "  {}. [{}:{}] {} → {}",
                i + 1,
                m.location
                    .file
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
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
        // Pre-flight: confirm the suite is green before mutating anything.
        // A red baseline would make every mutant look "killed" (a false 100%).
        verify_baseline_or_bail(&args.path, args.timeout, &multi_progress).await?;

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

/// Verify a green baseline before mutating, aborting the run if it is red.
///
/// Implements [RUNNER-BASELINE]: a failing unmutated suite makes every mutant
/// appear killed, so we refuse to produce a (bogus) mutation score.
async fn verify_baseline_or_bail(
    path: &std::path::Path,
    timeout: u64,
    mp: &MultiProgress,
) -> Result<()> {
    let pb = create_spinner(mp, "Verifying baseline (running unmutated tests)...");
    let status = runner::verify_baseline(path, timeout).await?;
    match status {
        runner::BaselineStatus::Passing => {
            pb.finish_with_message(format!("{} Baseline passed", "✓".green()));
            Ok(())
        }
        runner::BaselineStatus::Failing { details } => {
            pb.finish_with_message(format!("{} Baseline failed", "✗".red()));
            warn!("aborting: baseline test suite is red");
            anyhow::bail!(
                "Baseline test suite failed on unmutated code. Mutation results would be \
                 meaningless (every mutant would look killed). Fix the failing tests first.\n\n{}",
                details.trim()
            )
        }
    }
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

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    if score >= 80.0 {
        bar.green().to_string()
    } else if score >= 60.0 {
        bar.yellow().to_string()
    } else {
        bar.red().to_string()
    }
}
