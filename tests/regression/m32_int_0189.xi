// M32: Char arithmetic (add wraps as UInt8)
fn main() -> Int {
  var a: UInt8 = 'A' as UInt8;
  var b: UInt8 = 1;
  var c: UInt8 = a + b;
  if c == 66 as UInt8 { return 0; }
  return 1;
}
