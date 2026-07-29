// M32: UInt8 division (200 / 3 = 66)
fn main() -> Int {
  var a: UInt8 = 200;
  var b: UInt8 = 3;
  var c: UInt8 = a / b;
  if c == 66 as UInt8 { return 0; }
  return 1;
}
