// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_op_prec_006
pub fn run() -> Int {
    var x = 5;
    var y = 10;
    var z = 15;
    if x < y && y <= z && z > x { return 0; }
    return 1;
  }
use m21_op_prec_006.run;
fn main() -> Int { return run(); }
