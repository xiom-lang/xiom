// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float32 arithmetic -- add, sub, mul, div
fn main() -> Int {
  var a: Float32 = 8.0;
  var b: Float32 = 3.0;
  var sum: Float32 = a + b;
  var diff: Float32 = a - b;
  var prod: Float32 = a * b;
  var quot: Float32 = a / 2.0;
  if sum == 11.0 && diff == 5.0 && prod == 24.0 && quot == 4.0 {
    return 0;
  }
  return 1;
}
