// M34-W13: Byte swap -- swap bytes in a 32-bit value using shift + mask
fn swap16(x: UInt32) -> UInt32 {
  var lo: UInt32 = x & (0xFF as UInt32);
  var hi: UInt32 = (x >> 8) & (0xFF as UInt32);
  return (lo << 8) | hi;
}
fn main() -> Int {
  var a: UInt32 = 0x1234 as UInt32;
  var r: UInt32 = swap16(a);
  if r == 0x3412 as UInt32 { return 0; }
  return 1;
}
