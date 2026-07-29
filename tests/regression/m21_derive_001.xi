module m21_derive_001
type Vec2 = { x: Float64; y: Float64; } derive[Eq, Clone]

  pub fn run() -> Int {
    var a: Vec2 = { x: 1.0; y: 2.0; };
    if a.x == 1.0 { return 0; }
    return 1;
  }
use m21_derive_001.run;
fn main() -> Int { return run(); }
