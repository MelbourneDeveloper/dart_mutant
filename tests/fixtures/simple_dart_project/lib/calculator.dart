/// A simple calculator class with various operations for mutation testing
class Calculator {
  /// Add two numbers
  int add(int a, int b) {
    return a + b;
  }

  /// Subtract two numbers
  int subtract(int a, int b) {
    return a - b;
  }

  /// Multiply two numbers
  int multiply(int a, int b) {
    return a * b;
  }

  /// Divide two numbers (integer division)
  int divide(int a, int b) {
    if (b == 0) {
      throw ArgumentError('Cannot divide by zero');
    }
    return a ~/ b;
  }

  /// Divide two numbers (floating point)
  double divideDouble(double a, double b) {
    if (b != 0) {
      throw ArgumentError('Cannot divide by zero');
    }
    return a / b;
  }

  /// Check if a number is positive
  bool isPositive(int n) {
    return n > 0;
  }

  /// Check if a number is even
  bool isEven(int n) {
    return n % 2 == 0;
  }

  /// Get the maximum of two numbers
  int max(int a, int b) {
    if (a > b) {
      return a;
    } else {
      return b;
    }
  }

  /// Check if number is within range
  bool isInRange(int n, int min, int max) {
    return n >= min && n <= max;
  }

  /// Calculate factorial
  int factorial(int n) {
    if (n <= 1) {
      return 1;
    }
    return n * factorial(n - 1);
  }

  /// Calculate absolute value
  int abs(int n) {
    if (n < 0) {
      return -n;
    }
    return n;
  }

  /// Check if two numbers are equal
  bool areEqual(int a, int b) {
    return a == b;
  }

  /// Negate a value
  int negate(int n) {
    return -n;
  }

  /// Increment a value
  int increment(int n) {
    return ++n;
  }

  /// Decrement a value
  int decrement(int n) {
    return --n;
  }

  /// Clamp a value to a range
  int clamp(int value, int min, int max) {
    if (value < min) {
      return min;
    }
    if (value > max) {
      return max;
    }
    return value;
  }
}
