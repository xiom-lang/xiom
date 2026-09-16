// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_module_014
fn helper(x: Int) -> Int { return x * 2; }

  pub fn compute(x: Int) -> Int {
    return helper(x) + 1;
  }

  pub fn run() -> Int {
    var v = compute(5);
    if v == 11 { return 0; }
    return 1;
  }
use m21_module_014.run;
fn main() -> Int { return run(); }
