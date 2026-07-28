fn is_some_and(o: Option[Int], pred: fn(Int) -> Bool) -> Bool {
  match o { Some(v) => pred(v), None => false }
}
fn positive(x: Int) -> Bool { return x > 0; }
fn eq100(x: Int) -> Bool { return x == 100; }
fn always_true(x: Int) -> Bool { return true; }
fn main() -> Int {
  if !is_some_and(Some(42), positive) { return 1; }
  if is_some_and(Some(-5), positive) { return 2; }
  if is_some_and(None, always_true) { return 3; }
  if !is_some_and(Some(100), eq100) { return 4; }
  return 0;
}
