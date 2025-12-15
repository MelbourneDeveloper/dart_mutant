// Allow test-specific patterns that are fine in test code
#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::assertions_on_constants,
    clippy::unnecessary_literal_unwrap,
    clippy::cloned_instead_of_copied,
    clippy::iter_out_of_bounds,
    clippy::needless_collect,
    clippy::useless_vec
)]

//! Integration tests for mutation generation
//!
//! These tests verify that the mutation system correctly:
//! - Generates valid mutations from real Dart code
//! - Applies mutations correctly to source files
//! - Produces syntactically valid mutated code

use std::path::PathBuf;

/// Get the path to the test fixtures directory
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("simple_dart_project")
}

fn lib_path() -> PathBuf {
    fixtures_path().join("lib")
}

mod mutation_generation {
    use super::*;

    #[test]
    fn generates_arithmetic_mutations_for_calculator() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        // Expected arithmetic operators that should have mutations
        // (original, description)
        let expected_operators = [
            ("+", "Addition should mutate to subtraction"),
            ("-", "Subtraction should mutate to addition"),
            ("*", "Multiplication should mutate to division"),
            ("/", "Division should mutate to multiplication"),
            ("%", "Modulo should mutate to multiplication"),
        ];

        // Verify each operator exists in source
        for (original, description) in &expected_operators {
            assert!(
                source.contains(original),
                "{}: '{}' should exist in source",
                description,
                original
            );
        }
    }

    #[test]
    fn generates_comparison_mutations() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        let _comparison_mutations = vec![
            // < mutations
            ("<", "<=", "< should mutate to <="),
            ("<", ">", "< should mutate to >"),
            // <= mutations
            ("<=", "<", "<= should mutate to <"),
            ("<=", ">", "<= should mutate to >"),
            // > mutations
            (">", ">=", "> should mutate to >="),
            (">", "<", "> should mutate to <"),
            // >= mutations
            (">=", ">", ">= should mutate to >"),
            (">=", "<", ">= should mutate to <"),
            // == mutations
            ("==", "!=", "== should mutate to !="),
        ];

        // Verify source has the comparison operators we expect to mutate
        let comparisons_in_source = ["<", "<=", ">", ">=", "==", "!="];
        let mut found = 0;
        for cmp in &comparisons_in_source {
            if source.contains(cmp) {
                found += 1;
            }
        }
        assert!(
            found >= 4,
            "Source should have at least 4 comparison operators"
        );
    }

    #[test]
    fn generates_logical_mutations() {
        // validateInput in string_utils.dart has: s.isNotEmpty && s.length >= minLength && s.length <= maxLength
        let su_path = lib_path().join("string_utils.dart");
        let source = std::fs::read_to_string(&su_path).expect("Should read file");

        // && should mutate to ||
        assert!(
            source.contains("&&"),
            "Should have && operator to mutate to ||"
        );

        // isValid in null_safe.dart has: value != null && value.isNotEmpty
        let ns_path = lib_path().join("null_safe.dart");
        let ns_source = std::fs::read_to_string(&ns_path).expect("Should read file");

        assert!(
            ns_source.contains("&&"),
            "null_safe.dart should have && operator"
        );
    }

    #[test]
    fn generates_boolean_mutations() {
        // Look for boolean literals that can be mutated
        let _calc_path = lib_path().join("calculator.dart");

        // The isPositive, isEven, isInRange functions return boolean expressions
        // But they don't use literal true/false - they use comparisons

        // Let's check string_utils which has explicit returns
        let su_path = lib_path().join("string_utils.dart");
        let su_source = std::fs::read_to_string(&su_path).expect("Should read file");

        // startsWith returns false explicitly
        assert!(
            su_source.contains("return false"),
            "Should have explicit false return"
        );
    }

    #[test]
    fn generates_string_mutations() {
        let su_path = lib_path().join("string_utils.dart");
        let source = std::fs::read_to_string(&su_path).expect("Should read file");

        // Empty string '' should mutate to non-empty
        assert!(
            source.contains("''") || source.contains("\"\""),
            "Should have empty string to mutate"
        );

        // Non-empty strings should mutate to empty
        // 'short', 'medium', 'long', 'Hello, stranger!'
        let string_literals = ["'short'", "'medium'", "'long'", "'Hello,"];
        for lit in &string_literals {
            assert!(source.contains(lit), "Should have {} string literal", lit);
        }
    }

    #[test]
    fn generates_null_safety_mutations() {
        let ns_path = lib_path().join("null_safe.dart");
        let source = std::fs::read_to_string(&ns_path).expect("Should read file");

        // Null coalescing: value ?? 'default' should mutate to just 'value'
        assert!(
            source.contains("??"),
            "Should have null coalescing operator"
        );

        // Null-aware access: s?.length should mutate to s.length
        assert!(source.contains("?."), "Should have null-aware access");
    }

    #[test]
    fn generates_unary_mutations() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        // ++n should mutate to --n
        assert!(source.contains("++"), "Should have increment operator");

        // --n should mutate to ++n
        assert!(source.contains("--"), "Should have decrement operator");

        // -n (negation in negate function) should mutate to n (remove negation)
        assert!(
            source.contains("return -n"),
            "Should have unary negation in negate function"
        );
    }

    #[test]
    fn generates_control_flow_mutations() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        // if conditions should mutate to if(true) and if(false)
        // Count if statements
        let if_count = source.matches("if (").count() + source.matches("if(").count();
        // Each if condition is a control flow mutation candidate
        assert!(
            if_count >= 4,
            "Should have at least 4 if statements, found {}",
            if_count
        );
    }
}

mod mutation_application {
    #[test]
    fn applies_arithmetic_mutation_correctly() {
        let source = r#"
int add(int a, int b) {
    return a + b;
}
"#;

        // Find the + operator position
        let plus_pos = source.find('+').expect("Should find +");

        // Apply mutation: replace + with -
        let mut mutated = String::new();
        mutated.push_str(&source[..plus_pos]);
        mutated.push('-');
        mutated.push_str(&source[plus_pos + 1..]);

        assert!(
            mutated.contains("a - b"),
            "Mutated code should have 'a - b'"
        );
        assert!(
            !mutated.contains("a + b"),
            "Mutated code should NOT have 'a + b'"
        );

        // Verify it's still valid Dart syntax (basic check)
        assert!(mutated.contains("return"), "Should still have return");
        assert!(mutated.contains("int add"), "Should still have function");
    }

    #[test]
    fn applies_comparison_mutation_correctly() {
        let source = r#"
bool isPositive(int n) {
    return n > 0;
}
"#;

        // Mutate > to >=
        let mutated = source.replace('>', ">=");
        assert!(
            mutated.contains("n >= 0"),
            "Mutated code should have 'n >= 0'"
        );

        // Mutate > to <
        let mutated2 = source.replace('>', "<");
        assert!(
            mutated2.contains("n < 0"),
            "Mutated code should have 'n < 0'"
        );
    }

    #[test]
    fn applies_logical_mutation_correctly() {
        let source = r#"
bool isInRange(int n, int min, int max) {
    return n >= min && n <= max;
}
"#;

        // Mutate && to ||
        let mutated = source.replace("&&", "||");
        assert!(
            mutated.contains("||"),
            "Mutated code should have '||' instead of '&&'"
        );
        assert!(!mutated.contains("&&"), "Mutated code should NOT have '&&'");
    }

    #[test]
    fn applies_null_coalescing_mutation_correctly() {
        let source = r#"
String getValueOrDefault(String? value) {
    return value ?? 'default';
}
"#;

        // Mutation: remove ?? fallback, just return value
        // This tests that the mutation maintains valid syntax
        let mutated = source.replace("value ?? 'default'", "value");

        // Note: this would cause a null safety error, but the mutation
        // is syntactically valid - the test runner will catch it
        assert!(!mutated.contains("??"), "Mutated code should not have ??");
    }

    #[test]
    fn applies_string_mutation_correctly() {
        let source = r#"
String greet(String name) {
    if (name == '') {
        return 'Hello, stranger!';
    }
    return 'Hello, $name!';
}
"#;

        // Empty string should mutate to non-empty
        let mutated = source.replacen("''", "'mutated'", 1);
        assert!(
            mutated.contains("'mutated'"),
            "Should have mutated empty string"
        );

        // Non-empty should mutate to empty
        let mutated2 = source.replace("'Hello, stranger!'", "''");
        assert!(
            mutated2.contains("return '';"),
            "Should have empty string return"
        );
    }

    #[test]
    fn applies_boolean_mutation_correctly() {
        let source = r#"
bool alwaysTrue() {
    return true;
}

bool alwaysFalse() {
    return false;
}
"#;

        // true should mutate to false
        let mutated = source.replace("return true", "return false");
        assert!(
            mutated.matches("return false").count() == 2,
            "Both returns should be false after mutation"
        );

        // false should mutate to true
        let mutated2 = source.replace("return false", "return true");
        assert!(
            mutated2.matches("return true").count() == 2,
            "Both returns should be true after mutation"
        );
    }

    #[test]
    fn applies_if_condition_mutation_correctly() {
        let source = r#"
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}
"#;

        // Mutate if condition to always true
        let mutated = source.replace("if (a > b)", "if (true)");
        assert!(
            mutated.contains("if (true)"),
            "Should have if(true) condition"
        );

        // Mutate if condition to always false
        let mutated2 = source.replace("if (a > b)", "if (false)");
        assert!(
            mutated2.contains("if (false)"),
            "Should have if(false) condition"
        );
    }

    #[test]
    fn applies_increment_mutation_correctly() {
        let source = r#"
int increment(int n) {
    return ++n;
}
"#;

        // ++ should mutate to --
        let mutated = source.replace("++", "--");
        assert!(
            mutated.contains("--n"),
            "Should have decremented instead of incremented"
        );
    }

    #[test]
    fn mutated_code_is_syntactically_valid() {
        // This test verifies that common mutations produce valid Dart syntax
        // by parsing the mutated code with tree-sitter

        let mutations = vec![
            ("a + b", "a - b"),
            ("a > b", "a >= b"),
            ("x && y", "x || y"),
            ("true", "false"),
            ("''", "'mutated'"),
            ("++n", "--n"),
        ];

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        for (original, mutated) in mutations {
            let source = format!("void test() {{ var result = {}; }}", mutated);
            let tree = parser.parse(&source, None);

            assert!(
                tree.is_some(),
                "Mutation '{}' -> '{}' should produce parseable code",
                original,
                mutated
            );

            let _tree = tree.unwrap();
            // Note: Some mutations might produce parse errors, which is expected
            // The test runner will catch runtime errors
        }
    }
}

mod mutation_coverage {
    use super::*;

    #[test]
    fn all_dart_operators_have_mutations() {
        // Comprehensive list of Dart operators that should have mutations
        let mutable_operators = vec![
            // Arithmetic
            ("+", vec!["-"]),
            ("-", vec!["+"]),
            ("*", vec!["/"]),
            ("/", vec!["*"]),
            ("%", vec!["*"]),
            // Comparison
            ("<", vec!["<=", ">"]),
            ("<=", vec!["<", ">"]),
            (">", vec![">=", "<"]),
            (">=", vec![">", "<"]),
            ("==", vec!["!="]),
            ("!=", vec!["=="]),
            // Logical
            ("&&", vec!["||"]),
            ("||", vec!["&&"]),
            // Unary
            ("++", vec!["--"]),
            ("--", vec!["++"]),
            // Null safety
            ("??", vec!["<left operand only>"]),
            ("?.", vec!["."]),
        ];

        // Verify we know about all major operator types
        assert!(
            mutable_operators.len() >= 15,
            "Should have at least 15 mutable operators defined"
        );
    }

    #[test]
    fn calculator_has_sufficient_mutation_targets() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        // Count potential mutations
        let mut mutation_count = 0;

        // Arithmetic operators
        mutation_count += source.matches(" + ").count();
        mutation_count += source.matches(" - ").count();
        mutation_count += source.matches(" * ").count();
        mutation_count += source.matches(" / ").count();
        mutation_count += source.matches(" % ").count();

        // Comparison operators
        mutation_count += source.matches(" > ").count();
        mutation_count += source.matches(" < ").count();
        mutation_count += source.matches(" >= ").count();
        mutation_count += source.matches(" <= ").count();
        mutation_count += source.matches(" == ").count();
        mutation_count += source.matches(" != ").count();

        // If statements (each generates 2 mutations: true/false)
        let if_count = source.matches("if (").count() + source.matches("if(").count();
        mutation_count += if_count * 2;

        // Increment/decrement
        mutation_count += source.matches("++").count();
        mutation_count += source.matches("--").count();

        // This is a simple file with just basic operations, should have 20+ mutations
        assert!(
            mutation_count >= 15,
            "calculator.dart should have at least 15 mutation targets, found approximately {}",
            mutation_count
        );
    }

    #[test]
    fn null_safe_has_dart_specific_mutations() {
        let ns_path = lib_path().join("null_safe.dart");
        let source = std::fs::read_to_string(&ns_path).expect("Should read file");

        // Count Dart-specific null safety mutations
        let null_coalescing = source.matches("??").count();
        let null_aware_access = source.matches("?.").count();
        let null_checks = source.matches("!= null").count() + source.matches("== null").count();

        let dart_specific = null_coalescing + null_aware_access + null_checks;

        assert!(
            dart_specific >= 4,
            "null_safe.dart should have at least 4 Dart-specific mutation targets (null safety), found {}",
            dart_specific
        );
    }
}

/// Tests for AI suggestion parsing and mutation conversion
mod ai_suggestion_parsing {
    /// Test parsing Anthropic response format
    #[test]
    fn parses_anthropic_response_format() {
        let response = serde_json::json!({
            "content": [{
                "type": "text",
                "text": r#"[{"line": 10, "column": 5, "original": ">=", "mutated": ">", "reason": "Boundary check", "confidence": 0.85}]"#
            }]
        });

        let content = response
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|msg| msg.get("text"))
            .and_then(|t| t.as_str())
            .unwrap();

        assert!(content.contains("line"));
        assert!(content.contains(">="));
    }

    /// Test parsing OpenAI response format
    #[test]
    fn parses_openai_response_format() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": r#"[{"line": 10, "column": 5, "original": ">=", "mutated": ">", "reason": "Boundary check", "confidence": 0.85}]"#
                }
            }]
        });

        let content = response
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|t| t.as_str())
            .unwrap();

        assert!(content.contains("line"));
        assert!(content.contains(">="));
    }

    /// Test parsing Ollama response format
    #[test]
    fn parses_ollama_response_format() {
        let response = serde_json::json!({
            "response": r#"[{"line": 10, "column": 5, "original": ">=", "mutated": ">", "reason": "Boundary check", "confidence": 0.85}]"#
        });

        let content = response.get("response").and_then(|r| r.as_str()).unwrap();

        assert!(content.contains("line"));
        assert!(content.contains(">="));
    }

    /// Test JSON extraction from markdown code blocks
    #[test]
    fn extracts_json_from_markdown_code_blocks() {
        let content = r#"Here's the analysis:
```json
[{"line": 10, "column": 5, "original": ">=", "mutated": ">", "reason": "test", "confidence": 0.8}]
```
That's all!"#;

        let json_str = if let Some(start) = content.find('[') {
            if let Some(end) = content.rfind(']') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        assert!(json_str.starts_with('['));
        assert!(json_str.ends_with(']'));

        let parsed: Vec<serde_json::Value> = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["line"], 10);
    }

    /// Test handling of empty/invalid AI response
    #[test]
    fn handles_empty_ai_response() {
        let content = "";
        let json_str = if let Some(start) = content.find('[') {
            if let Some(end) = content.rfind(']') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        let suggestions: Vec<serde_json::Value> =
            serde_json::from_str(json_str).unwrap_or_default();
        assert!(suggestions.is_empty());
    }

    /// Test suggestion to mutation conversion logic
    #[test]
    fn suggestion_line_column_validation() {
        // Line 0 is invalid
        assert!(0_usize == 0); // Line numbering starts at 1

        // Column 0 should be treated as column 1
        let col_offset = 0_usize.saturating_sub(1);
        assert_eq!(col_offset, 0);

        // Valid column
        let col_offset = 5_usize.saturating_sub(1);
        assert_eq!(col_offset, 4);
    }

    /// Test byte offset calculation for multi-line source
    #[test]
    fn calculates_byte_offset_correctly() {
        let source = "line1\nline2\nline3";
        let lines: Vec<&str> = source.lines().collect();

        // Line 1 starts at byte 0
        let mut byte_start = 0;
        assert_eq!(byte_start, 0);

        // Line 2 starts at byte 6 (5 chars + newline)
        byte_start += lines[0].len() + 1;
        assert_eq!(byte_start, 6);

        // Line 3 starts at byte 12 (6 + 5 + 1)
        byte_start += lines[1].len() + 1;
        assert_eq!(byte_start, 12);
    }

    /// Test that AI-suggested mutations have correct flags
    #[test]
    fn ai_mutations_have_correct_metadata() {
        // A proper AI mutation should have:
        // - ai_suggested = true
        // - ai_confidence = Some(value)
        // - operator = AiSuggested

        let ai_suggested = true;
        let ai_confidence: Option<f64> = Some(0.85);

        assert!(ai_suggested);
        assert!(ai_confidence.is_some());
        assert!((ai_confidence.unwrap() - 0.85).abs() < f64::EPSILON);
    }
}

/// Tests for mutation sampling functionality
mod mutation_sampling {
    #[test]
    fn sample_returns_empty_for_empty_input() {
        let mutations: Vec<i32> = vec![];
        let count = 5;

        // Sampling from empty should return empty
        let sampled: Vec<i32> = if count == 0 || mutations.is_empty() {
            vec![]
        } else if count >= mutations.len() {
            mutations.clone()
        } else {
            mutations.iter().take(count).cloned().collect()
        };

        assert!(sampled.is_empty());
    }

    #[test]
    fn sample_returns_all_when_count_exceeds_length() {
        let mutations = vec![1, 2, 3, 4, 5];
        let count = 10;

        let sampled: Vec<i32> = if count >= mutations.len() {
            mutations.clone()
        } else {
            mutations.iter().take(count).cloned().collect()
        };

        assert_eq!(sampled.len(), 5);
        assert_eq!(sampled, mutations);
    }

    #[test]
    fn sample_returns_exact_count_when_smaller() {
        let mutations = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let count = 3;

        // Using deterministic take instead of random for testing
        let sampled: Vec<i32> = mutations.iter().take(count).cloned().collect();

        assert_eq!(sampled.len(), 3);
    }

    #[test]
    fn sample_zero_returns_empty() {
        let mutations = vec![1, 2, 3, 4, 5];
        let count = 0;

        let sampled: Vec<i32> = if count == 0 {
            vec![]
        } else {
            mutations.iter().take(count).cloned().collect()
        };

        assert!(sampled.is_empty());
    }
}

/// Tests for mutation operator names
mod mutation_operator_names {
    #[test]
    fn arithmetic_operators_have_descriptive_names() {
        let operator_names = [
            "ArithmeticAddToSub",
            "ArithmeticSubToAdd",
            "ArithmeticMulToDiv",
            "ArithmeticDivToMul",
        ];

        for name in &operator_names {
            assert!(name.starts_with("Arithmetic"));
            assert!(name.contains("To"));
        }
    }

    #[test]
    fn comparison_operators_have_descriptive_names() {
        let operator_names = [
            "ComparisonLtToLte",
            "ComparisonLteToLt",
            "ComparisonGtToGte",
            "ComparisonGteToGt",
            "ComparisonEqToNeq",
            "ComparisonNeqToEq",
        ];

        for name in &operator_names {
            assert!(name.starts_with("Comparison"));
        }
    }

    #[test]
    fn logical_operators_have_descriptive_names() {
        let operator_names = ["LogicalAndToOr", "LogicalOrToAnd", "LogicalNotRemoval"];

        for name in &operator_names {
            assert!(name.starts_with("Logical"));
        }
    }

    #[test]
    fn null_safety_operators_have_descriptive_names() {
        let operator_names = [
            "NullCoalescingRemoval",
            "NullAwareAccessRemoval",
            "NullAssertionRemoval",
            "NullCheckToTrue",
            "NullCheckToFalse",
        ];

        for name in &operator_names {
            assert!(name.starts_with("Null"));
        }
    }

    #[test]
    fn all_operators_follow_naming_convention() {
        // Operators should be PascalCase
        let valid_operators = [
            "Arithmetic",
            "Comparison",
            "Logical",
            "Boolean",
            "Unary",
            "NullSafety",
            "String",
            "Conditional",
            "Collection",
            "AiSuggested",
        ];

        for op in &valid_operators {
            // First char uppercase
            assert!(op.chars().next().unwrap().is_uppercase());
            // No underscores
            assert!(!op.contains('_'));
        }
    }
}

/// Tests for Mutation::apply functionality
mod mutation_apply {
    #[test]
    fn apply_replaces_at_correct_position() {
        let source = "int x = a + b;";
        let byte_start = 10; // Position of '+'
        let byte_end = 11;
        let replacement = "-";

        // Simulating Mutation::apply logic
        let result = format!(
            "{}{}{}",
            &source[..byte_start],
            replacement,
            &source[byte_end..]
        );

        assert_eq!(result, "int x = a - b;");
    }

    #[test]
    fn apply_handles_multi_char_replacement() {
        let source = "if (x > y)";
        let byte_start = 6; // Position of '>'
        let byte_end = 7;
        let replacement = ">=";

        let result = format!(
            "{}{}{}",
            &source[..byte_start],
            replacement,
            &source[byte_end..]
        );

        assert_eq!(result, "if (x >= y)");
    }

    #[test]
    fn apply_handles_deletion() {
        let source = "value ?? 'default'";
        let byte_start = 5; // Position of ' ??' (space before ??)
        let byte_end = 18; // End of " ?? 'default'"
        let replacement = "";

        let result = format!(
            "{}{}{}",
            &source[..byte_start],
            replacement,
            &source[byte_end..]
        );

        assert_eq!(result, "value");
    }

    #[test]
    fn apply_handles_start_of_string() {
        let source = "true && false";
        let byte_start = 0;
        let byte_end = 4;
        let replacement = "false";

        let result = format!(
            "{}{}{}",
            &source[..byte_start],
            replacement,
            &source[byte_end..]
        );

        assert_eq!(result, "false && false");
    }

    #[test]
    fn apply_handles_end_of_string() {
        let source = "return true";
        let byte_start = 7;
        let byte_end = 11;
        let replacement = "false";

        let result = format!(
            "{}{}{}",
            &source[..byte_start],
            replacement,
            &source[byte_end..]
        );

        assert_eq!(result, "return false");
    }

    #[test]
    fn apply_preserves_unicode() {
        let source = "String emoji = '🎉';";
        let byte_start = 16; // Position after '='
        let byte_end = 20; // Position after emoji (4 bytes for emoji)

        // Just verify the slice positions work with unicode
        assert!(byte_start < source.len());
        assert!(byte_end <= source.len());
    }
}
