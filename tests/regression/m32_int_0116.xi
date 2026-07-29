// M32: Int16 mul wraparound (16384 * 2 = -32768)
fn main() -> Int {
  var a: Int16 = 16384;
  var b: Int16 = 2;
  var c: Int16 = a * b;
  if c == -32768 as Int16 { return 0; }
  return 1;
}
