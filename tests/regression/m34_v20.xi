// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V20: Float32 full coverage -- all ops, comparisons, casts, array
fn main() -> Int {
  var a: Float32 = 7.0;
  var b: Float32 = 3.0;
  var sum: Float32 = a + b;
  var diff: Float32 = a - b;
  var prod: Float32 = a * b;
  var quot: Float32 = a / b;
  var neg: Float32 = -a;
  var f64: Float64 = a as Float64;
  var f32_arr = [1.5, 2.5, 3.5];
  var arr_sum: Float32 = f32_arr[0] + f32_arr[1] + f32_arr[2];
  if sum == 10.0 && diff == 4.0 && prod == 21.0 && neg == -7.0 && f64 == 7.0 && arr_sum == 7.5 {
    return 0;
  }
  return 1;
}
