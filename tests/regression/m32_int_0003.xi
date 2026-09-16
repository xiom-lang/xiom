// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 addition at boundaries
fn main() -> Int {
  var a: Int8 = 100;
  var b: Int8 = 20;
  var sum: Int8 = a + b;
  if sum == 120 as Int8 {
    return 0;
  }
  return 1;
}
