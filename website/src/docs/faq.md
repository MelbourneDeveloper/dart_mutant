---
layout: docs.njk
title: FAQ
---

# Frequently Asked Questions

## General

### What is mutation testing?

Mutation testing evaluates your test suite's quality by introducing deliberate bugs (mutations) into your code and checking if your tests catch them. Unlike code coverage, which only measures what code *runs*, mutation testing measures what code is actually *verified* by assertions.

### How is this different from code coverage?

| Metric | Measures | Example |
|--------|----------|---------|
| Code Coverage | Lines executed | A line runs = covered |
| Mutation Score | Bugs detected | A bug is caught = killed |

You can have 100% code coverage but only 50% mutation score if your tests execute code without verifying results.

### Why is my mutation score lower than my code coverage?

This is common! It means your tests execute code but don't make strong assertions. For example:

```dart
// This test gives 100% coverage of the function
test('calculates something', () {
  calculate(5, 3);  // No assertion!
});

// This test also kills mutations
test('calculates correctly', () {
  expect(calculate(5, 3), equals(8));
});
```

### What's a good mutation score?

| Score | Interpretation |
|-------|---------------|
| 90%+ | Excellent |
| 80-89% | Good |
| 70-79% | Acceptable |
| 60-69% | Needs improvement |
| <60% | Weak test suite |

Start by establishing a baseline, then gradually improve.

## Performance

### Why is mutation testing slow?

Each mutation requires a full test run. With 1000 mutations and 10-second test suite, that's ~3 hours. dart_mutant mitigates this with:

- **Parallel execution**: Run multiple mutations concurrently
- **Sampling**: Test a subset of mutations
- **Incremental mode**: Only test changed files

### How can I speed it up?

```bash
# More parallelism (uses all CPUs by default)
dart_mutant --parallel 16

# Sample mutations for quick feedback
dart_mutant --sample 100

# Shorter timeout
dart_mutant --timeout 10

# Only test changed files
dart_mutant --incremental --base-ref main
```

### How long should I expect mutation testing to take?

Rough estimates:

| Project Size | Mutations | Full Run | Sampled (100) |
|--------------|-----------|----------|---------------|
| Small (<10 files) | 200-500 | 5-15 min | 1-3 min |
| Medium (10-50 files) | 500-2000 | 15-60 min | 3-10 min |
| Large (50+ files) | 2000+ | 1-4 hours | 10-30 min |

## Results

### What does "survived" mean?

A mutation "survived" when all tests passed despite the code change. This usually means:

1. No test covers this code path
2. Tests execute the code but don't verify the result
3. The mutation is equivalent (functionally identical)

### What's an equivalent mutant?

An equivalent mutant produces the same behavior as the original code:

```dart
// Original
int index = 0;

// Equivalent mutant (both are zero)
int index = -0;
```

These are false positives and should be ignored.

### Should I aim for 100% mutation score?

No. Diminishing returns kick in around 85-90%. Some mutations are:

- Equivalent (impossible to kill)
- In non-critical code (logging, debug)
- Cost more to test than the risk they represent

Focus on critical business logic.

## Technical

### Does dart_mutant modify my files permanently?

No. dart_mutant:

1. Makes a backup of the original file
2. Applies mutation
3. Runs tests
4. Restores original file

If dart_mutant crashes, files are restored on next run.

### Which Dart versions are supported?

dart_mutant supports:

- Dart 2.17+ (null safety)
- Dart 3.x (full support)
- Flutter projects

### Does it work with Flutter?

Yes! dart_mutant runs `dart test` which works for Flutter packages. For Flutter app testing:

```bash
# Flutter test command
dart_mutant --test-command "flutter test"
```

### Why are some files excluded?

dart_mutant excludes:

- Generated files (`*.g.dart`, `*.freezed.dart`)
- Test files (`test/**`)
- Build outputs

Generated code is tested by the code generators, not your tests.

### Can I run it in CI?

dart_mutant is designed for CI:

```bash
# CI-friendly command
dart_mutant --quiet --threshold 80 --junit
```

See [CI/CD Integration](/docs/ci/) for full examples.

## Troubleshooting

### "No mutations found"

Check:
1. Are there `.dart` files in `lib/`?
2. Are all files excluded by patterns?
3. Is the code too simple (no operators to mutate)?

```bash
# See what's being discovered
dart_mutant --dry-run --verbose
```

### "Tests pass but mutation score is 0%"

This usually means tests aren't actually testing the mutated code. Check:

1. Are tests in `test/` directory?
2. Do tests import the mutated files?
3. Do tests have assertions?

### "Timeout" on every mutation

Your test suite may be slow or have flaky tests:

```bash
# Increase timeout
dart_mutant --timeout 60

# Run tests manually to check
dart test
```

### Build errors on mutations

Some mutations can create invalid Dart code. These are marked as "Error" and excluded from the score. This is normal for a small percentage of mutations.

## More Questions?

- Check the [GitHub Issues](https://github.com/user/dart_mutant/issues)
- Open a new issue if your question isn't answered
