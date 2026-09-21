// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Vec Int8 elements
fn main() -> Int {
  var v: Vec[Int8] = Vec[Int8].new();
  v.push(127);
  v.push(-128 as Int8);
  if v.len() == 2 { return 0; }
  return 1;
}
