// M32: Int8 all-ones (-1) bitwise AND with 127 = 127
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: Int8 = 127;
  var c: Int8 = a & b;
  if c == 127 as Int8 { return 0; }
  return 1;
}
