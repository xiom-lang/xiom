// M32: Bitwise NOT on Int
fn main() -> Int {
  var a: Int = 0;
  var result: Int = ~a;
  if result == -1 {
    return 0;
  }
  return 1;
}
