// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_003
pub fn run() -> Int {
    var r: Result[Int32, Int32] = Ok(1000000i32);
    match r {
      Ok(v) => if v == 1000000i32 { return 0; },
      Err(_) => return 1,
    }
  }
use m21_result_option_003.run;
fn main() -> Int { return run(); }
