---
layout: docs.njk
title: Reports
---

# Reports

dart_mutant generates detailed reports in multiple formats.

## HTML Report

The default report format - a beautiful dark-themed interactive report.

```bash
dart_mutant --html

# Open in browser automatically
dart_mutant --html --open
```

### Report Contents

The HTML report includes:

- **Summary Dashboard**: Overall mutation score with visual progress bar
- **File Breakdown**: Per-file scores and mutation counts
- **Mutation Details**: Click to expand each file and see individual mutations
- **Status Indicators**: Color-coded killed/survived/timeout/error status

### Output Location

```
./mutation-reports/
├── mutation-report.html
└── assets/
    └── style.css
```

## JSON Report

Stryker-compatible JSON format for integration with mutation testing dashboards.

```bash
dart_mutant --json
```

### Schema

```json
{
  "schemaVersion": "1",
  "thresholds": {
    "high": 80,
    "low": 60
  },
  "projectRoot": "/path/to/project",
  "files": {
    "lib/src/calculator.dart": {
      "language": "dart",
      "mutants": [
        {
          "id": "1",
          "mutatorName": "ArithmeticOperator",
          "replacement": "-",
          "location": {
            "start": { "line": 5, "column": 12 },
            "end": { "line": 5, "column": 13 }
          },
          "status": "Killed"
        }
      ]
    }
  }
}
```

### Stryker Dashboard Integration

Upload results to the [Stryker Dashboard](https://dashboard.stryker-mutator.io/):

```bash
dart_mutant --json

# Upload to dashboard
curl -X PUT \
  -H "Content-Type: application/json" \
  -H "Host: dashboard.stryker-mutator.io" \
  -d @mutation-reports/mutation-report.json \
  "https://dashboard.stryker-mutator.io/api/reports/github.com/user/project/main"
```

## JUnit XML Report

Standard JUnit format for CI/CD test result integration.

```bash
dart_mutant --junit
```

### Output

```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="Mutation Testing" tests="847" failures="108" time="245.3">
    <testcase name="calculator.dart:5 (+) -> (-)" classname="ArithmeticOperator">
      <!-- Killed mutation - passes -->
    </testcase>
    <testcase name="validator.dart:12 (>=) -> (>)" classname="ComparisonOperator">
      <failure message="Mutation survived">
        Tests did not detect mutation: >= changed to > at line 12
      </failure>
    </testcase>
  </testsuite>
</testsuites>
```

### CI Integration

Most CI systems automatically parse JUnit XML:

**GitHub Actions:**

```yaml
- uses: dorny/test-reporter@v1
  with:
    name: Mutation Tests
    path: mutation-reports/junit.xml
    reporter: java-junit
```

**GitLab CI:**

```yaml
artifacts:
  reports:
    junit: mutation-reports/junit.xml
```

## AI Report

Generate a markdown report optimized for AI assistants:

```bash
dart_mutant --ai-report
```

### Output

Creates `mutation-report-ai.md` - paste it directly into Claude, ChatGPT, or Copilot to have AI write your missing tests.

**Contents:**

- **Summary**: Mutation score, killed/survived counts
- **Surviving mutants by file**: Worst files first
- **Mutation details**: Exact line, original → mutated code
- **Test hints**: Specific guidance for each mutation type
- **Quick reference**: `file:line` format for easy navigation

### Example Workflow

```bash
# Generate the AI report
dart_mutant --ai-report

# Paste into your AI assistant:
```

```
Here's my mutation report. Write tests to kill these surviving mutants:

[paste mutation-report-ai.md contents]
```

The AI gets structured data about exactly what mutations survived and specific hints about what tests would catch them.

## Multiple Formats

Generate all formats at once:

```bash
dart_mutant --html --json --junit --ai-report
```

Output:

```
./mutation-reports/
├── mutation-report.html
├── mutation-report.json
├── mutation-report-ai.md
└── junit.xml
```

## Custom Output Directory

Specify a custom output location:

```bash
dart_mutant --html --output-dir ./reports/mutations
```

## Console Output

The default console output shows real-time progress:

```
  Discovering Dart files...
  Found 12 files, 847 mutation candidates

  Running mutation tests [████████████████████████████████████████] 847/847

  ═══════════════════════════════════════════════════════════════════════════════
                              MUTATION TESTING COMPLETE
  ═══════════════════════════════════════════════════════════════════════════════

  Mutation Score: 87.2%
  ████████████████████████████████████░░░░░░

  Killed:    739    Survived:  108    Timeout:   0    Error:  0
```

### Quiet Mode

For CI environments, use quiet mode:

```bash
dart_mutant --quiet

# Output:
# Mutation Score: 87.2% (739 killed, 108 survived)
```

### Verbose Mode

For debugging, use verbose mode:

```bash
dart_mutant --verbose

# Shows each mutation being tested
```

## Next Steps

- [Interpreting Results](/docs/interpreting/) - Understanding your reports
- [CI/CD Integration](/docs/ci/) - Automate report generation
