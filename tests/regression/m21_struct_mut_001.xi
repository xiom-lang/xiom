// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_001
type Point = { x: Int; y: Int; }
fn main() -> Int {
  var p: Point = Point{ x: 10; y: 20; };
  p.x = 42;
  if p.x == 42 && p.y == 20 { return 0; }
  return 1;
}
