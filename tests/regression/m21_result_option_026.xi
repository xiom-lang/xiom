// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_026
pub fn run() -> Int {
    var r: Result[Int, Int] = Ok(1);
    if r.is_ok() { return 0; }
    return 1;
  }
use m21_result_option_026.run;
fn main() -> Int { return run(); }
