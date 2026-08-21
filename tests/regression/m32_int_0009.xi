// M32: Int8 overflow guard -- mul near max
fn main() -> Int {
  var a: Int8 = 64;
  var b: Int8 = 2;
  var prod: Int8 = a * b;
  if prod == -128 as Int8 {
    return 0;
  }
  return 1;
}
