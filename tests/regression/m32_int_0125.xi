// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32: Int16 cast from Int8 positive (preserves value)
fn main() -> Int {
  var a: Int8 = 100;
  var b: Int16 = a as Int16;
  if b == 100 as Int16 { return 0; }
  return 1;
}
