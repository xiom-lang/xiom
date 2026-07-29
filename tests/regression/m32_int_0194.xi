// M32: Mixed sign compare Int8 < UInt8 (sign matters)
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: UInt8 = 0;
  if a < b as Int8 { return 0; }
  return 1;
}
