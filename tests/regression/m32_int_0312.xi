// M32: UInt16 right shift (32768 >> 8 = 128, logical)
fn main() -> Int {
  var a: UInt16 = 32768;
  var b: UInt16 = 8;
  var c: UInt16 = a >> b;
  if c == 128 as UInt16 { return 0; }
  return 1;
}
