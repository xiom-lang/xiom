// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Vec UInt16 elements with high values
fn main() -> Int {
  var v: Vec[UInt16] = Vec[UInt16].new();
  v.push(65535);
  if v[0] == 65535 as UInt16 { return 0; }
  return 1;
}
