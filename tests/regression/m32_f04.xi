// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float32 comparisons -- <, >, ==, !=, <=, >=
fn main() -> Int {
  var a: Float32 = 5.5;
  var b: Float32 = 1.5;
  var c: Float32 = 5.5;
  if a > b && b < a && a == c && a != b && b <= a && a >= c {
    return 0;
  }
  return 1;
}
