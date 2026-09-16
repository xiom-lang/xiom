// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Mixed Float32/Float64 operations with casts
fn main() -> Int {
  var f32: Float32 = 3.0;
  var f64: Float64 = f32 as Float64;
  var sum: Float64 = f64 + 4.0;
  var back: Float32 = sum as Float32;
  if f64 == 3.0 && sum == 7.0 && back == 7.0 {
    return 0;
  }
  return 1;
}
