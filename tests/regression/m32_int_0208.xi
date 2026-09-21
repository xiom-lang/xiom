// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Vec Int16 elements (narrow Vec requires Int element type)
fn main() -> Int {
  var v: Vec[Int] = Vec[Int].new();
  v.push(-32768);
  v.push(32767);
  if v[0] == -32768 && v[1] == 32767 { return 0; }
  return 1;
}
