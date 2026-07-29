// M32: Nested if with UInt8 conditions
fn main() -> Int {
  var a: UInt8 = 200;
  var b: UInt8 = 100;
  if a > b {
    if a == 200 as UInt8 { return 0; }
  }
  return 1;
}
