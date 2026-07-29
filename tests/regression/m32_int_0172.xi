// M32: UInt32 add wraparound (4294967295 + 1 = 0)
fn main() -> Int {
  var a: UInt32 = 4294967295;
  var b: UInt32 = 1;
  var c: UInt32 = a + b;
  if c == 0 as UInt32 { return 0; }
  return 1;
}
