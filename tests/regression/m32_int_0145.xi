// M32: Int64 div edge (min / -1)
fn main() -> Int {
  var a: Int64 = -9223372036854775808 as Int64;
  var b: Int64 = -1 as Int64;
  var c: Int64 = a / b;
  if c == a { return 0; }
  return 1;
}
