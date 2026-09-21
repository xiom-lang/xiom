// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_003
pub fn run() -> Int {
    var x = 9;
    if x == 0 { return 1; }
    elif x == 1 { return 1; }
    elif x == 2 { return 1; }
    elif x == 3 { return 1; }
    elif x == 4 { return 1; }
    elif x == 5 { return 1; }
    elif x == 6 { return 1; }
    elif x == 7 { return 1; }
    elif x == 8 { return 1; }
    elif x == 9 { return 0; }
    else { return 1; }
  }
use m21_if_chain_003.run;
fn main() -> Int { return run(); }
