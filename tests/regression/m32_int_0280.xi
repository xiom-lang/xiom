// M32: Function returning UInt16 with high value preserved
fn get_high16() -> UInt16 {
  return 65000;
}
fn main() -> Int {
  var x: UInt16 = get_high16();
  var y: Int = x as Int;
  if y == 65000 { return 0; }
  return 1;
}
