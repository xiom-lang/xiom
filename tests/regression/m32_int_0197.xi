// M32: Struct with Int8 field
fn main() -> Int {
  type Point8 = { x: Int8; y: Int8 };
  var p: Point8 = Point8{ x: -128 as Int8, y: 127 as Int8 };
  var sum: Int8 = p.x + p.y;
  if sum == -1 as Int8 { return 0; }
  return 1;
}
