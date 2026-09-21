// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 maximum value 9223372036854775807
fn main() -> Int {
  var x: Int64 = 9223372036854775807;
  var y: Int64 = 9223372036854775806;
  if x > y {
    return 0;
  }
  return 1;
}
