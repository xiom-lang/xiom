// M32: UInt8 wraparound addition chain (252+1=253, +1=254, +1=255, +1=0)
fn main() -> Int {
  var a: UInt8 = 252;
  var r: UInt8 = a + 4 as UInt8;
  if r == 0 as UInt8 { return 0; }
  return 1;
}
