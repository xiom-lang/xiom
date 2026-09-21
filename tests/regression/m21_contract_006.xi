// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_contract_006
fn max_val(a: Int, b: Int) -> Int
    ensures: result >= a
    ensures: result >= b
  {
    if a > b { return a; }
    return b;
  }

  pub fn run() -> Int {
    var m = max_val(7, 12);
    if m == 12 { return 0; }
    return 1;
  }
use m21_contract_006.run;
fn main() -> Int { return run(); }
