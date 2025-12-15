/// Code where mutations will cause type errors at compile time
///
/// These examples demonstrate cases where mutating operators will break
/// the type system, causing compile-time errors rather than runtime failures.

class TypeSensitive {
  /// String concatenation - mutating + to - will cause compile error
  /// because you can't subtract strings
  String concatenate(String a, String b) {
    return a + b;  // Mutation: a - b -> COMPILE ERROR (can't subtract strings)
  }

  /// List concatenation - mutating + to other operators causes compile error
  List<int> combineListsWithPlus(List<int> a, List<int> b) {
    return a + b;  // Mutation: a - b -> COMPILE ERROR (can't subtract lists)
  }

  /// Typed arithmetic that must return int
  int intOnlyMath(int a, int b) {
    return a + b;  // Mutation to / would return double, causing type error
  }

  /// Boolean operations on non-booleans cause compile errors
  bool checkEquality(int a, int b) {
    return a == b;  // Mutation: a && b -> COMPILE ERROR (can't && ints)
  }

  /// Return type mismatch when mutating
  int mustReturnInt(int n) {
    if (n > 0) {
      return n;    // Mutation: return null -> COMPILE ERROR (non-nullable)
    }
    return 0;
  }

  /// Null safety violations
  String nonNullableString(String s) {
    return s.toUpperCase();  // If we mutate s to null, compile error
  }

  /// Method that doesn't exist after mutation
  int useSpecificMethod(List<int> items) {
    return items.length;  // Mutation can't change to items.size (doesn't exist)
  }

  /// Increment/decrement on wrong types
  int incrementValue(int n) {
    return ++n;  // Works. But ++s on String would fail
  }

  /// Type-specific comparison
  bool compareStrings(String a, String b) {
    return a == b;  // Mutation: a > b on strings is allowed but a - b is not
  }
}

/// Class with final fields - mutations to assignment cause errors
class ImmutableData {
  final int value;
  final String name;

  ImmutableData(this.value, this.name);

  /// Can't mutate to reassign final
  int getValue() {
    return value;  // Trying to mutate to: value = 0; return value; -> ERROR
  }
}

/// Generic type constraints
class GenericMath<T extends num> {
  T add(T a, T b) {
    // This works because num supports +
    // But mutation to string concat would fail type bounds
    return (a + b) as T;
  }
}
