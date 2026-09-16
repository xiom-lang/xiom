// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 narrowing to Int8 (truncation)
fn main() -> Int {
  var a: Int64 = 127;
  var b: Int8 = a as Int8;
  if b == 127 as Int8 { return 0; }
  return 1;
}
