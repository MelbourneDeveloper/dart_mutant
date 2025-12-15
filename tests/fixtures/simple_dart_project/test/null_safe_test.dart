import 'package:test/test.dart';
import '../lib/null_safe.dart';

void main() {
  late NullSafetyExamples ns;

  setUp(() {
    ns = NullSafetyExamples();
  });

  group('NullSafetyExamples', () {
    group('getValueOrDefault', () {
      test('returns value when not null', () {
        expect(ns.getValueOrDefault('hello'), equals('hello'));
      });

      test('returns default when null', () {
        expect(ns.getValueOrDefault(null), equals('default'));
      });
    });

    group('getLength', () {
      test('returns length when not null', () {
        expect(ns.getLength('hello'), equals(5));
      });

      test('returns null when string is null', () {
        expect(ns.getLength(null), isNull);
      });
    });

    group('isValid', () {
      test('returns true for non-null non-empty', () {
        expect(ns.isValid('hello'), isTrue);
      });

      test('returns false for null', () {
        expect(ns.isValid(null), isFalse);
      });

      test('returns false for empty string', () {
        expect(ns.isValid(''), isFalse);
      });
    });

    group('processItems', () {
      test('returns first item when list has items', () {
        expect(ns.processItems([1, 2, 3]), equals(1));
      });

      test('returns 0 for empty list', () {
        expect(ns.processItems([]), equals(0));
      });

      test('returns 0 for null list', () {
        expect(ns.processItems(null), equals(0));
      });
    });

    group('getNestedValue', () {
      test('returns value when path exists', () {
        final data = {
          'outer': {'inner': 'value'}
        };
        expect(ns.getNestedValue(data, 'outer', 'inner'), equals('value'));
      });

      test('returns null when outer key missing', () {
        final data = {
          'other': {'inner': 'value'}
        };
        expect(ns.getNestedValue(data, 'outer', 'inner'), isNull);
      });

      test('returns null when data is null', () {
        expect(ns.getNestedValue(null, 'outer', 'inner'), isNull);
      });
    });

    group('describeValue', () {
      test('returns positive for positive number', () {
        expect(ns.describeValue(5), equals('positive'));
      });

      test('returns negative for negative number', () {
        expect(ns.describeValue(-5), equals('negative'));
      });

      test('returns zero for zero', () {
        expect(ns.describeValue(0), equals('zero'));
      });

      test('returns no value for null', () {
        expect(ns.describeValue(null), equals('no value'));
      });
    });
  });
}
