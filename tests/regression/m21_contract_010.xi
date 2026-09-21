// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_contract_010
fn is_even(n: Int) -> Bool
    ensures: result == (n % 2 == 0)
  {
    return n % 2 == 0;
  }

  pub fn run() -> Int {
    if is_even(4) && !is_even(7) { return 0; }
    return 1;
  }
use m21_contract_010.run;
fn main() -> Int { return run(); }
