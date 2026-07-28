// M34-W08: Bit isolation — mask & value extracts specific bits
fn main() -> Int {
  var a: Int = 0xABCD;
  // low byte: a & 0xFF
  var lo: Int = a & 0xFF;
  // mid byte: (a >> 8) & 0xFF
  var mid: Int = (a >> 8) & 0xFF;
  // high nibble of low byte: (a & 0xFF) >> 4
  var nib: Int = (a & 0xFF) >> 4;
  // extract using multiple masks
  var m1: Int = a & 0xF0;
  var m2: Int = a & 0x0F;
  var m3: Int = a & 0xAA;
  if lo == 0xCD && mid == 0xAB && nib == 0xC &&
     m1 == 0xC0 && m2 == 0x0D && m3 == 0x88 { return 0; }
  return 1;
}
