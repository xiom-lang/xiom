// M32: Bitwise AND on Int8
fn main() -> Int {
  var a: Int8 = 0x7F;
  var b: Int8 = 0x0F;
  var result: Int8 = a & b;
  if result == 15 as Int8 {
    return 0;
  }
  return 1;
}
