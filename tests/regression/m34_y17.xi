// M34-Y17: Result chain + enum + compound assign + derive + impl + module
type Rect = { w: Int; h: Int; } derive[Eq]
enum ScaleKind { Double, Halve, Square }
fn scale_double(r: Rect) -> Int {
  var nw = r.w;
  var nh = r.h;
  nw = nw * 2; nh = nh * 2;
  return nw + nh;
}
fn scale_halve(r: Rect) -> Int {
  if r.w % 2 != 0 || r.h % 2 != 0 { return 0; }
  return r.w / 2 + r.h / 2;
}
fn scale_square(r: Rect) -> Int {
  return r.w * r.w + r.h * r.h;
}
fn scale[T](r: Rect, k: ScaleKind) -> Int
  requires: r.w > 0
  requires: r.h > 0
{
  match k {
    Double => scale_double(r),
    Halve => scale_halve(r),
    Square => scale_square(r),
  }
}
interface Scalable { fn scale(self, k: ScaleKind) -> Int; }
impl Scalable for Rect {
  fn scale(self, k: ScaleKind) -> Int { return scale(self, k); }
}
module geom {
  pub fn do_scale(r: Rect, k: ScaleKind) -> Int { return scale(r, k); }
  pub fn area(r: Rect) -> Int { return r.w * r.h; }
}
use geom.do_scale;
use geom.area;
fn main() -> Int {
  var r = Rect{ w: 4; h: 6; };
  var a = area(r);
  if a != 24 { return 1; }
  var r2 = Rect{ w: 4; h: 6; };
  var s = do_scale(r2, ScaleKind.Double);
  if s != 20 { return 2; }
  var r3 = Rect{ w: 4; h: 6; };
  var s2 = do_scale(r3, ScaleKind.Square);
  if s2 != 52 { return 3; }
  return 0;
}
