// M32: Int32 comparisons at boundaries
fn main() -> Int {
  var min: Int32 = -2147483648 as Int32;
  var max: Int32 = 2147483647;
  if min < max { return 0; }
  return 1;
}
