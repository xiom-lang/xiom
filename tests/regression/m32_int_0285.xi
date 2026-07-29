// M32: UInt16 comparison: 50000 < 60000 must be true
fn main() -> Int {
  var a: UInt16 = 50000;
  var b: UInt16 = 60000;
  if a < b && b > a { return 0; }
  return 1;
}
