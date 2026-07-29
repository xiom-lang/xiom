// M32: Struct with UInt8 field
type Color = { r: UInt8; g: UInt8; b: UInt8 };
fn main() -> Int {
  var c: Color = Color{ r: 255, g: 255, b: 255 };
  var sum: UInt8 = c.r + c.g;
  if sum == 254 as UInt8 { return 0; }
  return 1;
}
