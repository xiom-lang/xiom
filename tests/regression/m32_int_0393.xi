// M32: Shl + shr roundtrip for UInt8 (no overflow)
fn main() -> Int {
  var a: UInt8 = 16 as UInt8;
  var b: UInt8 = a << 2;
  var c: UInt8 = b >> 2;
  if c == 16 as UInt8 { return 0; }
  return 1;
}
