// M32: Function returning struct with narrow int fields
type Vec3u8 = { x: UInt8; y: UInt8; z: UInt8 };
fn make_vec3u8() -> Vec3u8 {
  return Vec3u8{ x: 100, y: 200, z: 50 };
}
fn main() -> Int {
  var v: Vec3u8 = make_vec3u8();
  var sum: Int = v.x as Int + v.y as Int + v.z as Int;
  if sum == 350 { return 0; }
  return 1;
}
