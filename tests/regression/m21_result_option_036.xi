// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_036
fn compute(flag: Bool) -> Result[Int, Int] {
    if flag { return Ok(100); }
    return Err(-1);
  }

  pub fn run() -> Int {
    var r = compute(true);
    match r {
      Ok(v) => if v == 100 { return 0; },
      Err(_) => return 1,
    }
  }
use m21_result_option_036.run;
fn main() -> Int { return run(); }
