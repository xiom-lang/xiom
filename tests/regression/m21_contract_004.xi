// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_contract_004
fn safe_sub(a: Int, b: Int) -> Int
    requires: a >= b
    ensures: result >= 0
  {
    return a - b;
  }

  pub fn run() -> Int {
    var r = safe_sub(50, 20);
    if r == 30 { return 0; }
    return 1;
  }
use m21_contract_004.run;
fn main() -> Int { return run(); }
