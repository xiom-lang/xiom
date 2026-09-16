// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_op_prec_004
pub fn run() -> Int {
    var a = true;
    var b = false;
    var c = true;
    if a || b && c { return 0; }
    return 1;
  }
use m21_op_prec_004.run;
fn main() -> Int { return run(); }
