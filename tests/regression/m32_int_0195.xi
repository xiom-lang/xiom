// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Mixed sign compare Int16 vs UInt16 (safe range)
fn main() -> Int {
  var a: Int16 = -1 as Int16;
  var b: UInt16 = 1 as UInt16;
  if a < b as Int16 { return 0; }
  return 1;
}
