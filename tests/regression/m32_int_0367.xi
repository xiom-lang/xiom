// M32: Int16 min * -1 should still be min (overflow)
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int16 = -1 as Int16;
  var c: Int16 = a * b;
  if c == -32768 as Int16 { return 0; }
  return 1;
}
