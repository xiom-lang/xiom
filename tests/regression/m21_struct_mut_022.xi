// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_022
type Vector2 = { x: Float64; y: Float64; }
fn main() -> Int {
  var v: Vector2 = Vector2{ x: 1.0; y: 2.0; };
  v.x = 3.5;
  v.y = 4.5;
  if v.x == 3.5 && v.y == 4.5 { return 0; }
  return 1;
  return 1;
}
