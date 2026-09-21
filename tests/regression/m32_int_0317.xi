// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int8 sign extension from positive to Int (sext: 127 stays 127)
fn main() -> Int {
  var a: Int8 = 127;
  var b: Int = a as Int;
  if b == 127 { return 0; }
  return 1;
}
