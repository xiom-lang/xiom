// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 arithmetic shift right (min >> 63 = -1)
fn main() -> Int {
  var a: Int64 = -9223372036854775808 as Int64;
  var b: Int64 = 63;
  var c: Int64 = a >> b;
  if c == -1 as Int64 { return 0; }
  return 1;
}
