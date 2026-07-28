// M32: Int64 negative division
fn main() -> Int {
  var a: Int64 = -3000000000000;
  var b: Int64 = 1000000;
  var div: Int64 = a / b;
  if div == -3000000 {
    return 0;
  }
  return 1;
}
