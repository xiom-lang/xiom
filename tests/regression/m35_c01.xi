// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-C01: if without else -- single branch, no alternative path
fn main() -> Int {
  var x: Int = 0;
  if x == 0 { x = 42; }
  if x != 42 { return 1; }
  return 0;
}
