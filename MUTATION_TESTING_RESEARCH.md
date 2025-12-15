# Dart Mutation Testing Framework Research

## Executive Summary

This document presents research findings on implementing a mutation testing framework for Dart. After evaluating multiple approaches, the key decision is whether to:
1. **Extend the existing `mutation_test` package** (recommended for quick wins)
2. **Build a new AST-based tool in Rust** (recommended for long-term quality)
3. **Build a new AST-based tool in Go** (alternative to Rust)

---

## What is Mutation Testing?

Mutation testing evaluates test suite quality by introducing deliberate bugs (mutants) into source code and verifying tests detect them.

### Key Concepts
- **Mutant**: A small code modification (e.g., `+` → `-`, `true` → `false`)
- **Killed**: Test fails after mutation (good - test caught the bug)
- **Survived**: Test passes after mutation (bad - test missed the bug)
- **Mutation Score**: `killed / total mutants` (higher = better tests)
- **Equivalent Mutant**: Syntactically different but functionally identical (false positive)

### Why It Matters
- Code coverage only measures execution, not verification
- A line can be "covered" without meaningful assertions
- Mutation score measures actual fault-detection capability

---

## Mutation Operators (What to Mutate)

### Arithmetic Operators
| Original | Mutated |
|----------|---------|
| `+` | `-`, `*`, `/` |
| `-` | `+`, `*`, `/` |
| `*` | `/`, `+`, `-` |
| `/` | `*`, `+`, `-` |
| `%` | `*` |

### Comparison Operators
| Original | Mutated |
|----------|---------|
| `<` | `<=`, `>=`, `>` |
| `>` | `>=`, `<=`, `<` |
| `<=` | `<`, `>=`, `>` |
| `>=` | `>`, `<=`, `<` |
| `==` | `!=` |
| `!=` | `==` |

### Logical Operators
| Original | Mutated |
|----------|---------|
| `&&` | `\|\|` |
| `\|\|` | `&&` |
| `!expr` | `expr` |

### Boolean Literals
| Original | Mutated |
|----------|---------|
| `true` | `false` |
| `false` | `true` |

### Control Flow
- Empty `if`/`else` bodies
- Remove `break`/`continue`
- Swap `case` fallthrough

### Dart-Specific
- `??` → `&&` (null coalescing)
- `?.` → `.` (null-aware access)
- Remove `async`/`await`
- Swap `Future.value()` / `Future.error()`

---

## Existing Solution: `mutation_test` Package

**Repository**: [domohuhn/mutation-test](https://github.com/domohuhn/mutation-test)
**Pub.dev**: [mutation_test](https://pub.dev/packages/mutation_test)

### Strengths
- **160/160 pub points** (perfect score)
- **Active development** (v1.7.1 released 28 days ago)
- **Language agnostic** (XML-configured rules)
- **Multiple report formats**: HTML, JUnit/XUnit, Markdown, XML
- **Coverage integration**: Excludes untested code via lcov
- **Easy to use**: Zero-config for Dart projects

### How It Works
```
1. Text-based mutations via regex/literal replacement
2. Apply mutation → Run `dart test` → Check exit code → Revert
3. Generate reports with mutation score
```

### Limitations
- **No AST awareness**: Mutations are text-based, can't understand syntax
- **False positives**: May mutate comments, strings, or invalid locations
- **No semantic understanding**: Can't avoid equivalent mutants
- **Performance**: Sequential execution, no parallelization
- **No incremental caching**: Re-runs all tests for each mutation

### Extension Opportunities
1. Add more Dart-specific mutation rules
2. Implement parallel mutation execution
3. Add mutation caching/incremental testing
4. Integrate with Dart analyzer for smarter mutations
5. Add IDE integration (VSCode extension)

---

## Alternative: Build AST-Based Tool

### Why AST-Based?
- **Precision**: Only mutate valid code locations
- **No false positives**: Skip comments, strings, generated code
- **Smarter mutations**: Use type information for context-aware mutations
- **Equivalent mutant detection**: Some can be detected statically

### Option A: Rust with tree-sitter

**Key Libraries**:
- [tree-sitter-dart](https://crates.io/crates/tree-sitter-dart) - Dart grammar for tree-sitter
- [tree-sitter-edit](https://crates.io/crates/tree-sitter-edit) - Modify parsed trees

**Pros**:
- Excellent performance
- tree-sitter provides incremental parsing
- Rich ecosystem for CLI tools
- Cross-platform binaries

**Cons**:
- tree-sitter doesn't natively support code modification
- Need to build edit/rewrite layer
- Steeper learning curve

**Architecture**:
```
1. Parse Dart files with tree-sitter-dart
2. Walk AST to identify mutation candidates
3. Generate mutated source text
4. Run `dart test` for each mutant
5. Collect results and generate reports
```

### Option B: Go with go/ast style

**Inspiration**: [go-mutesting](https://github.com/zimmski/go-mutesting)

**Approach**:
- Use tree-sitter-dart (has Go bindings)
- Or shell out to Dart analyzer for AST

**Pros**:
- Simpler concurrency model
- Easy to build CLI tools
- Good cross-platform support

**Cons**:
- Less mature Dart tooling
- May need to shell out to Dart

### Option C: Dart with analyzer package

**Key Libraries**:
- `analyzer` - Parse Dart into AST
- `analyzer_plugin` - IDE integration
- `source_gen` / `code_builder` - Code generation

**Pros**:
- Native Dart, same ecosystem
- Full type information available
- Can integrate with `dart analyze`

**Cons**:
- `analyzer` package doesn't support modification
- Would need to build text-diff layer
- Potentially slower than native tools

---

## Architecture Patterns from Existing Tools

### Stryker (JS/TS/C#/Scala)
- **Mutation Schemata**: All mutants compiled into one binary with runtime flags
- **Pros**: Single compilation, fast switching
- **Cons**: Requires language-specific compilation hooks

### PITest (Java)
- **Bytecode Mutation**: Mutates compiled .class files
- **Multi-process**: Main controller + minion workers
- **Test Selection**: Only runs tests that cover mutated code
- **Pros**: Very fast, no recompilation
- **Cons**: Bytecode doesn't map clearly to source

### cargo-mutants (Rust)
- **Source Mutation**: Modifies source, recompiles
- **Parallel Execution**: Multiple mutations tested concurrently
- **Incremental**: Caches compilation artifacts

### go-mutesting (Go)
- **AST Mutation**: Uses Go's native AST packages
- **Mutator Interface**: Pluggable mutation operators
- **Blacklist**: MD5-based false positive tracking

---

## Recommendation

### Short-term: Extend `mutation_test`

The existing package is well-maintained and functional. Recommended improvements:
1. **Add Dart-specific mutation rules** (null-safety operators, async/await)
2. **Parallel execution** (run multiple mutants concurrently)
3. **Smarter filtering** (use Dart analyzer to skip invalid mutations)
4. **VSCode extension** (show surviving mutants inline)

### Long-term: Build AST-based tool in Rust

For a production-quality tool:
1. Use `tree-sitter-dart` for parsing
2. Build mutation operators as AST visitors
3. Generate source diffs for each mutation
4. Parallel test execution with process pools
5. Incremental testing (only re-test affected code)
6. HTML/JSON reports compatible with Stryker report format

---

## Implementation Phases (if building new)

### Phase 1: Core Parser & Mutations
- Set up Rust project with tree-sitter-dart
- Implement basic mutation operators (arithmetic, comparison, boolean)
- Generate mutated source files
- Run `dart test` and capture results

### Phase 2: Performance
- Parallel mutation testing
- Test selection (only run relevant tests)
- Incremental caching

### Phase 3: Reporting & Integration
- HTML reports (Stryker-compatible format)
- JUnit XML for CI
- VSCode extension for inline results

### Phase 4: Advanced Features
- Equivalent mutant detection
- Mutation schemata (compile once)
- Coverage-based mutation filtering

---

## Sources

- [Mutation Testing - Wikipedia](https://en.wikipedia.org/wiki/Mutation_testing)
- [Stryker Mutator](https://stryker-mutator.io/)
- [PITest](https://pitest.org/)
- [cargo-mutants](https://mutants.rs/)
- [go-mutesting](https://github.com/zimmski/go-mutesting)
- [mutation_test (Dart)](https://pub.dev/packages/mutation_test)
- [tree-sitter-dart](https://crates.io/crates/tree-sitter-dart)
- [Dart Analyzer Plugins](https://dart.dev/tools/analyzer-plugins)
- [PITest: So you want to build a mutation testing system](https://github.com/hcoles/pitest/blob/master/so_you_want_to_build_mutation_testing_system.md)
