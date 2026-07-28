// M32: UInt arithmetic — no negative values
fn main() -> Int {
  var a: UInt = 100;
  var b: UInt = 200;
  var diff: UInt = b - a;
  if diff == 100 {
    return 0;
  }
  return 1;
}
