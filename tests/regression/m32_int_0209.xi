// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Vec UInt8 elements
fn main() -> Int {
  var v: Vec[UInt8] = Vec[UInt8].new();
  v.push(0);
  v.push(255);
  if v[1] == 255 as UInt8 { return 0; }
  return 1;
}
