// M32: Int8 all bits set (-1)
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: Int8 = a + 1 as Int8;
  if b == 0 as Int8 { return 0; }
  return 1;
}
