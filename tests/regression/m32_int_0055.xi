// M32: Downcast Int64 -> Int32 truncation (large value truncates)
fn main() -> Int {
  var a: Int64 = 3000000000;
  var b: Int32 = a as Int32;
  if b == -1294967296 as Int32 {
    return 0;
  }
  return 1;
}
