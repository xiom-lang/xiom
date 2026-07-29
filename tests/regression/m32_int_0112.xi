// M32: Int8 cast from Int32 (truncation of 383 = 256 + 127)
fn main() -> Int {
  var a: Int32 = 383;
  var b: Int8 = a as Int8;
  if b == 127 as Int8 { return 0; }
  return 1;
}
