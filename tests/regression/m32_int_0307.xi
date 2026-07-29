// M32: UInt8 bitwise AND with sign bit set (128 & 128 = 128)
fn main() -> Int {
  var a: UInt8 = 128;
  var b: UInt8 = 128;
  var c: UInt8 = a & b;
  if c == 128 as UInt8 { return 0; }
  return 1;
}
