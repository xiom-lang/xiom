// M32: UInt32(2147483648) as Int64 must be positive, not negative (zext)
fn main() -> Int {
  var a: UInt32 = 2147483648;
  var b: Int64 = a as Int64;
  if b > 0 { return 0; }
  return 1;
}
