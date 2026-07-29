// M32: Int32 shift left (1 << 31 = -2147483648)
fn main() -> Int {
  var a: Int32 = 1;
  var b: Int32 = 31;
  var c: Int32 = a << b;
  if c == -2147483648 as Int32 { return 0; }
  return 1;
}
