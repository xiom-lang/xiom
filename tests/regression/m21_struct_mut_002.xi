module m21_struct_mut_002
type Point = { x: Int; y: Int; }

  pub fn run() -> Int {
    var p: Point = { x: 10; y: 20; };
    var old_x = p.x;
    p.x = 42;
    if old_x == 10 && p.x == 42 { return 0; }
    return 1;
  }
use m21_struct_mut_002.run;
fn main() -> Int { return run(); }
