// XIOM -- phase1_modules
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

module math {
  pub fn add(a: Int, b: Int) -> Int { return a + b; }
  pub fn mul(a: Int, b: Int) -> Int { return a * b; }
}
use math.add;
fn main() -> Int { return add(10, 20); }
