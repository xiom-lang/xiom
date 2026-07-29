// M32: UInt32 shift left (1 << 31 = 2147483648)
fn main() -> Int {
  var a: UInt32 = 1;
  var b: UInt32 = 31;
  var c: UInt32 = a << b;
  if c == 2147483648 as UInt32 { return 0; }
  return 1;
}
