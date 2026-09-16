// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_contract_008
fn clamp(val: Int, min: Int, max: Int) -> Int
    requires: min <= max
    ensures: result >= min
    ensures: result <= max
  {
    if val < min { return min; }
    if val > max { return max; }
    return val;
  }

  pub fn run() -> Int {
    var c = clamp(15, 0, 10);
    if c == 10 { return 0; }
    return 1;
  }
use m21_contract_008.run;
fn main() -> Int { return run(); }
