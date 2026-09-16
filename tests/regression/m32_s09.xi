// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S09: Struct spread -- copy struct with field override via helper
type Point = { x: Float64; y: Float64; }
fn copy_point(p: Point) -> Point { return Point{ x: p.x; y: p.y; }; }
fn with_x(p: Point, nx: Float64) -> Point { return Point{ x: nx; y: p.y; }; }
fn with_y(p: Point, ny: Float64) -> Point { return Point{ x: p.x; y: ny; }; }
fn main() -> Int {
  var p = Point{ x: 1.0; y: 2.0; };
  var q = copy_point(p);
  var r = with_x(p, 99.0);
  var s = with_y(p, 88.0);
  var t = with_x(with_y(p, 55.0), 44.0);
  if q.x == 1.0 && q.y == 2.0 && r.x == 99.0 && r.y == 2.0 && s.x == 1.0 && s.y == 88.0 && t.x == 44.0 && t.y == 55.0 { return 0; }
  return 1;
}
