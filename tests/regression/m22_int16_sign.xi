// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M22: Int16 sign extension -- negative values must work
fn main() -> Int {
  var neg: Int16 = -1;
  var pos: Int16 = 1;
  var sum: Int16 = neg + pos;
  if sum == 0 as Int16 {
    return 0;
  }
  return 1;
}
