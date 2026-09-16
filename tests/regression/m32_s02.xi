// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S02: Nested structs -- Point inside Rect
type Point = { x: Float64; y: Float64; }
type Rect = { top_left: Point; bottom_right: Point; }
fn main() -> Int {
  var tl = Point{ x: 0.0; y: 10.0; };
  var br = Point{ x: 10.0; y: 0.0; };
  var r = Rect{ top_left: tl; bottom_right: br; };
  if r.top_left.x == 0.0 && r.top_left.y == 10.0 && r.bottom_right.x == 10.0 && r.bottom_right.y == 0.0 { return 0; }
  return 1;
}
