// M32: Int8(-1) < UInt8(1) must be true (-1 sign-extended as signed)
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: UInt8 = 1;
  var ai: Int = a as Int;
  var bi: Int = b as Int;
  if ai < bi { return 0; }
  return 1;
}
