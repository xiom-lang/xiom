// Phase 3 associated-form gate: an unknown member falls through to the
// observed cascade (undefined interface twice, then cannot call at the dot).
interface Eq[T] {
  fn eq(a: T, b: T) -> Bool;
}

impl Eq[Int] {
  fn eq(a: Int, b: Int) -> Bool {
    return a == b;
  }
}

fn call[T: Eq](x: T, y: T) -> Bool {
  return Eq[T].nope(x, y);
}
