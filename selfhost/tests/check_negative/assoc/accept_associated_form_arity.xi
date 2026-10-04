// Phase 3 associated-form gate, accept case: Rust's generic-bound path does
// NOT enforce arity for associated-form calls.
interface Eq[T] {
  fn eq(a: T, b: T) -> Bool;
}

impl Eq[Int] {
  fn eq(a: Int, b: Int) -> Bool {
    return a == b;
  }
}

fn call[T: Eq](x: T) -> Bool {
  return Eq[T].eq(x);
}
