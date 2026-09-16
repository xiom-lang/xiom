// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Truncation Int64 to Int16 (low 16 bits)
fn main() -> Int {
  var a: Int64 = 131071;
  var b: Int16 = a as Int16;
  if b == -1 as Int16 { return 0; }
  return 1;
}
