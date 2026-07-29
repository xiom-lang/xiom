// M32: Int8 add wraparound (127 + 1 = -128)
fn main() -> Int {
  var a: Int8 = 127;
  var b: Int8 = 1;
  var c: Int8 = a + b;
  if c == -128 as Int8 { return 0; }
  return 1;
}
