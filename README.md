# dart_mutant

A blazingly fast mutation testing tool for Dart using tree-sitter AST parsing.

Unlike text-based mutation tools, dart_mutant understands your code structure and only creates valid, meaningful mutations that actually test your assertions.

## Quick Start

```bash
# Navigate to your Dart project
cd your_dart_project

# Run mutation testing
dart_mutant
```

That's it. The tool will:
1. Find all `.dart` files in `lib/`
2. Generate mutations (arithmetic, logical, comparison, null-safety, etc.)
3. Run `dart test` against each mutation
4. Generate an HTML report in `./mutation-reports/`

## Installation

```bash
# Clone and build
git clone https://github.com/user/dart_mutant
cd dart_mutant
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

## Usage Examples

### Basic Usage

```bash
# Run on current directory
dart_mutant

# Run on specific path
dart_mutant --path ./my_package

# Generate HTML report and open in browser
dart_mutant --html --open
```

### Performance Tuning

```bash
# Run 8 parallel test jobs (default: number of CPUs)
dart_mutant --parallel 8

# Test only 50 mutations for quick feedback
dart_mutant --sample 50

# Set timeout per mutation (default: 30s)
dart_mutant --timeout 15
```

### CI/CD Integration

```bash
# Fail if mutation score < 80%
dart_mutant --threshold 80

# Generate JUnit report for CI
dart_mutant --junit

# Quiet mode for cleaner logs
dart_mutant --quiet --threshold 80
```

### Incremental Testing

```bash
# Only test mutations in files changed since main
dart_mutant --incremental --base-ref main
```

### AI-Powered Mutations

```bash
# Use Claude to find high-value mutation locations
export ANTHROPIC_API_KEY=your_key
dart_mutant --ai anthropic

# Use local Ollama
dart_mutant --ai ollama --ollama-model codellama
```

### Filtering

```bash
# Only mutate specific files
dart_mutant --glob "lib/src/core/**/*.dart"

# Exclude patterns (generated code excluded by default)
dart_mutant --exclude "**/*.g.dart" --exclude "**/generated/**"
```

## Output

### HTML Report

A beautiful dark-themed HTML report showing:
- Overall mutation score
- Per-file breakdown
- Survived mutations (your tests didn't catch these!)
- Killed mutations (good - your tests caught the bug)

### JSON Report (Stryker-compatible)

```bash
dart_mutant --json
```

Generates a Stryker-compatible JSON report for integration with mutation testing dashboards.

## Mutation Operators

dart_mutant generates these mutation types:

| Category | Examples |
|----------|----------|
| Arithmetic | `+` → `-`, `*` → `/`, `++` → `--` |
| Comparison | `>` → `>=`, `==` → `!=`, `<` → `<=` |
| Logical | `&&` → `\|\|`, `!` → (removed) |
| Null Safety | `??` → (removed), `?.` → `.` |
| Control Flow | `if(x)` → `if(true)`, `if(false)` |
| Literals | `true` → `false`, `"string"` → `""` |

## Interpreting Results

- **Mutation Score**: Percentage of mutations killed by tests
- **Killed**: Your tests caught the mutation (good!)
- **Survived**: Your tests didn't detect the change (add more tests!)
- **Timeout**: Mutation caused infinite loop (counts as killed)
- **Error**: Mutation caused compile error (excluded from score)

A mutation score of 80%+ indicates strong test coverage. Survived mutations show exactly where your tests are weak.

## Example Output

```
    ██████╗  █████╗ ██████╗ ████████╗    ███╗   ███╗██╗   ██╗████████╗ █████╗ ███╗   ██╗████████╗
    ██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝    ████╗ ████║██║   ██║╚══██╔══╝██╔══██╗████╗  ██║╚══██╔══╝
    ██║  ██║███████║██████╔╝   ██║       ██╔████╔██║██║   ██║   ██║   ███████║██╔██╗ ██║   ██║
    ██║  ██║██╔══██║██╔══██╗   ██║       ██║╚██╔╝██║██║   ██║   ██║   ██╔══██║██║╚██╗██║   ██║
    ██████╔╝██║  ██║██║  ██║   ██║       ██║ ╚═╝ ██║╚██████╔╝   ██║   ██║  ██║██║ ╚████║   ██║
    ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝       ╚═╝     ╚═╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝

  Discovering Dart files...
  Found 12 files, 847 mutation candidates

  Running mutation tests [████████████████████████████████████████] 847/847

  ═══════════════════════════════════════════════════════════════════════════════
                              MUTATION TESTING COMPLETE
  ═══════════════════════════════════════════════════════════════════════════════

  Mutation Score: 87.2%
  ████████████████████████████████████░░░░░░

  Killed:    739    Survived:  108    Timeout:   0    Error:  0

  Report: ./mutation-reports/mutation-report.html
```

## License

MIT
