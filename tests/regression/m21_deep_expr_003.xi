// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_003
pub fn run() -> Int {
    var a = 1;
    var b = 2;
    var c = 3;
    var d = 4;
    var e = 5;
    var result = a + b * c - d / e + a * b + c - d * e + a;
    if result == -7 { return 0; }
    return 1;
  }
use m21_deep_expr_003.run;
fn main() -> Int { return run(); }
