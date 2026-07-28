// M32-S07: Struct return from function
type Point = { x: Float64; y: Float64; }
fn origin() -> Point { return Point{ x: 0.0; y: 0.0; }; }
fn main() -> Int {
  var p = origin();
  var q = Point{ x: 3.5; y: 7.5; };
  if p.x == 0.0 && p.y == 0.0 && q.x == 3.5 && q.y == 7.5 { return 0; }
  return 1;
}
