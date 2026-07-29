// M32: Int32 div edge (-2147483648 / -1 overflows)
fn main() -> Int {
  var a: Int32 = -2147483648 as Int32;
  var b: Int32 = -1 as Int32;
  var c: Int32 = a / b;
  if c == -2147483648 as Int32 { return 0; }
  return 1;
}
