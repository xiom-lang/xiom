// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: UInt arithmetic -- no negative values
fn main() -> Int {
  var a: UInt = 100;
  var b: UInt = 200;
  var diff: UInt = b - a;
  if diff == 100 {
    return 0;
  }
  return 1;
}
