// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 subtraction
fn main() -> Int {
  var a: Int32 = 500000;
  var b: Int32 = 1500000;
  var sub: Int32 = a - b;
  if sub == -1000000 as Int32 {
    return 0;
  }
  return 1;
}
