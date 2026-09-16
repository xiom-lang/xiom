// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_if_chain_006
pub fn run() -> Int {
    var x: Int16 = 1000i16;
    if x == 500i16 { return 1; }
    elif x == 1000i16 { return 0; }
    else { return 1; }
  }
use m21_if_chain_006.run;
fn main() -> Int { return run(); }
