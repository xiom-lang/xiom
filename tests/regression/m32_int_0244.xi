// M32: Int8 underflow sub with negative (-128 - 127 = 1)
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = 127;
  var c: Int8 = a - b;
  if c == 1 as Int8 { return 0; }
  return 1;
}
