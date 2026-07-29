// M32: Nested type alias with narrow int and field widening
type Pixel = { r: UInt8; g: UInt8; b: UInt8; a: UInt8 };
fn main() -> Int {
  var p: Pixel = Pixel{ r: 255, g: 128, b: 0, a: 255 };
  var gray: Int = p.r as Int + p.g as Int + p.b as Int;
  if gray == 383 { return 0; }
  return 1;
}
