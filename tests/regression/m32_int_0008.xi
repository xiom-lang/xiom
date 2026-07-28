// M32: Int8 overflow guard — add near max
fn main() -> Int {
  var a: Int8 = 120;
  var b: Int8 = 10;
  var sum: Int8 = a + b;
  if sum == -126 as Int8 {
    return 0;
  }
  return 1;
}
