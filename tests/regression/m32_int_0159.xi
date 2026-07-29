// M32: UInt8 shift left (1 << 7 = 128)
fn main() -> Int {
  var a: UInt8 = 1;
  var b: UInt8 = 7;
  var c: UInt8 = a << b;
  if c == 128 as UInt8 { return 0; }
  return 1;
}
