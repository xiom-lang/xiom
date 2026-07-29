// M32: UInt8 to Float32 roundtrip
fn main() -> Int {
  var a: UInt8 = 255;
  var f: Float32 = a as Float32;
  var b: UInt8 = f as UInt8;
  if b == a { return 0; }
  return 1;
}
