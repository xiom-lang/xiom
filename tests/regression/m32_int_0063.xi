// M32: Bitwise XOR on Int
fn main() -> Int {
  var a: Int = 0xFF;
  var b: Int = 0x0F;
  var result: Int = a ^ b;
  if result == 0xF0 {
    return 0;
  }
  return 1;
}
