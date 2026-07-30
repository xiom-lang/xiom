module m21_struct_mut_034
type Point = { x: Int; y: Int; }
fn main() -> Int {
  var p: Point = make_point(1, 2);
  p.x = 99;
  if p.x == 99 && p.y == 2 { return 0; }
  return 1;
  return 1;
}
