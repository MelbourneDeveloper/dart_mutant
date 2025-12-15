# dart_mutant

**Hunt down weak tests.** Blazingly fast mutation testing for Dart.

Code coverage lies. A line can be "covered" without being tested. dart_mutant injects bugs into your code to prove your tests actually catch them.

AST-based. Zero false positives. Every surviving mutant is a real gap.

## Quick Start

```bash
cd your_dart_project
dart_mutant
```

Open `./mutation-reports/mutation-report.html` to see what your tests missed.

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

Generate a report optimized for AI assistants:

```bash
dart_mutant --ai-report
```

Creates `mutation-report-ai.md` - paste directly into Claude, ChatGPT, or Copilot:

- **Surviving mutants grouped by file** (worst first)
- **Exact mutations** with line numbers
- **Test hints** for each mutation type
- **Copy-paste references** in `file:line` format

Have AI write your missing tests:

```
Here's my mutation report. Write tests to kill these surviving mutants:

[paste mutation-report-ai.md]
```

## Report Formats

| Flag | Output | Use Case |
|------|--------|----------|
| `--html` | Interactive HTML dashboard | Human review |
| `--json` | Stryker-compatible JSON | CI dashboards |
| `--ai-report` | LLM-optimized markdown | AI-assisted test writing |
| `--junit` | JUnit XML | CI test results |

## Mutation Operators

| Category | Mutations |
|----------|-----------|
| Arithmetic | `+` ↔ `-`, `*` ↔ `/`, `++` ↔ `--` |
| Comparison | `>` ↔ `>=`, `<` ↔ `<=`, `==` ↔ `!=` |
| Logical | `&&` ↔ `\|\|`, `!` removal |
| Null Safety | `??` removal, `?.` → `.` |
| Control Flow | `if(x)` → `if(true/false)` |
| Literals | `true` ↔ `false`, `"str"` → `""` |

## Results

- **Killed**: Test caught the mutation (good)
- **Survived**: Test missed it (write more tests!)
- **Timeout**: Infinite loop (counts as killed)

**80%+ mutation score = strong test suite.** Survived mutants show exactly where your tests are weak.

## AI-Powered Mutation Discovery

```bash
# Claude finds high-value mutation spots
export ANTHROPIC_API_KEY=your_key
dart_mutant --ai anthropic

# Local Ollama
dart_mutant --ai ollama --ollama-model codellama
```

## License

MIT
