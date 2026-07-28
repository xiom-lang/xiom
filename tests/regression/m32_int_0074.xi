// M32: Unary negation on Int32
fn main() -> Int {
  var a: Int32 = 1000;
  var b: Int32 = -a;
  if b == -1000 as Int32 {
    return 0;
  }
  return 1;
}
