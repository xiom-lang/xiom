// M32: Int16 division
fn main() -> Int {
  var a: Int16 = 30000;
  var b: Int16 = 7;
  var div: Int16 = a / b;
  if div == 4285 as Int16 {
    return 0;
  }
  return 1;
}
