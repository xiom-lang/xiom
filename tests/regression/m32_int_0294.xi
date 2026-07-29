// M32: UInt8 negation semantics (unsigned negation wraps)
fn main() -> Int {
  var a: UInt8 = 1;
  var b: UInt8 = 0 - a;
  var c: UInt8 = a + b;
  if c == 0 as UInt8 { return 0; }
  return 1;
}
