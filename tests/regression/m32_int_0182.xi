// M32: UInt64 mul wraparound (large * 2)
fn main() -> Int {
  var a: UInt64 = 9223372036854775808;
  var b: UInt64 = 2;
  var c: UInt64 = a * b;
  if c == 0 as UInt64 { return 0; }
  return 1;
}
