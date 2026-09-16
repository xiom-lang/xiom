// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Compound arithmetic expression
fn main() -> Int {
  var result: Int = (10 + 20) * (30 - 15) / 5;
  if result == 90 {
    return 0;
  }
  return 1;
}
