// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M30: Struct with derive[Eq] -- equality comparison
type Vec2 = { x: Int; y: Int; } derive[Eq]
fn main() -> Int {
  var a = Vec2{ x: 1; y: 2; };
  var b = Vec2{ x: 1; y: 2; };
  var c = Vec2{ x: 3; y: 4; };
  if a == b && a != c { return 0; }
  return 1;
}
