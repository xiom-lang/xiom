// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_002
pub fn run() -> Int {
    var x = 2;
    var y = x * x * x * x * x * x * x * x;
    if y == 256 { return 0; }
    return 1;
  }
use m21_deep_expr_002.run;
fn main() -> Int { return run(); }
