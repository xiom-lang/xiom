module m21_struct_mut_018
type Pos = { x: Int; y: Int; }

  fn shiftX(p: Pos, dx: Int) -> Pos {
    var result: Pos = p;
    result.x = result.x + dx;
    return result;
  }

pub fn run() -> Int {
    var p: Pos = { x: 10; y: 5; };
    var p2 = shiftX(p, 7);
    if p2.x == 17 && p2.y == 5 { return 0; }
    return 1;
  }
use m21_struct_mut_018.run;
fn main() -> Int { return run(); }
