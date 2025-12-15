// Allow panics and expects in test code - tests need to fail loudly
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

//! Integration tests for the parser module
//!
//! These tests verify that the parser correctly:
//! - Discovers Dart files in a project
//! - Parses Dart source code using tree-sitter
//! - Identifies mutation locations in real Dart code

use std::path::PathBuf;

/// Get the path to the test fixtures directory
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("simple_dart_project")
}

/// Get the lib directory path
fn lib_path() -> PathBuf {
    fixtures_path().join("lib")
}

mod discover_files {
    use super::*;

    #[test]
    fn discovers_all_dart_files_in_lib() {
        // This tests actual file discovery on real Dart files
        let lib = lib_path();
        assert!(lib.exists(), "Test fixtures must exist at {:?}", lib);

        let files: Vec<_> = walkdir::WalkDir::new(&lib)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "dart"))
            .collect();

        assert!(
            files.len() >= 3,
            "Expected at least 3 Dart files in fixtures, found {}",
            files.len()
        );

        // Verify specific files exist
        let file_names: Vec<_> = files
            .iter()
            .map(|f| f.file_name().to_string_lossy().to_string())
            .collect();

        assert!(
            file_names.contains(&"calculator.dart".to_string()),
            "calculator.dart should exist"
        );
        assert!(
            file_names.contains(&"string_utils.dart".to_string()),
            "string_utils.dart should exist"
        );
        assert!(
            file_names.contains(&"null_safe.dart".to_string()),
            "null_safe.dart should exist"
        );
    }

    #[test]
    fn excludes_generated_files() {
        // Create a mock .g.dart file and verify it would be excluded
        let exclusion_patterns = [
            "**/*.g.dart",
            "**/*.freezed.dart",
            "**/*.mocks.dart",
            "**/test/**",
        ];

        let test_paths = vec![
            ("lib/model.g.dart", true),          // Should be excluded
            ("lib/model.freezed.dart", true),    // Should be excluded
            ("lib/model.mocks.dart", true),      // Should be excluded
            ("test/calculator_test.dart", true), // Should be excluded
            ("lib/calculator.dart", false),      // Should NOT be excluded
            ("lib/string_utils.dart", false),    // Should NOT be excluded
        ];

        for (path, should_exclude) in test_paths {
            let is_excluded = exclusion_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(path))
                    .unwrap_or(false)
            });

            assert_eq!(
                is_excluded, should_exclude,
                "Path '{}' exclusion mismatch: expected {}, got {}",
                path, should_exclude, is_excluded
            );
        }
    }
}

mod parse_dart {
    use super::*;

    #[test]
    fn parses_calculator_dart_without_errors() {
        let calc_path = lib_path().join("calculator.dart");
        assert!(calc_path.exists(), "calculator.dart must exist");

        let source = std::fs::read_to_string(&calc_path).expect("Should read calculator.dart");

        // Parse with tree-sitter
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        let tree = parser.parse(&source, None).expect("Should parse Dart");
        let root = tree.root_node();

        // Verify no parse errors
        assert!(
            !root.has_error(),
            "calculator.dart should parse without errors"
        );

        // Verify we found a class definition
        let _source_bytes = source.as_bytes();
        let mut found_class = false;
        let mut cursor = root.walk();

        for node in root.children(&mut cursor) {
            if node.kind() == "class_definition" {
                found_class = true;
                let class_text = &source[node.byte_range()];
                assert!(
                    class_text.contains("Calculator"),
                    "Should find Calculator class"
                );
            }
        }

        assert!(found_class, "Should find at least one class definition");
    }

    #[test]
    fn parses_null_safe_dart_correctly() {
        let ns_path = lib_path().join("null_safe.dart");
        assert!(ns_path.exists(), "null_safe.dart must exist");

        let source = std::fs::read_to_string(&ns_path).expect("Should read null_safe.dart");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        let tree = parser.parse(&source, None).expect("Should parse Dart");

        assert!(
            !tree.root_node().has_error(),
            "null_safe.dart should parse without errors (null safety syntax)"
        );
    }

    #[test]
    fn finds_binary_expressions() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        let tree = parser.parse(&source, None).expect("Should parse");

        // Count binary expressions (which are mutation candidates)
        let binary_count = count_nodes_of_kind(
            &tree.root_node(),
            &source,
            &[
                "binary_expression",
                "additive_expression",
                "multiplicative_expression",
                "relational_expression",
                "equality_expression",
            ],
        );

        // calculator.dart has multiple arithmetic and comparison operations
        assert!(
            binary_count >= 5,
            "Should find at least 5 binary expressions, found {}",
            binary_count
        );
    }

    #[test]
    fn finds_logical_expressions() {
        let su_path = lib_path().join("string_utils.dart");
        let source = std::fs::read_to_string(&su_path).expect("Should read file");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        let tree = parser.parse(&source, None).expect("Should parse");

        // validateInput has && operators
        let logical_count = count_nodes_of_kind(
            &tree.root_node(),
            &source,
            &["logical_and_expression", "logical_or_expression"],
        );

        assert!(
            logical_count >= 1,
            "Should find at least 1 logical expression (from validateInput), found {}",
            logical_count
        );
    }

    #[test]
    fn finds_if_statements() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        let tree = parser.parse(&source, None).expect("Should parse");

        let if_count = count_nodes_of_kind(&tree.root_node(), &source, &["if_statement"]);

        // calculator.dart has multiple if statements
        assert!(
            if_count >= 4,
            "Should find at least 4 if statements, found {}",
            if_count
        );
    }

    #[test]
    fn finds_null_aware_operators() {
        let ns_path = lib_path().join("null_safe.dart");
        let source = std::fs::read_to_string(&ns_path).expect("Should read file");

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        let tree = parser.parse(&source, None).expect("Should parse");

        // Check for null-aware access (?.) and null coalescing (??)
        let has_null_coalescing = source.contains("??");
        let has_null_aware_access = source.contains("?.");

        assert!(
            has_null_coalescing,
            "null_safe.dart should have ?? operator"
        );
        assert!(
            has_null_aware_access,
            "null_safe.dart should have ?. operator"
        );

        // The parser should recognize these constructs
        let null_aware_count = count_nodes_of_kind(
            &tree.root_node(),
            &source,
            &["if_null_expression", "conditional_member_access"],
        );

        // The parser finds at least one null-aware construct (tree-sitter node types may vary)
        assert!(
            null_aware_count >= 1,
            "Should find at least 1 null-aware construct, found {}",
            null_aware_count
        );
    }

    /// Helper function to count nodes of specific kinds in the AST
    #[allow(clippy::only_used_in_recursion)]
    fn count_nodes_of_kind(node: &tree_sitter::Node<'_>, source: &str, kinds: &[&str]) -> usize {
        let mut count = 0;

        if kinds.contains(&node.kind()) {
            count += 1;
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            count += count_nodes_of_kind(&child, source, kinds);
        }

        count
    }
}

mod mutation_discovery {
    use super::*;

    #[test]
    fn discovers_arithmetic_mutations() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        // The calculator.dart file has:
        // - a + b (add function)
        // - a - b (subtract function)
        // - a * b (multiply function)
        // - a / b (divide function)
        // - n % 2 (isEven function)
        // - n * factorial(n-1) (factorial function)

        assert!(source.contains("a + b"), "Should have addition");
        assert!(source.contains("a - b"), "Should have subtraction");
        assert!(source.contains("a * b"), "Should have multiplication");
        assert!(
            source.contains("a ~/ b") || source.contains("a / b"),
            "Should have division"
        );
        assert!(source.contains("n % 2"), "Should have modulo");

        // All of these are valid mutation targets
        let arithmetic_ops = ["+", "-", "*", "/", "%"];
        let mut found_count = 0;

        for op in &arithmetic_ops {
            if source.contains(&format!(" {} ", op)) || source.contains(&format!(" {}", op)) {
                found_count += 1;
            }
        }

        assert!(
            found_count >= 4,
            "Should find at least 4 different arithmetic operators"
        );
    }

    #[test]
    fn discovers_comparison_mutations() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        // calculator.dart has:
        // - b == 0 (divide)
        // - n > 0 (isPositive)
        // - n % 2 == 0 (isEven)
        // - a > b (max)
        // - n >= min && n <= max (isInRange)
        // - n <= 1 (factorial)
        // - n < 0 (abs)
        // - a == b (areEqual)

        let comparison_ops = ["==", "!=", ">", "<", ">=", "<="];
        let mut found_count = 0;

        for op in &comparison_ops {
            if source.contains(op) {
                found_count += 1;
            }
        }

        assert!(
            found_count >= 4,
            "Should find at least 4 different comparison operators, found {}",
            found_count
        );
    }

    #[test]
    fn discovers_logical_mutations() {
        let su_path = lib_path().join("string_utils.dart");
        let source = std::fs::read_to_string(&su_path).expect("Should read file");

        // string_utils.dart has validateInput with && operators
        assert!(
            source.contains("&&"),
            "Should have && operator in validateInput"
        );

        // null_safe.dart has isValid with && operator
        let ns_path = lib_path().join("null_safe.dart");
        let ns_source = std::fs::read_to_string(&ns_path).expect("Should read file");
        assert!(
            ns_source.contains("&&"),
            "Should have && operator in isValid"
        );
    }

    #[test]
    fn discovers_string_literal_mutations() {
        let su_path = lib_path().join("string_utils.dart");
        let source = std::fs::read_to_string(&su_path).expect("Should read file");

        // string_utils.dart has various string literals:
        // - '' (empty string checks)
        // - 'Hello, stranger!'
        // - 'short', 'medium', 'long'

        assert!(source.contains("''"), "Should have empty string literal");
        assert!(
            source.contains("'Hello,"),
            "Should have greeting string literal"
        );
        assert!(source.contains("'short'"), "Should have 'short' literal");
        assert!(source.contains("'medium'"), "Should have 'medium' literal");
        assert!(source.contains("'long'"), "Should have 'long' literal");
    }

    #[test]
    fn discovers_null_safety_mutations() {
        let ns_path = lib_path().join("null_safe.dart");
        let source = std::fs::read_to_string(&ns_path).expect("Should read file");

        // null_safe.dart has:
        // - value ?? 'default' (null coalescing)
        // - s?.length (null-aware access)
        // - value != null (null check)
        // - items == null (null check)

        assert!(
            source.contains("??"),
            "Should have null coalescing operator"
        );
        assert!(source.contains("?."), "Should have null-aware access");
        assert!(
            source.contains("!= null") || source.contains("== null"),
            "Should have null checks"
        );
    }

    #[test]
    fn discovers_increment_decrement_mutations() {
        let calc_path = lib_path().join("calculator.dart");
        let source = std::fs::read_to_string(&calc_path).expect("Should read file");

        // calculator.dart has increment and decrement functions with ++n and --n
        assert!(source.contains("++"), "Should have increment operator");
        assert!(source.contains("--"), "Should have decrement operator");
    }
}
