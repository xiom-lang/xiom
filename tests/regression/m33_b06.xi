// M33-B06: Struct field access through borrow -- read struct fields via & reference
type Point = { x: Int; y: Int; }
fn get_x(p: &Point) -> Int { return p.x; }
fn main() -> Int {
  var pt = Point{ x: 5; y: 10; };
  var vx = get_x(&pt);
  if vx == 5 && pt.x == 5 && pt.y == 10 { return 0; }
  return 1;
}
