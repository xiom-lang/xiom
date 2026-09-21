// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 bitwise with negative
fn main() -> Int {
  var a: Int64 = -1 as Int64;
  var b: Int64 = -1 as Int64;
  var c: Int64 = a & b;
  if c == -1 as Int64 { return 0; }
  return 1;
}
