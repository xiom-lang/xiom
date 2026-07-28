// M32: UInt16 maximum value 65535
fn main() -> Int {
  var x: UInt16 = 65535;
  var y: UInt16 = 65534;
  if x > y && x == 65535 as UInt16 {
    return 0;
  }
  return 1;
}
