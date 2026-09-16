// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_010
pub fn run() -> Int {
    var opt: Option[Int8] = Some(127i8);
    match opt {
      Some(v) => if v == 127i8 { return 0; },
      None => return 1,
    }
  }
use m21_result_option_010.run;
fn main() -> Int { return run(); }
