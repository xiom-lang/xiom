// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float64 negative values
fn main() -> Int {
  var a: Float64 = -10.0;
  var b: Float64 = -5.0;
  var sum: Float64 = a + b;
  var diff: Float64 = a - b;
  var prod: Float64 = a * b;
  var quot: Float64 = b / a;
  if sum == -15.0 && diff == -5.0 && prod == 50.0 && quot == 0.5 {
    return 0;
  }
  return 1;
}
