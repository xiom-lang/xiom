// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Sign extend Int16 to Int64 (negative value)
fn main() -> Int {
  var a: Int16 = -32768 as Int16;
  var b: Int64 = a as Int64;
  if b == -32768 as Int64 { return 0; }
  return 1;
}
