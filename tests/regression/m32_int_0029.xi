// M32: Int32 overflow -- mul wraps
fn main() -> Int {
  var a: Int32 = 65536;
  var b: Int32 = 32768;
  var prod: Int32 = a * b;
  if prod == -2147483648 as Int32 {
    return 0;
  }
  return 1;
}
