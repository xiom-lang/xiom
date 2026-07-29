// M32: Int8 cast from Int16 (truncation of 256)
fn main() -> Int {
  var a: Int16 = 256;
  var b: Int8 = a as Int8;
  if b == 0 as Int8 { return 0; }
  return 1;
}
