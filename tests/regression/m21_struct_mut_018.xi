module m21_struct_mut_018
type Pos = { x: Int; y: Int; }
fn main() -> Int {
  var p: Pos = Pos{ x: 10; y: 5; };
  var p2 = shiftX(p, 7);
  if p2.x == 17 && p2.y == 5 { return 0; }
  return 1;
  return 1;
}
