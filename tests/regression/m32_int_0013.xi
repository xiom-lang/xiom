// M32: Int16 addition
fn main() -> Int {
  var a: Int16 = 10000;
  var b: Int16 = 20000;
  var sum: Int16 = a + b;
  if sum == 30000 as Int16 {
    return 0;
  }
  return 1;
}
