---
layout: docs.njk
title: CLI Options
---

# CLI Options

Complete reference for all dart_mutant command-line options.

## Usage

```bash
dart_mutant [OPTIONS]
```

## General Options

| Option         | Short | Description                          | Default           |
| -------------- | ----- | ------------------------------------ | ----------------- |
| `--path <DIR>` | `-p`  | Path to Dart project                 | Current directory |
| `--help`       | `-h`  | Show help message                    |                   |
| `--version`    | `-V`  | Show version                         |                   |
| `--quiet`      | `-q`  | Minimal output                       | false             |
| `--verbose`    | `-v`  | Detailed output                      | false             |
| `--dry-run`    |       | Show mutations without running tests | false             |

## Test Execution

| Option             | Description                     | Default   |
| ------------------ | ------------------------------- | --------- |
| `--parallel <N>`   | Number of parallel test jobs    | CPU count |
| `--timeout <SECS>` | Per-mutation timeout in seconds | 30        |
| `--sample <N>`     | Test only N random mutations    | All       |

### Examples

```bash
# Run 8 parallel jobs
dart_mutant --parallel 8

# Quick feedback with 50 mutations, 10s timeout
dart_mutant --sample 50 --timeout 10

# Preview mutations without running tests
dart_mutant --dry-run
```

## Filtering

| Option                | Description                                 |
| --------------------- | ------------------------------------------- |
| `--glob <PATTERN>`    | Only mutate files matching glob pattern     |
| `--exclude <PATTERN>` | Exclude files matching pattern (can repeat) |

### Default Exclusions

dart_mutant automatically excludes:

- `**/*.g.dart` (generated)
- `**/*.freezed.dart` (freezed)
- `**/*.gr.dart` (auto_route)
- `**/generated/**`

### Examples

```bash
# Only mutate core library
dart_mutant --glob "lib/src/core/**/*.dart"

# Exclude specific directories
dart_mutant --exclude "**/legacy/**" --exclude "**/deprecated/**"
```

## Incremental Mode

| Option             | Description                          |
| ------------------ | ------------------------------------ | ---- |
| `--incremental`    | Only test mutations in changed files |
| `--base-ref <REF>` | Git ref to compare against           | main |

### Examples

```bash
# Test only files changed since main
dart_mutant --incremental --base-ref main

# Compare against specific commit
dart_mutant --incremental --base-ref HEAD~5
```

## Output & Reports

| Option               | Description                      | Output Path                             |
| -------------------- | -------------------------------- | --------------------------------------- |
| `--html`             | Generate HTML report             | `mutation-reports/mutation-report.html` |
| `--json`             | Generate Stryker-compatible JSON | `mutation-reports/mutation-report.json` |
| `--junit`            | Generate JUnit XML               | `mutation-reports/junit.xml`            |
| `--open`             | Open HTML report in browser      |                                         |
| `--output-dir <DIR>` | Custom output directory          | `mutation-reports/`                     |

### Examples

```bash
# Generate all report formats
dart_mutant --html --json --junit

# Generate and open HTML report
dart_mutant --html --open

# Custom output directory
dart_mutant --html --output-dir ./reports
```

## CI/CD Options

| Option                  | Description                            |
| ----------------------- | -------------------------------------- |
| `--threshold <PERCENT>` | Fail if mutation score below threshold |

### Examples

```bash
# Fail build if score < 80%
dart_mutant --threshold 80

# CI-friendly output
dart_mutant --quiet --threshold 80 --junit
```

## AI-Powered Mutations

| Option                   | Description                                  |
| ------------------------ | -------------------------------------------- | ---------------------- |
| `--ai <PROVIDER>`        | AI provider: `anthropic`, `openai`, `ollama` |
| `--ollama-model <MODEL>` | Ollama model name                            | codellama              |
| `--ollama-url <URL>`     | Ollama API URL                               | http://localhost:11434 |

### Environment Variables

- `ANTHROPIC_API_KEY` - For Claude
- `OPENAI_API_KEY` - For GPT models

### Examples

```bash
# Use Claude for mutation suggestions
export ANTHROPIC_API_KEY=your_key
dart_mutant --ai anthropic

# Use local Ollama
dart_mutant --ai ollama --ollama-model codellama
```

## Exit Codes

| Code | Meaning                                            |
| ---- | -------------------------------------------------- |
| 0    | Success (score >= threshold or no threshold set)   |
| 1    | Mutation score below threshold                     |
| 2    | Error (invalid arguments, project not found, etc.) |

## Configuration File (Future)

A `dart_mutant.yaml` configuration file is planned for future releases.

## Next Steps

- [Quick Start](/docs/quickstart/) - Basic usage examples
- [Mutation Operators](/docs/operators/) - What gets mutated
- [CI/CD Integration](/docs/ci/) - CI pipeline setup
