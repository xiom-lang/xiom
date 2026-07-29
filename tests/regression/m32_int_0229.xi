// M32: Int8 div by -1 on max negative (-128 / -1 overflows)
fn main() -> Int {
  var a: Int8 = -128 as Int8;
  var b: Int8 = -1 as Int8;
  var c: Int8 = a / b;
  if c == -128 as Int8 { return 0; }
  return 1;
}
