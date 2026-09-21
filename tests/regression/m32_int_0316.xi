// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 sign extension from negative to Int64 (sext verification)
fn main() -> Int {
  var a: Int32 = -1 as Int32;
  var b: Int64 = a as Int64;
  if b == -1 as Int64 { return 0; }
  return 1;
}
