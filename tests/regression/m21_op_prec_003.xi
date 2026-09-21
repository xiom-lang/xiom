// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_op_prec_003
pub fn run() -> Int {
    var r = 12 / 3 / 2;
    if r == 2 { return 0; }
    return 1;
  }
use m21_op_prec_003.run;
fn main() -> Int { return run(); }
