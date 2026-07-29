module m21_struct_mut_019
type Rect = { x: Int; y: Int; w: Int; h: Int; }

  fn translate(r: Rect, dx: Int, dy: Int) -> Rect {
    var out: Rect = r;
    out.x = out.x + dx;
    out.y = out.y + dy;
    return out;
  }

  pub fn run() -> Int {
    var r: Rect = { x: 10; y: 20; w: 100; h: 50; };
    var r2 = translate(r, 5, 3);
    if r2.x == 15 && r2.y == 23 && r2.w == 100 && r2.h == 50 { return 0; }
    return 1;
  }
use m21_struct_mut_019.run;
fn main() -> Int { return run(); }
