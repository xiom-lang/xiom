// M34-V13: Float in struct field
type Point = { x: Float64; y: Float64; }
fn distance_sq(p: Point) -> Float64 {
  return p.x * p.x + p.y * p.y;
}
fn main() -> Int {
  var p = Point{ x: 3.0; y: 4.0; };
  var d2 = distance_sq(p);
  var neg = Point{ x: -1.0; y: -2.5; };
  if d2 == 25.0 && neg.x == -1.0 && neg.y == -2.5 {
    return 0;
  }
  return 1;
}
