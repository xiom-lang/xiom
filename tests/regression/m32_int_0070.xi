// M32: Right shift on UInt32 (logical for unsigned)
fn main() -> Int {
  var a: UInt32 = 2147483648;
  var result: UInt32 = a >> 31;
  if result == 1 as UInt32 {
    return 0;
  }
  return 1;
}
