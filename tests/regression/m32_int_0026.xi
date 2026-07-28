// M32: Int32 division
fn main() -> Int {
  var a: Int32 = 2000000000;
  var b: Int32 = 10000;
  var div: Int32 = a / b;
  if div == 200000 as Int32 {
    return 0;
  }
  return 1;
}
