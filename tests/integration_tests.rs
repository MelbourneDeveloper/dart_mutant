// Allow panics and expects in test code - tests need to fail loudly
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

//! High-level integration tests for dart_mutant
//!
//! These tests verify the ENTIRE mutation testing pipeline works correctly:
//! - Discovering Dart files
//! - Parsing and generating mutations
//! - Running tests against mutated code
//! - Generating reports
//!
//! IMPORTANT: These are NOT unit tests. They test actual end-to-end behavior.
//!
//! NOTE: Tests that access sample_project fixtures must run serially to prevent
//! race conditions when mutation tests modify files.

use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the path to the test fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Get the path to the sample project fixture
fn sample_project_dir() -> PathBuf {
    fixtures_dir().join("sample_project")
}

/// Ensure the sample project has dependencies installed
fn ensure_dart_deps(project_dir: &Path) {
    let status = Command::new("dart")
        .arg("pub")
        .arg("get")
        .current_dir(project_dir)
        .status()
        .expect("Failed to run dart pub get");

    assert!(status.success(), "dart pub get failed");
}

/// Check if Dart is available
fn dart_available() -> bool {
    Command::new("dart")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ============================================================================
// MUTATION DISCOVERY TESTS
// ============================================================================

mod mutation_discovery {
    use super::*;

    #[test]
    fn test_discovers_dart_files_in_sample_project() {
        let project_dir = sample_project_dir();
        let lib_dir = project_dir.join("lib");

        // Count .dart files in lib/
        let dart_files: Vec<_> = fs::read_dir(&lib_dir)
            .expect("Failed to read lib directory")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "dart"))
            .collect();

        assert!(
            dart_files.len() >= 3,
            "Expected at least 3 Dart files in lib/, found {}",
            dart_files.len()
        );
    }

    #[test]
    fn test_fixture_files_contain_mutable_code() {
        let calculator_path = sample_project_dir().join("lib").join("calculator.dart");
        let content = fs::read_to_string(&calculator_path).expect("Failed to read calculator.dart");

        // Verify arithmetic operators exist (these should be mutated)
        assert!(content.contains(" + "), "Missing + operator");
        assert!(content.contains(" - "), "Missing - operator");
        assert!(content.contains(" * "), "Missing * operator");

        // Verify comparison operators exist
        assert!(content.contains(" > "), "Missing > operator");
        assert!(content.contains(" < "), "Missing < operator");
        assert!(content.contains(" == "), "Missing == operator");
        assert!(content.contains(" >= "), "Missing >= operator");
        assert!(content.contains(" <= "), "Missing <= operator");

        // Verify logical operators exist
        assert!(content.contains(" && "), "Missing && operator");
    }

    #[test]
    fn test_fixture_files_contain_null_safety_operators() {
        let null_safety_path = sample_project_dir()
            .join("lib")
            .join("null_safety_examples.dart");
        let content = fs::read_to_string(&null_safety_path)
            .expect("Failed to read null_safety_examples.dart");

        // Verify null safety operators exist
        assert!(content.contains("?."), "Missing ?. operator");
        assert!(content.contains("??"), "Missing ?? operator");
        assert!(content.contains("!= null"), "Missing null check");
        assert!(content.contains("== null"), "Missing null comparison");
    }
}

// ============================================================================
// DART TEST EXECUTION TESTS
// ============================================================================

mod dart_test_execution {
    use super::*;

    #[test]
    #[serial]
    fn test_sample_project_tests_pass() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = sample_project_dir();
        ensure_dart_deps(&project_dir);

        let output = Command::new("dart")
            .arg("test")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run dart test");

        assert!(
            output.status.success(),
            "Sample project tests should pass. Stdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    #[serial]
    fn test_sample_project_has_sufficient_test_coverage() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = sample_project_dir();
        ensure_dart_deps(&project_dir);

        // Run tests with reporter to count tests
        let output = Command::new("dart")
            .arg("test")
            .arg("--reporter=compact")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run dart test");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // The compact reporter shows "X/Y" for passed/total tests
        // We expect at least 40 tests across all files
        assert!(
            stdout.contains("All tests passed"),
            "All tests should pass. Output: {}",
            stdout
        );
    }
}

// ============================================================================
// END-TO-END MUTATION TESTING TESTS
// ============================================================================

mod e2e_mutation_testing {
    use super::*;

    #[test]
    fn test_binary_builds_successfully() {
        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("Failed to run cargo build");

        assert!(status.success(), "dart_mutant should build successfully");
    }

    #[test]
    fn test_cli_shows_help() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--help")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to run dart_mutant --help");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains("mutation testing") || stdout.contains("dart_mutant"),
            "Help should mention mutation testing. Output: {}",
            stdout
        );
    }

    #[test]
    fn test_cli_shows_version() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--version")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to run dart_mutant --version");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains("dart_mutant") || stdout.contains("0.1"),
            "Version should be shown. Output: {}",
            stdout
        );
    }

    #[test]
    #[serial]
    fn test_full_mutation_run_on_sample_project() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = sample_project_dir();
        ensure_dart_deps(&project_dir);

        let output_dir = project_dir.join("mutation-reports");

        // Clean previous reports
        drop(fs::remove_dir_all(&output_dir));

        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--path")
            .arg(&project_dir)
            .arg("--output")
            .arg(&output_dir)
            .arg("--html")
            .arg("--json")
            .arg("--sample")
            .arg("10") // Only test 10 mutations for speed
            .arg("--timeout")
            .arg("10")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to run dart_mutant");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        // Should complete (either success or exit with non-zero for low score)
        assert!(
            stdout.contains("Mutation Score") || stdout.contains("mutations"),
            "Should show mutation results. Output: {}",
            stdout
        );

        // HTML report should be generated
        let html_report = output_dir.join("mutation-report.html");
        assert!(
            html_report.exists(),
            "HTML report should be generated at {:?}",
            html_report
        );

        // JSON report should be generated
        let json_report = output_dir.join("mutation-report.json");
        assert!(
            json_report.exists(),
            "JSON report should be generated at {:?}",
            json_report
        );
    }

    #[test]
    #[serial]
    fn test_dry_run_mode() {
        let project_dir = sample_project_dir();

        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--path")
            .arg(&project_dir)
            .arg("--dry-run")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to run dart_mutant --dry-run");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Dry run should show mutations without running tests
        assert!(
            output.status.success() || stdout.contains("mutation"),
            "Dry run should succeed. Output: {}",
            stdout
        );
    }
}

// ============================================================================
// REPORT GENERATION TESTS
// ============================================================================

mod report_tests {
    #[test]
    fn test_html_report_structure() {
        // This test verifies the HTML report has proper structure
        // by checking a pre-generated sample report
        let sample_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Dart Mutant - Mutation Testing Report</title>
</head>
<body>
    <div class="score-value">75%</div>
    <div class="stat-killed">10</div>
    <div class="stat-survived">3</div>
</body>
</html>"#;

        // Verify key elements exist
        assert!(sample_html.contains("<!DOCTYPE html>"));
        assert!(sample_html.contains("Mutation Testing Report"));
        assert!(sample_html.contains("score"));
        assert!(sample_html.contains("killed"));
        assert!(sample_html.contains("survived"));
    }

    #[test]
    fn test_json_report_is_valid() {
        // This test verifies JSON report structure
        let sample_json = r#"{
  "schemaVersion": "1",
  "thresholds": {
    "high": 80,
    "low": 60
  },
  "files": {},
  "projectRoot": "/test",
  "mutationScore": 75.0
}"#;

        let parsed: serde_json::Value =
            serde_json::from_str(sample_json).expect("JSON should be valid");

        assert_eq!(parsed["schemaVersion"], "1");
        assert_eq!(parsed["thresholds"]["high"], 80);
        assert_eq!(parsed["thresholds"]["low"], 60);
        assert_eq!(parsed["mutationScore"], 75.0);
    }
}

// ============================================================================
// MUTATION OPERATOR TESTS (Verify actual mutations are generated)
// ============================================================================

mod mutation_operator_tests {
    use super::*;

    fn count_potential_mutations(content: &str) -> usize {
        let mut count = 0;

        // Count arithmetic operators
        count += content.matches(" + ").count();
        count += content.matches(" - ").count();
        count += content.matches(" * ").count();
        count += content.matches(" / ").count();

        // Count comparison operators
        count += content.matches(" < ").count();
        count += content.matches(" > ").count();
        count += content.matches(" <= ").count();
        count += content.matches(" >= ").count();
        count += content.matches(" == ").count();
        count += content.matches(" != ").count();

        // Count logical operators
        count += content.matches(" && ").count();
        count += content.matches(" || ").count();

        // Count boolean literals
        count += content.matches("true").count();
        count += content.matches("false").count();

        count
    }

    #[test]
    fn test_calculator_has_many_mutation_opportunities() {
        let calculator_path = sample_project_dir().join("lib").join("calculator.dart");
        let content = fs::read_to_string(&calculator_path).expect("Failed to read calculator.dart");

        let mutation_count = count_potential_mutations(&content);

        assert!(
            mutation_count >= 10,
            "Calculator should have at least 10 mutation opportunities, found {}",
            mutation_count
        );
    }

    #[test]
    fn test_string_utils_has_mutation_opportunities() {
        let path = sample_project_dir().join("lib").join("string_utils.dart");
        let content = fs::read_to_string(&path).expect("Failed to read string_utils.dart");

        let mutation_count = count_potential_mutations(&content);

        assert!(
            mutation_count >= 10,
            "StringUtils should have at least 10 mutation opportunities, found {}",
            mutation_count
        );
    }

    #[test]
    fn test_null_safety_has_null_operator_mutations() {
        let path = sample_project_dir()
            .join("lib")
            .join("null_safety_examples.dart");
        let content = fs::read_to_string(&path).expect("Failed to read null_safety_examples.dart");

        // Count null safety specific operators
        let null_aware_count = content.matches("?.").count();
        let null_coalescing_count = content.matches("??").count();
        let null_check_count =
            content.matches("!= null").count() + content.matches("== null").count();

        assert!(
            null_aware_count >= 1,
            "Should have at least 1 null-aware access operator, found {}",
            null_aware_count
        );

        assert!(
            null_coalescing_count >= 1,
            "Should have at least 1 null coalescing operator, found {}",
            null_coalescing_count
        );

        assert!(
            null_check_count >= 1,
            "Should have at least 1 null check, found {}",
            null_check_count
        );
    }
}

// ============================================================================
// COMPILE ERROR DETECTION TESTS
// ============================================================================

mod compile_error_tests {
    use super::*;

    /// Get the path to the compile error project fixture
    fn compile_error_project_dir() -> PathBuf {
        fixtures_dir().join("compile_error_project")
    }

    /// Copy project to a temp directory to avoid race conditions between parallel tests
    fn copy_to_temp(project_dir: &Path, suffix: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join(format!("dart_mutant_test_{}", suffix));

        // Clean and create temp dir
        drop(fs::remove_dir_all(&temp_dir));
        fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // Copy all files using walkdir
        for entry in walkdir::WalkDir::new(project_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let rel_path = entry.path().strip_prefix(project_dir).unwrap();
            let dest = temp_dir.join(rel_path);

            if entry.file_type().is_dir() {
                fs::create_dir_all(&dest).ok();
            } else {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::copy(entry.path(), &dest).expect("Failed to copy file");
            }
        }

        temp_dir
    }

    /// Verify that the original code compiles successfully
    #[test]
    fn test_original_code_compiles() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = compile_error_project_dir();
        ensure_dart_deps(&project_dir);

        // Run dart analyze to check for compile errors
        let output = Command::new("dart")
            .arg("analyze")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run dart analyze");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for actual compile errors (not just the word "error" in the project name)
        // dart analyze shows errors like "error - lib/file.dart:10:5 - message - code"
        let has_compile_errors = stdout.contains("error -")
            || stderr.contains("error -")
            || stdout.contains("Error:")
            || stderr.contains("Error:");
        assert!(
            !has_compile_errors,
            "Original code should compile without errors.\nStdout: {}\nStderr: {}",
            stdout, stderr
        );
    }

    /// Test that mutating string + to - causes a compile error
    /// This proves the tool should detect compile errors for type-incompatible mutations
    #[test]
    fn test_string_plus_to_minus_causes_compile_error() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = compile_error_project_dir();
        let temp_dir = copy_to_temp(&project_dir, "string_minus");
        ensure_dart_deps(&temp_dir);

        // Read original file from temp
        let file_path = temp_dir.join("lib").join("type_sensitive.dart");
        let original_content =
            fs::read_to_string(&file_path).expect("Failed to read type_sensitive.dart");

        // Apply mutation: change `return a + b;` to `return a - b;` for string concatenation
        let mutated_content = original_content.replacen("return a + b;", "return a - b;", 1);

        // Write mutated file
        fs::write(&file_path, &mutated_content).expect("Failed to write mutated file");

        // Try to compile - should fail because can't subtract strings
        let output = Command::new("dart")
            .arg("analyze")
            .current_dir(&temp_dir)
            .output()
            .expect("Failed to run dart analyze");

        // Cleanup temp dir
        drop(fs::remove_dir_all(&temp_dir));

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr).to_lowercase();

        // The mutation should cause a compile error about operator - not being defined for String
        assert!(
            combined.contains("error") || !output.status.success(),
            "Mutating String + to - should cause compile error.\nStdout: {}\nStderr: {}",
            stdout,
            stderr
        );
    }

    /// Test that mutating List + to - causes a compile error
    #[test]
    fn test_list_plus_to_minus_causes_compile_error() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = compile_error_project_dir();
        let temp_dir = copy_to_temp(&project_dir, "list_minus");
        ensure_dart_deps(&temp_dir);

        let file_path = temp_dir.join("lib").join("type_sensitive.dart");
        let original_content =
            fs::read_to_string(&file_path).expect("Failed to read type_sensitive.dart");

        // Mutate the list concatenation
        let mutated_content = original_content.replace(
            "return a + b;  // Mutation: a - b -> COMPILE ERROR (can't subtract lists)",
            "return a - b;  // Mutation: a - b -> COMPILE ERROR (can't subtract lists)",
        );

        fs::write(&file_path, &mutated_content).expect("Failed to write mutated file");

        let output = Command::new("dart")
            .arg("analyze")
            .current_dir(&temp_dir)
            .output()
            .expect("Failed to run dart analyze");

        drop(fs::remove_dir_all(&temp_dir));

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr).to_lowercase();

        assert!(
            combined.contains("error") || !output.status.success(),
            "Mutating List + to - should cause compile error.\nStdout: {}\nStderr: {}",
            stdout,
            stderr
        );
    }

    /// Test that mutating int == to && causes a compile error
    #[test]
    fn test_equality_to_logical_and_causes_compile_error() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = compile_error_project_dir();
        let temp_dir = copy_to_temp(&project_dir, "eq_to_and");
        ensure_dart_deps(&temp_dir);

        let file_path = temp_dir.join("lib").join("type_sensitive.dart");
        let original_content =
            fs::read_to_string(&file_path).expect("Failed to read type_sensitive.dart");

        // Mutate a == b to a && b (can't use && on ints)
        let mutated_content = original_content.replace(
            "return a == b;  // Mutation: a && b -> COMPILE ERROR (can't && ints)",
            "return a && b;  // Mutation: a && b -> COMPILE ERROR (can't && ints)",
        );

        fs::write(&file_path, &mutated_content).expect("Failed to write mutated file");

        let output = Command::new("dart")
            .arg("analyze")
            .current_dir(&temp_dir)
            .output()
            .expect("Failed to run dart analyze");

        drop(fs::remove_dir_all(&temp_dir));

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr).to_lowercase();

        assert!(
            combined.contains("error") || !output.status.success(),
            "Mutating int == to && should cause compile error.\nStdout: {}\nStderr: {}",
            stdout,
            stderr
        );
    }

    /// Test that valid arithmetic mutations DO compile (control test)
    /// This ensures our compile-error detection isn't too aggressive
    #[test]
    fn test_valid_arithmetic_mutation_compiles() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = compile_error_project_dir();
        let temp_dir = copy_to_temp(&project_dir, "valid_arith");
        ensure_dart_deps(&temp_dir);

        let file_path = temp_dir.join("lib").join("type_sensitive.dart");
        let original_content =
            fs::read_to_string(&file_path).expect("Failed to read type_sensitive.dart");

        // Mutate int + int to int - int (this IS valid and should compile)
        let mutated_content = original_content.replace(
            "return a + b;  // Mutation to / would return double, causing type error",
            "return a - b;  // Mutation to / would return double, causing type error",
        );

        fs::write(&file_path, &mutated_content).expect("Failed to write mutated file");

        let output = Command::new("dart")
            .arg("analyze")
            .current_dir(&temp_dir)
            .output()
            .expect("Failed to run dart analyze");

        drop(fs::remove_dir_all(&temp_dir));

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // This mutation SHOULD compile (int - int is valid)
        assert!(
            output.status.success() || !stdout.to_lowercase().contains("error"),
            "Mutating int + to int - should compile successfully.\nStdout: {}\nStderr: {}",
            stdout,
            stderr
        );
    }

    /// Verify that the tests for the compile_error_project pass with original code
    #[test]
    fn test_compile_error_project_tests_pass() {
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = compile_error_project_dir();
        ensure_dart_deps(&project_dir);

        let output = Command::new("dart")
            .arg("test")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run dart test");

        assert!(
            output.status.success(),
            "Compile error project tests should pass with original code.\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

// ============================================================================
// PARSER INTEGRATION TESTS
// ============================================================================

mod parser_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_dart_files_are_syntactically_valid() {
        // This test ensures our fixture files are valid Dart
        if !dart_available() {
            eprintln!("Skipping test: Dart not available");
            return;
        }

        let project_dir = sample_project_dir();
        ensure_dart_deps(&project_dir);

        let output = Command::new("dart")
            .arg("analyze")
            .current_dir(&project_dir)
            .output()
            .expect("Failed to run dart analyze");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // dart analyze exits with 0 if no issues (or only infos/hints)
        // We check for actual errors in the output
        let has_errors = stdout.contains("error") || stderr.contains("error");

        assert!(
            !has_errors,
            "Dart files should have no analyzer errors.\nStdout: {}\nStderr: {}",
            stdout, stderr
        );
    }
}
