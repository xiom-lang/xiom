// M32: Mixed sign cast Int8 to UInt8 (-1 -> 255)
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: UInt8 = a as UInt8;
  if b == 255 as UInt8 { return 0; }
  return 1;
}
