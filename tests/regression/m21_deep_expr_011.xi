// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_deep_expr_011
pub fn run() -> Int {
    var s = "a";
    s = s.concat("b");
    s = s.concat("c");
    s = s.concat("d");
    s = s.concat("e");
    s = s.concat("f");
    s = s.concat("g");
    s = s.concat("h");
    if s == "abcdefgh" { return 0; }
    return 1;
  }
use m21_deep_expr_011.run;
fn main() -> Int { return run(); }
