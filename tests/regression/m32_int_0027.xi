// M32: Int32 modulo
fn main() -> Int {
  var a: Int32 = 2000000000;
  var b: Int32 = 9999;
  var mod_val: Int32 = a % b;
  if mod_val == 200 as Int32 {
    return 0;
  }
  return 1;
}
