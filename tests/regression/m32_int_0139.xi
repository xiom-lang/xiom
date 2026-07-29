// M32: Int32 cast from Int16 (sign extension)
fn main() -> Int {
  var a: Int16 = -1 as Int16;
  var b: Int32 = a as Int32;
  if b == -1 as Int32 { return 0; }
  return 1;
}
