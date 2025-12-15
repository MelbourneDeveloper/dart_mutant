---
layout: docs.njk
title: Mutation Operators
---

# Mutation Operators

dart_mutant includes 40+ mutation operators organized by category. Each operator represents a small, targeted change that a real bug might introduce.

## Arithmetic Operators

Mutations that change mathematical operations.

| Original | Mutated To |
|----------|------------|
| `a + b` | `a - b`, `a * b`, `a / b` |
| `a - b` | `a + b`, `a * b`, `a / b` |
| `a * b` | `a / b`, `a + b`, `a - b` |
| `a / b` | `a * b`, `a + b`, `a - b` |
| `a % b` | `a * b` |
| `a++` | `a--` |
| `a--` | `a++` |
| `++a` | `--a` |
| `--a` | `++a` |

**Example:**
```dart
// Original
int calculate(int x, int y) => x + y;

// Mutant
int calculate(int x, int y) => x - y;  // + → -
```

## Comparison Operators

Mutations that change relational comparisons.

| Original | Mutated To |
|----------|------------|
| `a < b` | `a <= b`, `a >= b`, `a > b` |
| `a > b` | `a >= b`, `a <= b`, `a < b` |
| `a <= b` | `a < b`, `a >= b`, `a > b` |
| `a >= b` | `a > b`, `a <= b`, `a < b` |
| `a == b` | `a != b` |
| `a != b` | `a == b` |

**Example:**
```dart
// Original
bool isAdult(int age) => age >= 18;

// Mutant
bool isAdult(int age) => age > 18;  // >= → >
```

## Logical Operators

Mutations that change boolean logic.

| Original | Mutated To |
|----------|------------|
| `a && b` | `a \|\| b` |
| `a \|\| b` | `a && b` |
| `!a` | `a` (negation removed) |

**Example:**
```dart
// Original
bool canAccess(bool isAdmin, bool isOwner) => isAdmin && isOwner;

// Mutant
bool canAccess(bool isAdmin, bool isOwner) => isAdmin || isOwner;  // && → ||
```

## Boolean Literals

Direct true/false swaps.

| Original | Mutated To |
|----------|------------|
| `true` | `false` |
| `false` | `true` |

**Example:**
```dart
// Original
bool isEnabled = true;

// Mutant
bool isEnabled = false;  // true → false
```

## Null Safety Operators

Dart-specific null-aware mutations.

| Original | Mutated To |
|----------|------------|
| `a ?? b` | `a` (null coalescing removed) |
| `a?.b` | `a.b` (null-aware removed) |
| `a ??= b` | `a = b` (null-aware assignment) |

**Example:**
```dart
// Original
String getName(User? user) => user?.name ?? 'Anonymous';

// Mutant 1
String getName(User? user) => user.name ?? 'Anonymous';  // ?. → .

// Mutant 2
String getName(User? user) => user?.name;  // ?? removed (returns null)
```

## Control Flow

Mutations that affect branching and loops.

| Original | Mutated To |
|----------|------------|
| `if (condition)` | `if (true)` |
| `if (condition)` | `if (false)` |
| `while (condition)` | `while (false)` |
| `break` | (removed) |
| `continue` | (removed) |

**Example:**
```dart
// Original
void process(bool shouldRun) {
  if (shouldRun) {
    doWork();
  }
}

// Mutant 1
void process(bool shouldRun) {
  if (true) {  // condition → true
    doWork();
  }
}

// Mutant 2
void process(bool shouldRun) {
  if (false) {  // condition → false
    doWork();
  }
}
```

## String Literals

Mutations for string values.

| Original | Mutated To |
|----------|------------|
| `"any string"` | `""` (empty string) |
| `""` | `"mutant"` |

> **Note:** String mutations are limited to avoid noise. Only strings in meaningful positions are mutated.

## Assignment Operators

Compound assignment mutations.

| Original | Mutated To |
|----------|------------|
| `a += b` | `a -= b` |
| `a -= b` | `a += b` |
| `a *= b` | `a /= b` |
| `a /= b` | `a *= b` |

## Return Values

Mutations that change what functions return.

| Original | Mutated To |
|----------|------------|
| `return value` | `return null` (for nullable types) |
| `return true` | `return false` |
| `return false` | `return true` |

## Excluded from Mutation

dart_mutant automatically excludes:

- **Generated code**: `*.g.dart`, `*.freezed.dart`, `*.gr.dart`
- **Comments**: Both line and block comments
- **Import/export statements**: Package imports
- **Annotations**: `@override`, `@deprecated`, etc.
- **Constant expressions**: `const` values that would break compilation

## Next Steps

- [Filtering](/docs/filtering/) - Control what gets mutated
- [Interpreting Results](/docs/interpreting/) - Understand surviving mutants
