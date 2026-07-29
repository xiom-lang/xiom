// M32: UInt8 to Int8 zero extension vs sign extension
fn main() -> Int {
  var a: UInt8 = 255;
  var b: Int16 = a as Int16;
  if b == 255 as Int16 { return 0; }
  return 1;
}
