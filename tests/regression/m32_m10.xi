// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-M10: Module exports -- multiple pub items exported
module exports {
  pub fn add(a: Int, b: Int) -> Int { return a + b; }
  pub fn mul(a: Int, b: Int) -> Int { return a * b; }
  pub const PI: Int = 3;
  pub type Pair = { first: Int; second: Int; }
}
use exports.add;
use exports.mul;
use exports.PI;
fn main() -> Int {
  if add(1, 2) == 3 && mul(3, 4) == 12 && PI == 3 { return 0; }
  return 1;
}
