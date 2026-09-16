// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Very small Float64 values (0.000001)
fn main() -> Int {
  var tiny: Float64 = 0.000001;
  var one: Float64 = 1.0;
  var product: Float64 = tiny * 1000000.0;
  if tiny > 0.0 && tiny < one && product == 1.0 {
    return 0;
  }
  return 1;
}
