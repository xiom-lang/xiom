// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_020
pub fn run() -> Int {
    var a = 1;
    var b = 2;
    var c = 3;
    if a < b && b < c { return 0; }
    elif a == b { return 1; }
    elif b == c { return 1; }
    else { return 1; }
  }
use m21_if_chain_020.run;
fn main() -> Int { return run(); }
