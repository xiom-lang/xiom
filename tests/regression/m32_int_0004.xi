// M32: Int8 subtraction crossing zero
fn main() -> Int {
  var a: Int8 = -128;
  var b: Int8 = 1;
  var sub: Int8 = a - b;
  if sub == 127 as Int8 {
    return 0;
  }
  return 1;
}
