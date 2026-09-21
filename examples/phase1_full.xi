// XIOM -- phase1_full
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

type Point = {
  x: Float64;
  y: Float64;
} derive[Eq, Clone, Display]

fn distance(a: &Point, b: &Point) -> Float64 {
  let dx = a.x - b.x;
  let dy = a.y - b.y;
  return dx * dx + dy * dy;
}

fn make_point(x: Float64, y: Float64) -> Point {
  return Point{ x: x, y: y };
}

fn main() -> Int {
  let p1 = make_point(0.0, 0.0);
  let p2 = Point{ x: 3.0, y: 4.0 };
  let d = distance(&p1, &p2);
  let r = 0;
  return r;
}
