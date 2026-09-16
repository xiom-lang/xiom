// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float32 to Float64 cast (widening)
fn main() -> Int {
  var f32: Float32 = 3.14;
  var f64: Float64 = f32 as Float64;
  if f64 > 3.13 && f64 < 3.15 {
    return 0;
  }
  return 1;
}
