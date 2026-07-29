// M32: Int16 add wraparound (32767 + 1 = -32768)
fn main() -> Int {
  var a: Int16 = 32767;
  var b: Int16 = 1;
  var c: Int16 = a + b;
  if c == -32768 as Int16 { return 0; }
  return 1;
}
