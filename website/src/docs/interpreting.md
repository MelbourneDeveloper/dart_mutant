---
layout: docs.njk
title: Interpreting Results
---

# Interpreting Results

Understanding what your mutation testing results mean and how to improve your test suite.

## Mutation Statuses

### Killed (Good)

A mutation is **killed** when at least one test fails after the mutation is applied.

```
✓ Killed: x + y → x - y
  Test failed: "should add two numbers correctly"
```

**This is good!** Your tests correctly detected the bug.

### Survived (Needs Attention)

A mutation **survives** when all tests pass despite the code change.

```
✗ Survived: x >= 18 → x > 18
  All tests passed
```

**This needs attention!** Either:

- Your tests don't cover this case
- Your tests execute this code but don't verify the result
- The mutation is equivalent (functionally identical)

### Timeout

A mutation causes a **timeout** when tests don't complete within the time limit.

```
⏱ Timeout: while(i < n) → while(true)
```

Timeouts usually indicate infinite loops. **Timeouts count as killed** since they represent detectable bugs.

### Error

A mutation causes an **error** when the code fails to compile or crashes before tests run.

```
⚠ Error: a?.b → a.b
  NullPointerException during compilation
```

**Errors are excluded** from the mutation score since they don't represent realistic bugs.

## Mutation Score

The mutation score measures your test suite's fault-detection capability:

```
Mutation Score = Killed / (Killed + Survived)
```

### Interpreting Scores

| Score  | Interpretation                           |
| ------ | ---------------------------------------- |
| 90%+   | Excellent - comprehensive test suite     |
| 80-89% | Good - strong coverage with minor gaps   |
| 70-79% | Acceptable - some improvement needed     |
| 60-69% | Fair - significant gaps in test coverage |
| <60%   | Poor - tests miss many potential bugs    |

### Score vs. Code Coverage

| Metric         | What It Measures                       |
| -------------- | -------------------------------------- |
| Code Coverage  | Lines/branches _executed_ during tests |
| Mutation Score | Code _verified_ by assertions          |

> A 100% code coverage with 60% mutation score means 40% of your code is executed but not actually tested!

## Analyzing Survived Mutants

When a mutation survives, investigate:

### 1. Missing Test Case

The most common cause. Add a test that would catch this bug:

```dart
// Survived: price * 0.9 → price * 1.1

// Add test:
test('applies 10% discount', () {
  expect(applyDiscount(100), equals(90)); // Not 110!
});
```

### 2. Weak Assertion

Tests execute the code but don't verify the result:

```dart
// BAD: Only checks it doesn't throw
test('calculates total', () {
  expect(() => calculateTotal(items), returnsNormally);
});

// GOOD: Verifies the actual result
test('calculates total', () {
  expect(calculateTotal(items), equals(150.0));
});
```

### 3. Equivalent Mutant

Some mutations produce functionally identical code:

```dart
// Original
int index = 0;

// Mutant (equivalent if index is never negative)
int index = -0;
```

These are false positives - you can ignore them.

## Prioritizing Improvements

Focus on:

### High-Value Mutations

1. **Business logic** - Core calculations, validations
2. **Security checks** - Authentication, authorization
3. **Data transformations** - Parsing, serialization
4. **Conditional logic** - if/else, switch statements

### Lower Priority

1. **Logging statements** - Usually don't affect correctness
2. **Debug code** - Should be removed anyway
3. **Generated code** - Tested upstream

## Example Analysis

```
File: lib/src/cart.dart
Mutation Score: 72% (18 killed, 7 survived)

Survived Mutations:
1. Line 45: total += item.price  →  total -= item.price
2. Line 52: quantity > 0         →  quantity >= 0
3. Line 67: discount ?? 0        →  discount
```

**Analysis:**

1. **Line 45**: Critical! Adding items should increase total. Missing assertion.
2. **Line 52**: Edge case - what happens with quantity = 0? Add test.
3. **Line 67**: What if discount is null? Verify default behavior.

## Tracking Progress

Track mutation score over time:

```bash
# Save historical data
dart_mutant --json >> mutation-history.jsonl
```

Set incremental goals:

- Week 1: Reach 70%
- Week 2: Reach 75%
- Week 3: Reach 80%

## Next Steps

- [Mutation Operators](/docs/operators/) - Understand what's being mutated
- [Filtering](/docs/filtering/) - Exclude non-critical code
- [CI/CD Integration](/docs/ci/) - Enforce thresholds
