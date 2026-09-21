// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 increment loop wraparound
fn main() -> Int {
  var i: Int16 = 32765;
  while i < 32767 {
    i = i + 1 as Int16;
  }
  var j: Int16 = i + 1 as Int16;
  if j == -32768 as Int16 { return 0; }
  return 1;
}
