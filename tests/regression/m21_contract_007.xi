// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_contract_007
type PositiveInt = {
    value: Int;
    invariant: value > 0;
  }

  fn make_positive(v: Int) -> PositiveInt
    requires: v > 0
  {
    return { value: v; };
  }

  pub fn run() -> Int {
    var p = make_positive(5);
    if p.value == 5 { return 0; }
    return 1;
  }
use m21_contract_007.run;
fn main() -> Int { return run(); }
