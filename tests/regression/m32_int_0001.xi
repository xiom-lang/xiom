// M32: Int8 minimum value -128
fn main() -> Int {
  var x: Int8 = -128;
  var y: Int8 = -127;
  if x < y && x == -128 as Int8 {
    return 0;
  }
  return 1;
}
