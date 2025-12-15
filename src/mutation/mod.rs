//! Mutation types and operators for Dart code
//!
//! This module defines the different kinds of mutations that can be applied
//! to Dart source code, inspired by Stryker's comprehensive operator set.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Location of a mutation in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Status of a mutant after testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutantStatus {
    /// Test failed (mutation was detected) - this is good!
    Killed,
    /// All tests passed (mutation was not detected) - this is bad!
    Survived,
    /// Test timed out (likely infinite loop) - counts as killed
    Timeout,
    /// No test coverage for the mutated code
    NoCoverage,
    /// Error occurred during testing
    Error,
    /// Not yet tested
    Pending,
}

/// Represents a single mutation that can be applied to source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mutation {
    /// Unique identifier for this mutation
    pub id: String,

    /// Location of this mutation in source
    pub location: SourceLocation,

    /// The type of mutation operator used
    pub operator: MutationOperator,

    /// The original source code being replaced
    pub original: String,

    /// The mutated replacement code
    pub mutated: String,

    /// Human-readable description of the mutation
    pub description: String,

    /// All possible replacement options
    pub replacements: Vec<String>,

    /// Whether this mutation was suggested by AI
    #[serde(default)]
    pub ai_suggested: bool,

    /// AI confidence score (0.0 - 1.0) if AI suggested
    #[serde(default)]
    pub ai_confidence: Option<f64>,
}

impl Mutation {
    /// Create a new mutation
    pub fn new(
        file_path: PathBuf,
        byte_start: usize,
        byte_end: usize,
        line: usize,
        column: usize,
        original: String,
        replacement: String,
        operator: MutationOperator,
    ) -> Self {
        let id = format!(
            "{:x}",
            md5::compute(format!(
                "{}:{}:{}:{}",
                file_path.display(),
                line,
                original,
                replacement
            ))
        );
        let description = format!("{}: {} → {}", operator.name(), original, replacement);

        Self {
            id,
            location: SourceLocation {
                file: file_path.clone(),
                start_line: line,
                start_col: column,
                end_line: line,
                end_col: column + original.len(),
                byte_start,
                byte_end,
            },
            operator,
            original,
            mutated: replacement.clone(),
            description,
            replacements: vec![replacement],
            ai_suggested: false,
            ai_confidence: None,
        }
    }

    /// Apply this mutation to the given source code
    pub fn apply(&self, source: &str) -> String {
        // Validate byte indices
        if self.location.byte_start > source.len() || self.location.byte_end > source.len() {
            tracing::warn!(
                "Mutation byte indices out of bounds: start={}, end={}, source_len={}",
                self.location.byte_start,
                self.location.byte_end,
                source.len()
            );
            return source.to_owned();
        }

        let mut result = String::with_capacity(source.len());
        result.push_str(source.get(..self.location.byte_start).unwrap_or_default());
        result.push_str(&self.mutated);
        result.push_str(source.get(self.location.byte_end..).unwrap_or_default());
        result
    }
}

/// Categories of mutation operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationOperator {
    // General categories (used by parser)
    Arithmetic,
    Comparison,
    Logical,
    Boolean,
    Unary,
    Assignment,
    NullSafety,
    String,
    Collection,
    Conditional,
    Return,
    Async,
    Literal,
    Bitwise,
    Other,

    // Specific arithmetic mutations
    ArithmeticAddToSub,
    ArithmeticSubToAdd,
    ArithmeticMulToDiv,
    ArithmeticDivToMul,
    ArithmeticModToMul,

    // Specific comparison mutations
    ComparisonLtToLte,
    ComparisonLtToGt,
    ComparisonLtToGte,
    ComparisonLteToLt,
    ComparisonLteToGt,
    ComparisonLteToGte,
    ComparisonGtToGte,
    ComparisonGtToLt,
    ComparisonGtToLte,
    ComparisonGteToGt,
    ComparisonGteToLt,
    ComparisonGteToLte,
    ComparisonEqToNeq,
    ComparisonNeqToEq,

    // Specific logical mutations
    LogicalAndToOr,
    LogicalOrToAnd,
    LogicalNotRemoval,

    // Specific boolean mutations
    BooleanTrueToFalse,
    BooleanFalseToTrue,

    // Specific unary mutations
    UnaryMinusRemoval,
    UnaryPlusMinus,
    UnaryIncrementToDecrement,
    UnaryDecrementToIncrement,
    UnaryPreToPost,
    UnaryPostToPre,

    // Specific assignment mutations
    AssignmentAddToSub,
    AssignmentSubToAdd,
    AssignmentMulToDiv,
    AssignmentDivToMul,

    // Dart Null Safety
    NullCoalescingRemoval,  // ?? → left operand
    NullAwareAccessRemoval, // ?. → .
    NullAssertionRemoval,   // x! → x
    NullCheckToTrue,        // x != null → true
    NullCheckToFalse,       // x == null → false

    // String mutations
    StringEmptyToNonEmpty,
    StringNonEmptyToEmpty,

    // Collection mutations
    CollectionEmptyCheck,    // isEmpty → isNotEmpty
    CollectionNotEmptyCheck, // isNotEmpty → isEmpty
    CollectionAddRemoval,    // .add() → nothing
    CollectionFirstToLast,   // .first → .last
    CollectionLastToFirst,   // .last → .first

    // Control Flow mutations
    ControlFlowIfConditionTrue,
    ControlFlowIfConditionFalse,
    ControlFlowRemoveElse,
    ControlFlowBreakRemoval,
    ControlFlowContinueRemoval,
    ControlFlowReturnRemoval,

    // Async mutations
    AsyncAwaitRemoval,
    AsyncFutureValueToError,

    // Method Calls
    MethodCallRemoval,

    // AI-Suggested (custom mutations)
    AiSuggested,
}

impl MutationOperator {
    /// Get a human-readable name for this operator
    pub fn name(&self) -> &'static str {
        match self {
            // General categories
            Self::Arithmetic => "Arithmetic Operator",
            Self::Comparison => "Comparison Operator",
            Self::Logical => "Logical Operator",
            Self::Boolean => "Boolean Literal",
            Self::Unary => "Unary Operator",
            Self::Assignment => "Assignment Operator",
            Self::NullSafety => "Null Safety Operator",
            Self::String => "String Literal",
            Self::Collection => "Collection Operation",
            Self::Conditional => "Conditional",
            Self::Return => "Return Statement",
            Self::Async => "Async Operation",
            Self::Literal => "Literal Value",
            Self::Bitwise => "Bitwise Operator",
            Self::Other => "Other",

            // Arithmetic
            Self::ArithmeticAddToSub => "Arithmetic: + → -",
            Self::ArithmeticSubToAdd => "Arithmetic: - → +",
            Self::ArithmeticMulToDiv => "Arithmetic: * → /",
            Self::ArithmeticDivToMul => "Arithmetic: / → *",
            Self::ArithmeticModToMul => "Arithmetic: % → *",

            // Comparison
            Self::ComparisonLtToLte => "Comparison: < → <=",
            Self::ComparisonLtToGt => "Comparison: < → >",
            Self::ComparisonLtToGte => "Comparison: < → >=",
            Self::ComparisonLteToLt => "Comparison: <= → <",
            Self::ComparisonLteToGt => "Comparison: <= → >",
            Self::ComparisonLteToGte => "Comparison: <= → >=",
            Self::ComparisonGtToGte => "Comparison: > → >=",
            Self::ComparisonGtToLt => "Comparison: > → <",
            Self::ComparisonGtToLte => "Comparison: > → <=",
            Self::ComparisonGteToGt => "Comparison: >= → >",
            Self::ComparisonGteToLt => "Comparison: >= → <",
            Self::ComparisonGteToLte => "Comparison: >= → <=",
            Self::ComparisonEqToNeq => "Comparison: == → !=",
            Self::ComparisonNeqToEq => "Comparison: != → ==",

            // Logical
            Self::LogicalAndToOr => "Logical: && → ||",
            Self::LogicalOrToAnd => "Logical: || → &&",
            Self::LogicalNotRemoval => "Logical: !x → x",

            // Boolean
            Self::BooleanTrueToFalse => "Boolean: true → false",
            Self::BooleanFalseToTrue => "Boolean: false → true",

            // Unary
            Self::UnaryMinusRemoval => "Unary: -x → x",
            Self::UnaryPlusMinus => "Unary: +x → -x",
            Self::UnaryIncrementToDecrement => "Unary: ++ → --",
            Self::UnaryDecrementToIncrement => "Unary: -- → ++",
            Self::UnaryPreToPost => "Unary: ++x → x++",
            Self::UnaryPostToPre => "Unary: x++ → ++x",

            // Assignment
            Self::AssignmentAddToSub => "Assignment: += → -=",
            Self::AssignmentSubToAdd => "Assignment: -= → +=",
            Self::AssignmentMulToDiv => "Assignment: *= → /=",
            Self::AssignmentDivToMul => "Assignment: /= → *=",

            // Null Safety
            Self::NullCoalescingRemoval => "Null: x ?? y → x",
            Self::NullAwareAccessRemoval => "Null: x?.y → x.y",
            Self::NullAssertionRemoval => "Null: x! → x",
            Self::NullCheckToTrue => "Null: x != null → true",
            Self::NullCheckToFalse => "Null: x == null → false",

            // String
            Self::StringEmptyToNonEmpty => "String: '' → 'mutated'",
            Self::StringNonEmptyToEmpty => "String: 'x' → ''",

            // Collection
            Self::CollectionEmptyCheck => "Collection: isEmpty → isNotEmpty",
            Self::CollectionNotEmptyCheck => "Collection: isNotEmpty → isEmpty",
            Self::CollectionAddRemoval => "Collection: .add() removal",
            Self::CollectionFirstToLast => "Collection: .first → .last",
            Self::CollectionLastToFirst => "Collection: .last → .first",

            // Control Flow
            Self::ControlFlowIfConditionTrue => "Control: if(x) → if(true)",
            Self::ControlFlowIfConditionFalse => "Control: if(x) → if(false)",
            Self::ControlFlowRemoveElse => "Control: else removal",
            Self::ControlFlowBreakRemoval => "Control: break removal",
            Self::ControlFlowContinueRemoval => "Control: continue removal",
            Self::ControlFlowReturnRemoval => "Control: return removal",

            // Async
            Self::AsyncAwaitRemoval => "Async: await removal",
            Self::AsyncFutureValueToError => "Async: Future.value → Future.error",

            // Method
            Self::MethodCallRemoval => "Method: call removal",

            // AI
            Self::AiSuggested => "AI Suggested",
        }
    }
}

/// Sample a subset of mutations for quicker testing
pub fn sample_mutations(mutations: &[Mutation], count: usize) -> Vec<Mutation> {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    if count >= mutations.len() {
        return mutations.to_vec();
    }

    let mut sampled: Vec<_> = mutations.to_vec();
    sampled.shuffle(&mut rng);
    sampled.truncate(count);
    sampled
}
