// M32: Int8(-1) as UInt8 should be 255, then -1 shift right vs 255 shift right
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: UInt8 = a as UInt8;
  var si: Int8 = a >> 1;
  var ui: UInt8 = b >> 1;
  if si == -1 as Int8 && ui == 127 as UInt8 { return 0; }
  return 1;
}
