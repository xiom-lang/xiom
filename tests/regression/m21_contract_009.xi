// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_contract_009
type NonEmptyVec = {
    data: Vec[Int];
    invariant: data.len() > 0;
  }

  pub fn run() -> Int {
    var v: NonEmptyVec = { data: [42]; };
    if v.data[0] == 42 { return 0; }
    return 1;
  }
use m21_contract_009.run;
fn main() -> Int { return run(); }
