// M32: Hex literal operations
fn main() -> Int {
  var a: Int = 0xDEAD;
  var b: Int = 0xBEEF;
  var sum: Int = a + b;
  if sum == 0x19D9C {
    return 0;
  }
  return 1;
}
