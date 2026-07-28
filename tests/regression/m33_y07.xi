// M33-Y07: module + const + type + generic fn + enum payload + contract + impl + match + diff
const PI: Float64 = 3.14159;
type Circle = { radius: Float64; }
enum ShapeKind { Square(side: Float64), Circ(c: Circle) }
fn area[T](s: ShapeKind) -> Float64
  ensures: result >= 0.0
{
  match s {
    Square(side) => side * side,
    Circ(c) => PI * c.radius * c.radius,
  }
}
fn area_sq(side: Float64) -> Float64 { return side * side; }
interface ShapeOps { fn measure(self) -> Float64; }
impl ShapeOps for Circle {
  fn measure(self) -> Float64 { return self.radius * self.radius * 3.14159; }
}
module geom {
  pub fn do_area(s: ShapeKind) -> Float64 { return area(s); }
  pub fn do_sq(side: Float64) -> Float64 { return area_sq(side); }
  pub fn via_impl(c: Circle) -> Float64 { return c.measure(); }
}
use geom.do_area;
use geom.do_sq;
use geom.via_impl;
enum Route { Direct, Sq, Impl }
fn compute_route(r: Route, s: ShapeKind, c: Circle) -> Float64 {
  match r { Direct => do_area(s), Sq => do_sq(3.0), Impl => via_impl(c), }
}
fn main() -> Int {
  var side: Float64 = 3.0;
  var cir = Circle{ radius: 1.0; };
  var sq = ShapeKind.Square(side);
  var r1 = compute_route(Route.Direct, sq, cir);
  var r2 = compute_route(Route.Sq, sq, cir);
  if r1 == r2 && r1 == 9.0 { return 0; }
  return 1;
}
