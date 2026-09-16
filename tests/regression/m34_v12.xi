// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V12: Multiply-accumulate (a * b + c chained)
fn mul_add(a: Float64, b: Float64, c: Float64) -> Float64 {
  return a * b + c;
}
fn main() -> Int {
  var a: Float64 = 2.0;
  var b: Float64 = 3.0;
  var c: Float64 = 4.0;
  var result: Float64 = mul_add(a, b, c);
  var chain: Float64 = a * b * 5.0 + c;
  if result == 10.0 && chain == 34.0 {
    return 0;
  }
  return 1;
}
