// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 sign extension negative to Int (sext: -1 stays -1)
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: Int = a as Int;
  if b == -1 { return 0; }
  return 1;
}
