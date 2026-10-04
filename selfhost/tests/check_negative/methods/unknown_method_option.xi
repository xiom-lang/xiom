// Phase 3 method gate: unknown method on an Option receiver.
fn f(o: Option[Int]) -> Int {
  o.nope();
  return 0;
}
