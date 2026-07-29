// M32: Mixed sign compare Int16 vs UInt16
fn main() -> Int {
  var a: Int16 = 32767;
  var b: UInt16 = 32768;
  if a < b as Int16 { return 0; }
  return 1;
}
