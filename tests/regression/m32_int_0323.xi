// M32: Char comparison: 'A'(65) < 'z'(122) as UInt8
fn main() -> Int {
  var a: UInt8 = 'A' as UInt8;
  var z: UInt8 = 'z' as UInt8;
  if a < z { return 0; }
  return 1;
}
