// M32-S03: Field copy — copy fields between two struct instances
type Point = { x: Float64; y: Float64; }
fn main() -> Int {
  var a = Point{ x: 5.0; y: 10.0; };
  var b = Point{ x: a.x; y: a.y; };
  b.x = 99.0;
  if a.x == 5.0 && a.y == 10.0 && b.x == 99.0 && b.y == 10.0 { return 0; }
  return 1;
}
