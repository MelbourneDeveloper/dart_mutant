---
layout: docs.njk
title: Quick Start
---

# Quick Start

Get mutation testing running on your Dart project in under a minute.

## Basic Usage

Navigate to your Dart project and run:

```bash
cd your_dart_project
dart_mutant
```

That's it! dart_mutant will:

1. **Discover** all `.dart` files in `lib/`
2. **Parse** each file and find mutation candidates
3. **Test** each mutation by running `dart test`
4. **Generate** an HTML report in `./mutation-reports/`

## Sample Output

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

## View the Report

Open the HTML report in your browser:

```bash
dart_mutant --html --open
```

Or manually open `./mutation-reports/mutation-report.html`.

## Quick Feedback Mode

Testing all mutations can take a while. For quick feedback during development:

```bash
# Test only 50 random mutations
dart_mutant --sample 50

# Shorter timeout for faster iteration
dart_mutant --timeout 10
```

## Target Specific Files

Focus on files you're working on:

```bash
# Only mutate files matching pattern
dart_mutant --glob "lib/src/core/**/*.dart"

# Only mutate files changed since main
dart_mutant --incremental --base-ref main
```

## Common Options

| Option           | Description                          |
| ---------------- | ------------------------------------ |
| `--path <dir>`   | Project directory (default: current) |
| `--parallel <n>` | Number of parallel test jobs         |
| `--sample <n>`   | Test only n random mutations         |
| `--timeout <s>`  | Per-mutation timeout in seconds      |
| `--html`         | Generate HTML report                 |
| `--open`         | Open report in browser               |
| `--quiet`        | Minimal output                       |

## Next Steps

- [CLI Options](/docs/cli/) - Full list of options
- [Mutation Operators](/docs/operators/) - What gets mutated
- [Interpreting Results](/docs/interpreting/) - Understanding your score
