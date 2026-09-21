// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_005
pub fn run() -> Int {
    var a = true;
    var b = false;
    var c = true;
    var d = false;
    var e = true;
    var result = a && b || c && d || e && a || b && c;
    if result { return 0; }
    return 1;
  }
use m21_deep_expr_005.run;
fn main() -> Int { return run(); }
