/// String utility functions for mutation testing
class StringUtils {
  /// Check if a string is empty
  bool isEmpty(String s) {
    return s == '';
  }

  /// Check if a string is not empty
  bool isNotEmpty(String s) {
    return s != '';
  }

  /// Get greeting message
  String greet(String name) {
    if (name == '') {
      return 'Hello, stranger!';
    }
    return 'Hello, $name!';
  }

  /// Check if string starts with prefix
  bool startsWith(String s, String prefix) {
    if (s.isEmpty || prefix.isEmpty) {
      return false;
    }
    return s.startsWith(prefix);
  }

  /// Combine two strings
  String combine(String a, String b) {
    return a + b;
  }

  /// Get string length category
  String lengthCategory(String s) {
    if (s.length < 5) {
      return 'short';
    } else if (s.length <= 10) {
      return 'medium';
    } else {
      return 'long';
    }
  }

  /// Check if all conditions are met
  bool validateInput(String s, int minLength, int maxLength) {
    return s.isNotEmpty && s.length >= minLength && s.length <= maxLength;
  }
}
