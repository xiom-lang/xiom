// M32-E04: Enum with multi-field payload, match extract
enum Point { Origin, Coord(x: Int, y: Int) }
fn main() -> Int {
  var p = Point.Coord(10, 20);
  match p {
    Origin => { return 1; }
    Coord(x, y) => {
      if x != 10 { return 2; }
      if y != 20 { return 3; }
    }
  }
  var o = Point.Origin;
  match o {
    Origin => {}
    Coord(_, _) => { return 4; }
  }
  return 0;
}
