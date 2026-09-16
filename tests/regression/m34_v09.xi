// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V09: Float32 to Int truncation, Int to Float64
fn main() -> Int {
  var f32: Float32 = 42.7;
  var i32: Int = f32 as Int;
  var f64: Float64 = 42.7;
  var i64: Int = f64 as Int;
  var neg_f: Float64 = -3.9;
  var neg_i: Int = neg_f as Int;
  if i32 == 42 && i64 == 42 && neg_i == -3 {
    return 0;
  }
  return 1;
}
