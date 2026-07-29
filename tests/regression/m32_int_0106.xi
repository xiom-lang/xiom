// M32: Int8 shift left (1 << 7 = -128)
fn main() -> Int {
  var a: Int8 = 1;
  var b: Int8 = 7;
  var c: Int8 = a << b;
  if c == -128 as Int8 { return 0; }
  return 1;
}
