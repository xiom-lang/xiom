// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_001
pub fn add(a: Int, b: Int) -> Int { return a + b; }

  pub fn run() -> Int {
    var result = add(3, 4);
    if result == 7 { return 0; }
    return 1;
  }
use m21_module_001.run;
fn main() -> Int { return run(); }
