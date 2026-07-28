// M32: Unary negation on Int8
fn main() -> Int {
  var a: Int8 = -128;
  var b: Int8 = -a;
  if b == -128 as Int8 {
    return 0;
  }
  return 1;
}
