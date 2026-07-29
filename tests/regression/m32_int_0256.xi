// M32: UInt16(32768) as Int32 must be 32768, not -32768 (zext vs sext)
fn main() -> Int {
  var a: UInt16 = 32768;
  var b: Int32 = a as Int32;
  if b == 32768 as Int32 { return 0; }
  return 1;
}
