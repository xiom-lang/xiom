// M32: Right shift on Int (arithmetic for signed)
fn main() -> Int {
  var a: Int = 1024;
  var result: Int = a >> 10;
  if result == 1 {
    return 0;
  }
  return 1;
}
