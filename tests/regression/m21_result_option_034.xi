// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_034
pub fn run() -> Int {
    var r: Result[Int, Int] = Ok(5);
    var result = 0;
    match r {
      Ok(v) => result = v * 10,
      Err(_) => result = -1,
    }
    if result == 50 { return 0; }
    return 1;
  }
use m21_result_option_034.run;
fn main() -> Int { return run(); }
