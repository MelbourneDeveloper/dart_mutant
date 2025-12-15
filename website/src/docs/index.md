---
layout: docs.njk
title: Introduction
---

# Introduction to dart_mutant

**dart_mutant** is a blazingly fast mutation testing tool for Dart that uses tree-sitter AST parsing to generate syntactically valid mutations.

## What is Mutation Testing?

Mutation testing evaluates your test suite's quality by introducing deliberate bugs (mutants) into your source code and checking if your tests catch them.

> **Key insight:** Code coverage only tells you what code _runs_ during tests. Mutation testing tells you what code is actually _verified_ by assertions.

### How It Works

1. **Generate Mutants**: dart_mutant parses your code and creates small modifications (e.g., `+` → `-`, `true` → `false`)
2. **Run Tests**: For each mutation, your test suite runs
3. **Analyze Results**:
   - **Killed** = tests failed (good - they caught the bug)
   - **Survived** = tests passed (bad - they missed the bug)

The **mutation score** (`killed / total`) measures your test suite's fault-detection capability.

## Why dart_mutant?

### AST-Based Precision

Unlike regex-based mutation tools, dart_mutant:

- Parses code into an Abstract Syntax Tree
- Only creates syntactically valid mutations
- Never mutates comments, strings (unless intentional), or generated code
- Understands Dart-specific constructs like null-safety operators

### Performance

- Written in Rust for maximum speed
- Parallel test execution
- Incremental testing (only test changed code)
- Smart sampling for large codebases

### Beautiful Reports

- Dark-themed HTML reports
- Per-file mutation score breakdown
- Click to see exactly which mutations survived
- Stryker-compatible JSON format

## Quick Example

```bash
# In your Dart project directory
dart_mutant

# Output:
#   Found 12 files, 847 mutation candidates
#   Running mutation tests [████████████████████] 847/847
#
#   Mutation Score: 87.2%
#   Killed: 739  Survived: 108
```

## Next Steps

- [Installation](/docs/installation/) - Get dart_mutant running
- [Quick Start](/docs/quickstart/) - Your first mutation test run
- [CLI Options](/docs/cli/) - All command-line options
