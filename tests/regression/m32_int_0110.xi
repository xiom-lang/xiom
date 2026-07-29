// M32: Int8 sign extension from negative -1
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: Int16 = a as Int16;
  if b == -1 as Int16 { return 0; }
  return 1;
}
