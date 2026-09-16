// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M34-V03: Zero and negative zero (0.0 vs -0.0)
fn main() -> Int {
  var pos_zero: Float64 = 0.0;
  var neg_zero: Float64 = -0.0;
  if pos_zero == 0.0 && neg_zero == 0.0 && pos_zero == neg_zero {
    return 0;
  }
  return 1;
}
