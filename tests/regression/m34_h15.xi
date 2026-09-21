// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H15: Cast in struct field -- narrow field assignment via cast
type Point = { x: Int32; y: Int32; }
fn main() -> Int {
  var bigX: Int64 = 100;
  var bigY: Int64 = 200;
  var p = Point{ x: bigX as Int32; y: bigY as Int32; };
  if p.x == 100 as Int32 && p.y == 200 as Int32 { return 0; }
  return 1;
}
