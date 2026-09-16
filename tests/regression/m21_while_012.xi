// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_while_012
pub fn run() -> Int {
    var opt: Option[Int] = Some(42);
    var found = false;
    var count = 0;
    while count < 3 {
      match opt {
        Some(v) => if v == 42 { found = true; },
        None => { },
      }
      count = count + 1;
    }
    if found { return 0; }
    return 1;
  }
use m21_while_012.run;
fn main() -> Int { return run(); }
