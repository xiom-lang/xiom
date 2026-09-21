// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 sign extension positive to Int64 (32767 stays 32767)
fn main() -> Int {
  var a: Int16 = 32767;
  var b: Int64 = a as Int64;
  if b == 32767 as Int64 { return 0; }
  return 1;
}
