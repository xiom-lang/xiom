// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-U06: Pointer to struct int fields -- address-of struct member
type Vec2 = { x: Int; y: Int; }
fn main() -> Int {
  var v = Vec2{ x: 3; y: 4; };
  if v.x == 3 && v.y == 4 { return 0; }
  return 1;
}
