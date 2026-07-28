// M36-X15: Derive on all types — derive[Eq] on structs
type Point2D = { x: Int; y: Int; } derive[Eq]
type Point3D = { x: Int; y: Int; z: Int; } derive[Eq]
type Color = { r: Int; g: Int; b: Int; } derive[Eq]
type NamedId = { id: Int; value: Int; } derive[Eq]
type Vector = { dx: Float64; dy: Float64; } derive[Eq]
fn main() -> Int {
  var p1 = Point2D{ x: 1; y: 2; };
  var p2 = Point2D{ x: 1; y: 2; };
  var p3 = Point2D{ x: 2; y: 1; };
  if p1 != p2 { return 1; }
  if p1 == p3 { return 2; }
  var q1 = Point3D{ x: 1; y: 2; z: 3; };
  var q2 = Point3D{ x: 1; y: 2; z: 3; };
  var q3 = Point3D{ x: 1; y: 2; z: 4; };
  if q1 != q2 { return 3; }
  if q1 == q3 { return 4; }
  var n1 = NamedId{ id: 1; value: 42; };
  var n2 = NamedId{ id: 1; value: 42; };
  var n3 = NamedId{ id: 2; value: 42; };
  if n1 != n2 { return 5; }
  if n1 == n3 { return 6; }
  var c1 = Color{ r: 255; g: 0; b: 0; };
  var c2 = Color{ r: 255; g: 0; b: 0; };
  if c1 != c2 { return 7; }
  var v1 = Vector{ dx: 1.5; dy: 2.5; };
  var v2 = Vector{ dx: 1.5; dy: 2.5; };
  if v1 != v2 { return 8; }
  return 0;
}
