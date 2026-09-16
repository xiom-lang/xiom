// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-H08: Float32->Float64 widening -- single to double precision
fn main() -> Int {
  var f32: Float32 = 2.71828;
  var f64: Float64 = f32 as Float64;
  if f64 > 2.71827 && f64 < 2.71829 { return 0; }
  return 1;
}
