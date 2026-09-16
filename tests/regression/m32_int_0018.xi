// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 overflow -- add near max wraps
fn main() -> Int {
  var a: Int16 = 32767;
  var b: Int16 = 1;
  var sum: Int16 = a + b;
  if sum == -32768 as Int16 {
    return 0;
  }
  return 1;
}
