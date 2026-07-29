// M32: UInt8 all-ones OR with 0 = 255
fn main() -> Int {
  var a: UInt8 = 255;
  var b: UInt8 = 0;
  var c: UInt8 = a | b;
  if c == 255 as UInt8 { return 0; }
  return 1;
}
