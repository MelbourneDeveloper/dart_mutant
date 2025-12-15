//! Command-line interface for dart_mutant

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum AiProvider {
    #[default]
    None,
    /// Use Anthropic Claude for smart mutation placement
    Anthropic,
    /// Use OpenAI GPT for smart mutation placement
    OpenAI,
    /// Use local Ollama model for smart mutation placement
    Ollama,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "dart_mutant",
    author = "Christian Findlay",
    version,
    about = "🧬 AST-powered mutation testing for Dart",
    long_about = r#"
dart_mutant is a blazingly fast mutation testing tool for Dart that uses
tree-sitter for precise, syntax-aware mutations.

Unlike text-based mutation tools, dart_mutant understands your code structure
and only creates valid, meaningful mutations that test your assertions.

FEATURES:
    • AST-based mutations (no invalid syntax)
    • Parallel test execution
    • Beautiful HTML reports (Stryker-compatible)
    • AI-powered smart mutation placement
    • Incremental testing with caching

EXAMPLES:
    # Run mutation testing on current directory
    dart_mutant

    # Run on specific directory with 8 parallel jobs
    dart_mutant --path ./lib --jobs 8

    # Use AI to find high-value mutation locations
    dart_mutant --ai anthropic

    # Set minimum mutation score threshold
    dart_mutant --threshold 80

    # Test only a sample of mutations for quick feedback
    dart_mutant --sample 50

    # Incremental mode - only test changed files
    dart_mutant --incremental --base-ref main
"#
)]
pub struct Args {
    /// Path to Dart project or file
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Glob pattern for files to mutate
    #[arg(short, long, default_value = "lib/**/*.dart")]
    pub glob: String,

    /// Glob patterns to exclude
    #[arg(short, long, default_values_t = vec![
        "**/*.g.dart".to_string(),
        "**/*.freezed.dart".to_string(),
        "**/*.mocks.dart".to_string(),
        "**/generated/**".to_string(),
        "**/test/**".to_string(),
        "**/*_test.dart".to_string(),
    ])]
    pub exclude: Vec<String>,

    /// Number of parallel mutation test jobs
    #[arg(short = 'j', long, default_value_t = num_cpus())]
    pub parallel: usize,

    /// Timeout per mutation test in seconds
    #[arg(short, long, default_value = "30")]
    pub timeout: u64,

    /// Minimum mutation score threshold (0-100)
    #[arg(long, default_value = "0")]
    pub threshold: f64,

    /// Output directory for reports
    #[arg(short, long, default_value = "./mutation-reports")]
    pub output: PathBuf,

    /// Only generate mutations without running tests (dry run)
    #[arg(long)]
    pub dry_run: bool,

    /// Quiet mode - minimal output
    #[arg(short, long)]
    pub quiet: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Test command to run (default: dart test)
    #[arg(long, default_value = "dart test")]
    pub test_command: String,

    /// Sample number of mutations to test (0 = all)
    #[arg(long)]
    pub sample: Option<usize>,

    /// Mutation operators to use (default: all)
    #[arg(long, value_delimiter = ',')]
    pub operators: Option<Vec<String>>,

    /// Only mutate lines covered by tests (requires coverage file)
    #[arg(long)]
    pub coverage_file: Option<PathBuf>,

    /// Generate incremental results (cache killed/survived status)
    #[arg(long)]
    pub incremental: bool,

    /// Path to incremental cache file
    #[arg(long, default_value = ".dart_mutant_cache")]
    pub cache_file: PathBuf,

    /// Git base ref for incremental mode
    #[arg(long, default_value = "main")]
    pub base_ref: String,

    // ===== AI-Powered Mutations =====

    /// Enable AI-powered smart mutation placement
    #[arg(long, value_enum, default_value = "none")]
    pub ai: AiProvider,

    /// API key for AI provider (or set ANTHROPIC_API_KEY / OPENAI_API_KEY env var)
    #[arg(long, env = "DART_MUTANT_AI_KEY")]
    pub ai_key: Option<String>,

    /// Ollama model name (for --ai ollama)
    #[arg(long, default_value = "codellama")]
    pub ollama_model: String,

    /// Ollama server URL
    #[arg(long, default_value = "http://localhost:11434")]
    pub ollama_url: String,

    /// Maximum number of AI-suggested mutations per file
    #[arg(long, default_value = "10")]
    pub ai_max_per_file: usize,

    // ===== Report Options =====

    /// Generate HTML report
    #[arg(long, default_value_t = true)]
    pub html: bool,

    /// Generate JSON report (Stryker-compatible format)
    #[arg(long)]
    pub json: bool,

    /// Generate JUnit XML report for CI
    #[arg(long)]
    pub junit: bool,

    /// Generate AI-optimized markdown report for LLM consumption
    #[arg(long)]
    pub ai_report: bool,

    /// Open HTML report in browser after completion
    #[arg(long)]
    pub open: bool,
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

impl Args {
    /// Check if AI mutation suggestions are enabled
    pub fn is_ai_enabled(&self) -> bool {
        !matches!(self.ai, AiProvider::None)
    }

    pub fn get_ai_api_key(&self) -> Option<String> {
        self.ai_key.clone().or_else(|| match self.ai {
            AiProvider::Anthropic => std::env::var("ANTHROPIC_API_KEY").ok(),
            AiProvider::OpenAI => std::env::var("OPENAI_API_KEY").ok(),
            AiProvider::Ollama | AiProvider::None => None,
        })
    }
}
