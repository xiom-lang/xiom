// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 cast from Int8 (sign extension negative)
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: Int64 = a as Int64;
  if b == -1 as Int64 { return 0; }
  return 1;
}
