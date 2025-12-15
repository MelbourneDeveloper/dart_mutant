import 'package:test/test.dart';
import 'package:simple_dart_project/string_utils.dart';

void main() {
  late StringUtils utils;

  setUp(() {
    utils = StringUtils();
  });

  group('StringUtils', () {
    group('isEmpty', () {
      test('returns true for empty string', () {
        expect(utils.isEmpty(''), isTrue);
      });

      test('returns false for non-empty string', () {
        expect(utils.isEmpty('hello'), isFalse);
      });
    });

    group('isNotEmpty', () {
      test('returns false for empty string', () {
        expect(utils.isNotEmpty(''), isFalse);
      });

      test('returns true for non-empty string', () {
        expect(utils.isNotEmpty('hello'), isTrue);
      });
    });

    group('greet', () {
      test('greets with name', () {
        expect(utils.greet('World'), equals('Hello, World!'));
      });

      test('greets stranger when empty', () {
        expect(utils.greet(''), equals('Hello, stranger!'));
      });
    });

    group('startsWith', () {
      test('returns true when starts with prefix', () {
        expect(utils.startsWith('hello', 'hel'), isTrue);
      });

      test('returns false when does not start with prefix', () {
        expect(utils.startsWith('hello', 'bye'), isFalse);
      });

      test('returns false for empty string', () {
        expect(utils.startsWith('', 'a'), isFalse);
      });

      test('returns false for empty prefix', () {
        expect(utils.startsWith('hello', ''), isFalse);
      });
    });

    group('combine', () {
      test('combines two strings', () {
        expect(utils.combine('hello', 'world'), equals('helloworld'));
      });
    });

    group('lengthCategory', () {
      test('returns short for < 5 chars', () {
        expect(utils.lengthCategory('hi'), equals('short'));
      });

      test('returns medium for 5-10 chars', () {
        expect(utils.lengthCategory('hello'), equals('medium'));
      });

      test('returns long for > 10 chars', () {
        expect(utils.lengthCategory('hello world!'), equals('long'));
      });
    });

    group('validateInput', () {
      test('returns true for valid input', () {
        expect(utils.validateInput('hello', 3, 10), isTrue);
      });

      test('returns false for empty input', () {
        expect(utils.validateInput('', 3, 10), isFalse);
      });

      test('returns false for too short', () {
        expect(utils.validateInput('hi', 3, 10), isFalse);
      });

      test('returns false for too long', () {
        expect(utils.validateInput('hello world!', 3, 10), isFalse);
      });
    });
  });
}
