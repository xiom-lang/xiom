// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int32 sign extension from Int8 negative
fn main() -> Int {
  var a: Int8 = -1 as Int8;
  var b: Int32 = a as Int32;
  if b == -1 as Int32 { return 0; }
  return 1;
}
