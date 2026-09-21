// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_007
pub fn run() -> Int {
    var r: Result[Int32, Int32] = Err(-999999i32);
    match r {
      Ok(_) => return 1,
      Err(e) => if e == -999999i32 { return 0; },
    }
  }
use m21_result_option_007.run;
fn main() -> Int { return run(); }
