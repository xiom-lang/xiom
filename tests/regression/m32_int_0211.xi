// M32: Function param Int8
fn add8(a: Int8, b: Int8) -> Int8 {
  return a + b;
}
fn main() -> Int {
  var c: Int8 = add8(127, 1);
  if c == -128 as Int8 { return 0; }
  return 1;
}
