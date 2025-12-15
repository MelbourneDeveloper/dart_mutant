import 'package:test/test.dart';
import 'package:sample_project/null_safety_examples.dart';

void main() {
  late NullSafetyExamples examples;

  setUp(() {
    examples = NullSafetyExamples();
  });

  group('getLengthOrDefault', () {
    test('returns length when not null', () {
      expect(examples.getLengthOrDefault('hello', 0), equals(5));
    });

    test('returns default when null', () {
      expect(examples.getLengthOrDefault(null, 10), equals(10));
    });

    test('returns zero for empty string', () {
      expect(examples.getLengthOrDefault('', 5), equals(0));
    });
  });

  group('getNestedValue', () {
    test('returns null for null map', () {
      expect(examples.getNestedValue(null), isNull);
    });

    test('returns null for missing key', () {
      expect(examples.getNestedValue({'other': 'value'}), isNull);
    });

    test('returns null for null user', () {
      expect(examples.getNestedValue({'user': null}), isNull);
    });

    test('returns value when present', () {
      expect(examples.getNestedValue({
        'user': {'name': 'John'}
      }), equals('John'));
    });
  });

  group('hasValue', () {
    test('returns false for null', () {
      expect(examples.hasValue<String>(null), isFalse);
    });

    test('returns true for value', () {
      expect(examples.hasValue<String>('hello'), isTrue);
    });

    test('returns true for empty string', () {
      expect(examples.hasValue<String>(''), isTrue);
    });
  });

  group('getOrThrow', () {
    test('returns value when present', () {
      expect(examples.getOrThrow<String>('hello', 'error'), equals('hello'));
    });

    test('throws when null', () {
      expect(
        () => examples.getOrThrow<String>(null, 'Value is required'),
        throwsArgumentError,
      );
    });
  });

  group('mapIfPresent', () {
    test('returns null for null input', () {
      expect(examples.mapIfPresent<int, String>(null, (i) => i.toString()), isNull);
    });

    test('transforms when present', () {
      expect(examples.mapIfPresent<int, String>(42, (i) => i.toString()), equals('42'));
    });
  });

  group('firstNonNull', () {
    test('returns null for all null', () {
      expect(examples.firstNonNull<int>([null, null, null]), isNull);
    });

    test('returns first non-null', () {
      expect(examples.firstNonNull<int>([null, 5, 10]), equals(5));
    });

    test('returns first when not null', () {
      expect(examples.firstNonNull<int>([1, 2, 3]), equals(1));
    });

    test('returns null for empty list', () {
      expect(examples.firstNonNull<int>([]), isNull);
    });
  });

  group('safeGet', () {
    test('returns null for negative index', () {
      expect(examples.safeGet([1, 2, 3], -1), isNull);
    });

    test('returns null for out of bounds index', () {
      expect(examples.safeGet([1, 2, 3], 5), isNull);
    });

    test('returns value for valid index', () {
      expect(examples.safeGet([1, 2, 3], 1), equals(2));
    });

    test('returns first element', () {
      expect(examples.safeGet([1, 2, 3], 0), equals(1));
    });

    test('returns last element', () {
      expect(examples.safeGet([1, 2, 3], 2), equals(3));
    });
  });

  group('safeSubstring', () {
    test('returns null for null string', () {
      expect(examples.safeSubstring(null, 0, 3), isNull);
    });

    test('returns substring for valid string', () {
      expect(examples.safeSubstring('hello', 0, 3), equals('hel'));
    });
  });

  group('startsWithSafe', () {
    test('returns null for null string', () {
      expect(examples.startsWithSafe(null, 'he'), isNull);
    });

    test('returns true for matching prefix', () {
      expect(examples.startsWithSafe('hello', 'he'), isTrue);
    });

    test('returns false for non-matching prefix', () {
      expect(examples.startsWithSafe('hello', 'wo'), isFalse);
    });
  });
}
