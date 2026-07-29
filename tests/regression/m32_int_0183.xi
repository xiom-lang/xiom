// M32: UInt64 div at max
fn main() -> Int {
  var a: UInt64 = 18446744073709551615;
  var b: UInt64 = 1;
  var c: UInt64 = a / b;
  if c == a { return 0; }
  return 1;
}
