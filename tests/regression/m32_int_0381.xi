// M32: Int16 arithmetic shift right: -32768 >> 15 = -1
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int16 = a >> 15;
  if b == -1 as Int16 { return 0; }
  return 1;
}
