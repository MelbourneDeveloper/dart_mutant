/// Side-effect-heavy code whose only mutatable constructs are statement-level
/// calls to void-returning methods (their return value is discarded). Used to
/// verify dart_mutant generates "remove the call" mutations. See GitHub issue #4.
class Worker {
  void run(Worker other) {
    other.validate();
    other.persist();
    notify();
  }

  Future<void> flush(Worker other) async {
    await other.drain();
  }

  // The call below is a braceless `if` body. Removing it would leave
  // `if (flag)` with no statement — invalid Dart — so it must NOT be mutated.
  void maybeSkip(bool flag, Worker other) {
    if (flag) other.skip();
  }

  void validate() {}
  void persist() {}
  void notify() {}
  void skip() {}
  Future<void> drain() async {}
}
