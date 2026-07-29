// M32: UInt32 div at max
fn main() -> Int {
  var a: UInt32 = 4294967295;
  var b: UInt32 = 1;
  var c: UInt32 = a / b;
  if c == a { return 0; }
  return 1;
}
