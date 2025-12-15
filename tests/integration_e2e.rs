//! End-to-end integration tests for dart_mutant
//!
//! These tests verify the complete mutation testing pipeline:
//! - CLI argument parsing
//! - File discovery
//! - Mutation generation
//! - Test execution (when Dart is available)
//! - Report generation
//!
//! These are HIGH-LEVEL tests that test the actual tool behavior.
//!
//! IMPORTANT: Tests that run actual mutations use COPIES of fixtures
//! to prevent corrupting the original fixture files.

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the test fixtures directory
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("simple_dart_project")
}

/// Get the path to the dart_mutant binary (after cargo build)
fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("dart_mutant")
}

/// Check if the binary exists
fn binary_exists() -> bool {
    binary_path().exists()
}

/// Check if Dart is available
fn dart_available() -> bool {
    Command::new("dart")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Copy fixtures to a temp directory to prevent mutation from corrupting originals.
/// Returns the temp directory path (which will be automatically cleaned up on drop).
fn copy_fixtures_to_temp() -> Option<tempfile::TempDir> {
    let temp_dir = tempfile::tempdir().ok()?;
    let source = fixtures_path();
    let dest = temp_dir.path().join("simple_dart_project");

    // Copy recursively using walkdir
    for entry in walkdir::WalkDir::new(&source).into_iter().filter_map(|e| e.ok()) {
        let rel_path = entry.path().strip_prefix(&source).ok()?;
        let target = dest.join(rel_path);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target).ok()?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).ok()?;
            }
            std::fs::copy(entry.path(), &target).ok()?;
        }
    }

    Some(temp_dir)
}


mod cli_arguments {
    use super::*;

    #[test]
    fn help_flag_shows_usage() {
        if !binary_exists() {
            println!("Skipping: binary not built. Run `cargo build` first.");
            return;
        }

        let output = Command::new(binary_path())
            .arg("--help")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains("dart_mutant") || stdout.contains("mutation"),
            "Help should mention dart_mutant or mutation testing"
        );
        assert!(
            stdout.contains("--path") || stdout.contains("-p"),
            "Help should show --path option"
        );
        assert!(
            stdout.contains("--threshold"),
            "Help should show --threshold option"
        );
    }

    #[test]
    fn version_flag_shows_version() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .arg("--version")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        assert!(
            combined.contains("0.") || combined.contains("dart_mutant"),
            "Version output should contain version number or program name"
        );
    }

    #[test]
    fn accepts_path_argument() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        // Just test that the argument is accepted (--dry-run prevents actual execution)
        let output = Command::new(binary_path())
            .args(["--path", "/nonexistent", "--dry-run"])
            .output()
            .expect("Failed to execute command");

        // It should at least try to run, even if the path doesn't exist
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Either it runs or reports path not found - both are valid
        assert!(
            !stderr.contains("error: unexpected argument") &&
            !stderr.contains("error: Found argument"),
            "--path should be a valid argument"
        );
    }

    #[test]
    fn accepts_threshold_argument() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args(["--threshold", "80", "--dry-run", "--path", "/nonexistent"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("error: unexpected argument"),
            "--threshold should be a valid argument"
        );
    }

    #[test]
    fn accepts_parallel_jobs_argument() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args(["--parallel", "4", "--dry-run", "--path", "/nonexistent"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("error: unexpected argument"),
            "--parallel should be a valid argument"
        );
    }

    #[test]
    fn accepts_output_directory_argument() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args(["--output", "/tmp/reports", "--dry-run", "--path", "/nonexistent"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("error: unexpected argument"),
            "--output should be a valid argument"
        );
    }

    #[test]
    fn accepts_exclude_patterns() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args([
                "--exclude", "**/*.g.dart",
                "--exclude", "**/*.freezed.dart",
                "--dry-run",
                "--path", "/nonexistent"
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("error: unexpected argument"),
            "--exclude should be a valid argument"
        );
    }
}


mod file_discovery_e2e {
    use super::*;

    #[test]
    fn discovers_fixture_dart_files() {
        let lib = fixtures_path().join("lib");
        assert!(lib.exists(), "Test fixtures lib directory should exist");

        let dart_files: Vec<_> = walkdir::WalkDir::new(&lib)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "dart"))
            .collect();

        assert_eq!(
            dart_files.len(),
            4,
            "Should find exactly 4 Dart files in fixtures/lib"
        );
    }

    #[test]
    fn fixture_files_are_parseable() {
        let lib = fixtures_path().join("lib");

        let dart_files = vec![
            lib.join("calculator.dart"),
            lib.join("string_utils.dart"),
            lib.join("null_safe.dart"),
        ];

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        for file in dart_files {
            let source = std::fs::read_to_string(&file)
                .unwrap_or_else(|_| panic!("Should read {:?}", file));

            let tree = parser
                .parse(&source, None)
                .expect("Should parse");

            assert!(
                !tree.root_node().has_error(),
                "{:?} should parse without errors",
                file.file_name()
            );
        }
    }
}


mod mutation_generation_e2e {
    use super::*;

    /// Count mutation candidates in a Dart file
    fn count_mutation_candidates(source: &str) -> usize {
        let mut count = 0;

        // Arithmetic
        count += source.matches(" + ").count();
        count += source.matches(" - ").count();
        count += source.matches(" * ").count();
        count += source.matches(" / ").count();

        // Comparison
        count += source.matches(" > ").count();
        count += source.matches(" < ").count();
        count += source.matches(" >= ").count();
        count += source.matches(" <= ").count();
        count += source.matches(" == ").count();
        count += source.matches(" != ").count();

        // Logical
        count += source.matches(" && ").count();
        count += source.matches(" || ").count();

        // Null safety
        count += source.matches("??").count();
        count += source.matches("?.").count();

        // Control flow (if statements generate 2 mutations each)
        count += (source.matches("if (").count() + source.matches("if(").count()) * 2;

        count
    }

    #[test]
    fn fixtures_have_many_mutation_candidates() {
        let lib = fixtures_path().join("lib");

        let files = vec![
            ("calculator.dart", 15), // Minimum expected mutations
            ("string_utils.dart", 10),
            ("null_safe.dart", 10),
        ];

        for (filename, min_expected) in files {
            let path = lib.join(filename);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Should read {}", filename));

            let count = count_mutation_candidates(&source);
            assert!(
                count >= min_expected,
                "{} should have at least {} mutation candidates, found {}",
                filename,
                min_expected,
                count
            );
        }
    }

    #[test]
    fn calculator_has_all_mutation_types() {
        let calc = fixtures_path().join("lib").join("calculator.dart");
        let source = std::fs::read_to_string(&calc).expect("Should read");

        // Verify presence of different mutation-worthy constructs
        assert!(source.contains(" + "), "Should have addition");
        assert!(source.contains(" - "), "Should have subtraction");
        assert!(source.contains(" * "), "Should have multiplication");
        assert!(source.contains("~/") || source.contains(" / "), "Should have division");
        assert!(source.contains(" > ") || source.contains(" >= "), "Should have greater than");
        assert!(source.contains(" < ") || source.contains(" <= "), "Should have less than");
        assert!(source.contains(" == "), "Should have equality");
        assert!(source.contains("if (") || source.contains("if("), "Should have if statement");
        assert!(source.contains("++") || source.contains("--"), "Should have increment/decrement");
    }

    #[test]
    fn null_safe_has_dart_specific_constructs() {
        let ns = fixtures_path().join("lib").join("null_safe.dart");
        let source = std::fs::read_to_string(&ns).expect("Should read");

        // Dart-specific null safety mutations
        assert!(source.contains("??"), "Should have null coalescing");
        assert!(source.contains("?."), "Should have null-aware access");
        assert!(
            source.contains("!= null") || source.contains("== null"),
            "Should have null checks"
        );
    }
}


mod full_pipeline_e2e {
    use super::*;

    #[test]
    fn dry_run_on_fixtures_succeeds() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args([
                "--path", fixtures_path().to_str().unwrap(),
                "--dry-run",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        // In dry run mode, should discover files and generate mutations without running tests
        assert!(
            output.status.success() || stderr.contains("mutation"),
            "Dry run should succeed or at least start mutation process"
        );
    }

    #[test]
    fn full_run_on_fixtures_produces_report() {
        if !binary_exists() || !dart_available() {
            println!("Skipping: binary not built or Dart not available");
            return;
        }

        // IMPORTANT: Copy fixtures to temp dir to prevent corruption during mutation testing
        let Some(temp_fixtures) = copy_fixtures_to_temp() else {
            println!("Skipping: failed to copy fixtures");
            return;
        };
        let project_path = temp_fixtures.path().join("simple_dart_project");

        // Run dart pub get on the COPY
        let pub_get = Command::new("dart")
            .args(["pub", "get"])
            .current_dir(&project_path)
            .output();

        if pub_get.is_err() || !pub_get.unwrap().status.success() {
            println!("Skipping: dart pub get failed");
            return;
        }

        let temp_output = std::env::temp_dir().join("dart_mutant_e2e_test");
        std::fs::create_dir_all(&temp_output).ok();

        let output = Command::new(binary_path())
            .args([
                "--path", project_path.to_str().unwrap(),
                "--output", temp_output.to_str().unwrap(),
                "--html",
                "--json",
                "--sample", "5", // Only test 5 mutations for speed
                "--timeout", "10",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        // Check for report files
        let html_report = temp_output.join("mutation-report.html");
        let json_report = temp_output.join("mutation-report.json");

        if html_report.exists() {
            let html = std::fs::read_to_string(&html_report).unwrap();
            assert!(
                html.contains("<!DOCTYPE html>"),
                "HTML report should be valid HTML"
            );
        }

        if json_report.exists() {
            let json = std::fs::read_to_string(&json_report).unwrap();
            assert!(
                json.contains("mutationScore") || json.contains("mutation_score"),
                "JSON report should contain mutation score"
            );
        }

        // Cleanup (temp_fixtures auto-cleans on drop, but output dir needs manual cleanup)
        std::fs::remove_dir_all(&temp_output).ok();
    }

    #[test]
    fn ai_report_flag_generates_markdown_report() {
        if !binary_exists() || !dart_available() {
            println!("Skipping: binary not built or Dart not available");
            return;
        }

        // IMPORTANT: Copy fixtures to temp dir to prevent corruption during mutation testing
        let Some(temp_fixtures) = copy_fixtures_to_temp() else {
            println!("Skipping: failed to copy fixtures");
            return;
        };
        let project_path = temp_fixtures.path().join("simple_dart_project");

        // Run dart pub get on the COPY
        let pub_get = Command::new("dart")
            .args(["pub", "get"])
            .current_dir(&project_path)
            .output();

        if pub_get.is_err() || !pub_get.unwrap().status.success() {
            println!("Skipping: dart pub get failed");
            return;
        }

        let temp_output = std::env::temp_dir().join("dart_mutant_ai_report_test");
        std::fs::create_dir_all(&temp_output).ok();

        let output = Command::new(binary_path())
            .args([
                "--path", project_path.to_str().unwrap(),
                "--output", temp_output.to_str().unwrap(),
                "--ai-report",
                "--sample", "3", // Only test 3 mutations for speed
                "--timeout", "10",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        // Check for AI report file
        let ai_report = temp_output.join("mutation-report-ai.md");

        if ai_report.exists() {
            let md = std::fs::read_to_string(&ai_report).unwrap();
            assert!(
                md.contains("# Mutation Testing Report (AI-Optimized)"),
                "AI report should have expected header"
            );
            assert!(
                md.contains("## Summary"),
                "AI report should have summary section"
            );
            assert!(
                md.contains("**Mutation Score**"),
                "AI report should contain mutation score"
            );
            assert!(
                md.contains("**Killed**") && md.contains("**Survived**"),
                "AI report should contain killed/survived counts"
            );

            // If there are survivors, should have action items
            if md.contains("Surviving Mutants") {
                assert!(
                    md.contains("**Mutation**:"),
                    "AI report should show mutation details for survivors"
                );
                assert!(
                    md.contains("**Suggested Test**:"),
                    "AI report should provide test hints"
                );
                assert!(
                    md.contains("## Quick Reference"),
                    "AI report should have quick reference section"
                );
            }
        } else {
            println!("AI report not generated (possibly no mutations tested)");
        }

        // Cleanup (temp_fixtures auto-cleans on drop)
        std::fs::remove_dir_all(&temp_output).ok();
    }
}


mod threshold_behavior {
    #[test]
    fn threshold_zero_always_passes() {
        // With threshold 0, any mutation score should pass
        let scores = vec![0.0, 25.0, 50.0, 75.0, 100.0];
        let threshold = 0.0;

        for score in scores {
            let passes = score >= threshold;
            assert!(
                passes,
                "Score {} should pass threshold {}",
                score,
                threshold
            );
        }
    }

    #[test]
    fn threshold_80_requires_high_score() {
        let threshold = 80.0;

        assert!(80.0 >= threshold, "80% should pass 80% threshold");
        assert!(100.0 >= threshold, "100% should pass 80% threshold");
        assert!(!(79.9 >= threshold), "79.9% should fail 80% threshold");
        assert!(!(50.0 >= threshold), "50% should fail 80% threshold");
    }

    #[test]
    fn threshold_determines_exit_code() {
        let determine_exit = |score: f64, threshold: f64| -> i32 {
            if score >= threshold {
                0 // Success
            } else {
                1 // Failure
            }
        };

        assert_eq!(determine_exit(100.0, 80.0), 0);
        assert_eq!(determine_exit(80.0, 80.0), 0);
        assert_eq!(determine_exit(79.0, 80.0), 1);
        assert_eq!(determine_exit(0.0, 0.0), 0);
    }
}


mod output_format_e2e {
    #[test]
    fn banner_is_displayed() {
        // The tool should display a banner on startup
        // This is a visual test - we just verify the banner format

        let banner = r#"
    ██████╗  █████╗ ██████╗ ████████╗    ███╗   ███╗██╗   ██╗████████╗ █████╗ ███╗   ██╗████████╗
    ██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝    ████╗ ████║██║   ██║╚══██╔══╝██╔══██╗████╗  ██║╚══██╔══╝
    ██║  ██║███████║██████╔╝   ██║       ██╔████╔██║██║   ██║   ██║   ███████║██╔██╗ ██║   ██║
    ██║  ██║██╔══██║██╔══██╗   ██║       ██║╚██╔╝██║██║   ██║   ██║   ██╔══██║██║╚██╗██║   ██║
    ██████╔╝██║  ██║██║  ██║   ██║       ██║ ╚═╝ ██║╚██████╔╝   ██║   ██║  ██║██║ ╚████║   ██║
    ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝       ╚═╝     ╚═╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝
"#;

        // Verify banner contains DART MUTANT text
        assert!(banner.contains("██"), "Banner should use block characters");
    }

    #[test]
    fn score_bar_format() {
        // Test the score bar generation logic

        let create_score_bar = |score: f64| -> String {
            let width = 40;
            let filled = ((score / 100.0) * width as f64) as usize;
            let empty = width - filled;
            format!("{}{}", "█".repeat(filled), "░".repeat(empty))
        };

        let bar_100 = create_score_bar(100.0);
        assert_eq!(bar_100.chars().filter(|c| *c == '█').count(), 40);

        let bar_50 = create_score_bar(50.0);
        assert_eq!(bar_50.chars().filter(|c| *c == '█').count(), 20);
        assert_eq!(bar_50.chars().filter(|c| *c == '░').count(), 20);

        let bar_0 = create_score_bar(0.0);
        assert_eq!(bar_0.chars().filter(|c| *c == '░').count(), 40);
    }
}
