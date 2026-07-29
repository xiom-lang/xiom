// M32: Unary minus on UInt8 literal: 0 - 1 wraparound
fn main() -> Int {
  var a: UInt8 = 0;
  var b: UInt8 = -a;
  if b == 0 as UInt8 { return 0; }
  return 1;
}
