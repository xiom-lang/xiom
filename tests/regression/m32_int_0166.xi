// M32: UInt16 mul wraparound (256 * 256 = 0)
fn main() -> Int {
  var a: UInt16 = 256;
  var b: UInt16 = 256;
  var c: UInt16 = a * b;
  if c == 0 as UInt16 { return 0; }
  return 1;
}
