// M32: Function return Int8
fn get_max8() -> Int8 {
  return 127;
}
fn main() -> Int {
  var x: Int8 = get_max8();
  var y: Int8 = x + 1 as Int8;
  if y == -128 as Int8 { return 0; }
  return 1;
}
