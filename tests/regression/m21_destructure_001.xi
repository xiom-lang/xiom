// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_destructure_001
pub fn run() -> Int {
    var x = 10;
    var y = 20;
    var t = (x, y);
    if t.0 == 10 && t.1 == 20 { return 0; }
    return 1;
  }
use m21_destructure_001.run;
fn main() -> Int { return run(); }
