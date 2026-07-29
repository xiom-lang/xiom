// M32: Int32 multiple chained operations
fn main() -> Int {
  var a: Int32 = 1073741824;
  var b: Int32 = a * 2 as Int32;
  if b == -2147483648 as Int32 { return 0; }
  return 1;
}
