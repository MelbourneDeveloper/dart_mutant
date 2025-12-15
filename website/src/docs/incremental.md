---
layout: docs.njk
title: Incremental Testing
---

# Incremental Testing

Test only mutations in files that have changed - perfect for PR checks and fast feedback.

## Basic Usage

```bash
# Test only files changed since main
dart_mutant --incremental --base-ref main
```

## How It Works

1. **Git Diff**: dart_mutant runs `git diff --name-only base_ref HEAD`
2. **Filter**: Only `.dart` files in `lib/` are considered
3. **Mutate**: Mutations are generated only for changed files
4. **Test**: Standard mutation testing on the subset

## Options

### Base Reference

Compare against any git ref:

```bash
# Compare against main branch
dart_mutant --incremental --base-ref main

# Compare against specific branch
dart_mutant --incremental --base-ref origin/develop

# Compare against tag
dart_mutant --incremental --base-ref v1.2.0

# Compare against specific commit
dart_mutant --incremental --base-ref abc1234

# Compare against previous commit
dart_mutant --incremental --base-ref HEAD~1
```

### Default Base

If `--base-ref` is not specified, dart_mutant uses:

1. `main` if it exists
2. `master` if main doesn't exist
3. Error if neither exists

## CI Integration

### GitHub Actions

```yaml
name: Mutation Test (Incremental)

on:
  pull_request:
    branches: [main]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0 # Full history for git diff

      - name: Setup Dart
        uses: dart-lang/setup-dart@v1

      - name: Install dart_mutant
        run: |
          curl -L https://github.com/user/dart_mutant/releases/latest/download/dart_mutant-x86_64-unknown-linux-gnu.tar.gz | tar xz
          sudo mv dart_mutant /usr/local/bin/

      - name: Run incremental mutation tests
        run: |
          dart pub get
          dart_mutant --incremental --base-ref origin/main --threshold 70
```

> **Important**: Use `fetch-depth: 0` to get full git history for accurate diffs.

### GitLab CI

```yaml
mutation-test:
  script:
    - dart_mutant --incremental --base-ref origin/$CI_MERGE_REQUEST_TARGET_BRANCH_NAME
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
```

## Strategies

### PR Checks: Incremental Only

Fast feedback on changed code:

```yaml
# On PR
dart_mutant --incremental --threshold 70
```

### Main Branch: Full Run

Comprehensive testing on merge:

```yaml
# On push to main
dart_mutant --threshold 80
```

### Hybrid Approach

Incremental for quick feedback, full run nightly:

```yaml
# PR workflow
name: PR Mutation Test
on: pull_request
jobs:
  mutation:
    runs-on: ubuntu-latest
    steps:
      - run: dart_mutant --incremental --threshold 60

# Nightly workflow
name: Nightly Mutation Test
on:
  schedule:
    - cron: '0 2 * * *'
jobs:
  mutation:
    runs-on: ubuntu-latest
    steps:
      - run: dart_mutant --threshold 80 --html
```

## Performance Comparison

| Mode                   | Files | Mutations | Time   |
| ---------------------- | ----- | --------- | ------ |
| Full                   | 50    | 2,000     | 15 min |
| Incremental (3 files)  | 3     | 120       | 1 min  |
| Incremental (10 files) | 10    | 400       | 4 min  |

## Limitations

1. **New bugs in unchanged code**: Incremental mode won't catch regressions in files you didn't change
2. **Cross-file dependencies**: Changing a function signature might break tests for unchanged files
3. **Git required**: Must be in a git repository

## Best Practices

### 1. Use Full Runs Regularly

Don't rely solely on incremental mode:

```bash
# Weekly full mutation test
dart_mutant --threshold 80
```

### 2. Lower Threshold for Incremental

Be lenient on incremental (new code) but strict on full runs:

```bash
# Incremental - 70% threshold
dart_mutant --incremental --threshold 70

# Full - 80% threshold
dart_mutant --threshold 80
```

### 3. Fetch Full History in CI

Shallow clones break incremental mode:

```yaml
# GitHub Actions
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
```

## Troubleshooting

### "No changed files found"

```bash
# Check what git sees
git diff --name-only main HEAD

# Ensure base ref exists
git branch -a | grep main
```

### "fatal: bad revision"

```bash
# Fetch the base branch
git fetch origin main:main

# Or use origin/main
dart_mutant --incremental --base-ref origin/main
```

## Next Steps

- [CI/CD Integration](/docs/ci/) - Full CI setup guide
- [Filtering](/docs/filtering/) - Additional file filtering options
