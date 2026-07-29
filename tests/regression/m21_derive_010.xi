module m21_derive_010
type Vec3D = { x: Float64; y: Float64; z: Float64; } derive[Eq, Clone, Display]

  pub fn run() -> Int {
    var v: Vec3D = { x: 1.0; y: 2.0; z: 3.0; };
    if v.x + v.y + v.z == 6.0 { return 0; }
    return 1;
  }
use m21_derive_010.run;
fn main() -> Int { return run(); }
