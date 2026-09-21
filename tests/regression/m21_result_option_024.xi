// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_024
pub fn run() -> Int {
    var opt: Option[Int] = Some(42);
    if opt.is_some() { return 0; }
    return 1;
  }
use m21_result_option_024.run;
fn main() -> Int { return run(); }
