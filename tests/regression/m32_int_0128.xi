// M32: Int32 sub wraparound (-2147483648 - 1 = 2147483647)
fn main() -> Int {
  var a: Int32 = -2147483648 as Int32;
  var b: Int32 = 1;
  var c: Int32 = a - b;
  if c == 2147483647 as Int32 { return 0; }
  return 1;
}
