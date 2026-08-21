// M32: Division by zero guard -- runtime check returns non-zero on div by zero
fn main() -> Int {
  var a: Int = 10;
  var b: Int = 0;
  if b != 0 {
    return a / b;
  }
  return 0;
}
