// M32: Mixed sign cast UInt8 to Int8 (255 -> -1)
fn main() -> Int {
  var a: UInt8 = 255;
  var b: Int8 = a as Int8;
  if b == -1 as Int8 { return 0; }
  return 1;
}
