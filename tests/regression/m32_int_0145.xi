// M32: Int64 div edge (MIN / 1 = MIN)
fn main() -> Int {
  var a: Int64 = -9223372036854775808 as Int64;
  var b: Int64 = 1;
  var c: Int64 = a / b;
  if c == a { return 0; }
  return 1;
}
