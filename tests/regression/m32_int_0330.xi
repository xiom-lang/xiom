// M32: UInt8 0 to Bool is false
fn main() -> Int {
  var a: UInt8 = 0;
  var b: Bool = a as Bool;
  if !b { return 0; }
  return 1;
}
