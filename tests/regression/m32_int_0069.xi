// M32: Left shift on UInt32
fn main() -> Int {
  var a: UInt32 = 1;
  var result: UInt32 = a << 31;
  if result == 2147483648 as UInt32 {
    return 0;
  }
  return 1;
}
