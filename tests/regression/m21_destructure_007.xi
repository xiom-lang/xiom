// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_destructure_007
fn swap(a: Int, b: Int) -> (Int, Int) {
    return (b, a);
  }

  pub fn run() -> Int {
    var s = swap(10, 20);
    if s.0 == 20 && s.1 == 10 { return 0; }
    return 1;
  }
use m21_destructure_007.run;
fn main() -> Int { return run(); }
