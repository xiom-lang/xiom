// M32: Int8(127) < UInt8(128) with mixed sign widening
fn main() -> Int {
  var a: Int8 = 127;
  var b: UInt8 = 128;
  var ai: Int = a as Int;
  var bi: Int = b as Int;
  if bi == 128 && ai < bi { return 0; }
  return 1;
}
