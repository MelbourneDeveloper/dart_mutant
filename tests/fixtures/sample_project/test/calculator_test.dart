import 'package:test/test.dart';
import 'package:sample_project/calculator.dart';

void main() {
  late Calculator calc;

  setUp(() {
    calc = Calculator();
  });

  group('add', () {
    test('adds positive numbers', () {
      expect(calc.add(2, 3), equals(5));
    });

    test('adds negative numbers', () {
      expect(calc.add(-2, -3), equals(-5));
    });

    test('adds mixed numbers', () {
      expect(calc.add(-2, 5), equals(3));
    });

    test('adds zero', () {
      expect(calc.add(5, 0), equals(5));
    });
  });

  group('subtract', () {
    test('subtracts positive numbers', () {
      expect(calc.subtract(5, 3), equals(2));
    });

    test('subtracts resulting in negative', () {
      expect(calc.subtract(3, 5), equals(-2));
    });

    test('subtracts zero', () {
      expect(calc.subtract(5, 0), equals(5));
    });
  });

  group('multiply', () {
    test('multiplies positive numbers', () {
      expect(calc.multiply(4, 3), equals(12));
    });

    test('multiplies by zero', () {
      expect(calc.multiply(5, 0), equals(0));
    });

    test('multiplies negative numbers', () {
      expect(calc.multiply(-2, -3), equals(6));
    });
  });

  group('divide', () {
    test('divides evenly', () {
      expect(calc.divide(10, 2), equals(5));
    });

    test('divides with remainder', () {
      expect(calc.divide(7, 2), equals(3));
    });

    test('throws on divide by zero', () {
      expect(() => calc.divide(5, 0), throwsArgumentError);
    });
  });

  group('isPositive', () {
    test('returns true for positive', () {
      expect(calc.isPositive(5), isTrue);
    });

    test('returns false for zero', () {
      expect(calc.isPositive(0), isFalse);
    });

    test('returns false for negative', () {
      expect(calc.isPositive(-5), isFalse);
    });
  });

  group('isEven', () {
    test('returns true for even', () {
      expect(calc.isEven(4), isTrue);
    });

    test('returns false for odd', () {
      expect(calc.isEven(5), isFalse);
    });

    test('zero is even', () {
      expect(calc.isEven(0), isTrue);
    });
  });

  group('abs', () {
    test('returns same for positive', () {
      expect(calc.abs(5), equals(5));
    });

    test('returns positive for negative', () {
      expect(calc.abs(-5), equals(5));
    });

    test('returns zero for zero', () {
      expect(calc.abs(0), equals(0));
    });
  });

  group('isInRange', () {
    test('returns true when in range', () {
      expect(calc.isInRange(5, 1, 10), isTrue);
    });

    test('returns true at min boundary', () {
      expect(calc.isInRange(1, 1, 10), isTrue);
    });

    test('returns true at max boundary', () {
      expect(calc.isInRange(10, 1, 10), isTrue);
    });

    test('returns false below range', () {
      expect(calc.isInRange(0, 1, 10), isFalse);
    });

    test('returns false above range', () {
      expect(calc.isInRange(11, 1, 10), isFalse);
    });
  });

  group('max', () {
    test('returns first when larger', () {
      expect(calc.max(10, 5), equals(10));
    });

    test('returns second when larger', () {
      expect(calc.max(5, 10), equals(10));
    });

    test('returns either when equal', () {
      expect(calc.max(5, 5), equals(5));
    });
  });

  group('clamp', () {
    test('returns value when in range', () {
      expect(calc.clamp(5, 1, 10), equals(5));
    });

    test('returns min when below', () {
      expect(calc.clamp(0, 1, 10), equals(1));
    });

    test('returns max when above', () {
      expect(calc.clamp(15, 1, 10), equals(10));
    });

    test('returns value at min boundary', () {
      expect(calc.clamp(1, 1, 10), equals(1));
    });

    test('returns value at max boundary', () {
      expect(calc.clamp(10, 1, 10), equals(10));
    });
  });

  group('negate', () {
    test('negates positive', () {
      expect(calc.negate(5), equals(-5));
    });

    test('negates negative', () {
      expect(calc.negate(-5), equals(5));
    });

    test('negates zero', () {
      expect(calc.negate(0), equals(0));
    });
  });

  group('increment', () {
    test('increments positive', () {
      expect(calc.increment(5), equals(6));
    });

    test('increments negative', () {
      expect(calc.increment(-5), equals(-4));
    });

    test('increments zero', () {
      expect(calc.increment(0), equals(1));
    });
  });

  group('decrement', () {
    test('decrements positive', () {
      expect(calc.decrement(5), equals(4));
    });

    test('decrements negative', () {
      expect(calc.decrement(-5), equals(-6));
    });

    test('decrements zero', () {
      expect(calc.decrement(0), equals(-1));
    });
  });
}
