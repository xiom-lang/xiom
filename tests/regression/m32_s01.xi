// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-S01: Basic struct with Float64 fields -- create/access/mutate
type Point = { x: Float64; y: Float64; }
fn main() -> Int {
  var p = Point{ x: 1.0; y: 2.0; };
  var sum: Float64 = p.x + p.y;
  p.x = 5.0;
  if sum == 3.0 && p.x == 5.0 && p.y == 2.0 { return 0; }
  return 1;
}
