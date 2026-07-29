// M32: UInt8 to Bool: non-zero is truthy
fn main() -> Int {
  var a: UInt8 = 255;
  var b: Bool = a as Bool;
  if b { return 0; }
  return 1;
}
