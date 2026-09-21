// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M33-A16: Array of struct -- array of Point structs
type Point = { x: Float64; y: Float64; }
fn main() -> Int {
  var pts = [Point{ x: 1.0; y: 2.0; }, Point{ x: 3.0; y: 4.0; }, Point{ x: 5.0; y: 6.0; }];
  var sx: Float64 = 0.0;
  var sy: Float64 = 0.0;
  var i: Int = 0;
  while i < 3 {
    sx += pts[i].x;
    sy += pts[i].y;
    i += 1;
  }
  if sx == 9.0 && sy == 12.0 { return 0; }
  return 1;
}
