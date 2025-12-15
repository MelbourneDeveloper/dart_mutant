---
layout: docs.njk
title: CI/CD Integration
---

# CI/CD Integration

Integrate dart_mutant into your continuous integration pipeline to enforce mutation score thresholds.

## GitHub Actions

```yaml
name: Mutation Testing

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Dart
        uses: dart-lang/setup-dart@v1
        with:
          sdk: stable

      - name: Install dependencies
        run: dart pub get

      - name: Install dart_mutant
        run: |
          curl -L https://github.com/user/dart_mutant/releases/latest/download/dart_mutant-x86_64-unknown-linux-gnu.tar.gz | tar xz
          sudo mv dart_mutant /usr/local/bin/

      - name: Run mutation tests
        run: dart_mutant --threshold 80 --junit --quiet

      - name: Upload mutation report
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: mutation-report
          path: mutation-reports/
```

## GitLab CI

```yaml
mutation-test:
  image: dart:stable
  stage: test
  before_script:
    - curl -L https://github.com/user/dart_mutant/releases/latest/download/dart_mutant-x86_64-unknown-linux-gnu.tar.gz | tar xz
    - mv dart_mutant /usr/local/bin/
    - dart pub get
  script:
    - dart_mutant --threshold 80 --junit --quiet
  artifacts:
    when: always
    paths:
      - mutation-reports/
    reports:
      junit: mutation-reports/junit.xml
```

## CircleCI

```yaml
version: 2.1

jobs:
  mutation-test:
    docker:
      - image: dart:stable
    steps:
      - checkout
      - run:
          name: Install dart_mutant
          command: |
            curl -L https://github.com/user/dart_mutant/releases/latest/download/dart_mutant-x86_64-unknown-linux-gnu.tar.gz | tar xz
            mv dart_mutant /usr/local/bin/
      - run:
          name: Install dependencies
          command: dart pub get
      - run:
          name: Run mutation tests
          command: dart_mutant --threshold 80 --junit --quiet
      - store_artifacts:
          path: mutation-reports/
      - store_test_results:
          path: mutation-reports/

workflows:
  test:
    jobs:
      - mutation-test
```

## Azure Pipelines

```yaml
trigger:
  - main

pool:
  vmImage: 'ubuntu-latest'

steps:
  - task: UseDartSDK@0
    inputs:
      version: 'stable'

  - script: |
      curl -L https://github.com/user/dart_mutant/releases/latest/download/dart_mutant-x86_64-unknown-linux-gnu.tar.gz | tar xz
      sudo mv dart_mutant /usr/local/bin/
    displayName: 'Install dart_mutant'

  - script: dart pub get
    displayName: 'Install dependencies'

  - script: dart_mutant --threshold 80 --junit --quiet
    displayName: 'Run mutation tests'

  - task: PublishTestResults@2
    condition: always()
    inputs:
      testResultsFormat: 'JUnit'
      testResultsFiles: 'mutation-reports/junit.xml'

  - task: PublishBuildArtifacts@1
    condition: always()
    inputs:
      pathtoPublish: 'mutation-reports/'
      artifactName: 'mutation-report'
```

## CI Options

### Threshold Enforcement

Fail the build if mutation score drops below target:

```bash
# Fail if score < 80%
dart_mutant --threshold 80

# Exit code: 0 = passed, 1 = below threshold
```

### JUnit Report

Generate JUnit XML for CI test result integration:

```bash
dart_mutant --junit
# Creates: mutation-reports/junit.xml
```

### Quiet Mode

Minimal output for cleaner CI logs:

```bash
dart_mutant --quiet --threshold 80
```

### Incremental Mode

Only test mutations in changed files (great for PRs):

```bash
# Test mutations only in files changed vs main
dart_mutant --incremental --base-ref origin/main
```

### Sampling

For large codebases, test a subset for faster feedback:

```bash
# Test 100 random mutations
dart_mutant --sample 100 --threshold 75
```

## Best Practices

### 1. Start with a Low Threshold

Begin with a low threshold (e.g., 50%) and gradually increase as you improve your tests:

```yaml
# Week 1
dart_mutant --threshold 50

# After improvements
dart_mutant --threshold 70

# Target
dart_mutant --threshold 80
```

### 2. Use Incremental Mode for PRs

Full mutation testing on every PR can be slow. Use incremental mode:

```yaml
# On PR
dart_mutant --incremental --base-ref ${% raw %}{{ github.base_ref }}{% endraw %}

# On main branch merge - full run
dart_mutant --threshold 80
```

### 3. Cache Dart Dependencies

Speed up CI by caching:

```yaml
- uses: actions/cache@v3
  with:
    path: ~/.pub-cache
    key: ${% raw %}{{ runner.os }}{% endraw %}-pub-${% raw %}{{ hashFiles('**/pubspec.lock') }}{% endraw %}
```

### 4. Run Mutation Tests on Schedule

Full mutation tests can run overnight on a schedule:

```yaml
on:
  schedule:
    - cron: '0 2 * * *' # 2 AM daily
```

## Output Formats

| Format | Flag      | Use Case                      |
| ------ | --------- | ----------------------------- |
| HTML   | `--html`  | Human-readable report         |
| JSON   | `--json`  | Stryker dashboard integration |
| JUnit  | `--junit` | CI test result integration    |

## Next Steps

- [Interpreting Results](/docs/interpreting/) - Understand mutation scores
- [Incremental Testing](/docs/incremental/) - Faster CI runs
