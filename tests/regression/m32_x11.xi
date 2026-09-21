// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-X11: Combinatorial + Differential -- nested compute vs flat compute with struct+match
type Point = { x: Int; y: Int; }
enum DistMethod { Nested, Flat }
fn dist_nested(p1: Point, p2: Point) -> Int {
  var dx = p1.x - p2.x;
  var dy = p1.y - p2.y;
  return dx * dx + dy * dy;
}
fn dist_flat(p1: Point, p2: Point) -> Int {
  return (p1.x - p2.x) * (p1.x - p2.x) + (p1.y - p2.y) * (p1.y - p2.y);
}
fn compute(m: DistMethod, p1: Point, p2: Point) -> Int {
  match m { Nested => dist_nested(p1, p2), Flat => dist_flat(p1, p2), }
}
fn main() -> Int {
  var p1 = Point{ x: 0; y: 0; };
  var p2 = Point{ x: 3; y: 4; };
  var p3 = Point{ x: 5; y: 6; };
  var p4 = Point{ x: 8; y: 10; };
  var r1 = compute(DistMethod.Nested, p1, p2);
  var r2 = compute(DistMethod.Flat, p1, p2);
  var r3 = compute(DistMethod.Nested, p3, p4);
  var r4 = compute(DistMethod.Flat, p3, p4);
  if r1 == r2 && r3 == r4 && r1 == 25 && r3 == 25 { return 0; }
  return 1;
}
