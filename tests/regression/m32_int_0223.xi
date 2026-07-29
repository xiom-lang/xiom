// M32: Int8 zero wraparound identity
fn main() -> Int {
  var a: Int8 = 0;
  var b: Int8 = a + 127 as Int8;
  var c: Int8 = b + 127 as Int8;
  if c == -2 as Int8 { return 0; }
  return 1;
}
