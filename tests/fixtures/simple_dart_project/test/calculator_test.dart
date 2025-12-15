import 'package:test/test.dart';
import '../lib/calculator.dart';

void main() {
  late Calculator calc;

  setUp(() {
    calc = Calculator();
  });

  group('Calculator', () {
    group('add', () {
      test('adds two positive numbers', () {
        expect(calc.add(2, 3), equals(5));
      });

      test('adds negative numbers', () {
        expect(calc.add(-1, -1), equals(-2));
      });

      test('adds zero', () {
        expect(calc.add(5, 0), equals(5));
      });
    });

    group('subtract', () {
      test('subtracts two numbers', () {
        expect(calc.subtract(5, 3), equals(2));
      });

      test('handles negative result', () {
        expect(calc.subtract(3, 5), equals(-2));
      });
    });

    group('multiply', () {
      test('multiplies two numbers', () {
        expect(calc.multiply(3, 4), equals(12));
      });

      test('multiplies by zero', () {
        expect(calc.multiply(5, 0), equals(0));
      });
    });

    group('divide', () {
      test('divides two numbers', () {
        expect(calc.divide(10, 2), equals(5.0));
      });

      test('throws on divide by zero', () {
        expect(() => calc.divide(10, 0), throwsArgumentError);
      });
    });

    group('isPositive', () {
      test('returns true for positive', () {
        expect(calc.isPositive(5), isTrue);
      });

      test('returns false for negative', () {
        expect(calc.isPositive(-5), isFalse);
      });

      test('returns false for zero', () {
        expect(calc.isPositive(0), isFalse);
      });
    });

    group('isEven', () {
      test('returns true for even', () {
        expect(calc.isEven(4), isTrue);
      });

      test('returns false for odd', () {
        expect(calc.isEven(3), isFalse);
      });
    });

    group('max', () {
      test('returns larger value', () {
        expect(calc.max(3, 5), equals(5));
      });

      test('returns first when larger', () {
        expect(calc.max(7, 2), equals(7));
      });

      test('handles equal values', () {
        expect(calc.max(4, 4), equals(4));
      });
    });

    group('isInRange', () {
      test('returns true when in range', () {
        expect(calc.isInRange(5, 1, 10), isTrue);
      });

      test('returns true on lower boundary', () {
        expect(calc.isInRange(1, 1, 10), isTrue);
      });

      test('returns true on upper boundary', () {
        expect(calc.isInRange(10, 1, 10), isTrue);
      });

      test('returns false when below range', () {
        expect(calc.isInRange(0, 1, 10), isFalse);
      });

      test('returns false when above range', () {
        expect(calc.isInRange(11, 1, 10), isFalse);
      });
    });

    group('factorial', () {
      test('returns 1 for 0', () {
        expect(calc.factorial(0), equals(1));
      });

      test('returns 1 for 1', () {
        expect(calc.factorial(1), equals(1));
      });

      test('calculates factorial of 5', () {
        expect(calc.factorial(5), equals(120));
      });
    });

    group('abs', () {
      test('returns positive value unchanged', () {
        expect(calc.abs(5), equals(5));
      });

      test('returns absolute of negative', () {
        expect(calc.abs(-5), equals(5));
      });

      test('returns 0 for 0', () {
        expect(calc.abs(0), equals(0));
      });
    });

    group('areEqual', () {
      test('returns true for equal values', () {
        expect(calc.areEqual(5, 5), isTrue);
      });

      test('returns false for different values', () {
        expect(calc.areEqual(5, 3), isFalse);
      });
    });

    group('negate', () {
      test('negates positive', () {
        expect(calc.negate(5), equals(-5));
      });

      test('negates negative', () {
        expect(calc.negate(-5), equals(5));
      });
    });

    group('increment', () {
      test('increments value', () {
        expect(calc.increment(5), equals(6));
      });
    });

    group('decrement', () {
      test('decrements value', () {
        expect(calc.decrement(5), equals(4));
      });
    });
  });
}
