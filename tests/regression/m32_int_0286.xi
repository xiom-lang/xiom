// M32: UInt32 comparison at high boundary (3000000000 > 2000000000)
fn main() -> Int {
  var a: UInt32 = 3000000000;
  var b: UInt32 = 2000000000;
  if a > b { return 0; }
  return 1;
}
