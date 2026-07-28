// M32: Int8 maximum value 127
fn main() -> Int {
  var x: Int8 = 127;
  var y: Int8 = 126;
  if x > y && x == 127 as Int8 {
    return 0;
  }
  return 1;
}
