// M32: UInt8 comparison with high values (200 < 255)
fn main() -> Int {
  var a: UInt8 = 200;
  var b: UInt8 = 255;
  if a < b && b > a { return 0; }
  return 1;
}
