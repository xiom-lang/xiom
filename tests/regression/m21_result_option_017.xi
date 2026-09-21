// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_017
pub fn run() -> Int {
    var opt: Option[Int] = None;
    var v = opt.unwrap_or(42);
    if v == 42 { return 0; }
    return 1;
  }
use m21_result_option_017.run;
fn main() -> Int { return run(); }
