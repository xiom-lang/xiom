// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_006
pub fn run() -> Int {
    var a = 10;
    var b = 5;
    var c = 2;
    var d = 3;
    var e = 7;
    var x = a > b && b > c && c < d && d < e && e > a;
    if !x { return 0; }
    return 1;
  }
use m21_deep_expr_006.run;
fn main() -> Int { return run(); }
