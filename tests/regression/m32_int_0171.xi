// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt16 comparisons
fn main() -> Int {
  var min: UInt16 = 0;
  var max: UInt16 = 65535;
  if min <= max && max > min { return 0; }
  return 1;
}
