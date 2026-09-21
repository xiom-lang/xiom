// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 minimum value -9223372036854775808
fn main() -> Int {
  var x: Int64 = -9223372036854775808;
  var y: Int64 = -9223372036854775807;
  if x < y {
    return 0;
  }
  return 1;
}
