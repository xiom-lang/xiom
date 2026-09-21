// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_op_prec_007
pub fn run() -> Int {
    var a = 10;
    var b = 20;
    var c = 30;
    if a + b == 30 && b + c == 50 { return 0; }
    return 1;
  }
use m21_op_prec_007.run;
fn main() -> Int { return run(); }
