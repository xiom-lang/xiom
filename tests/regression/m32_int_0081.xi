// M32: Integer in struct field
type Point = { x: Int; y: Int; }
fn main() -> Int {
  var p = Point{ x: 10; y: 20; };
  var sum: Int = p.x + p.y;
  if sum == 30 {
    return 0;
  }
  return 1;
}
