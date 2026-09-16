// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Integer comparison chain -- all operators
fn main() -> Int {
  var a: Int = 10;
  var b: Int = 5;
  var c: Int = 10;
  if a > b && a >= c && b < a && b <= c && a == c && a != b {
    return 0;
  }
  return 1;
}
