// M32: UInt16 wraparound addition (65530 + 10 = 4)
fn main() -> Int {
  var a: UInt16 = 65530;
  var b: UInt16 = 10;
  var c: UInt16 = a + b;
  if c == 4 as UInt16 { return 0; }
  return 1;
}
