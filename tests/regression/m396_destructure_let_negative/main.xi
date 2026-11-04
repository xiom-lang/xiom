// m396 lock: `let (a, b)` bindings are immutable; only the explicit
// `var (a, b)` form is assignable.
fn main() -> Int {
  let (a, b) = (1, 2);
  a = 3;
  return b;
}
