// M32: UInt8(254) as UInt16 preserves value (zext identity)
fn main() -> Int {
  var a: UInt8 = 254;
  var b: UInt16 = a as UInt16;
  if b == 254 as UInt16 { return 0; }
  return 1;
}
