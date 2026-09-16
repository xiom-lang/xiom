// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-Y11: derive struct + generic enum payload + method contract + match + impl + module + diff
type Rect = { w: Int; h: Int; } derive[Eq]
enum Shape { RectShape(r: Rect), Circle(radius: Int) }
fn perimeter[T](s: Shape) -> Int
  requires: true
  ensures: result >= 0
{
  match s {
    RectShape(r) => 2 * (r.w + r.h),
    Circle(radius) => 2 * radius * 3,
  }
}
fn perim_rect_direct(r: Rect) -> Int { return 2 * (r.w + r.h); }
interface Geometry { fn perim(self) -> Int; }
impl Geometry for Rect {
  fn perim(self) -> Int { return 2 * (self.w + self.h); }
}
module types {
  pub fn do_perimeter(s: Shape) -> Int { return perimeter(s); }
  pub fn do_direct(r: Rect) -> Int { return perim_rect_direct(r); }
  pub fn via_geom(r: Rect) -> Int { return r.perim(); }
}
use types.do_perimeter;
use types.do_direct;
use types.via_geom;
enum Path { MatchP, Direct, Impl }
fn compute(p: Path, s: Shape, r: Rect) -> Int {
  match p { MatchP => do_perimeter(s), Direct => do_direct(r), Impl => via_geom(r), }
}
fn main() -> Int {
  var rect = Rect{ w: 4; h: 5; };
  var shape = Shape.RectShape(rect);
  var r1 = compute(Path.MatchP, shape, rect);
  var r2 = compute(Path.Direct, shape, rect);
  var r3 = compute(Path.Impl, shape, rect);
  if r1 == r2 && r2 == r3 && r1 == 18 { return 0; }
  return 1;
}
