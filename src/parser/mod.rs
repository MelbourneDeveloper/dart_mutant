//! Dart parser using tree-sitter for AST-based mutation discovery
//!
//! This module parses Dart source files and identifies locations where
//! mutations can be applied safely and meaningfully.

use crate::mutation::{Mutation, MutationOperator};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser, Tree};
use walkdir::WalkDir;

/// Discover all Dart files in the given path, excluding specified patterns
pub fn discover_dart_files(path: &Path, exclude_patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();

        // Only include .dart files
        if file_path.extension().map_or(false, |ext| ext == "dart") {
            let path_str = file_path.to_string_lossy();

            // Check exclusion patterns
            let excluded = exclude_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
            });

            if !excluded {
                // Skip generated files by convention
                let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
                if !filename.ends_with(".g.dart")
                    && !filename.ends_with(".freezed.dart")
                    && !filename.ends_with(".mocks.dart")
                {
                    files.push(file_path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

/// Parse a Dart file and find all possible mutation locations
pub fn parse_and_find_mutations(file_path: &Path) -> Result<Vec<Mutation>> {
    let source = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let tree = parse_dart(&source)?;
    let mut mutations = Vec::new();

    find_mutations_in_tree(&tree, &source, file_path, &mut mutations);

    Ok(mutations)
}

/// Parse Dart source code into a tree-sitter AST
fn parse_dart(source: &str) -> Result<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_dart::language())
        .context("Failed to load Dart grammar")?;

    parser
        .parse(source, None)
        .context("Failed to parse Dart source")
}

/// Recursively walk the AST and find mutation candidates
fn find_mutations_in_tree(
    tree: &Tree,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    let root = tree.root_node();
    find_mutations_in_node(root, source, file_path, mutations);
}

fn find_mutations_in_node(
    node: Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    let node_kind = node.kind();

    // Match different node types for mutation opportunities
    match node_kind {
        // Binary expressions: arithmetic, comparison, logical
        "binary_expression" | "multiplicative_expression" | "additive_expression" => {
            find_binary_mutations(&node, source, file_path, mutations);
        }

        "relational_expression" | "equality_expression" => {
            find_comparison_mutations(&node, source, file_path, mutations);
        }

        "logical_and_expression" | "logical_or_expression" => {
            find_logical_mutations(&node, source, file_path, mutations);
        }

        // Unary expressions: !, -, ++, --
        "unary_expression" | "prefix_expression" | "postfix_expression" => {
            find_unary_mutations(&node, source, file_path, mutations);
        }

        // Boolean literals
        "true" | "false" => {
            mutations.push(create_boolean_mutation(&node, source, file_path));
        }

        // Null-aware operators
        "if_null_expression" => {
            find_null_coalescing_mutation(&node, source, file_path, mutations);
        }

        "conditional_member_access" => {
            find_null_aware_access_mutation(&node, source, file_path, mutations);
        }

        // If statements
        "if_statement" => {
            find_if_statement_mutations(&node, source, file_path, mutations);
        }

        // String literals
        "string_literal" => {
            find_string_mutation(&node, source, file_path, mutations);
        }

        _ => {}
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_mutations_in_node(child, source, file_path, mutations);
    }
}

fn get_node_text<'a>(node: &Node<'_>, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or_default()
}

fn find_binary_mutations(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    // Look for operator in children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let text = get_node_text(&child, source);

        let replacements: Vec<(&str, MutationOperator)> = match text {
            "+" => vec![("-", MutationOperator::ArithmeticAddToSub)],
            "-" => vec![("+", MutationOperator::ArithmeticSubToAdd)],
            "*" => vec![("/", MutationOperator::ArithmeticMulToDiv)],
            "/" => vec![("*", MutationOperator::ArithmeticDivToMul)],
            "%" => vec![("*", MutationOperator::ArithmeticModToMul)],
            _ => continue,
        };

        for (replacement, operator) in replacements {
            mutations.push(Mutation::new(
                file_path.to_path_buf(),
                child.start_byte(),
                child.end_byte(),
                child.start_position().row + 1,
                child.start_position().column + 1,
                text.to_owned(),
                replacement.to_owned(),
                operator,
            ));
        }
    }
}

fn find_comparison_mutations(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let text = get_node_text(&child, source);

        let replacements: Vec<(&str, MutationOperator)> = match text {
            "<" => vec![
                ("<=", MutationOperator::ComparisonLtToLte),
                (">", MutationOperator::ComparisonLtToGt),
            ],
            "<=" => vec![
                ("<", MutationOperator::ComparisonLteToLt),
                (">", MutationOperator::ComparisonLteToGt),
            ],
            ">" => vec![
                (">=", MutationOperator::ComparisonGtToGte),
                ("<", MutationOperator::ComparisonGtToLt),
            ],
            ">=" => vec![
                (">", MutationOperator::ComparisonGteToGt),
                ("<", MutationOperator::ComparisonGteToLt),
            ],
            "==" => vec![("!=", MutationOperator::ComparisonEqToNeq)],
            "!=" => vec![("==", MutationOperator::ComparisonNeqToEq)],
            _ => continue,
        };

        for (replacement, operator) in replacements {
            mutations.push(Mutation::new(
                file_path.to_path_buf(),
                child.start_byte(),
                child.end_byte(),
                child.start_position().row + 1,
                child.start_position().column + 1,
                text.to_owned(),
                replacement.to_owned(),
                operator,
            ));
        }
    }
}

fn find_logical_mutations(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let text = get_node_text(&child, source);

        let (replacement, operator) = match text {
            "&&" => ("||", MutationOperator::LogicalAndToOr),
            "||" => ("&&", MutationOperator::LogicalOrToAnd),
            _ => continue,
        };

        mutations.push(Mutation::new(
            file_path.to_path_buf(),
            child.start_byte(),
            child.end_byte(),
            child.start_position().row + 1,
            child.start_position().column + 1,
            text.to_owned(),
            replacement.to_owned(),
            operator,
        ));
    }
}

fn find_unary_mutations(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    let text = get_node_text(node, source);

    // Remove negation operator
    if let Some(replacement) = text.strip_prefix('!') {
        if !replacement.is_empty() {
            mutations.push(Mutation::new(
                file_path.to_path_buf(),
                node.start_byte(),
                node.end_byte(),
                node.start_position().row + 1,
                node.start_position().column + 1,
                text.to_owned(),
                replacement.to_owned(),
                MutationOperator::LogicalNotRemoval,
            ));
        }
    }

    // Swap increment/decrement
    if text.starts_with("++") || text.ends_with("++") {
        let replacement = text.replace("++", "--");
        mutations.push(Mutation::new(
            file_path.to_path_buf(),
            node.start_byte(),
            node.end_byte(),
            node.start_position().row + 1,
            node.start_position().column + 1,
            text.to_owned(),
            replacement,
            MutationOperator::UnaryIncrementToDecrement,
        ));
    } else if text.starts_with("--") || text.ends_with("--") {
        let replacement = text.replace("--", "++");
        mutations.push(Mutation::new(
            file_path.to_path_buf(),
            node.start_byte(),
            node.end_byte(),
            node.start_position().row + 1,
            node.start_position().column + 1,
            text.to_owned(),
            replacement,
            MutationOperator::UnaryDecrementToIncrement,
        ));
    }
}

fn create_boolean_mutation(node: &Node<'_>, source: &str, file_path: &Path) -> Mutation {
    let original = get_node_text(node, source);
    let (replacement, operator) = if original == "true" {
        ("false", MutationOperator::BooleanTrueToFalse)
    } else {
        ("true", MutationOperator::BooleanFalseToTrue)
    };

    Mutation::new(
        file_path.to_path_buf(),
        node.start_byte(),
        node.end_byte(),
        node.start_position().row + 1,
        node.start_position().column + 1,
        original.to_owned(),
        replacement.to_owned(),
        operator,
    )
}

fn find_null_coalescing_mutation(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    // x ?? y -> x (remove fallback)
    if let Some(left) = node.child(0) {
        let left_text = get_node_text(&left, source);
        let full_text = get_node_text(node, source);

        mutations.push(Mutation::new(
            file_path.to_path_buf(),
            node.start_byte(),
            node.end_byte(),
            node.start_position().row + 1,
            node.start_position().column + 1,
            full_text.to_owned(),
            left_text.to_owned(),
            MutationOperator::NullCoalescingRemoval,
        ));
    }
}

fn find_null_aware_access_mutation(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    let text = get_node_text(node, source);

    // x?.y -> x.y
    if text.contains("?.") {
        let replacement = text.replace("?.", ".");
        mutations.push(Mutation::new(
            file_path.to_path_buf(),
            node.start_byte(),
            node.end_byte(),
            node.start_position().row + 1,
            node.start_position().column + 1,
            text.to_owned(),
            replacement,
            MutationOperator::NullAwareAccessRemoval,
        ));
    }
}

fn find_if_statement_mutations(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    // Find the condition - usually in parentheses
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "parenthesized_expression" {
            let cond_text = get_node_text(&child, source);

            // if(x) -> if(true)
            mutations.push(Mutation::new(
                file_path.to_path_buf(),
                child.start_byte(),
                child.end_byte(),
                child.start_position().row + 1,
                child.start_position().column + 1,
                cond_text.to_owned(),
                "(true)".to_owned(),
                MutationOperator::ControlFlowIfConditionTrue,
            ));

            // if(x) -> if(false)
            mutations.push(Mutation::new(
                file_path.to_path_buf(),
                child.start_byte(),
                child.end_byte(),
                child.start_position().row + 1,
                child.start_position().column + 1,
                cond_text.to_owned(),
                "(false)".to_owned(),
                MutationOperator::ControlFlowIfConditionFalse,
            ));

            break;
        }
    }
}

fn find_string_mutation(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    let text = get_node_text(node, source);

    // Skip interpolated strings
    if text.contains('$') {
        return;
    }

    let quote_char = if text.starts_with('\'') { '\'' } else { '"' };
    let inner = text
        .trim_start_matches(quote_char)
        .trim_end_matches(quote_char);

    if inner.is_empty() {
        // Empty -> non-empty
        mutations.push(Mutation::new(
            file_path.to_path_buf(),
            node.start_byte(),
            node.end_byte(),
            node.start_position().row + 1,
            node.start_position().column + 1,
            text.to_owned(),
            format!("{}mutated{}", quote_char, quote_char),
            MutationOperator::StringEmptyToNonEmpty,
        ));
    } else {
        // Non-empty -> empty
        mutations.push(Mutation::new(
            file_path.to_path_buf(),
            node.start_byte(),
            node.end_byte(),
            node.start_position().row + 1,
            node.start_position().column + 1,
            text.to_owned(),
            format!("{}{}", quote_char, quote_char),
            MutationOperator::StringNonEmptyToEmpty,
        ));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dart() {
        let source = r#"
            void main() {
                var x = 1 + 2;
                if (x > 0) {
                    print(x);
                }
            }
        "#;

        let tree = parse_dart(source).unwrap();
        assert!(!tree.root_node().has_error());
    }
}
