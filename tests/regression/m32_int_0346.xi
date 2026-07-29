// M32: UInt8(128) as UInt16(128) as Int32 should = 128
fn main() -> Int {
  var a: UInt8 = 128;
  var b: Int32 = a as Int32;
  if b == 128 as Int32 { return 0; }
  return 1;
}
