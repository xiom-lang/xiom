// M32: UInt8(128) as Int16 must be 128, not -128 (zext vs sext bug)
fn main() -> Int {
  var a: UInt8 = 128;
  var b: Int16 = a as Int16;
  if b == 128 as Int16 { return 0; }
  return 1;
}
