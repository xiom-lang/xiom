// M32: Integer comparison chain — all operators
fn main() -> Int {
  var a: Int = 10;
  var b: Int = 5;
  var c: Int = 10;
  if a > b && a >= c && b < a && b <= c && a == c && a != b {
    return 0;
  }
  return 1;
}
