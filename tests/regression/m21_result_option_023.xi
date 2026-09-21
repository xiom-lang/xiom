// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_023
pub fn run() -> Int {
    var opt: Option[Int] = Some(5);
    match opt {
      Some(v) => if v > 0 { return 0; },
      None => return 1,
    }
  }
use m21_result_option_023.run;
fn main() -> Int { return run(); }
