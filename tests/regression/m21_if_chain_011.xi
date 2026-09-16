// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_011
pub fn run() -> Int {
    var x = 10;
    var y = 20;
    if x > 0 && y > 10 { return 0; }
    elif x > 0 { return 1; }
    else { return 1; }
  }
use m21_if_chain_011.run;
fn main() -> Int { return run(); }
