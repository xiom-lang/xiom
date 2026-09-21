// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 comparisons at boundaries
fn main() -> Int {
  var min: Int64 = -9223372036854775808 as Int64;
  var max: Int64 = 9223372036854775807;
  if min < max && max > 0 && min < 0 { return 0; }
  return 1;
}
