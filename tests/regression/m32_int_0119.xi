// M32: Int16 mod with negative (-32768 % 5 = -3)
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int16 = 5;
  var c: Int16 = a % b;
  if c == -3 as Int16 { return 0; }
  return 1;
}
