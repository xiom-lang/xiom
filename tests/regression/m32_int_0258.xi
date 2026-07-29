// M32: UInt16(40000) as Int should be 40000, not -25536 (zext vs sext)
fn main() -> Int {
  var a: UInt16 = 40000;
  var b: Int = a as Int;
  if b == 40000 { return 0; }
  return 1;
}
