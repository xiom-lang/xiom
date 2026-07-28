// M32: Integer in array-like struct access
type Vec3 = { x: Int; y: Int; z: Int; }
fn dot(a: Vec3, b: Vec3) -> Int {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}
fn main() -> Int {
  var v1 = Vec3{ x: 1; y: 2; z: 3; };
  var v2 = Vec3{ x: 4; y: 5; z: 6; };
  var result: Int = dot(v1, v2);
  if result == 32 {
    return 0;
  }
  return 1;
}
