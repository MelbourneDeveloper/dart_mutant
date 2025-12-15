/// Examples demonstrating Dart null safety for mutation testing
class NullSafetyExamples {
  /// Get length or default using null coalescing
  int getLengthOrDefault(String? s, int defaultValue) {
    return s?.length ?? defaultValue;
  }

  /// Safely access nested property
  String? getNestedValue(Map<String, dynamic>? data) {
    return data?['user']?['name'] as String?;
  }

  /// Get a safe substring
  String? safeSubstring(String? s, int start, int end) {
    return s?.substring(start, end);
  }

  /// Check if string starts with prefix (null-safe)
  bool? startsWithSafe(String? s, String prefix) {
    return s?.startsWith(prefix);
  }

  /// Check if value is not null
  bool hasValue<T>(T? value) {
    return value != null;
  }

  /// Get value or throw
  T getOrThrow<T>(T? value, String message) {
    if (value == null) {
      throw ArgumentError(message);
    }
    return value;
  }

  /// Transform if not null
  R? mapIfPresent<T, R>(T? value, R Function(T) transform) {
    if (value == null) {
      return null;
    }
    return transform(value);
  }

  /// Get first non-null value
  T? firstNonNull<T>(List<T?> values) {
    for (final value in values) {
      if (value != null) {
        return value;
      }
    }
    return null;
  }

  /// Safe list access
  T? safeGet<T>(List<T> list, int index) {
    if (index < 0 || index >= list.length) {
      return null;
    }
    return list[index];
  }
}
