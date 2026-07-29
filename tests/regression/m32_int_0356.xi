// M32: UInt8 0xAA (170) bit pattern through sext vs zext
fn main() -> Int {
  var a: UInt8 = 170;
  var b: Int16 = a as Int16;
  if b == 170 as Int16 { return 0; }
  return 1;
}
