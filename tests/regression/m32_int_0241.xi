// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Truncation Int32 to Int8 (low byte)
fn main() -> Int {
  var a: Int32 = 511;
  var b: Int8 = a as Int8;
  if b == -1 as Int8 { return 0; }
  return 1;
}
