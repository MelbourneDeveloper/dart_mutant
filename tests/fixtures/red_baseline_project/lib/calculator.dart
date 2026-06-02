/// A tiny calculator used as a red-baseline fixture for dart_mutant.
///
/// The accompanying test suite intentionally fails on this UNMUTATED code so
/// that we can verify dart_mutant refuses to run on a red baseline rather than
/// reporting every mutant as killed (GitHub issue #5).
class Calculator {
  /// Returns the sum of [a] and [b].
  int add(int a, int b) => a + b;

  /// Returns `true` when [value] is strictly greater than zero.
  bool isPositive(int value) => value > 0;
}
