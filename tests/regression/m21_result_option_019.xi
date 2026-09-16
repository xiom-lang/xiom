// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_019
pub fn run() -> Int {
    var r: Result[Int, Int] = Err(500);
    match r {
      Ok(_) => return 1,
      Err(code) => if code == 500 { return 0; },
    }
  }
use m21_result_option_019.run;
fn main() -> Int { return run(); }
