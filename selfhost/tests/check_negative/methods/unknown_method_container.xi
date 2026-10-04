// Phase 3 method gate: unknown method on a builtin container receiver.
fn main() -> Int {
  var v = Vec[Int].new();
  v.nope();
  return 0;
}
