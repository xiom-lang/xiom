// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_struct_mut_002
type Point = { x: Int; y: Int; }
fn main() -> Int {
  var p: Point = Point{ x: 10; y: 20; };
  var old_x = p.x;
  p.x = 42;
  if old_x == 10 && p.x == 42 { return 0; }
  return 1;
}
