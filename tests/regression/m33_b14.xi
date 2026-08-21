// M33-B14: Borrow with generic -- generic fn takes &T, dereferences and returns
fn deref[T](x: &T) -> T { return *x; }
fn main() -> Int {
  var a = 88;
  var b = deref(&a);
  if b == 88 { return 0; }
  return 1;
}
