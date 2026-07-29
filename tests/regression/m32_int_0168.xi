// M32: UInt16 bitwise XOR
fn main() -> Int {
  var a: UInt16 = 65535;
  var b: UInt16 = a ^ a;
  if b == 0 as UInt16 { return 0; }
  return 1;
}
