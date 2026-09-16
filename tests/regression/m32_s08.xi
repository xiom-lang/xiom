// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S08: Struct as function parameter
type Point = { x: Float64; y: Float64; }
fn distance_sq(p: Point) -> Float64 { return p.x * p.x + p.y * p.y; }
fn add_points(a: Point, b: Point) -> Point { return Point{ x: a.x + b.x; y: a.y + b.y; }; }
fn main() -> Int {
  var a = Point{ x: 3.0; y: 4.0; };
  var b = Point{ x: 1.0; y: 2.0; };
  var d: Float64 = distance_sq(a);
  var c = add_points(a, b);
  if d == 25.0 && c.x == 4.0 && c.y == 6.0 { return 0; }
  return 1;
}
