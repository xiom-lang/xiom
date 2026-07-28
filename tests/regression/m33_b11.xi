// M33-B11: Borrow in if scope — & ref to struct used within conditional branch
type Cell = { v: Int; }
fn read_cell(c: &Cell) -> Int { return c.v; }
fn main() -> Int {
  var a = Cell{ v: 25; };
  var result = 0;
  if true {
    result = read_cell(&a);
  }
  if result == 25 && a.v == 25 { return 0; }
  return 1;
}
