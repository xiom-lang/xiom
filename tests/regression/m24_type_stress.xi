// M24: Type stress -- many types, generics, nesting
type Point2D = { x: Float64; y: Float64; }
type Point3D = { x: Float64; y: Float64; z: Float64; }
type Named = { name: Str; val: Int; }
type Boxed = { id: Int; inner: Point2D; }
type Wrapper = { id: Int; point: Point2D; named: Named; }
fn make_point(x: Float64, y: Float64) -> Point2D { return Point2D{ x: x; y: y; }; }
fn make_wrapper(id: Int) -> Wrapper {
  var p = make_point(1.0, 2.0);
  var n = Named{ name: "test"; val: 42; };
  return Wrapper{ id: id; point: p; named: n; };
}
fn main() -> Int {
  var w = make_wrapper(100);
  if w.id == 100 && w.named.val == 42 { return 0; }
  return 1;
}
