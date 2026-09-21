// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Integer in struct field
type Point = { x: Int; y: Int; }
fn main() -> Int {
  var p = Point{ x: 10; y: 20; };
  var sum: Int = p.x + p.y;
  if sum == 30 {
    return 0;
  }
  return 1;
}
