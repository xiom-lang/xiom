// M32: Int32 multiplication
fn main() -> Int {
  var a: Int32 = 10000;
  var b: Int32 = 200000;
  var prod: Int32 = a * b;
  if prod == 2000000000 as Int32 {
    return 0;
  }
  return 1;
}
