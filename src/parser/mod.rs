//! Dart parser using tree-sitter for AST-based mutation discovery
//!
//! This module parses Dart source files and identifies locations where
//! mutations can be applied safely and meaningfully.

use crate::mutation::{Mutation, MutationOperator};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, trace, warn};
use tree_sitter::{Node, Parser, Tree};
use walkdir::{DirEntry, WalkDir};

/// Non-hidden directory names pruned during discovery: generated build output
/// and vendored dependencies that never contain project source worth mutating.
const PRUNED_DIRS: &[&str] = &["build", "node_modules"];

/// Returns `true` when a directory entry should be pruned (never descended into).
///
/// Hidden directories (`.dart_tool`, `.git`, `.fvm`, …) and [`PRUNED_DIRS`] are
/// pruned. Pruning `.dart_tool` is critical: it holds symlinks into the pub
/// cache, and descending through them makes discovery appear to hang forever on
/// real Dart projects. The scan root itself (depth 0) is never pruned.
fn is_pruned_dir(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() || entry.depth() == 0 {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    let prune = name.starts_with('.') || PRUNED_DIRS.contains(&name.as_ref());
    if prune {
        debug!(dir = %entry.path().display(), "pruning directory from discovery");
    }
    prune
}

/// Returns `true` when a file is generated Dart code that must not be mutated.
fn is_generated_dart(file_path: &Path) -> bool {
    let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
    filename.ends_with(".g.dart")
        || filename.ends_with(".freezed.dart")
        || filename.ends_with(".mocks.dart")
}

/// Returns `true` when `path_str` matches any of the glob exclusion `patterns`.
fn matches_exclude(patterns: &[String], path_str: &str) -> bool {
    patterns
        .iter()
        .any(|pattern| glob::Pattern::new(pattern).is_ok_and(|p| p.matches(path_str)))
}

/// Running tallies recorded while classifying discovered files, for logging.
#[derive(Default)]
struct DiscoveryStats {
    excluded: usize,
    generated: usize,
}

/// Classify one filesystem entry, pushing real Dart source onto `files`.
fn classify_entry(
    file_path: &Path,
    exclude_patterns: &[String],
    files: &mut Vec<PathBuf>,
    stats: &mut DiscoveryStats,
) {
    if file_path.extension().map_or(true, |ext| ext != "dart") {
        return;
    }
    let path_str = file_path.to_string_lossy();
    if matches_exclude(exclude_patterns, &path_str) {
        stats.excluded += 1;
        debug!(file = %file_path.display(), "excluded by pattern");
    } else if is_generated_dart(file_path) {
        stats.generated += 1;
        debug!(file = %file_path.display(), "skipped generated file");
    } else {
        debug!(file = %file_path.display(), "discovered Dart file");
        files.push(file_path.to_path_buf());
    }
}

/// Discover all Dart files in the given path, excluding specified patterns.
///
/// Symlinks are NOT followed and heavy directories are pruned (see
/// [`is_pruned_dir`]) so discovery cannot wander into the pub cache. The full
/// directory tree is otherwise walked to unlimited depth.
pub fn discover_dart_files(path: &Path, exclude_patterns: &[String]) -> Result<Vec<PathBuf>> {
    info!(root = %path.display(), excludes = exclude_patterns.len(), "discovering Dart files");

    let mut files = Vec::new();
    let mut stats = DiscoveryStats::default();
    let mut dirs_scanned = 0_usize;

    let walker = WalkDir::new(path).follow_links(false).into_iter();
    for entry in walker.filter_entry(|e| !is_pruned_dir(e)) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warn!(%error, "skipping unreadable path during discovery");
                continue;
            }
        };
        if entry.file_type().is_dir() {
            dirs_scanned += 1;
            trace!(dir = %entry.path().display(), depth = entry.depth(), "scanning directory");
            continue;
        }
        classify_entry(entry.path(), exclude_patterns, &mut files, &mut stats);
    }

    info!(
        found = files.len(),
        dirs_scanned,
        excluded = stats.excluded,
        generated = stats.generated,
        "Dart file discovery complete"
    );
    if files.is_empty() {
        warn!(root = %path.display(), "no Dart files found — check --path and --exclude patterns");
    }
    Ok(files)
}

/// Parse a Dart file and find all possible mutation locations
pub fn parse_and_find_mutations(file_path: &Path) -> Result<Vec<Mutation>> {
    trace!(file = %file_path.display(), "parsing file for mutations");
    let source = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let tree = parse_dart(&source)?;
    if tree.root_node().has_error() {
        warn!(file = %file_path.display(), "tree-sitter reported syntax errors; mutation discovery may be incomplete");
    }
    let mut mutations = Vec::new();

    find_mutations_in_tree(&tree, &source, file_path, &mut mutations);

    debug!(file = %file_path.display(), count = mutations.len(), "found mutations in file");
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

        // Statement-level calls to (likely void) functions/methods
        "expression_statement" => {
            find_method_call_removal_mutation(&node, source, file_path, mutations);
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

/// Remove a statement-level call to a (likely void-returning) function/method.
///
/// Implements [PARSER-VOID-CALL]. Calls whose return value is discarded (e.g.
/// `order.validate();`, `save(order);`, `await flush();`) usually exist for
/// their side effects; deleting the whole statement surfaces tests that only
/// assert return values. The replacement is empty, which is always valid Dart.
fn find_method_call_removal_mutation(
    node: &Node<'_>,
    source: &str,
    file_path: &Path,
    mutations: &mut Vec<Mutation>,
) {
    if !is_side_effect_call_statement(node) {
        return;
    }
    mutations.push(Mutation::new(
        file_path.to_path_buf(),
        node.start_byte(),
        node.end_byte(),
        node.start_position().row + 1,
        node.start_position().column + 1,
        get_node_text(node, source).to_owned(),
        String::new(),
        MutationOperator::MethodCallRemoval,
    ));
}

/// Returns `true` when `node` is an expression statement whose value is
/// discarded and whose expression is a call (including an awaited call).
///
/// Assignments (`x = f();`), cascades (`o..a()..b();`) and bare field accesses
/// (`o.field;`) are excluded: only the first kinds below combined with an
/// invocation count. The statement must also live directly in a brace block —
/// deleting the lone statement of a braceless body (`if (c) f();`) would yield
/// invalid Dart. Implements [PARSER-VOID-CALL].
fn is_side_effect_call_statement(node: &Node<'_>) -> bool {
    node.parent().is_some_and(|parent| parent.kind() == "block")
        && node.named_child(0).is_some_and(|inner| {
            matches!(
                inner.kind(),
                "member_access" | "unary_expression" | "await_expression"
            ) && contains_arguments(&inner)
        })
}

/// Returns `true` when `node` has an `arguments` descendant — i.e. it contains
/// an invocation rather than only field accesses.
fn contains_arguments(node: &Node<'_>) -> bool {
    (0..node.child_count()).any(|i| {
        node.child(i)
            .is_some_and(|child| child.kind() == "arguments" || contains_arguments(&child))
    })
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
    if is_library_directive_string(node) {
        return;
    }

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

fn is_library_directive_string(node: &Node<'_>) -> bool {
    let mut current = *node;

    while let Some(parent) = current.parent() {
        if matches!(
            parent.kind(),
            "import_or_export" | "part_directive" | "part_of_directive"
        ) {
            return true;
        }
        current = parent;
    }

    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::mutation::MutationOperator;
    use std::path::{Path, PathBuf};

    fn write_dart_file(dir: &tempfile::TempDir, path: &str, source: &str) -> PathBuf {
        let file_path = dir.path().join(path);
        let parent = file_path.parent().unwrap_or(dir.path());
        std::fs::create_dir_all(parent).unwrap();
        std::fs::write(&file_path, source).unwrap();
        file_path
    }

    fn parse_mutations(source: &str) -> Vec<Mutation> {
        let dir = tempfile::tempdir().unwrap();
        let file_path = write_dart_file(&dir, "lib/sample.dart", source);
        parse_and_find_mutations(&file_path).unwrap()
    }

    fn has_operator(mutations: &[Mutation], operator: MutationOperator) -> bool {
        mutations.iter().any(|m| m.operator == operator)
    }

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

    #[test]
    fn test_string_mutations_skip_library_directives() {
        let source = r#"
            import 'package:example/foo.dart';
            export 'src/bar.dart';
            part 'src/baz.dart';
            part of 'package:example/library.dart';

            const greeting = 'hello';
        "#;
        let tree = parse_dart(source).unwrap();
        let mut mutations = Vec::new();

        find_mutations_in_tree(&tree, source, Path::new("sample.dart"), &mut mutations);

        let string_mutations: Vec<_> = mutations
            .iter()
            .filter(|m| matches!(m.operator, MutationOperator::StringNonEmptyToEmpty))
            .map(|m| m.original.as_str())
            .collect();

        assert_eq!(
            string_mutations,
            vec!["'hello'"],
            "import/export/part/part of directives must not be mutated"
        );
    }

    #[test]
    fn test_parse_and_find_mutations_covers_core_operator_families() {
        let source = r#"
            import 'package:example/foo.dart';

            class Example {
              int run(int a, int b, bool flag, String? value) {
                var sum = a + b;
                var diff = a - b;
                var prod = a * b;
                var div = a / b;
                var mod = a % b;
                var lt = a < b;
                var lte = a <= b;
                var gt = a > b;
                var gte = a >= b;
                var eq = a == b;
                var neq = a != b;
                var anded = flag && a > 0;
                var ored = flag || b > 0;
                var negated = !flag;
                ++a;
                b--;
                var truth = true;
                var lie = false;
                var fallback = value ?? 'fallback';
                var length = value?.length;
                if (a > 0) {
                  return 1;
                }
                var empty = '';
                var word = 'hello';
                var interpolated = 'value $a';
                return sum + diff + prod + div.toInt() + mod + length!;
              }
            }
        "#;
        let mutations = parse_mutations(source);
        assert!(has_operator(
            &mutations,
            MutationOperator::ArithmeticAddToSub
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ArithmeticSubToAdd
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ArithmeticMulToDiv
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ArithmeticDivToMul
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ArithmeticModToMul
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ComparisonLtToLte
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ComparisonLteToLt
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ComparisonGtToGte
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ComparisonGteToGt
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ComparisonEqToNeq
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ComparisonNeqToEq
        ));
        assert!(has_operator(&mutations, MutationOperator::LogicalAndToOr));
        assert!(has_operator(&mutations, MutationOperator::LogicalOrToAnd));
        assert!(has_operator(
            &mutations,
            MutationOperator::LogicalNotRemoval
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::UnaryIncrementToDecrement
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::UnaryDecrementToIncrement
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::BooleanTrueToFalse
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::BooleanFalseToTrue
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::NullCoalescingRemoval
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ControlFlowIfConditionTrue
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::ControlFlowIfConditionFalse
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::StringEmptyToNonEmpty
        ));
        assert!(has_operator(
            &mutations,
            MutationOperator::StringNonEmptyToEmpty
        ));
    }

    /// Default excludes mirror the CLI so discovery tests reflect real runs.
    fn default_excludes() -> Vec<String> {
        vec![
            "**/*.g.dart".to_string(),
            "**/*_test.dart".to_string(),
            "**/test/**".to_string(),
        ]
    }

    fn file_names(files: &[PathBuf]) -> Vec<String> {
        files
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect()
    }

    #[test]
    fn test_discover_recurses_into_deeply_nested_directories() {
        let dir = tempfile::tempdir().unwrap();
        write_dart_file(&dir, "lib/a/b/c/d/e/deep.dart", "void main() {}");
        write_dart_file(&dir, "lib/top.dart", "void main() {}");

        let files = discover_dart_files(dir.path(), &[]).unwrap();

        let names = file_names(&files);
        assert!(names.contains(&"deep.dart".to_string()), "got {names:?}");
        assert!(names.contains(&"top.dart".to_string()), "got {names:?}");
    }

    #[test]
    fn test_discover_prunes_dart_tool_and_dependency_dirs() {
        let dir = tempfile::tempdir().unwrap();
        write_dart_file(&dir, "lib/real.dart", "void main() {}");
        write_dart_file(&dir, ".dart_tool/pkg/cached.dart", "void main() {}");
        write_dart_file(&dir, "build/generated.dart", "void main() {}");
        write_dart_file(&dir, "node_modules/dep/lib.dart", "void main() {}");

        let names = file_names(&discover_dart_files(dir.path(), &[]).unwrap());

        assert_eq!(names, vec!["real.dart".to_string()], "got {names:?}");
    }

    #[test]
    fn test_discover_skips_generated_files() {
        let dir = tempfile::tempdir().unwrap();
        write_dart_file(&dir, "lib/model.dart", "void main() {}");
        write_dart_file(&dir, "lib/model.g.dart", "void main() {}");
        write_dart_file(&dir, "lib/model.freezed.dart", "void main() {}");
        write_dart_file(&dir, "lib/model.mocks.dart", "void main() {}");

        let names = file_names(&discover_dart_files(dir.path(), &[]).unwrap());

        assert_eq!(names, vec!["model.dart".to_string()], "got {names:?}");
    }

    #[test]
    fn test_discover_respects_exclude_patterns() {
        let dir = tempfile::tempdir().unwrap();
        write_dart_file(&dir, "lib/calc.dart", "void main() {}");
        write_dart_file(&dir, "test/calc_test.dart", "void main() {}");

        let names = file_names(&discover_dart_files(dir.path(), &default_excludes()).unwrap());

        assert_eq!(names, vec!["calc.dart".to_string()], "got {names:?}");
    }

    #[test]
    fn test_discover_empty_tree_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("lib")).unwrap();

        let files = discover_dart_files(dir.path(), &[]).unwrap();

        assert!(files.is_empty(), "got {files:?}");
    }

    #[cfg(unix)]
    #[test]
    fn test_discover_does_not_follow_symlinks() {
        let outside = tempfile::tempdir().unwrap();
        write_dart_file(&outside, "pkg/external.dart", "void main() {}");

        let dir = tempfile::tempdir().unwrap();
        write_dart_file(&dir, "lib/real.dart", "void main() {}");
        std::os::unix::fs::symlink(outside.path(), dir.path().join("lib").join("linked")).unwrap();

        let names = file_names(&discover_dart_files(dir.path(), &[]).unwrap());

        assert_eq!(names, vec!["real.dart".to_string()], "got {names:?}");
    }

    #[test]
    fn test_string_mutations_skip_interpolated_and_directive_strings() {
        let source = r#"
            export 'src/library.dart';

            String message(int value) {
              var interpolated = 'value $value';
              return 'plain';
            }
        "#;
        let mutations = parse_mutations(source);
        let originals: Vec<_> = mutations
            .iter()
            .filter(|m| matches!(m.operator, MutationOperator::StringNonEmptyToEmpty))
            .map(|m| m.original.as_str())
            .collect();

        assert_eq!(originals, vec!["'plain'"]);
    }
}
