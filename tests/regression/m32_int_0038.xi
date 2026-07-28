// M32: Int64 overflow — add near max wraps
fn main() -> Int {
  var a: Int64 = 9223372036854775807;
  var b: Int64 = 1;
  var sum: Int64 = a + b;
  if sum == -9223372036854775808 {
    return 0;
  }
  return 1;
}
