// M34-J14: Cross-module type reference -- modules referencing each other's types
module types {
  pub type Coord = { x: Int; y: Int; }
  pub fn origin() -> Coord { return Coord{ x: 0; y: 0; }; }
}
module ops {
  use types.Coord;
  pub fn add_coords(a: Coord, b: Coord) -> Coord {
    return Coord{ x: a.x + b.x, y: a.y + b.y };
  }
  pub fn scale_coord(c: Coord, s: Int) -> Coord {
    return Coord{ x: c.x * s, y: c.y * s };
  }
}
use types.Coord;
use ops.add_coords;
use ops.scale_coord;
fn main() -> Int {
  var a = Coord{ x: 3, y: 4 };
  var b = Coord{ x: 7, y: 1 };
  var sum = add_coords(a, b);
  var scaled = scale_coord(a, 2);
  if sum.x == 10 && sum.y == 5 && scaled.x == 6 && scaled.y == 8 { return 0; }
  return 1;
}
