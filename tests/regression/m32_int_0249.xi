// M32: UInt8 max boundary operations (add near max)
fn main() -> Int {
  var a: UInt8 = 254;
  var b: UInt8 = 1;
  var c: UInt8 = a + b;
  var d: UInt8 = c + b;
  if c == 255 as UInt8 && d == 0 as UInt8 { return 0; }
  return 1;
}
