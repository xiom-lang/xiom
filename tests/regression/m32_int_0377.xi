// M32: Int16(16384) * Int16(4) = 0 (wraps twice)
fn main() -> Int {
  var a: Int16 = 16384;
  var b: Int16 = 4;
  var c: Int16 = a * b;
  if c == 0 as Int16 { return 0; }
  return 1;
}
