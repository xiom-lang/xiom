// M32: Int8 overflow add with large negative values
fn main() -> Int {
  var a: Int8 = -64 as Int8;
  var b: Int8 = -65 as Int8;
  var c: Int8 = a + b;
  if c == 127 as Int8 { return 0; }
  return 1;
}
