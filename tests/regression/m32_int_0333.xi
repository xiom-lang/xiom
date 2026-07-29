// M32: Swap two Int16 values with XOR trick
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int16 = 32767;
  a = a ^ b;
  b = a ^ b;
  a = a ^ b;
  if a == 32767 && b == -32768 as Int16 { return 0; }
  return 1;
}
