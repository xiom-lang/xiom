// M32: Shl + shr roundtrip for Int8 (no overflow)
fn main() -> Int {
  var a: Int8 = 1 as Int8;
  var b: Int8 = a << 3;
  var c: Int8 = b >> 3;
  if c == 1 as Int8 { return 0; }
  return 1;
}
