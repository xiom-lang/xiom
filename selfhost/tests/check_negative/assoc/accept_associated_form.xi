// Phase 3 associated-form gate, accept case: `Eq[T].eq` dispatches to the
// bound interface member.
interface Eq[T] {
  fn eq(a: T, b: T) -> Bool;
}

impl Eq[Int] {
  fn eq(a: Int, b: Int) -> Bool {
    return a == b;
  }
}

fn call[T: Eq](x: T, y: T) -> Bool {
  return Eq[T].eq(x, y);
}

fn main() -> Int {
  if call(1, 2) { return 1; }
  return 0;
}
