// M32: UInt16 at max boundary (65535 + 1 = 0)
fn main() -> Int {
  var max: UInt16 = 65535;
  var x: UInt16 = max + 1 as UInt16;
  if x == 0 as UInt16 { return 0; }
  return 1;
}
