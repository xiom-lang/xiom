// M32: UInt32 right shift (2147483648 >> 1 = 1073741824, logical)
fn main() -> Int {
  var a: UInt32 = 2147483648;
  var b: UInt32 = 1;
  var c: UInt32 = a >> b;
  if c == 1073741824 as UInt32 { return 0; }
  return 1;
}
