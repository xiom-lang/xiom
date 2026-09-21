// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-Z25: multi_module+use+generic+struct+enum+match+while+contract+derive+Option+compound_assign
type Point = { x: Int; y: Int; } derive[Eq]
enum Quadrant { Q1, Q2, Q3, Q4, Origin, Axis }
fn classify(p: Point) -> Quadrant {
  if p.x == 0 && p.y == 0 { return Quadrant.Origin; }
  if p.x == 0 || p.y == 0 { return Quadrant.Axis; }
  if p.x > 0 && p.y > 0 { return Quadrant.Q1; }
  if p.x < 0 && p.y > 0 { return Quadrant.Q2; }
  if p.x < 0 && p.y < 0 { return Quadrant.Q3; }
  return Quadrant.Q4;
}
fn dist_from_origin(p: Point) -> Int
  requires: p.x >= 0
  requires: p.y >= 0
  ensures: result >= 0
{
  var dx = p.x; var dy = p.y;
  return dx * dx + dy * dy;
}
module geo {
  pub fn quadrant(p: Point) -> Quadrant { return classify(p); }
  pub fn dist_sq(p: Point) -> Int { return dist_from_origin(p); }
  pub fn origin() -> Point { return Point{ x: 0; y: 0; }; }
}
module transform {
  pub fn move_pt(p: Point, dx: Int, dy: Int) -> Point { var r = p; r.x += dx; r.y += dy; return r; }
}
use geo.quadrant;
use geo.dist_sq;
use geo.origin;
use transform.move_pt;
fn main() -> Int {
  var p1 = origin();
  var p2 = move_pt(p1, 3, 4);
  var q = quadrant(p2);
  var d = dist_sq(p2);
  var chk = 0;
  if q == Quadrant.Q1 { chk += 1; }
  if d == 25 { chk += 1; }
  var p3 = move_pt(p2, -5, 0);
  if quadrant(p3) == Quadrant.Q2 { chk += 1; }
  if chk == 3 { return 0; }
  return 1;
}
