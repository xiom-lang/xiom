// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float32 negative values
fn main() -> Int {
  var a: Float32 = -20.0;
  var b: Float32 = -4.0;
  var sum: Float32 = a + b;
  var diff: Float32 = a - b;
  var prod: Float32 = a * b;
  var quot: Float32 = a / b;
  if sum == -24.0 && diff == -16.0 && prod == 80.0 && quot == 5.0 {
    return 0;
  }
  return 1;
}
