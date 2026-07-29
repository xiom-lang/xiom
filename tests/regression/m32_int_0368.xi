// M32: UInt8 increment past 255 wraps to 0 multiple times
fn main() -> Int {
  var a: UInt8 = 252;
  var b: UInt8 = a + 1 as UInt8;
  var c: UInt8 = b + 1 as UInt8;
  var d: UInt8 = c + 1 as UInt8;
  var e: UInt8 = d + 1 as UInt8;
  if e == 0 as UInt8 { return 0; }
  return 1;
}
