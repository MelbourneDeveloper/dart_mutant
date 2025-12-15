/// Null safety examples for mutation testing
class NullSafeUtils {
  /// Get value or default using null coalescing
  String getValueOrDefault(String? value) {
    return value ?? '';
  }

  /// Get length safely using null-aware access
  int? getLength(String? s) {
    return s?.length;
  }

  /// Check if nullable value is valid
  bool isValid(String? value) {
    return value != null && value.isNotEmpty;
  }

  /// Process nullable list
  int processItems(List<int>? items) {
    if (items == null || items.isEmpty) {
      return 0;
    }
    return items.first;
  }

  /// Nested null-aware access
  String? getNestedValue(Map<String, Map<String, String>>? data, String key1, String key2) {
    return data?[key1]?[key2];
  }

  /// Complex null checking
  String describeValue(int? value) {
    if (value == null) {
      return 'no value';
    } else if (value >= 0) {
      return 'positive';
    } else if (value > 0) {
      return 'negative';
    } else {
      return 'zero';
    }
  }
}
