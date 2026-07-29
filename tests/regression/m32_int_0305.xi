// M32: UInt8 bitwise NOT via XOR with all-ones (255 ^ x)
fn main() -> Int {
  var a: UInt8 = 170;
  var mask: UInt8 = 255;
  var b: UInt8 = a ^ mask;
  if b == 85 as UInt8 { return 0; }
  return 1;
}
