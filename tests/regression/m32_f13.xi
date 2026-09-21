// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Very large Float64 values (1e308)
fn main() -> Int {
  var big: Float64 = 1.0e308;
  var small: Float64 = 1.0;
  if big > small {
    return 0;
  }
  return 1;
}
