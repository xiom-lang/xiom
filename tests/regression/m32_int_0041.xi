// M32: UInt8 maximum value 255
fn main() -> Int {
  var x: UInt8 = 255;
  var y: UInt8 = 254;
  if x > y && x == 255 as UInt8 {
    return 0;
  }
  return 1;
}
