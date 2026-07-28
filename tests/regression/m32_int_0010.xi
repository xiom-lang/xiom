// M32: Int8 negative division
fn main() -> Int {
  var a: Int8 = -100;
  var b: Int8 = 3;
  var div: Int8 = a / b;
  if div == -33 as Int8 {
    return 0;
  }
  return 1;
}
