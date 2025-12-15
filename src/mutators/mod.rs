//! Mutation operators for Dart code
//!
//! This module provides AST-based mutation operators that transform Dart code
//! in semantically meaningful ways to test your test suite's effectiveness.

use tree_sitter::Node;

/// Represents a specific mutation that can be applied to code
#[derive(Debug, Clone)]
pub struct MutationOp {
    /// Human-readable name of this mutation
    pub name: &'static str,
    /// The mutation operator category
    pub category: MutatorCategory,
    /// Original code snippet
    pub original: String,
    /// Mutated code snippet
    pub replacement: String,
    /// Byte offset in source where mutation starts
    pub start_byte: usize,
    /// Byte offset in source where mutation ends
    pub end_byte: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub column: usize,
}

/// Categories of mutation operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MutatorCategory {
    Arithmetic,
    Comparison,
    Logical,
    Boolean,
    Unary,
    Assignment,
    NullSafety,
    String,
    Collection,
    ControlFlow,
}

impl MutatorCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "arithmetic" => Some(Self::Arithmetic),
            "comparison" => Some(Self::Comparison),
            "logical" => Some(Self::Logical),
            "boolean" => Some(Self::Boolean),
            "unary" => Some(Self::Unary),
            "assignment" => Some(Self::Assignment),
            "null_safety" | "nullsafety" => Some(Self::NullSafety),
            "string" => Some(Self::String),
            "collection" => Some(Self::Collection),
            "control_flow" | "controlflow" => Some(Self::ControlFlow),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Arithmetic => "arithmetic",
            Self::Comparison => "comparison",
            Self::Logical => "logical",
            Self::Boolean => "boolean",
            Self::Unary => "unary",
            Self::Assignment => "assignment",
            Self::NullSafety => "null_safety",
            Self::String => "string",
            Self::Collection => "collection",
            Self::ControlFlow => "control_flow",
        }
    }
}

/// Trait for mutation operators
pub trait Mutator: Send + Sync {
    /// Returns the category of this mutator
    fn category(&self) -> MutatorCategory;

    /// Check if this mutator can handle the given AST node
    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool;

    /// Generate all possible mutations for the given node
    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp>;
}

// ============================================================================
// ARITHMETIC MUTATOR
// ============================================================================

/// Mutates arithmetic operators: + - * / % ~/
pub struct ArithmeticMutator;

impl Mutator for ArithmeticMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::Arithmetic
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() == "binary_expression" {
            if let Some(op_node) = node.child_by_field_name("operator") {
                let op = &source[op_node.start_byte()..op_node.end_byte()];
                return matches!(op, b"+" | b"-" | b"*" | b"/" | b"%" | b"~/");
            }
        }
        false
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();

        if let Some(op_node) = node.child_by_field_name("operator") {
            let op = String::from_utf8_lossy(&source[op_node.start_byte()..op_node.end_byte()]);
            let start = op_node.start_byte();
            let end = op_node.end_byte();
            let line = op_node.start_position().row + 1;
            let column = op_node.start_position().column;

            let replacements: Vec<&str> = match op.as_ref() {
                "+" => vec!["-", "*"],
                "-" => vec!["+", "*"],
                "*" => vec!["/", "+"],
                "/" => vec!["*", "~/"],
                "%" => vec!["*", "/"],
                "~/" => vec!["/", "%"],
                _ => vec![],
            };

            for replacement in replacements {
                mutations.push(MutationOp {
                    name: "ArithmeticOperatorReplacement",
                    category: MutatorCategory::Arithmetic,
                    original: op.to_string(),
                    replacement: replacement.to_string(),
                    start_byte: start,
                    end_byte: end,
                    line,
                    column,
                });
            }
        }

        mutations
    }
}

// ============================================================================
// COMPARISON MUTATOR
// ============================================================================

/// Mutates comparison operators: < > <= >= == !=
pub struct ComparisonMutator;

impl Mutator for ComparisonMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::Comparison
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() == "binary_expression" {
            if let Some(op_node) = node.child_by_field_name("operator") {
                let op = &source[op_node.start_byte()..op_node.end_byte()];
                return matches!(op, b"<" | b">" | b"<=" | b">=" | b"==" | b"!=");
            }
        }
        false
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();

        if let Some(op_node) = node.child_by_field_name("operator") {
            let op = String::from_utf8_lossy(&source[op_node.start_byte()..op_node.end_byte()]);
            let start = op_node.start_byte();
            let end = op_node.end_byte();
            let line = op_node.start_position().row + 1;
            let column = op_node.start_position().column;

            let replacements: Vec<&str> = match op.as_ref() {
                "<" => vec!["<=", ">", ">="],
                ">" => vec![">=", "<", "<="],
                "<=" => vec!["<", ">=", ">"],
                ">=" => vec![">", "<=", "<"],
                "==" => vec!["!="],
                "!=" => vec!["=="],
                _ => vec![],
            };

            for replacement in replacements {
                mutations.push(MutationOp {
                    name: "ComparisonOperatorReplacement",
                    category: MutatorCategory::Comparison,
                    original: op.to_string(),
                    replacement: replacement.to_string(),
                    start_byte: start,
                    end_byte: end,
                    line,
                    column,
                });
            }
        }

        mutations
    }
}

// ============================================================================
// LOGICAL MUTATOR
// ============================================================================

/// Mutates logical operators: && ||
pub struct LogicalMutator;

impl Mutator for LogicalMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::Logical
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() == "binary_expression" {
            if let Some(op_node) = node.child_by_field_name("operator") {
                let op = &source[op_node.start_byte()..op_node.end_byte()];
                return matches!(op, b"&&" | b"||");
            }
        }
        false
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();

        if let Some(op_node) = node.child_by_field_name("operator") {
            let op = String::from_utf8_lossy(&source[op_node.start_byte()..op_node.end_byte()]);
            let start = op_node.start_byte();
            let end = op_node.end_byte();
            let line = op_node.start_position().row + 1;
            let column = op_node.start_position().column;

            let replacement = match op.as_ref() {
                "&&" => "||",
                "||" => "&&",
                _ => return mutations,
            };

            mutations.push(MutationOp {
                name: "LogicalOperatorReplacement",
                category: MutatorCategory::Logical,
                original: op.to_string(),
                replacement: replacement.to_string(),
                start_byte: start,
                end_byte: end,
                line,
                column,
            });
        }

        mutations
    }
}

// ============================================================================
// BOOLEAN MUTATOR
// ============================================================================

/// Mutates boolean literals: true <-> false
pub struct BooleanMutator;

impl Mutator for BooleanMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::Boolean
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() == "true" || node.kind() == "false" {
            return true;
        }
        if node.kind() == "identifier" {
            let text = &source[node.start_byte()..node.end_byte()];
            return matches!(text, b"true" | b"false");
        }
        false
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);
        let start = node.start_byte();
        let end = node.end_byte();
        let line = node.start_position().row + 1;
        let column = node.start_position().column;

        let replacement = match text.as_ref() {
            "true" => "false",
            "false" => "true",
            _ => return vec![],
        };

        vec![MutationOp {
            name: "BooleanLiteralReplacement",
            category: MutatorCategory::Boolean,
            original: text.to_string(),
            replacement: replacement.to_string(),
            start_byte: start,
            end_byte: end,
            line,
            column,
        }]
    }
}

// ============================================================================
// UNARY MUTATOR
// ============================================================================

/// Mutates unary operators: ! ++ -- -
pub struct UnaryMutator;

impl Mutator for UnaryMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::Unary
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        match node.kind() {
            "unary_expression" | "prefix_expression" | "postfix_expression" => true,
            _ => {
                if let Some(first_child) = node.child(0) {
                    let text = &source[first_child.start_byte()..first_child.end_byte()];
                    matches!(text, b"!" | b"++" | b"--" | b"-")
                } else {
                    false
                }
            }
        }
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();

        // Handle negation removal (!)
        if node.kind() == "unary_expression" || node.kind() == "prefix_expression" {
            if let Some(op_child) = node.child(0) {
                let op = String::from_utf8_lossy(&source[op_child.start_byte()..op_child.end_byte()]);

                if op == "!" {
                    // Remove the negation
                    if let Some(operand) = node.child(1) {
                        let operand_text = String::from_utf8_lossy(
                            &source[operand.start_byte()..operand.end_byte()],
                        );
                        mutations.push(MutationOp {
                            name: "NegationRemoval",
                            category: MutatorCategory::Unary,
                            original: format!("!{}", operand_text),
                            replacement: operand_text.to_string(),
                            start_byte: node.start_byte(),
                            end_byte: node.end_byte(),
                            line: node.start_position().row + 1,
                            column: node.start_position().column,
                        });
                    }
                }
            }
        }

        // Handle increment/decrement
        let node_text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);
        if node_text.contains("++") || node_text.contains("--") {
            let line = node.start_position().row + 1;
            let column = node.start_position().column;

            if node_text.contains("++") {
                mutations.push(MutationOp {
                    name: "IncrementToDecrement",
                    category: MutatorCategory::Unary,
                    original: node_text.to_string(),
                    replacement: node_text.replace("++", "--").to_string(),
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    line,
                    column,
                });
            }
            if node_text.contains("--") {
                mutations.push(MutationOp {
                    name: "DecrementToIncrement",
                    category: MutatorCategory::Unary,
                    original: node_text.to_string(),
                    replacement: node_text.replace("--", "++").to_string(),
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    line,
                    column,
                });
            }
        }

        mutations
    }
}

// ============================================================================
// ASSIGNMENT MUTATOR
// ============================================================================

/// Mutates compound assignment operators: += -= *= /= etc.
pub struct AssignmentMutator;

impl Mutator for AssignmentMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::Assignment
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        if node.kind() == "assignment_expression" {
            let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);
            return text.contains("+=")
                || text.contains("-=")
                || text.contains("*=")
                || text.contains("/=")
                || text.contains("%=")
                || text.contains("??=");
        }
        false
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();
        let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);
        let line = node.start_position().row + 1;
        let column = node.start_position().column;

        let ops = [
            ("+=", vec!["-=", "*="]),
            ("-=", vec!["+=", "*="]),
            ("*=", vec!["/=", "+="]),
            ("/=", vec!["*=", "-="]),
            ("%=", vec!["*=", "/="]),
            ("??=", vec!["="]),
        ];

        for (orig, replacements) in ops {
            if text.contains(orig) {
                for repl in replacements {
                    mutations.push(MutationOp {
                        name: "CompoundAssignmentReplacement",
                        category: MutatorCategory::Assignment,
                        original: text.to_string(),
                        replacement: text.replacen(orig, repl, 1).to_string(),
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                        line,
                        column,
                    });
                }
            }
        }

        mutations
    }
}

// ============================================================================
// NULL SAFETY MUTATOR
// ============================================================================

/// Mutates Dart null safety operators: ?. ?? ! ?.
pub struct NullSafetyMutator;

impl Mutator for NullSafetyMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::NullSafety
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);

        // Check for null-aware operators
        text.contains("?.") || text.contains("??") || text.contains("?[") || text.ends_with("!")
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();
        let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);
        let line = node.start_position().row + 1;
        let column = node.start_position().column;

        // ?. -> . (remove null safety)
        if text.contains("?.") {
            mutations.push(MutationOp {
                name: "NullAwareAccessRemoval",
                category: MutatorCategory::NullSafety,
                original: text.to_string(),
                replacement: text.replace("?.", ".").to_string(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                line,
                column,
            });
        }

        // ?? -> left operand only (remove fallback)
        if text.contains("??") && !text.contains("??=") {
            // Find the ?? and take left side only
            if let Some(idx) = text.find("??") {
                let left = text[..idx].trim();
                mutations.push(MutationOp {
                    name: "NullCoalescingRemoval",
                    category: MutatorCategory::NullSafety,
                    original: text.to_string(),
                    replacement: left.to_string(),
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    line,
                    column,
                });
            }
        }

        // ?[ -> [ (remove null-aware subscript)
        if text.contains("?[") {
            mutations.push(MutationOp {
                name: "NullAwareSubscriptRemoval",
                category: MutatorCategory::NullSafety,
                original: text.to_string(),
                replacement: text.replace("?[", "[").to_string(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                line,
                column,
            });
        }

        // Remove trailing ! (bang operator)
        if text.ends_with("!") && !text.ends_with("!=") {
            mutations.push(MutationOp {
                name: "BangOperatorRemoval",
                category: MutatorCategory::NullSafety,
                original: text.to_string(),
                replacement: text[..text.len() - 1].to_string(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                line,
                column,
            });
        }

        mutations
    }
}

// ============================================================================
// STRING MUTATOR
// ============================================================================

/// Mutates string literals
pub struct StringMutator;

impl Mutator for StringMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::String
    }

    fn can_mutate(&self, node: &Node, _source: &[u8]) -> bool {
        matches!(node.kind(), "string_literal" | "string")
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();
        let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);
        let line = node.start_position().row + 1;
        let column = node.start_position().column;

        // Empty string mutation
        if text.len() > 2 {
            // Has content between quotes
            let quote_char = text.chars().next().unwrap_or('"');
            let empty = if text.starts_with("'''") || text.starts_with("\"\"\"") {
                format!("{}{}{}{}{}{}",
                    quote_char, quote_char, quote_char,
                    quote_char, quote_char, quote_char)
            } else {
                format!("{}{}", quote_char, quote_char)
            };

            mutations.push(MutationOp {
                name: "StringEmptyMutation",
                category: MutatorCategory::String,
                original: text.to_string(),
                replacement: empty,
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                line,
                column,
            });
        }

        // Mutate string content (add "MUTATED_" prefix)
        if text.len() > 2 {
            let inner_start = if text.starts_with("'''") || text.starts_with("\"\"\"") { 3 } else { 1 };
            let inner_end = text.len() - inner_start;
            if inner_end > inner_start {
                let prefix = &text[..inner_start];
                let suffix = &text[inner_end..];
                let inner = &text[inner_start..inner_end];
                let mutated = format!("{}MUTATED_{}{}", prefix, inner, suffix);

                mutations.push(MutationOp {
                    name: "StringContentMutation",
                    category: MutatorCategory::String,
                    original: text.to_string(),
                    replacement: mutated,
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    line,
                    column,
                });
            }
        }

        mutations
    }
}

// ============================================================================
// COLLECTION MUTATOR
// ============================================================================

/// Mutates collection operations
pub struct CollectionMutator;

impl Mutator for CollectionMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::Collection
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);

        // Common collection methods
        text.contains(".add(")
            || text.contains(".remove(")
            || text.contains(".isEmpty")
            || text.contains(".isNotEmpty")
            || text.contains(".first")
            || text.contains(".last")
            || text.contains(".length")
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();
        let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);
        let line = node.start_position().row + 1;
        let column = node.start_position().column;

        // isEmpty <-> isNotEmpty
        if text.contains(".isEmpty") {
            mutations.push(MutationOp {
                name: "CollectionEmptyCheck",
                category: MutatorCategory::Collection,
                original: text.to_string(),
                replacement: text.replace(".isEmpty", ".isNotEmpty").to_string(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                line,
                column,
            });
        }

        if text.contains(".isNotEmpty") {
            mutations.push(MutationOp {
                name: "CollectionEmptyCheck",
                category: MutatorCategory::Collection,
                original: text.to_string(),
                replacement: text.replace(".isNotEmpty", ".isEmpty").to_string(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                line,
                column,
            });
        }

        // first <-> last
        if text.contains(".first") && !text.contains(".firstWhere") {
            mutations.push(MutationOp {
                name: "CollectionBoundaryAccess",
                category: MutatorCategory::Collection,
                original: text.to_string(),
                replacement: text.replace(".first", ".last").to_string(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                line,
                column,
            });
        }

        if text.contains(".last") && !text.contains(".lastWhere") {
            mutations.push(MutationOp {
                name: "CollectionBoundaryAccess",
                category: MutatorCategory::Collection,
                original: text.to_string(),
                replacement: text.replace(".last", ".first").to_string(),
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                line,
                column,
            });
        }

        mutations
    }
}

// ============================================================================
// CONTROL FLOW MUTATOR
// ============================================================================

/// Mutates control flow: if conditions, return statements
pub struct ControlFlowMutator;

impl Mutator for ControlFlowMutator {
    fn category(&self) -> MutatorCategory {
        MutatorCategory::ControlFlow
    }

    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        match node.kind() {
            "if_statement" | "while_statement" | "for_statement" => true,
            "return_statement" => {
                let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);
                text.contains("return true") || text.contains("return false")
            }
            _ => false,
        }
    }

    fn generate_mutations(&self, node: &Node, source: &[u8]) -> Vec<MutationOp> {
        let mut mutations = Vec::new();
        let line = node.start_position().row + 1;
        let column = node.start_position().column;

        match node.kind() {
            "if_statement" => {
                // Negate the condition
                if let Some(condition) = node.child_by_field_name("condition") {
                    let cond_text = String::from_utf8_lossy(
                        &source[condition.start_byte()..condition.end_byte()],
                    );

                    // Wrap in negation
                    let negated = if cond_text.starts_with("!") && !cond_text.starts_with("!=") {
                        // Remove existing negation
                        cond_text[1..].to_string()
                    } else {
                        format!("!({})", cond_text)
                    };

                    mutations.push(MutationOp {
                        name: "ConditionNegation",
                        category: MutatorCategory::ControlFlow,
                        original: cond_text.to_string(),
                        replacement: negated,
                        start_byte: condition.start_byte(),
                        end_byte: condition.end_byte(),
                        line: condition.start_position().row + 1,
                        column: condition.start_position().column,
                    });
                }
            }
            "return_statement" => {
                let text = String::from_utf8_lossy(&source[node.start_byte()..node.end_byte()]);

                if text.contains("return true") {
                    mutations.push(MutationOp {
                        name: "ReturnValueMutation",
                        category: MutatorCategory::ControlFlow,
                        original: text.to_string(),
                        replacement: text.replace("return true", "return false").to_string(),
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                        line,
                        column,
                    });
                }

                if text.contains("return false") {
                    mutations.push(MutationOp {
                        name: "ReturnValueMutation",
                        category: MutatorCategory::ControlFlow,
                        original: text.to_string(),
                        replacement: text.replace("return false", "return true").to_string(),
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                        line,
                        column,
                    });
                }
            }
            _ => {}
        }

        mutations
    }
}

// ============================================================================
// MUTATOR REGISTRY
// ============================================================================

/// Returns all available mutators
pub fn all_mutators() -> Vec<Box<dyn Mutator>> {
    vec![
        Box::new(ArithmeticMutator),
        Box::new(ComparisonMutator),
        Box::new(LogicalMutator),
        Box::new(BooleanMutator),
        Box::new(UnaryMutator),
        Box::new(AssignmentMutator),
        Box::new(NullSafetyMutator),
        Box::new(StringMutator),
        Box::new(CollectionMutator),
        Box::new(ControlFlowMutator),
    ]
}

/// Returns mutators filtered by enabled categories
pub fn get_mutators(enabled: &[String]) -> Vec<Box<dyn Mutator>> {
    let enabled_categories: Vec<MutatorCategory> = enabled
        .iter()
        .filter_map(|s| MutatorCategory::from_str(s))
        .collect();

    if enabled_categories.is_empty() {
        return all_mutators();
    }

    all_mutators()
        .into_iter()
        .filter(|m| enabled_categories.contains(&m.category()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutator_category_from_str() {
        assert_eq!(MutatorCategory::from_str("arithmetic"), Some(MutatorCategory::Arithmetic));
        assert_eq!(MutatorCategory::from_str("ARITHMETIC"), Some(MutatorCategory::Arithmetic));
        assert_eq!(MutatorCategory::from_str("null_safety"), Some(MutatorCategory::NullSafety));
        assert_eq!(MutatorCategory::from_str("unknown"), None);
    }

    #[test]
    fn test_all_mutators_returns_10() {
        assert_eq!(all_mutators().len(), 10);
    }

    #[test]
    fn test_get_mutators_filters() {
        let mutators = get_mutators(&["arithmetic".to_string(), "logical".to_string()]);
        assert_eq!(mutators.len(), 2);
    }
}
