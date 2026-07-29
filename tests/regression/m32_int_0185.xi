// M32: UInt64 shift left (1 << 63)
fn main() -> Int {
  var a: UInt64 = 1;
  var b: UInt64 = 63;
  var c: UInt64 = a << b;
  if c == 9223372036854775808 as UInt64 { return 0; }
  return 1;
}
