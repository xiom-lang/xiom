// M32: Int64 neg of min value
fn main() -> Int {
  var a: Int64 = -9223372036854775808 as Int64;
  var b: Int64 = -a;
  if b == a { return 0; }
  return 1;
}
