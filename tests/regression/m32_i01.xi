// M32: Int8 range classification with elif
fn main() -> Int {
  var x: Int8 = 127;
  if x > 100 as Int8 {
    return 0;
  } elif x > 50 as Int8 {
    return 1;
  } elif x > 0 as Int8 {
    return 2;
  } else {
    return 3;
  }
  return 4;
}
