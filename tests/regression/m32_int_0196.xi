// M32: Mixed arithmetic Int8 + UInt8 (promotion)
fn main() -> Int {
  var a: Int8 = 100;
  var b: UInt8 = 27;
  var c: Int8 = a + b as Int8;
  if c == 127 as Int8 { return 0; }
  return 1;
}
