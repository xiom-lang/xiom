// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float64 to Float32 cast (narrowing)
fn main() -> Int {
  var f64: Float64 = 2.5;
  var f32: Float32 = f64 as Float32;
  if f32 > 2.49 && f32 < 2.51 {
    return 0;
  }
  return 1;
}
