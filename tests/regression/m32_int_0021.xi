// M32: Int32 minimum value -2147483648
fn main() -> Int {
  var x: Int32 = -2147483648;
  var y: Int32 = -2147483647;
  if x < y && x == -2147483648 as Int32 {
    return 0;
  }
  return 1;
}
