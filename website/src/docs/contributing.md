---
layout: docs.njk
title: Contributing
description: Contribute to dart_mutant. Learn how to build from source, run tests, add mutation operators, and submit pull requests.
---

# Contributing

dart_mutant is open source and welcomes contributions!

## Ways to Contribute

### Report Issues

Found a bug or have a feature request? [Open an issue](https://github.com/Nimblesite/dart_mutant/issues/new) with:

- dart_mutant version (`dart_mutant --version`)
- Dart version (`dart --version`)
- Steps to reproduce
- Expected vs actual behavior

### Improve Documentation

Documentation lives in the same repository. Improvements welcome:

- Fix typos or unclear explanations
- Add examples
- Translate to other languages

### Submit Code

Pull requests are welcome for:

- Bug fixes
- New mutation operators
- Performance improvements
- Report format enhancements

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- [Dart SDK](https://dart.dev/get-dart) 3.x
- Git

### Clone and Build

```bash
git clone https://github.com/Nimblesite/dart_mutant
cd dart_mutant
cargo build
```

### Run Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_arithmetic_mutations

# Run with output
cargo test -- --nocapture
```

### Run Lints

```bash
# Clippy (strictest settings)
cargo clippy

# Format check
cargo fmt --check
```

## Code Style

### Rust Guidelines

dart_mutant follows strict Rust conventions:

- **No `unwrap()`/`expect()`** in production code - use `?` operator
- **No `unsafe`** code
- **Pure functions** where possible
- **Small, focused functions** (max ~100 lines)
- **Descriptive names** - no single-letter variables except closures

### Error Handling

Use `anyhow::Result` for error propagation:

```rust
use anyhow::{Context, Result};

fn parse_file(path: &Path) -> Result<Mutations> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read file")?;

    parse_dart(&content)
        .context("Failed to parse Dart code")
}
```

### Documentation

Public APIs must have doc comments:

```rust
/// Finds all arithmetic mutations in the given AST.
///
/// # Arguments
/// * `ast` - The parsed Dart AST
///
/// # Returns
/// A vector of mutations for arithmetic operators
pub fn find_arithmetic_mutations(ast: &Tree) -> Vec<Mutation> {
    // ...
}
```

## Project Structure

```
src/
├── main.rs         # CLI entry point
├── cli/            # Argument parsing
├── parser/         # tree-sitter parsing
├── mutation/       # Mutation types and operators
├── runner/         # Test execution
├── report/         # Report generation
└── ai/             # AI integration

tests/
├── integration_*.rs   # Integration tests
└── fixtures/          # Test Dart projects
```

## Adding a Mutation Operator

1. **Define the operator** in `src/mutation/operators.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum MutationOperator {
    // ... existing operators
    MyNewOperator,
}
```

2. **Implement detection** in `src/parser/mod.rs`:

```rust
fn find_my_new_mutations(cursor: &mut TreeCursor, source: &str) -> Vec<Mutation> {
    // Walk AST and find mutation candidates
}
```

3. **Add tests** in `tests/integration_mutation.rs`:

```rust
#[test]
fn test_my_new_mutation() {
    let source = r#"
        void test() {
            // code that should be mutated
        }
    "#;

    let mutations = find_mutations(source);
    assert!(mutations.iter().any(|m| m.operator == MutationOperator::MyNewOperator));
}
```

4. **Update documentation** in `website/src/docs/operators.md`

## Pull Request Process

1. **Fork** the repository
2. **Create a branch**: `git checkout -b feature/my-feature`
3. **Make changes** with tests
4. **Run checks**: `cargo test && cargo clippy && cargo fmt --check`
5. **Commit** with descriptive message
6. **Push** and open PR

### PR Checklist

- [ ] Tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Documentation updated if needed
- [ ] CHANGELOG updated for user-facing changes

## Release Process

Releases are fully automated via GitHub Actions. Pushing a `vMAJOR.MINOR.PATCH`
tag is the only manual step — the workflow sets `Cargo.toml`'s version from the
tag (the tag is the source of truth; it is not committed back to `main`):

1. Update `CHANGELOG.md`
2. Create git tag: `git tag v0.5.1`
3. Push tag: `git push origin v0.5.1`

The release workflow then automatically:

- Builds binaries for macOS (Apple Silicon), Linux (x86_64), and Windows (x86_64)
- Publishes the GitHub release with those archives attached
- Updates the [Homebrew formula](https://github.com/Nimblesite/homebrew-tap)
  (macOS + Linux) and the [Scoop manifest](https://github.com/Nimblesite/scoop-bucket)
  (Windows) with the new version and checksums
- Redeploys the website

## Code of Conduct

Be respectful. We're all here to make better software.

## Questions?

- [GitHub Discussions](https://github.com/Nimblesite/dart_mutant/discussions)
- [Open an Issue](https://github.com/Nimblesite/dart_mutant/issues)

Thank you for contributing!
