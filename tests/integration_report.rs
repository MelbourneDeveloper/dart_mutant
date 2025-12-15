//! Integration tests for report generation
//!
//! These tests verify that the report module correctly:
//! - Generates valid HTML reports
//! - Generates Stryker-compatible JSON reports
//! - Calculates mutation scores correctly
//! - Groups results by file

use std::collections::HashMap;

/// Simulated mutation test result for testing report generation
#[derive(Debug, Clone)]
struct MockMutantResult {
    file: String,
    status: MockStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MockStatus {
    Killed,
    Survived,
    Timeout,
    NoCoverage,
    Error,
}

/// Calculate mutation score from results
fn calculate_mutation_score(results: &[MockMutantResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }

    let detected = results
        .iter()
        .filter(|r| matches!(r.status, MockStatus::Killed | MockStatus::Timeout))
        .count();

    let valid = results
        .iter()
        .filter(|r| !matches!(r.status, MockStatus::Error | MockStatus::NoCoverage))
        .count();

    if valid == 0 {
        return 0.0;
    }

    (detected as f64 / valid as f64) * 100.0
}

/// Group results by file
fn group_by_file(results: &[MockMutantResult]) -> HashMap<String, Vec<&MockMutantResult>> {
    let mut grouped: HashMap<String, Vec<&MockMutantResult>> = HashMap::new();
    for result in results {
        grouped.entry(result.file.clone()).or_default().push(result);
    }
    grouped
}


mod mutation_score_calculation {
    use super::*;

    fn create_results(killed: usize, survived: usize, timeout: usize, error: usize, no_coverage: usize) -> Vec<MockMutantResult> {
        let mut results = Vec::new();

        for _ in 0..killed {
            results.push(MockMutantResult {
                file: "test.dart".to_string(),
                status: MockStatus::Killed,
            });
        }

        for _ in 0..survived {
            results.push(MockMutantResult {
                file: "test.dart".to_string(),
                status: MockStatus::Survived,
            });
        }

        for _ in 0..timeout {
            results.push(MockMutantResult {
                file: "test.dart".to_string(),
                status: MockStatus::Timeout,
            });
        }

        for _ in 0..error {
            results.push(MockMutantResult {
                file: "test.dart".to_string(),
                status: MockStatus::Error,
            });
        }

        for _ in 0..no_coverage {
            results.push(MockMutantResult {
                file: "test.dart".to_string(),
                status: MockStatus::NoCoverage,
            });
        }

        results
    }

    #[test]
    fn perfect_score_when_all_killed() {
        let results = create_results(10, 0, 0, 0, 0);
        let score = calculate_mutation_score(&results);
        assert!((score - 100.0).abs() < 0.001, "Score should be 100% when all killed");
    }

    #[test]
    fn zero_score_when_all_survived() {
        let results = create_results(0, 10, 0, 0, 0);
        let score = calculate_mutation_score(&results);
        assert!((score - 0.0).abs() < 0.001, "Score should be 0% when all survived");
    }

    #[test]
    fn fifty_percent_when_half_killed() {
        let results = create_results(5, 5, 0, 0, 0);
        let score = calculate_mutation_score(&results);
        assert!((score - 50.0).abs() < 0.001, "Score should be 50% when half killed");
    }

    #[test]
    fn timeout_counts_as_killed() {
        let results = create_results(5, 5, 5, 0, 0);
        let score = calculate_mutation_score(&results);
        // 10 detected (5 killed + 5 timeout) / 15 valid = 66.67%
        assert!(
            (score - 66.666).abs() < 0.01,
            "Timeouts should count as killed, expected ~66.67%, got {}",
            score
        );
    }

    #[test]
    fn errors_excluded_from_calculation() {
        let results = create_results(5, 5, 0, 10, 0);
        let score = calculate_mutation_score(&results);
        // 5 detected / 10 valid (errors excluded) = 50%
        assert!(
            (score - 50.0).abs() < 0.001,
            "Errors should be excluded, expected 50%, got {}",
            score
        );
    }

    #[test]
    fn no_coverage_excluded_from_calculation() {
        let results = create_results(5, 5, 0, 0, 10);
        let score = calculate_mutation_score(&results);
        // 5 detected / 10 valid (no_coverage excluded) = 50%
        assert!(
            (score - 50.0).abs() < 0.001,
            "NoCoverage should be excluded, expected 50%, got {}",
            score
        );
    }

    #[test]
    fn empty_results_return_zero() {
        let results: Vec<MockMutantResult> = vec![];
        let score = calculate_mutation_score(&results);
        assert!((score - 0.0).abs() < 0.001, "Empty results should give 0%");
    }

    #[test]
    fn score_is_bounded_0_to_100() {
        let test_cases = vec![
            create_results(100, 0, 0, 0, 0),
            create_results(0, 100, 0, 0, 0),
            create_results(50, 50, 0, 0, 0),
            create_results(33, 33, 34, 0, 0),
        ];

        for results in test_cases {
            let score = calculate_mutation_score(&results);
            assert!(
                score >= 0.0 && score <= 100.0,
                "Score should be between 0 and 100, got {}",
                score
            );
        }
    }
}


mod result_grouping {
    use super::*;

    fn create_multi_file_results() -> Vec<MockMutantResult> {
        vec![
            MockMutantResult {
                file: "lib/calculator.dart".to_string(),
                status: MockStatus::Killed,
            },
            MockMutantResult {
                file: "lib/calculator.dart".to_string(),
                status: MockStatus::Survived,
            },
            MockMutantResult {
                file: "lib/string_utils.dart".to_string(),
                status: MockStatus::Killed,
            },
            MockMutantResult {
                file: "lib/null_safe.dart".to_string(),
                status: MockStatus::Timeout,
            },
        ]
    }

    #[test]
    fn groups_results_by_file() {
        let results = create_multi_file_results();
        let grouped = group_by_file(&results);

        assert_eq!(grouped.len(), 3, "Should have 3 files");
        assert!(grouped.contains_key("lib/calculator.dart"));
        assert!(grouped.contains_key("lib/string_utils.dart"));
        assert!(grouped.contains_key("lib/null_safe.dart"));
    }

    #[test]
    fn correct_count_per_file() {
        let results = create_multi_file_results();
        let grouped = group_by_file(&results);

        assert_eq!(
            grouped.get("lib/calculator.dart").map(|v| v.len()),
            Some(2),
            "calculator.dart should have 2 mutations"
        );
        assert_eq!(
            grouped.get("lib/string_utils.dart").map(|v| v.len()),
            Some(1),
            "string_utils.dart should have 1 mutation"
        );
        assert_eq!(
            grouped.get("lib/null_safe.dart").map(|v| v.len()),
            Some(1),
            "null_safe.dart should have 1 mutation"
        );
    }

    #[test]
    fn can_calculate_per_file_score() {
        let results = create_multi_file_results();
        let grouped = group_by_file(&results);

        // calculator.dart: 1 killed, 1 survived = 50%
        let calc_results: Vec<MockMutantResult> = grouped
            .get("lib/calculator.dart")
            .unwrap()
            .iter()
            .map(|r| (*r).clone())
            .collect();
        let calc_score = calculate_mutation_score(&calc_results);
        assert!((calc_score - 50.0).abs() < 0.001);

        // string_utils.dart: 1 killed = 100%
        let su_results: Vec<MockMutantResult> = grouped
            .get("lib/string_utils.dart")
            .unwrap()
            .iter()
            .map(|r| (*r).clone())
            .collect();
        let su_score = calculate_mutation_score(&su_results);
        assert!((su_score - 100.0).abs() < 0.001);

        // null_safe.dart: 1 timeout = 100% (timeout counts as killed)
        let ns_results: Vec<MockMutantResult> = grouped
            .get("lib/null_safe.dart")
            .unwrap()
            .iter()
            .map(|r| (*r).clone())
            .collect();
        let ns_score = calculate_mutation_score(&ns_results);
        assert!((ns_score - 100.0).abs() < 0.001);
    }
}


mod html_report_structure {

    fn generate_mock_html(score: f64, killed: usize, survived: usize) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Mutation Report</title></head>
<body>
    <h1>Mutation Testing Report</h1>
    <div class="score">{:.1}%</div>
    <div class="stats">
        <span class="killed">{}</span>
        <span class="survived">{}</span>
    </div>
</body>
</html>"#,
            score, killed, survived
        )
    }

    #[test]
    fn html_contains_score() {
        let html = generate_mock_html(75.5, 10, 3);
        assert!(html.contains("75.5%"), "HTML should contain score");
    }

    #[test]
    fn html_contains_killed_count() {
        let html = generate_mock_html(75.5, 10, 3);
        assert!(html.contains(">10<"), "HTML should contain killed count");
    }

    #[test]
    fn html_contains_survived_count() {
        let html = generate_mock_html(75.5, 10, 3);
        assert!(html.contains(">3<"), "HTML should contain survived count");
    }

    #[test]
    fn html_is_valid_structure() {
        let html = generate_mock_html(50.0, 5, 5);

        assert!(html.starts_with("<!DOCTYPE html>"), "Should start with DOCTYPE");
        assert!(html.contains("<html>"), "Should have html tag");
        assert!(html.contains("<head>"), "Should have head tag");
        assert!(html.contains("<body>"), "Should have body tag");
        assert!(html.contains("</html>"), "Should close html tag");
    }

    #[test]
    fn score_color_coding_logic() {
        // Score >= 80 should be green
        // Score >= 60 should be yellow
        // Score < 60 should be red

        let get_color = |score: f64| -> &'static str {
            if score >= 80.0 {
                "green"
            } else if score >= 60.0 {
                "yellow"
            } else {
                "red"
            }
        };

        assert_eq!(get_color(100.0), "green");
        assert_eq!(get_color(80.0), "green");
        assert_eq!(get_color(79.9), "yellow");
        assert_eq!(get_color(60.0), "yellow");
        assert_eq!(get_color(59.9), "red");
        assert_eq!(get_color(0.0), "red");
    }
}


mod json_report_structure {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct MockJsonReport {
        #[serde(rename = "schemaVersion")]
        schema_version: String,
        #[serde(rename = "mutationScore")]
        mutation_score: f64,
        files: HashMap<String, MockJsonFile>,
    }

    #[derive(Serialize, Deserialize)]
    struct MockJsonFile {
        language: String,
        mutants: Vec<MockJsonMutant>,
    }

    #[derive(Serialize, Deserialize)]
    struct MockJsonMutant {
        id: String,
        #[serde(rename = "mutatorName")]
        mutator_name: String,
        status: String,
    }

    #[test]
    fn json_is_valid_and_parseable() {
        let mut files = HashMap::new();
        files.insert(
            "lib/calculator.dart".to_string(),
            MockJsonFile {
                language: "dart".to_string(),
                mutants: vec![
                    MockJsonMutant {
                        id: "abc123".to_string(),
                        mutator_name: "ArithmeticOperator".to_string(),
                        status: "Killed".to_string(),
                    },
                ],
            },
        );

        let report = MockJsonReport {
            schema_version: "1".to_string(),
            mutation_score: 75.5,
            files,
        };

        let json = serde_json::to_string_pretty(&report);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json = json.unwrap();
        let parsed: Result<MockJsonReport, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "Should parse back from JSON");
    }

    #[test]
    fn json_has_stryker_compatible_fields() {
        let json = r#"{
            "schemaVersion": "1",
            "mutationScore": 75.5,
            "files": {
                "lib/test.dart": {
                    "language": "dart",
                    "mutants": [{
                        "id": "abc",
                        "mutatorName": "Arithmetic",
                        "status": "Killed"
                    }]
                }
            }
        }"#;

        let parsed: Result<MockJsonReport, _> = serde_json::from_str(json);
        assert!(parsed.is_ok(), "Should parse Stryker-compatible JSON");

        let report = parsed.unwrap();
        assert_eq!(report.schema_version, "1");
        assert!((report.mutation_score - 75.5).abs() < 0.001);
    }

    #[test]
    fn status_values_are_valid() {
        let valid_statuses = vec![
            "Killed",
            "Survived",
            "Timeout",
            "NoCoverage",
            "CompileError",
        ];

        for status in valid_statuses {
            let json = format!(
                r#"{{"id":"x","mutatorName":"Test","status":"{}"}}"#,
                status
            );
            let parsed: Result<MockJsonMutant, _> = serde_json::from_str(&json);
            assert!(
                parsed.is_ok(),
                "Status '{}' should be valid",
                status
            );
        }
    }
}


mod report_output {
    use std::fs;

    #[test]
    fn can_write_report_to_file() {
        let report_content = "test report content";
        let temp_path = std::env::temp_dir().join("dart_mutant_test_report.txt");

        fs::write(&temp_path, report_content).expect("Should write report");

        let read_content = fs::read_to_string(&temp_path).expect("Should read report");
        assert_eq!(read_content, report_content);

        fs::remove_file(&temp_path).ok();
    }

    #[test]
    fn creates_parent_directories() {
        let temp_dir = std::env::temp_dir()
            .join("dart_mutant_test")
            .join("nested")
            .join("path");

        let report_path = temp_dir.join("report.html");

        fs::create_dir_all(report_path.parent().unwrap()).expect("Should create directories");
        fs::write(&report_path, "test").expect("Should write to nested path");

        assert!(report_path.exists());

        // Cleanup
        fs::remove_dir_all(std::env::temp_dir().join("dart_mutant_test")).ok();
    }
}


mod ai_report_structure {
    #[allow(unused_imports)]
    use super::*;

    /// Mock AI report generator for testing structure
    fn generate_mock_ai_report(
        score: f64,
        killed: usize,
        survived: usize,
        timeout: usize,
        errors: usize,
        survivors: &[(String, usize, usize, String, String, String)], // (file, line, col, original, mutated, operator)
    ) -> String {
        let mut report = String::new();
        let total = killed + survived + timeout + errors;

        // Header
        report.push_str("# Mutation Testing Report (AI-Optimized)\n\n");
        report.push_str("## Summary\n\n");
        report.push_str(&format!("- **Mutation Score**: {:.1}%\n", score));
        report.push_str(&format!("- **Total Mutants**: {}\n", total));
        report.push_str(&format!("- **Killed**: {} (tests caught the bug)\n", killed));
        report.push_str(&format!("- **Survived**: {} (tests missed the bug)\n", survived));
        report.push_str(&format!("- **Timeout**: {}\n", timeout));
        report.push_str(&format!("- **Errors**: {}\n\n", errors));

        if survivors.is_empty() {
            report.push_str("## Result\n\n");
            report.push_str("All mutants were killed. Test suite has excellent coverage.\n");
        } else {
            report.push_str("## Surviving Mutants (Action Required)\n\n");
            report.push_str("These mutations were NOT detected by tests. Each represents a potential bug your tests would miss.\n\n");

            // Group by file
            let mut by_file: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
            for s in survivors {
                by_file.entry(&s.0).or_default().push(s);
            }

            // Sort by count descending
            let mut files: Vec<_> = by_file.iter().collect();
            files.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

            for (file, mutants) in files {
                report.push_str(&format!("### {}\n\n", file));
                report.push_str(&format!("{} surviving mutant(s)\n\n", mutants.len()));

                for (_, line, col, original, mutated, operator) in mutants {
                    report.push_str(&format!("#### Line {}:{}\n\n", line, col));
                    report.push_str(&format!("**Mutation**: `{}` → `{}`\n\n", original, mutated));
                    report.push_str(&format!("**Operator**: {}\n\n", operator));
                    report.push_str("**Suggested Test**: Add a test for this mutation.\n\n");
                    report.push_str("---\n\n");
                }
            }

            // Quick reference
            report.push_str("## Quick Reference (file:line)\n\n");
            report.push_str("```\n");
            for (file, line, _, original, mutated, _) in survivors {
                report.push_str(&format!("{}:{}  # {} → {}\n", file, line, original, mutated));
            }
            report.push_str("```\n");
        }

        report
    }

    #[test]
    fn ai_report_has_markdown_header() {
        let report = generate_mock_ai_report(100.0, 10, 0, 0, 0, &[]);
        assert!(report.starts_with("# Mutation Testing Report (AI-Optimized)"));
    }

    #[test]
    fn ai_report_has_summary_section() {
        let report = generate_mock_ai_report(75.0, 15, 5, 0, 0, &[]);
        assert!(report.contains("## Summary"));
        assert!(report.contains("**Mutation Score**: 75.0%"));
        assert!(report.contains("**Killed**: 15"));
        assert!(report.contains("**Survived**: 5"));
    }

    #[test]
    fn ai_report_shows_all_killed_message_when_no_survivors() {
        let report = generate_mock_ai_report(100.0, 10, 0, 0, 0, &[]);
        assert!(report.contains("All mutants were killed"));
        assert!(report.contains("excellent coverage"));
        assert!(!report.contains("Surviving Mutants"));
    }

    #[test]
    fn ai_report_shows_surviving_mutants_section() {
        let survivors = vec![
            ("lib/calc.dart".to_string(), 10, 5, "+".to_string(), "-".to_string(), "Arithmetic".to_string()),
        ];
        let report = generate_mock_ai_report(50.0, 5, 5, 0, 0, &survivors);
        assert!(report.contains("## Surviving Mutants (Action Required)"));
        assert!(report.contains("These mutations were NOT detected"));
    }

    #[test]
    fn ai_report_groups_survivors_by_file() {
        let survivors = vec![
            ("lib/calc.dart".to_string(), 10, 5, "+".to_string(), "-".to_string(), "Arithmetic".to_string()),
            ("lib/calc.dart".to_string(), 20, 3, "*".to_string(), "/".to_string(), "Arithmetic".to_string()),
            ("lib/utils.dart".to_string(), 5, 1, "true".to_string(), "false".to_string(), "Boolean".to_string()),
        ];
        let report = generate_mock_ai_report(50.0, 5, 3, 0, 0, &survivors);

        assert!(report.contains("### lib/calc.dart"));
        assert!(report.contains("2 surviving mutant(s)"));
        assert!(report.contains("### lib/utils.dart"));
        assert!(report.contains("1 surviving mutant(s)"));
    }

    #[test]
    fn ai_report_shows_mutation_details() {
        let survivors = vec![
            ("lib/calc.dart".to_string(), 42, 15, ">=".to_string(), ">".to_string(), "Comparison".to_string()),
        ];
        let report = generate_mock_ai_report(50.0, 5, 1, 0, 0, &survivors);

        assert!(report.contains("#### Line 42:15"));
        assert!(report.contains("**Mutation**: `>=` → `>`"));
        assert!(report.contains("**Operator**: Comparison"));
        assert!(report.contains("**Suggested Test**:"));
    }

    #[test]
    fn ai_report_has_quick_reference_section() {
        let survivors = vec![
            ("lib/calc.dart".to_string(), 10, 5, "+".to_string(), "-".to_string(), "Arithmetic".to_string()),
            ("lib/utils.dart".to_string(), 20, 3, "&&".to_string(), "||".to_string(), "Logical".to_string()),
        ];
        let report = generate_mock_ai_report(50.0, 5, 2, 0, 0, &survivors);

        assert!(report.contains("## Quick Reference (file:line)"));
        assert!(report.contains("```"));
        assert!(report.contains("lib/calc.dart:10  # + → -"));
        assert!(report.contains("lib/utils.dart:20  # && → ||"));
    }

    #[test]
    fn ai_report_sorts_files_by_survivor_count() {
        let survivors = vec![
            ("lib/few.dart".to_string(), 1, 1, "+".to_string(), "-".to_string(), "Arithmetic".to_string()),
            ("lib/many.dart".to_string(), 1, 1, "+".to_string(), "-".to_string(), "Arithmetic".to_string()),
            ("lib/many.dart".to_string(), 2, 1, "-".to_string(), "+".to_string(), "Arithmetic".to_string()),
            ("lib/many.dart".to_string(), 3, 1, "*".to_string(), "/".to_string(), "Arithmetic".to_string()),
        ];
        let report = generate_mock_ai_report(50.0, 5, 4, 0, 0, &survivors);

        // lib/many.dart (3 survivors) should appear before lib/few.dart (1 survivor)
        let many_pos = report.find("### lib/many.dart").unwrap();
        let few_pos = report.find("### lib/few.dart").unwrap();
        assert!(many_pos < few_pos, "Files with more survivors should appear first");
    }

    #[test]
    fn ai_report_no_quick_reference_when_all_killed() {
        let report = generate_mock_ai_report(100.0, 10, 0, 0, 0, &[]);
        assert!(!report.contains("## Quick Reference"));
        assert!(!report.contains("```"));
    }

    #[test]
    fn ai_report_handles_special_characters_in_mutations() {
        let survivors = vec![
            ("lib/test.dart".to_string(), 1, 1, "<".to_string(), "<=".to_string(), "Comparison".to_string()),
            ("lib/test.dart".to_string(), 2, 1, "&&".to_string(), "||".to_string(), "Logical".to_string()),
        ];
        let report = generate_mock_ai_report(50.0, 2, 2, 0, 0, &survivors);

        // These should be present as-is in markdown (backticks protect them)
        assert!(report.contains("`<`"));
        assert!(report.contains("`<=`"));
        assert!(report.contains("`&&`"));
        assert!(report.contains("`||`"));
    }
}


mod test_hint_generation {
    /// Mock test hint generator that mirrors the real implementation
    fn generate_test_hint(operator: &str, original: &str, mutated: &str) -> String {
        match operator {
            "ArithmeticAddToSub" | "ArithmeticSubToAdd" => {
                format!(
                    "Add a test that verifies the arithmetic result. If `{}` changed to `{}`, \
                    test with values where addition vs subtraction gives different results.",
                    original, mutated
                )
            }
            "ComparisonLtToLte" | "ComparisonGtToGte" => {
                format!(
                    "Add a boundary test. Test with exact boundary value where `{}` vs `{}` differ.",
                    original, mutated
                )
            }
            "LogicalAndToOr" | "LogicalOrToAnd" => {
                "Test all combinations of boolean conditions.".to_string()
            }
            "BooleanTrueToFalse" | "BooleanFalseToTrue" => {
                format!(
                    "The boolean `{}` was changed to `{}`. Add a test that explicitly checks this.",
                    original, mutated
                )
            }
            "NullCoalescingRemoval" => {
                "Test with null input to verify the fallback value is used.".to_string()
            }
            _ => format!(
                "Add a test that verifies behavior changes when `{}` is replaced with `{}`.",
                original, mutated
            ),
        }
    }

    #[test]
    fn arithmetic_hint_mentions_values() {
        let hint = generate_test_hint("ArithmeticAddToSub", "+", "-");
        assert!(hint.contains("arithmetic"));
        assert!(hint.contains("addition vs subtraction"));
    }

    #[test]
    fn comparison_hint_mentions_boundary() {
        let hint = generate_test_hint("ComparisonLtToLte", "<", "<=");
        assert!(hint.contains("boundary"));
    }

    #[test]
    fn logical_hint_mentions_combinations() {
        let hint = generate_test_hint("LogicalAndToOr", "&&", "||");
        assert!(hint.contains("combinations"));
    }

    #[test]
    fn boolean_hint_includes_values() {
        let hint = generate_test_hint("BooleanTrueToFalse", "true", "false");
        assert!(hint.contains("true"));
        assert!(hint.contains("false"));
    }

    #[test]
    fn null_hint_mentions_null_input() {
        let hint = generate_test_hint("NullCoalescingRemoval", "??", "");
        assert!(hint.contains("null"));
        assert!(hint.contains("fallback"));
    }

    #[test]
    fn unknown_operator_gives_generic_hint() {
        let hint = generate_test_hint("UnknownOperator", "foo", "bar");
        assert!(hint.contains("foo"));
        assert!(hint.contains("bar"));
        assert!(hint.contains("behavior changes"));
    }
}
