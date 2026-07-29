// M32: Shl + shr roundtrip for UInt8 (logical)
fn main() -> Int {
  var a: UInt8 = 128;
  var b: UInt8 = a << 2;
  var c: UInt8 = b >> 2;
  if c == 32 as UInt8 { return 0; }
  return 1;
}
