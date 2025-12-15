//! Beautiful HTML and JSON report generation
//!
//! Generates Stryker-compatible reports with stunning visuals.
//! Uses the Toxic Lab theme from the dart_mutant website.

mod css;

use crate::mutation::MutantStatus;
use crate::runner::MutantTestResult;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

/// Helper trait for MutantStatus display
pub trait MutantStatusDisplay {
    /// Get CSS class for this status
    fn css_class(&self) -> &'static str;
    /// Get emoji for this status
    fn emoji(&self) -> &'static str;
}

impl MutantStatusDisplay for MutantStatus {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Killed => "killed",
            Self::Survived => "survived",
            Self::Timeout => "timeout",
            Self::NoCoverage => "no-coverage",
            Self::Error | Self::Pending => "error",
        }
    }

    fn emoji(&self) -> &'static str {
        match self {
            Self::Killed => "✅",
            Self::Survived => "🔴",
            Self::Timeout => "⏱️",
            Self::NoCoverage => "🚫",
            Self::Error | Self::Pending => "⚠️",
        }
    }
}

/// Overall mutation testing results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub total: usize,
    pub killed: usize,
    pub survived: usize,
    pub timeout: usize,
    pub no_coverage: usize,
    pub errors: usize,
    pub mutation_score: f64,
}

impl Default for MutationResult {
    fn default() -> Self {
        Self {
            total: 0,
            killed: 0,
            survived: 0,
            timeout: 0,
            no_coverage: 0,
            errors: 0,
            mutation_score: 0.0,
        }
    }
}

impl MutationResult {
    pub fn from_results(results: &[MutantTestResult]) -> Self {
        let mut r = Self::default();
        r.total = results.len();

        for result in results {
            match result.status {
                MutantStatus::Killed => r.killed += 1,
                MutantStatus::Survived => r.survived += 1,
                MutantStatus::Timeout => r.timeout += 1,
                MutantStatus::NoCoverage => r.no_coverage += 1,
                MutantStatus::Error | MutantStatus::Pending => r.errors += 1,
            }
        }

        let detected = r.killed + r.timeout;
        let valid = r.total - r.errors - r.no_coverage;
        r.mutation_score = if valid > 0 {
            (detected as f64 / valid as f64) * 100.0
        } else {
            0.0
        };

        r
    }
}

/// Generate a beautiful HTML report
pub fn generate_html_report(
    result: &MutationResult,
    test_results: &[MutantTestResult],
    dart_files: &[std::path::PathBuf],
    output_path: &Path,
) -> Result<()> {
    // Group results by file
    let mut by_file: HashMap<String, Vec<&MutantTestResult>> = HashMap::new();
    for r in test_results {
        let file = r.mutation.location.file.display().to_string();
        by_file.entry(file).or_default().push(r);
    }

    // Calculate per-file stats
    let mut file_stats: Vec<FileStats> = by_file
        .iter()
        .map(|(file, results)| {
            let total = results.len();
            let killed = results
                .iter()
                .filter(|r| matches!(r.status, MutantStatus::Killed | MutantStatus::Timeout))
                .count();
            let score = if total > 0 {
                (killed as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            FileStats {
                file: file.clone(),
                total,
                killed,
                score,
                mutants: results.iter().map(|r| (*r).clone()).collect(),
            }
        })
        .collect();

    file_stats.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let html = generate_html_content(result, &file_stats, dart_files.len());

    std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
    std::fs::write(output_path, html).context("Failed to write HTML report")?;

    Ok(())
}

#[derive(Debug, Clone)]
struct FileStats {
    file: String,
    total: usize,
    killed: usize,
    score: f64,
    mutants: Vec<MutantTestResult>,
}

fn generate_html_content(
    result: &MutationResult,
    file_stats: &[FileStats],
    total_files: usize,
) -> String {
    let score_class = if result.mutation_score >= 80.0 {
        "high"
    } else if result.mutation_score >= 60.0 {
        "medium"
    } else {
        "low"
    };

    let files_html: String = file_stats
        .iter()
        .map(|f| generate_file_section(f))
        .collect();

    let report_css = css::get_report_css();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🧬 Dart Mutant - Mutation Testing Report</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&family=Orbitron:wght@700&display=swap" rel="stylesheet">
    <style>
{report_css}
    </style>
</head>
<body>
    <div class="container">
        <header class="header">
            <div class="logo">
                <span class="logo-icon">🧬</span>
                <span>Dart Mutant</span>
            </div>
            <p class="tagline">AST-Powered Mutation Testing for Dart</p>
        </header>

        <div class="score-card">
            <div class="score-display">
                <div class="score-label">Mutation Score</div>
                <div class="score-value {score_class}">{score:.0}%</div>
                <div class="score-bar">
                    <div class="score-bar-fill {score_class}" style="width: {score}%"></div>
                </div>
            </div>

            <div class="stats-grid">
                <div class="stat-card stat-total">
                    <div class="stat-value">{total}</div>
                    <div class="stat-label">Total Mutants</div>
                </div>
                <div class="stat-card stat-killed">
                    <div class="stat-value">{killed}</div>
                    <div class="stat-label">Killed ✅</div>
                </div>
                <div class="stat-card stat-survived">
                    <div class="stat-value">{survived}</div>
                    <div class="stat-label">Survived 🔴</div>
                </div>
                <div class="stat-card stat-timeout">
                    <div class="stat-value">{timeout}</div>
                    <div class="stat-label">Timeout ⏱️</div>
                </div>
                <div class="stat-card stat-no-coverage">
                    <div class="stat-value">{no_coverage}</div>
                    <div class="stat-label">No Coverage 🚫</div>
                </div>
                <div class="stat-card stat-error">
                    <div class="stat-value">{errors}</div>
                    <div class="stat-label">Errors ⚠️</div>
                </div>
            </div>
        </div>

        <section>
            <h2 class="section-title">Files ({total_files} files, {file_count} with mutations)</h2>
            <div class="filter-controls">
                <span class="filter-label">Filter:</span>
                <label class="filter-checkbox">
                    <input type="checkbox" id="hideKilled">
                    <span>Hide killed mutants (show survivors only)</span>
                </label>
            </div>
            {files_html}
        </section>

        <footer class="footer">
            Generated by <a href="https://github.com/user/dart_mutant">dart_mutant</a> •
            Mutation testing helps you write better tests by finding gaps in your test coverage
        </footer>
    </div>

    <script>
        document.querySelectorAll('.file-header').forEach(header => {{
            header.addEventListener('click', () => {{
                header.parentElement.classList.toggle('expanded');
            }});
        }});

        // Filter toggle for hiding killed mutants
        const hideKilledCheckbox = document.getElementById('hideKilled');
        hideKilledCheckbox.addEventListener('change', () => {{
            const hideKilled = hideKilledCheckbox.checked;

            // Toggle visibility of killed/timeout mutants
            document.querySelectorAll('.mutant-item').forEach(item => {{
                const isKilled = item.classList.contains('killed') || item.classList.contains('timeout');
                if (hideKilled && isKilled) {{
                    item.classList.add('hidden');
                }} else {{
                    item.classList.remove('hidden');
                }}
            }});

            // Hide file cards that have no visible mutants
            document.querySelectorAll('.file-card').forEach(card => {{
                const visibleMutants = card.querySelectorAll('.mutant-item:not(.hidden)');
                if (visibleMutants.length === 0) {{
                    card.classList.add('all-hidden');
                }} else {{
                    card.classList.remove('all-hidden');
                }}
            }});
        }});
    </script>
</body>
</html>"#,
        report_css = report_css,
        score = result.mutation_score,
        score_class = score_class,
        total = result.total,
        killed = result.killed,
        survived = result.survived,
        timeout = result.timeout,
        no_coverage = result.no_coverage,
        errors = result.errors,
        total_files = total_files,
        file_count = file_stats.len(),
        files_html = files_html,
    )
}

fn generate_file_section(file_stats: &FileStats) -> String {
    let score_class = if file_stats.score >= 80.0 {
        "high"
    } else if file_stats.score >= 60.0 {
        "medium"
    } else {
        "low"
    };

    let mutants_html: String = file_stats
        .mutants
        .iter()
        .map(|m| {
            let status_class = MutantStatusDisplay::css_class(&m.status);
            let status_emoji = MutantStatusDisplay::emoji(&m.status);
            format!(
                r#"<div class="mutant-item {status_class}">
                    <div class="mutant-status">{status_emoji}</div>
                    <div class="mutant-details">
                        <div class="mutant-location">Line {line}:{col}</div>
                        <div class="mutant-description">{description}</div>
                        <div class="mutant-code">
                            <span class="code-original">{original}</span>
                            →
                            <span class="code-replacement">{replacement}</span>
                        </div>
                    </div>
                </div>"#,
                status_class = status_class,
                status_emoji = status_emoji,
                line = m.mutation.location.start_line,
                col = m.mutation.location.start_col,
                description = html_escape(&m.mutation.description),
                original = html_escape(&m.mutation.original),
                replacement = html_escape(&m.mutation.mutated),
            )
        })
        .collect();

    format!(
        r#"<div class="file-card">
            <div class="file-header">
                <span class="file-name">{file}</span>
                <div class="file-stats">
                    <span class="file-mutants">{killed}/{total} killed</span>
                    <span class="file-score {score_class}">{score:.0}%</span>
                </div>
            </div>
            <div class="file-content">
                {mutants_html}
            </div>
        </div>"#,
        file = html_escape(&file_stats.file),
        killed = file_stats.killed,
        total = file_stats.total,
        score = file_stats.score,
        score_class = score_class,
        mutants_html = mutants_html,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Generate a JSON report (Stryker-compatible format)
pub fn generate_json_report(
    result: &MutationResult,
    test_results: &[MutantTestResult],
    output_path: &Path,
) -> Result<()> {
    let report = JsonReport {
        schema_version: "1".to_string(),
        thresholds: Thresholds { high: 80, low: 60 },
        files: generate_json_files(test_results),
        project_root: std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        mutation_score: result.mutation_score,
    };

    let json = serde_json::to_string_pretty(&report)?;
    std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
    std::fs::write(output_path, json).context("Failed to write JSON report")?;

    Ok(())
}

#[derive(Serialize)]
struct JsonReport {
    #[serde(rename = "schemaVersion")]
    schema_version: String,
    thresholds: Thresholds,
    files: HashMap<String, JsonFile>,
    #[serde(rename = "projectRoot")]
    project_root: String,
    #[serde(rename = "mutationScore")]
    mutation_score: f64,
}

#[derive(Serialize)]
struct Thresholds {
    high: u32,
    low: u32,
}

#[derive(Serialize)]
struct JsonFile {
    language: String,
    mutants: Vec<JsonMutant>,
}

#[derive(Serialize)]
struct JsonMutant {
    id: String,
    #[serde(rename = "mutatorName")]
    mutator_name: String,
    replacement: String,
    status: String,
    location: JsonLocation,
    description: String,
}

#[derive(Serialize)]
struct JsonLocation {
    start: JsonPosition,
    end: JsonPosition,
}

#[derive(Serialize)]
struct JsonPosition {
    line: usize,
    column: usize,
}

fn generate_json_files(results: &[MutantTestResult]) -> HashMap<String, JsonFile> {
    let mut files: HashMap<String, JsonFile> = HashMap::new();

    for result in results {
        let file = result.mutation.location.file.display().to_string();

        let mutant = JsonMutant {
            id: result.mutation.id.clone(),
            mutator_name: result.mutation.operator.name().to_string(),
            replacement: result.mutation.mutated.clone(),
            status: match result.status {
                MutantStatus::Killed => "Killed",
                MutantStatus::Survived => "Survived",
                MutantStatus::Timeout => "Timeout",
                MutantStatus::NoCoverage => "NoCoverage",
                MutantStatus::Error | MutantStatus::Pending => "CompileError",
            }
            .to_string(),
            location: JsonLocation {
                start: JsonPosition {
                    line: result.mutation.location.start_line,
                    column: result.mutation.location.start_col,
                },
                end: JsonPosition {
                    line: result.mutation.location.end_line,
                    column: result.mutation.location.end_col,
                },
            },
            description: result.mutation.description.clone(),
        };

        files
            .entry(file)
            .or_insert_with(|| JsonFile {
                language: "dart".to_string(),
                mutants: vec![],
            })
            .mutants
            .push(mutant);
    }

    files
}

/// Generate an AI-friendly markdown report optimized for LLM consumption
///
/// This report is structured to help AI assistants quickly understand:
/// - What code has surviving mutants (test gaps)
/// - What changes were made that tests didn't catch
/// - What kind of tests would catch each mutant
pub fn generate_ai_report(
    result: &MutationResult,
    test_results: &[MutantTestResult],
    output_path: &Path,
) -> Result<()> {
    let mut report = String::new();

    // Header with summary
    report.push_str("# Mutation Testing Report (AI-Optimized)\n\n");
    report.push_str("## Summary\n\n");
    let _ = writeln!(
        report,
        "- **Mutation Score**: {:.1}%",
        result.mutation_score
    );
    let _ = writeln!(report, "- **Total Mutants**: {}", result.total);
    let _ = writeln!(
        report,
        "- **Killed**: {} (tests caught the bug)",
        result.killed
    );
    let _ = writeln!(
        report,
        "- **Survived**: {} (tests missed the bug)",
        result.survived
    );
    let _ = writeln!(report, "- **Timeout**: {}", result.timeout);
    let _ = writeln!(report, "- **Errors**: {}\n", result.errors);

    // Group survived mutants by file
    let mut survived_by_file: HashMap<String, Vec<&MutantTestResult>> = HashMap::new();
    for r in test_results {
        if matches!(r.status, MutantStatus::Survived) {
            let file = r.mutation.location.file.display().to_string();
            survived_by_file.entry(file).or_default().push(r);
        }
    }

    if survived_by_file.is_empty() {
        report.push_str("## Result\n\n");
        report.push_str("All mutants were killed. Test suite has excellent coverage.\n");
    } else {
        report.push_str("## Surviving Mutants (Action Required)\n\n");
        report.push_str("These mutations were NOT detected by tests. Each represents a potential bug your tests would miss.\n\n");

        // Sort files by number of survivors (worst first)
        let mut files: Vec<_> = survived_by_file.iter().collect();
        files.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (file, mutants) in files {
            let _ = writeln!(report, "### {}\n", file);
            let _ = writeln!(report, "{} surviving mutant(s)\n", mutants.len());

            for mutant in mutants {
                let m = &mutant.mutation;
                let _ = writeln!(
                    report,
                    "#### Line {}:{}\n",
                    m.location.start_line, m.location.start_col
                );
                let _ = writeln!(report, "**Mutation**: `{}` → `{}`\n", m.original, m.mutated);
                let _ = writeln!(report, "**Operator**: {}\n", m.operator.name());

                // Generate test hint based on operator
                let test_hint = generate_test_hint(&m.operator, &m.original, &m.mutated);
                let _ = writeln!(report, "**Suggested Test**: {}\n", test_hint);

                report.push_str("---\n\n");
            }
        }
    }

    // Add section for easy copy-paste file:line references
    if !survived_by_file.is_empty() {
        report.push_str("## Quick Reference (file:line)\n\n");
        report.push_str("```\n");
        for (file, mutants) in &survived_by_file {
            for mutant in mutants {
                let _ = writeln!(
                    report,
                    "{}:{}  # {} → {}",
                    file,
                    mutant.mutation.location.start_line,
                    mutant.mutation.original,
                    mutant.mutation.mutated
                );
            }
        }
        report.push_str("```\n");
    }

    std::fs::create_dir_all(output_path.parent().unwrap_or(Path::new(".")))?;
    std::fs::write(output_path, report).context("Failed to write AI report")?;

    Ok(())
}

/// Generate a test hint based on the mutation operator
fn generate_test_hint(
    operator: &crate::mutation::MutationOperator,
    original: &str,
    mutated: &str,
) -> String {
    use crate::mutation::MutationOperator;

    match operator {
        // Arithmetic
        MutationOperator::Arithmetic
        | MutationOperator::ArithmeticAddToSub
        | MutationOperator::ArithmeticSubToAdd => {
            format!(
                "Add a test that verifies the arithmetic result. If `{}` changed to `{}`, \
                test with values where addition vs subtraction gives different results (e.g., non-zero operands).",
                original, mutated
            )
        }
        MutationOperator::ArithmeticMulToDiv | MutationOperator::ArithmeticDivToMul => {
            format!(
                "Test with values where `{}` vs `{}` produce different results. \
                Avoid values like 1 or 0 that may give same result for both operations.",
                original, mutated
            )
        }
        MutationOperator::ArithmeticModToMul => {
            "Test modulo operation with values that produce a remainder (not evenly divisible)."
                .to_string()
        }

        // Comparison
        MutationOperator::Comparison
        | MutationOperator::ComparisonLtToLte
        | MutationOperator::ComparisonLteToLt
        | MutationOperator::ComparisonGtToGte
        | MutationOperator::ComparisonGteToGt => {
            format!(
                "Add a boundary test. Test with exact boundary value where `{}` vs `{}` differ. \
                If testing `<` vs `<=`, use the exact boundary value.",
                original, mutated
            )
        }
        MutationOperator::ComparisonLtToGt
        | MutationOperator::ComparisonLteToGte
        | MutationOperator::ComparisonGtToLt
        | MutationOperator::ComparisonGteToLte => {
            "Add tests for values on both sides of the comparison. \
            Test with value less than, equal to, and greater than the boundary."
                .to_string()
        }
        MutationOperator::ComparisonEqToNeq | MutationOperator::ComparisonNeqToEq => {
            format!(
                "Test both equality and inequality. If `{}` → `{}`, ensure tests verify \
                both the equal case AND the not-equal case.",
                original, mutated
            )
        }

        // Logical
        MutationOperator::Logical
        | MutationOperator::LogicalAndToOr
        | MutationOperator::LogicalOrToAnd => {
            "Test all combinations of boolean conditions. For `&&` vs `||`, test cases where \
            one condition is true and the other is false."
                .to_string()
        }
        MutationOperator::LogicalNotRemoval => {
            "Add tests for both true and false outcomes of the negated expression. \
            Ensure the test fails when negation is removed."
                .to_string()
        }

        // Boolean
        MutationOperator::Boolean
        | MutationOperator::BooleanTrueToFalse
        | MutationOperator::BooleanFalseToTrue => {
            format!(
                "The boolean `{}` was changed to `{}`. Add a test that \
                explicitly checks this boolean's effect on behavior.",
                original, mutated
            )
        }

        // Unary
        MutationOperator::Unary
        | MutationOperator::UnaryIncrementToDecrement
        | MutationOperator::UnaryDecrementToIncrement => {
            "Test that the value changes in the expected direction. \
            Verify increment increases and decrement decreases."
                .to_string()
        }
        MutationOperator::UnaryMinusRemoval | MutationOperator::UnaryPlusMinus => {
            "Test with positive and negative values to ensure sign is handled correctly."
                .to_string()
        }
        MutationOperator::UnaryPreToPost | MutationOperator::UnaryPostToPre => {
            "Test that uses the return value of the increment/decrement expression. \
            Pre vs post increment differ in what value is returned."
                .to_string()
        }

        // Null Safety
        MutationOperator::NullSafety | MutationOperator::NullCoalescingRemoval => {
            "Test with null input to verify the fallback value is used. \
            The `??` operator's right side should be tested."
                .to_string()
        }
        MutationOperator::NullAwareAccessRemoval => {
            "Test with null object to ensure null-safe access (`?.`) prevents crash. \
            Verify behavior when the object is null vs non-null."
                .to_string()
        }
        MutationOperator::NullAssertionRemoval => {
            "Test with non-null values to ensure assertion (`!`) behavior is correct.".to_string()
        }
        MutationOperator::NullCheckToTrue | MutationOperator::NullCheckToFalse => {
            "Test with both null and non-null values to verify null check works correctly."
                .to_string()
        }

        // String
        MutationOperator::String
        | MutationOperator::StringEmptyToNonEmpty
        | MutationOperator::StringNonEmptyToEmpty => {
            "Test with both empty and non-empty strings. Verify behavior differs appropriately."
                .to_string()
        }

        // Control Flow
        MutationOperator::Conditional
        | MutationOperator::ControlFlowIfConditionTrue
        | MutationOperator::ControlFlowIfConditionFalse => {
            "Add tests that exercise both branches of the if statement. \
            Ensure tests verify behavior when condition is true AND when false."
                .to_string()
        }
        MutationOperator::ControlFlowRemoveElse => {
            "Test the else branch explicitly. Verify behavior when the if condition is false."
                .to_string()
        }
        MutationOperator::ControlFlowBreakRemoval
        | MutationOperator::ControlFlowContinueRemoval => {
            "Test loop termination/continuation. Verify the loop stops or continues at the right time."
                .to_string()
        }
        MutationOperator::ControlFlowReturnRemoval => {
            "Test early return conditions. Verify function returns expected value at the return point."
                .to_string()
        }

        // Collection
        MutationOperator::Collection
        | MutationOperator::CollectionEmptyCheck
        | MutationOperator::CollectionNotEmptyCheck => {
            "Test with empty collection AND non-empty collection. \
            Verify isEmpty/isNotEmpty checks affect behavior."
                .to_string()
        }
        MutationOperator::CollectionFirstToLast | MutationOperator::CollectionLastToFirst => {
            "Test with collection having different first and last elements. \
            Verify correct element is accessed."
                .to_string()
        }
        MutationOperator::CollectionAddRemoval => {
            "Verify the collection modification occurs. Test that add() actually adds the element."
                .to_string()
        }

        // Other - catch-all for any other operators
        _ => format!(
            "Add a test that verifies the behavior changes when `{}` is replaced with `{}`.",
            original, mutated
        ),
    }
}
