module m21_struct_mut_001
type Point = { x: Int; y: Int; }
fn main() -> Int {
  var p: Point = Point{ x: 10; y: 20; };
  p.x = 42;
  if p.x == 42 && p.y == 20 { return 0; }
  return 1;
}
