# dart_mutant

Fast mutation testing for Dart. AST-based. No invalid mutations.

## Quick Start

```bash
cd your_dart_project
dart_mutant
```

Done. Check `./mutation-reports/mutation-report.html` for results.

## Installation

```bash
git clone https://github.com/user/dart_mutant
cd dart_mutant
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

## Key Options

```bash
# Parallel execution (default: CPU count)
dart_mutant --parallel 8

# Quick feedback with sampling
dart_mutant --sample 50

# CI threshold - fail if score < 80%
dart_mutant --threshold 80

# Incremental - only test changed files
dart_mutant --incremental --base-ref main
```

## AI Report

Generate a markdown report optimized for AI assistants:

```bash
dart_mutant --ai-report
```

This creates `mutation-report-ai.md` - paste it directly into Claude, ChatGPT, or Copilot. The AI gets:

- **Surviving mutants grouped by file** (worst files first)
- **Exact mutations**: what changed, line numbers
- **Test hints**: specific guidance for each mutation type
- **Copy-paste references**: `file:line` format for quick navigation

Use it to have AI write the missing tests for you:

```
Here's my mutation report. Write tests to kill these surviving mutants:

[paste mutation-report-ai.md]
```

## Report Types

| Flag | Output | Use Case |
|------|--------|----------|
| `--html` | Beautiful HTML dashboard | Human review |
| `--json` | Stryker-compatible JSON | CI dashboards |
| `--ai-report` | LLM-optimized markdown | AI-assisted test writing |

## Mutation Operators

| Category | Mutations |
|----------|-----------|
| Arithmetic | `+` ↔ `-`, `*` ↔ `/`, `++` ↔ `--` |
| Comparison | `>` ↔ `>=`, `<` ↔ `<=`, `==` ↔ `!=` |
| Logical | `&&` ↔ `\|\|`, `!` removal |
| Null Safety | `??` removal, `?.` → `.` |
| Control Flow | `if(x)` → `if(true/false)` |
| Literals | `true` ↔ `false`, `"str"` → `""` |

## Understanding Results

- **Killed**: Test caught the mutation (good)
- **Survived**: Test missed it (write more tests!)
- **Timeout**: Infinite loop (counts as killed)

80%+ mutation score = strong coverage. Survived mutants show exactly where tests are weak.

## AI-Powered Mutation Discovery

```bash
# Use Claude to find high-value mutation spots
export ANTHROPIC_API_KEY=your_key
dart_mutant --ai anthropic

# Or local Ollama
dart_mutant --ai ollama --ollama-model codellama
```

## License

MIT
