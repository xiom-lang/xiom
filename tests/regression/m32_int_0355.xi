// M32: Int32 truncation chain Int32 -> Int16 -> Int8 on negative
fn main() -> Int {
  var a: Int32 = -1 as Int32;
  var b: Int16 = a as Int16;
  var c: Int8 = b as Int8;
  if c == -1 as Int8 { return 0; }
  return 1;
}
