import 'package:test/test.dart';
import 'package:compile_error_project/type_sensitive.dart';

void main() {
  late TypeSensitive ts;

  setUp(() {
    ts = TypeSensitive();
  });

  group('concatenate', () {
    test('joins two strings', () {
      expect(ts.concatenate('hello', ' world'), equals('hello world'));
    });

    test('handles empty strings', () {
      expect(ts.concatenate('', 'test'), equals('test'));
      expect(ts.concatenate('test', ''), equals('test'));
    });
  });

  group('combineListsWithPlus', () {
    test('combines two lists', () {
      expect(ts.combineListsWithPlus([1, 2], [3, 4]), equals([1, 2, 3, 4]));
    });

    test('handles empty lists', () {
      expect(ts.combineListsWithPlus([], [1, 2]), equals([1, 2]));
    });
  });

  group('intOnlyMath', () {
    test('adds two integers', () {
      expect(ts.intOnlyMath(5, 3), equals(8));
    });

    test('handles negative numbers', () {
      expect(ts.intOnlyMath(-5, 3), equals(-2));
    });
  });

  group('checkEquality', () {
    test('returns true for equal values', () {
      expect(ts.checkEquality(5, 5), isTrue);
    });

    test('returns false for different values', () {
      expect(ts.checkEquality(5, 3), isFalse);
    });
  });

  group('mustReturnInt', () {
    test('returns positive number', () {
      expect(ts.mustReturnInt(5), equals(5));
    });

    test('returns zero for non-positive', () {
      expect(ts.mustReturnInt(0), equals(0));
      expect(ts.mustReturnInt(-5), equals(0));
    });
  });

  group('nonNullableString', () {
    test('converts to uppercase', () {
      expect(ts.nonNullableString('hello'), equals('HELLO'));
    });
  });

  group('useSpecificMethod', () {
    test('returns list length', () {
      expect(ts.useSpecificMethod([1, 2, 3]), equals(3));
    });

    test('returns zero for empty list', () {
      expect(ts.useSpecificMethod([]), equals(0));
    });
  });

  group('incrementValue', () {
    test('increments value', () {
      expect(ts.incrementValue(5), equals(6));
    });

    test('handles zero', () {
      expect(ts.incrementValue(0), equals(1));
    });
  });

  group('compareStrings', () {
    test('returns true for equal strings', () {
      expect(ts.compareStrings('hello', 'hello'), isTrue);
    });

    test('returns false for different strings', () {
      expect(ts.compareStrings('hello', 'world'), isFalse);
    });
  });

  group('ImmutableData', () {
    test('returns stored value', () {
      final data = ImmutableData(42, 'test');
      expect(data.getValue(), equals(42));
    });
  });

  group('GenericMath', () {
    test('adds integers', () {
      final math = GenericMath<int>();
      expect(math.add(2, 3), equals(5));
    });

    test('adds doubles', () {
      final math = GenericMath<double>();
      expect(math.add(2.5, 3.5), equals(6.0));
    });
  });
}
