module m21_struct_mut_022
type Vector2 = { x: Float64; y: Float64; }

  pub fn run() -> Int {
    var v: Vector2 = { x: 1.0; y: 2.0; };
    v.x = 3.5;
    v.y = 4.5;
    if v.x == 3.5 && v.y == 4.5 { return 0; }
    return 1;
  }
use m21_struct_mut_022.run;
fn main() -> Int { return run(); }
