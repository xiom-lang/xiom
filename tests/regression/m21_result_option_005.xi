// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_005
pub fn run() -> Int {
    var r: Result[Int, Int] = Err(-1);
    match r {
      Ok(_) => return 1,
      Err(e) => if e == -1 { return 0; },
    }
  }
use m21_result_option_005.run;
fn main() -> Int { return run(); }
