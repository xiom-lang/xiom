// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M11: Wildcard import -- use module.* to import all pub items
module math {
  pub fn add(a: Int, b: Int) -> Int { return a + b; }
  pub fn mul(a: Int, b: Int) -> Int { return a * b; }
}
use math.*;
fn main() -> Int {
  if add(3, 4) == 7 && mul(3, 4) == 12 { return 0; }
  return 1;
}
