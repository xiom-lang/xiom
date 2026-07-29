// M32: Narrow int in if condition
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = 127 as Int8;
  if a < b && a == -128 as Int8 && b == 127 as Int8 { return 0; }
  return 1;
}
