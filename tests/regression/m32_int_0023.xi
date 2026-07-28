// M32: Int32 addition
fn main() -> Int {
  var a: Int32 = 1000000;
  var b: Int32 = 2000000;
  var sum: Int32 = a + b;
  if sum == 3000000 as Int32 {
    return 0;
  }
  return 1;
}
