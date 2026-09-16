// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float64 to Int conversion (truncation)
fn main() -> Int {
  var f: Float64 = 42.7;
  var i: Int = f as Int;
  if i == 42 {
    return 0;
  }
  return 1;
}
