// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_038
pub fn run() -> Int {
    var r: Result[Int, Int] = Ok(200);
    match r {
      Ok(code) if code >= 200 => return 0,
      Ok(_) => return 1,
      Err(e) if e < 0 => return 1,
      Err(_) => return 1,
    }
  }
use m21_result_option_038.run;
fn main() -> Int { return run(); }
