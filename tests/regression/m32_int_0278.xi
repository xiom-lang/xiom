// M32: Function taking UInt16 param widening to Int
fn widen16(a: UInt16) -> Int {
  return a as Int;
}
fn main() -> Int {
  var x: UInt16 = 50000;
  var y: Int = widen16(x);
  if y == 50000 { return 0; }
  return 1;
}
