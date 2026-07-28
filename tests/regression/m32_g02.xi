// M32-G02: Two-param generic function
fn pair[T](a: T, b: T) -> Int {
  return 0;
}
fn main() -> Int {
  var a: Int = pair(1, 2);
  var b: Int = pair(-5, -5);
  if a != 0 || b != 0 { return 1; }
  return 0;
}
