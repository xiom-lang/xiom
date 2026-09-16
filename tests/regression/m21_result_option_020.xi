// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_020
pub fn run() -> Int {
    var r: Result[Int, Int] = Ok(10);
    match r {
      Ok(v) => if v > 0 { return 0; },
      Err(e) => if e < 0 { return 1; },
    }
    return 1;
  }
use m21_result_option_020.run;
fn main() -> Int { return run(); }
