// M32: Shl + shr roundtrip: (x << 4) >> 4 for Int8 mask
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = a << 4;
  var c: Int8 = b >> 4;
  if c == -8 as Int8 { return 0; }
  return 1;
}
