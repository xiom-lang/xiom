// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Struct with Int8 field
type Point8 = { x: Int8; y: Int8; }
fn main() -> Int {
  var p: Point8 = Point8{ x: (-128) as Int8; y: 127 as Int8; };
  var sum: Int8 = p.x + p.y;
  if sum == (-1) as Int8 { return 0; }
  return 1;
}
