// M32: Int64 negative modulo
fn main() -> Int {
  var a: Int64 = -3000000000007;
  var b: Int64 = 1000000;
  var mod_val: Int64 = a % b;
  if mod_val == -7 {
    return 0;
  }
  return 1;
}
