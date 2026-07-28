// M32: Logical NOT on integer comparison results
fn main() -> Int {
  var a: Int = 5;
  var b: Int = 10;
  if !(a > b) && !(a == b) && (a < b) {
    return 0;
  }
  return 1;
}
