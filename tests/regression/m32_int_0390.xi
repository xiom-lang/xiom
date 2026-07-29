// M32: Multiple comparison operators: Int8 >= and <=
fn main() -> Int {
  var min: Int8 = -128 as Int8;
  var max: Int8 = 127;
  if min <= -128 as Int8 && max >= 127 && min <= max && max >= min { return 0; }
  return 1;
}
