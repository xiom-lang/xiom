// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_op_prec_008
pub fn run() -> Int {
    var r = 3 + 4 * 2 - 6 / 3 + 1;
    if r == 10 { return 0; }
    return 1;
  }
use m21_op_prec_008.run;
fn main() -> Int { return run(); }
