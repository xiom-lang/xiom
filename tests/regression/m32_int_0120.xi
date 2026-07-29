// M32: Int16 shift left (1 << 15 = -32768)
fn main() -> Int {
  var a: Int16 = 1;
  var b: Int16 = 15;
  var c: Int16 = a << b;
  if c == -32768 as Int16 { return 0; }
  return 1;
}
