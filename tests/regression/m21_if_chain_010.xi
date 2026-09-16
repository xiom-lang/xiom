// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_010
pub fn run() -> Int {
    var s = "beta";
    if s == "alpha" { return 1; }
    elif s == "beta" { return 0; }
    elif s == "gamma" { return 1; }
    else { return 1; }
  }
use m21_if_chain_010.run;
fn main() -> Int { return run(); }
