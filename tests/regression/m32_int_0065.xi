// M32: Left shift on Int
fn main() -> Int {
  var a: Int = 1;
  var result: Int = a << 10;
  if result == 1024 {
    return 0;
  }
  return 1;
}
