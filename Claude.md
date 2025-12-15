# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Rules

## Core Principles
- Ignoring tests = ILLEGAL
- Zero DUPLICATION. DRY AF!!! Always check for existing code before creating new code
- 100% Test Coverage is only the start of code quality
- No unit tests. Only COARSE tests that actually TEST TESTS.
- Beautiful output report
- Do not use Git unless asked to

## Rust Quality Standards
- All lints at highest strictness (see Cargo.toml `[lints]` section)
- `unsafe` code is forbidden (`unsafe_code = "deny"`)
- No `.unwrap()` or `.expect()` in production code - use `?` with proper error types
- No `panic!`, `todo!`, `unimplemented!` - handle all cases explicitly

## Functional Programming Style
- Follow FP style code with `Result<T,E>` and `Option<T>`
- Expressions over statements - prefer `match`, `if let`, iterator chains
- Pure functions where possible - minimize side effects
- Prefer `map`, `and_then`, `unwrap_or_else` over imperative control flow
- Use early returns with `?` operator for clean error propagation

## Code Structure
- Small, focused functions (clippy::too_many_lines warns at 100 lines)
- Low cognitive complexity (clippy::cognitive_complexity enabled)
- Descriptive variable names - no single letters except in closures
- Group related functionality into modules
- Public APIs must have documentation (`missing_docs = "warn"`)

## Project Overview

dart_mutant is a Rust-based mutation testing tool for Dart that uses tree-sitter for AST-based mutations. It parses Dart code, generates syntactically valid mutations, runs tests against each mutation, and generates beautiful HTML/JSON reports.

## Build & Test Commands

```bash
# Build
cargo build
cargo build --release

# Run all tests
cargo test

# Run specific integration test
cargo test --test integration_e2e
cargo test --test integration_parser
cargo test --test integration_mutation
cargo test --test integration_runner
cargo test --test integration_report

# Run with all lints (strictest settings configured in Cargo.toml)
cargo clippy

# Run the tool
cargo run -- --path ./path/to/dart/project --dry-run
cargo run -- --help

# Build release
cargo build --release
```

## Architecture

```
src/
├── main.rs       # Entry point, CLI orchestration, progress display
├── cli/mod.rs    # Clap argument definitions (Args, AiProvider)
├── parser/mod.rs # Tree-sitter Dart parsing, mutation discovery (find_*_mutations functions)
├── mutation/     # Mutation types, operators (40+ MutationOperator variants), sampling
│   ├── mod.rs    # Mutation struct, MutantStatus, SourceLocation
│   └── operators.rs
├── runner/mod.rs # Parallel test execution with tokio, file mutation/restoration
├── report/mod.rs # HTML (beautiful dark theme) and JSON (Stryker-compatible) report generation
└── ai/mod.rs     # AI-powered mutation suggestions (Anthropic/OpenAI/Ollama)
```

### Data Flow
1. `parser::discover_dart_files` finds .dart files, excluding generated code
2. `parser::parse_and_find_mutations` uses tree-sitter to walk AST and find mutation points
3. `runner::run_mutation_tests` applies each mutation, runs `dart test`, restores file
4. `report::generate_*_report` creates HTML/JSON output with per-file breakdown

### Key Types
- `Mutation` - describes a single code mutation (location, original, replacement, operator)
- `MutantTestResult` - result after running tests (status: Killed/Survived/Timeout/Error)
- `MutationResult` - aggregate stats (mutation_score, killed, survived counts)

## Code Style Rules

- **Result<T, E>** - Use anyhow::Result for error handling, thiserror for custom errors
- **Expressions over statements** - Prefer functional style, match expressions, iterator chains
- **Pure functions** - Minimize side effects, use immutable data where possible
- **No unit tests** - Only coarse integration tests that test real behavior (tests/ directory)
- **DRY** - Check for existing code before creating new; zero tolerance for duplication
- **All lints maxed** - Cargo.toml has strictest Clippy/rustc settings; unsafe_code = "deny"

## Testing Philosophy

Tests must actually TEST TESTS - they verify mutation testing catches weak test suites. Integration tests in `tests/` use `tests/fixtures/simple_dart_project/` as a real Dart project fixture.

## CI Notes

- Requires Dart SDK for full integration tests (runner tests skip gracefully if Dart unavailable)
- Binary must be built before E2E tests: `cargo build && cargo test`
