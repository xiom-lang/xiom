// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_012
pub fn run() -> Int {
    var opt: Option[Int] = None;
    match opt {
      Some(_) => return 1,
      None => return 0,
    }
  }
use m21_result_option_012.run;
fn main() -> Int { return run(); }
