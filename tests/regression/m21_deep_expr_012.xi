// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_012
pub fn run() -> Int {
    var x = 4;
    var y = 3;
    var z = 2;
    var w = 5;
    var r = x * y + z * w - x / z + y * w - z + x % y;
    if r == 34 { return 0; }
    return 1;
  }
use m21_deep_expr_012.run;
fn main() -> Int { return run(); }
