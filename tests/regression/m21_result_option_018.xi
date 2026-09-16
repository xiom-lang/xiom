// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_018
pub fn run() -> Int {
    var r: Result[Int, Int] = Ok(50);
    match r {
      Ok(v) => if v == 50 { return 0; },
      Err(_) => { },
    }
    return 1;
  }
use m21_result_option_018.run;
fn main() -> Int { return run(); }
