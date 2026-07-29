// M32: UInt8 left shift crossing byte boundary (64 << 2 = 0 with wraparound in UInt8)
fn main() -> Int {
  var a: UInt8 = 64;
  var b: UInt8 = 2;
  var c: UInt8 = a << b;
  if c == 0 as UInt8 { return 0; }
  return 1;
}
