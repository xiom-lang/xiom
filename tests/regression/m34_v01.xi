// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V01: Very large Float64 values (1e100, 1e200)
fn main() -> Int {
  var a: Float64 = 1.0e100;
  var b: Float64 = 1.0e200;
  if a > 0.0 && b > 0.0 && b > a {
    return 0;
  }
  return 1;
}
