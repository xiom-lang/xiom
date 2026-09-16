// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T09: String comparison -- == and != operators
fn main() -> Int {
  var a: Str = "apple";
  var b: Str = "apple";
  var c: Str = "orange";
  if a == b && a != c && b != c { return 0; }
  return 1;
}
