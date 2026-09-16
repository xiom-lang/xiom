// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-B18: Struct borrow -- &mut on struct, replace whole struct through reference
type Vec2 = { x: Int; y: Int; }
fn set_x(v: &mut Vec2, val: Int) { *v = Vec2{ x: val; y: v.y; }; }
fn set_y(v: &mut Vec2, val: Int) { *v = Vec2{ x: v.x; y: val; }; }
fn main() -> Int {
  var p = Vec2{ x: 0; y: 0; };
  set_x(&mut p, 3);
  set_y(&mut p, 7);
  if p.x == 3 && p.y == 7 { return 0; }
  return 1;
}
