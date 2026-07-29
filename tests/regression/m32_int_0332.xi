// M32: Swap two UInt8 values with XOR trick (a ^= b; b ^= a; a ^= b)
fn main() -> Int {
  var a: UInt8 = 200;
  var b: UInt8 = 55;
  a = a ^ b;
  b = a ^ b;
  a = a ^ b;
  if a == 55 as UInt8 && b == 200 as UInt8 { return 0; }
  return 1;
}
