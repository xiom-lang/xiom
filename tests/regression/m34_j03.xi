// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-J03: Module with pub type -- public struct types in modules
module geometry {
  pub type Point = { x: Int; y: Int; }
  pub type Size = { w: Int; h: Int; }
  pub fn origin() -> Point { return Point{ x: 0; y: 0; }; }
  pub fn area(s: Size) -> Int { return s.w * s.h; }
}
use geometry.origin;
use geometry.Size;
use geometry.area;
fn main() -> Int {
  var p = origin();
  var s = Size{ w: 10, h: 5 };
  var a = area(s);
  if p.x == 0 && p.y == 0 && a == 50 { return 0; }
  return 1;
}
