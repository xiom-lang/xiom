// M34-J18: Module with derive -- struct with derive inside modules
module models {
  pub type Point = { x: Int; y: Int; } derive[Eq]
  pub type Vec3 = { a: Float64; b: Float64; c: Float64; } derive[Eq]
  pub fn make_origin() -> Point { return Point{ x: 0; y: 0; }; }
  pub fn make_unit() -> Vec3 { return Vec3{ a: 1.0; b: 1.0; c: 1.0; }; }
}
use models.Point;
use models.Vec3;
use models.make_origin;
use models.make_unit;
fn main() -> Int {
  var p1 = make_origin();
  var p2 = make_origin();
  if p1 == p2 { } else { return 1; }
  var v1 = Vec3{ a: 1.0, b: 1.0, c: 1.0 };
  var v2 = make_unit();
  if v1 == v2 { return 0; }
  return 2;
}
