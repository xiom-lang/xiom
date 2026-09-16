// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int64 mod with negative (safe range)
fn main() -> Int {
  var a: Int64 = -100 as Int64;
  var b: Int64 = 11;
  var c: Int64 = a % b;
  if c == -1 as Int64 { return 0; }
  return 1;
}
