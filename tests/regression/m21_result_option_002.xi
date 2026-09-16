// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_002
pub fn run() -> Int {
    var r: Result[Int16, Int16] = Ok(32767i16);
    match r {
      Ok(v) => if v == 32767i16 { return 0; },
      Err(_) => return 1,
    }
  }
use m21_result_option_002.run;
fn main() -> Int { return run(); }
