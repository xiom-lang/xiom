// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_012
pub fn run() -> Int {
    var x = 0;
    var y = 100;
    if x > 0 || y > 50 { return 0; }
    elif y == 0 { return 1; }
    else { return 1; }
  }
use m21_if_chain_012.run;
fn main() -> Int { return run(); }
