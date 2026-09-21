// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float comparison edge cases -- all six operators on Float64 and Float32
fn main() -> Int {
  var f64a: Float64 = 100.0;
  var f64b: Float64 = 50.0;
  var f64c: Float64 = 100.0;
  var f32a: Float32 = 7.5;
  var f32b: Float32 = 3.25;
  var f32c: Float32 = 7.5;
  if f64a > f64b && f64b < f64a && f64a == f64c && f64a != f64b && f64b <= f64a && f64a >= f64c &&
     f32a > f32b && f32b < f32a && f32a == f32c && f32a != f32b && f32b <= f32a && f32a >= f32c {
    return 0;
  }
  return 1;
}
