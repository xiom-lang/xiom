// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_destructure_004
pub fn run() -> Int {
    var t = (1, true, "three");
    var a = t.0;
    var b = t.1;
    var c = t.2;
    if a == 1 && b && c == "three" { return 0; }
    return 1;
  }
use m21_destructure_004.run;
fn main() -> Int { return run(); }
