// M32: UInt8(255) as Int64 must be 255, not -1 (zext vs sext)
fn main() -> Int {
  var a: UInt8 = 255;
  var b: Int64 = a as Int64;
  if b == 255 as Int64 { return 0; }
  return 1;
}
