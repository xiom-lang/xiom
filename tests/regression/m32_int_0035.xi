// M32: Int64 multiplication
fn main() -> Int {
  var a: Int64 = 1000000;
  var b: Int64 = 2000000;
  var prod: Int64 = a * b;
  if prod == 2000000000000 {
    return 0;
  }
  return 1;
}
