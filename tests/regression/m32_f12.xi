// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Very small Float32 values (0.000001)
fn main() -> Int {
  var tiny: Float32 = 0.000001;
  var zero: Float32 = 0.0;
  var one: Float32 = 1.0;
  if tiny > zero && tiny < one {
    return 0;
  }
  return 1;
}
