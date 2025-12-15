import 'package:test/test.dart';
import 'package:sample_project/string_utils.dart';

void main() {
  late StringUtils utils;

  setUp(() {
    utils = StringUtils();
  });

  group('isEmpty', () {
    test('returns true for empty string', () {
      expect(utils.isEmpty(''), isTrue);
    });

    test('returns false for non-empty', () {
      expect(utils.isEmpty('hello'), isFalse);
    });

    test('returns false for whitespace', () {
      expect(utils.isEmpty(' '), isFalse);
    });
  });

  group('isNotEmpty', () {
    test('returns false for empty string', () {
      expect(utils.isNotEmpty(''), isFalse);
    });

    test('returns true for non-empty', () {
      expect(utils.isNotEmpty('hello'), isTrue);
    });

    test('returns true for whitespace', () {
      expect(utils.isNotEmpty(' '), isTrue);
    });
  });

  group('greet', () {
    test('returns stranger greeting for empty name', () {
      expect(utils.greet(''), equals('Hello, stranger!'));
    });

    test('returns personalized greeting', () {
      expect(utils.greet('Alice'), equals('Hello, Alice!'));
    });

    test('handles names with spaces', () {
      expect(utils.greet('John Doe'), equals('Hello, John Doe!'));
    });
  });

  group('startsWith', () {
    test('returns false for empty string', () {
      expect(utils.startsWith('', 'a'), isFalse);
    });

    test('returns false for empty prefix', () {
      expect(utils.startsWith('hello', ''), isFalse);
    });

    test('returns true when starts with prefix', () {
      expect(utils.startsWith('hello', 'hel'), isTrue);
    });

    test('returns false when does not start with prefix', () {
      expect(utils.startsWith('hello', 'world'), isFalse);
    });

    test('returns true for exact match', () {
      expect(utils.startsWith('hello', 'hello'), isTrue);
    });
  });

  group('combine', () {
    test('combines two strings', () {
      expect(utils.combine('hello', ' world'), equals('hello world'));
    });

    test('handles empty first string', () {
      expect(utils.combine('', 'world'), equals('world'));
    });

    test('handles empty second string', () {
      expect(utils.combine('hello', ''), equals('hello'));
    });

    test('handles both empty', () {
      expect(utils.combine('', ''), equals(''));
    });
  });

  group('lengthCategory', () {
    test('returns short for empty', () {
      expect(utils.lengthCategory(''), equals('short'));
    });

    test('returns short for 4 chars', () {
      expect(utils.lengthCategory('test'), equals('short'));
    });

    test('returns medium for 5 chars', () {
      expect(utils.lengthCategory('hello'), equals('medium'));
    });

    test('returns medium for 10 chars', () {
      expect(utils.lengthCategory('1234567890'), equals('medium'));
    });

    test('returns long for 11 chars', () {
      expect(utils.lengthCategory('hello world'), equals('long'));
    });
  });

  group('validateInput', () {
    test('returns false for empty string', () {
      expect(utils.validateInput('', 1, 10), isFalse);
    });

    test('returns false for too short', () {
      expect(utils.validateInput('hi', 5, 10), isFalse);
    });

    test('returns false for too long', () {
      expect(utils.validateInput('hello world!', 1, 5), isFalse);
    });

    test('returns true for valid length', () {
      expect(utils.validateInput('hello', 1, 10), isTrue);
    });

    test('returns true for exact min length', () {
      expect(utils.validateInput('hi', 2, 10), isTrue);
    });

    test('returns true for exact max length', () {
      expect(utils.validateInput('hello', 1, 5), isTrue);
    });
  });
}
