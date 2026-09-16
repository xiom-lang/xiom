// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M04: use module -- import multiple functions via use
module ops {
  pub fn add(a: Int, b: Int) -> Int { return a + b; }
  pub fn sub(a: Int, b: Int) -> Int { return a - b; }
}
use ops.add;
use ops.sub;
fn main() -> Int {
  if add(10, 5) == 15 && sub(10, 5) == 5 { return 0; }
  return 1;
}
