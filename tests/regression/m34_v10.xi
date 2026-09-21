// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V10: Float comparisons -- >= operator stress
fn main() -> Int {
  var a: Float64 = 5.0;
  var b: Float64 = 5.0;
  var c: Float64 = 3.0;
  var d: Float64 = 7.0;
  if a >= b && a >= c && d >= a && a <= b && c <= a && a <= d {
    return 0;
  }
  return 1;
}
