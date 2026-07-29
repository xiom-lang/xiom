// M32: UInt8(255) as Int16 must be 255, not -1 (zext vs sext bug)
fn main() -> Int {
  var a: UInt8 = 255;
  var b: Int16 = a as Int16;
  if b == 255 as Int16 { return 0; }
  return 1;
}
