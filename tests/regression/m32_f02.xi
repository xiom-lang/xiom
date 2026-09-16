// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Float64 comparisons -- <, >, ==, !=, <=, >=
fn main() -> Int {
  var a: Float64 = 3.14;
  var b: Float64 = 2.718;
  var c: Float64 = 3.14;
  if a > b && b < a && a == c && a != b && b <= a && a >= c {
    return 0;
  }
  return 1;
}
