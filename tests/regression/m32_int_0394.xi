// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Mixed float-int narrowing: Float64 to Int32
fn main() -> Int {
  var f: Float64 = 32767.5;
  var i: Int32 = f as Int32;
  if i == 32767 as Int32 { return 0; }
  return 1;
}
