// M32-M08: pub type — public struct type in module
module geom {
  pub type Point3D = { x: Int; y: Int; z: Int; }
  pub fn origin() -> Point3D { return Point3D{ x: 0; y: 0; z: 0; }; }
}
use geom.Point3D;
fn main() -> Int {
  var p = geom.origin();
  if p.x == 0 && p.y == 0 && p.z == 0 { return 0; }
  return 1;
}
