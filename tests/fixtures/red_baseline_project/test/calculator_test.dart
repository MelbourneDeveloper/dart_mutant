import 'package:red_baseline_project/calculator.dart';
import 'package:test/test.dart';

void main() {
  // This expectation is INTENTIONALLY wrong so the suite fails on UNMUTATED
  // code (a "red baseline"). dart_mutant must refuse to run and must NOT report
  // every mutant as killed. See GitHub issue #5.
  test('add (intentionally wrong to create a red baseline)', () {
    final calc = Calculator();
    expect(calc.add(2, 2), equals(5));
  });
}
