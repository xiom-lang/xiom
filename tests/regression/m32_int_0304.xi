// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Vec UInt32 elements with large values
fn main() -> Int {
  var v: Vec[UInt32] = Vec[UInt32].new();
  v.push(3000000000);
  if v[0] == 3000000000 as UInt32 { return 0; }
  return 1;
}
