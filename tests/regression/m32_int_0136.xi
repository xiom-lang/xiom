// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 comparisons at boundaries
fn main() -> Int {
  var min: Int32 = -2147483648 as Int32;
  var max: Int32 = 2147483647;
  if min < max { return 0; }
  return 1;
}
