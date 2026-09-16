// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module b {
  pub type Color2 = {
    r: Int;
    g: Int;
    b: Int;
  }

  fn test_b() -> Int {
    var c = Color2{ r: 255, g: 128, b: 64 };
    if c.r == 255 { return 1; }
    return 0;
  }

  pub fn run_b() -> Int { return test_b(); }
}

use b.run_b;

fn main() -> Int { return run_b(); }
