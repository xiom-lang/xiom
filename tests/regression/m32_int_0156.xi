// M32: UInt8 div at max value (255 / 1 = 255)
fn main() -> Int {
  var a: UInt8 = 255;
  var b: UInt8 = 1;
  var c: UInt8 = a / b;
  if c == 255 as UInt8 { return 0; }
  return 1;
}
