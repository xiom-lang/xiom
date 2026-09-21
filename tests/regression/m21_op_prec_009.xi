// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_op_prec_009
pub fn run() -> Int {
    var r = 8 / 2 * 4;
    if r == 16 { return 0; }
    return 1;
  }
use m21_op_prec_009.run;
fn main() -> Int { return run(); }
