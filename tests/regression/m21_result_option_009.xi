// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_009
pub fn run() -> Int {
    var opt: Option[Int16] = Some(100i16);
    match opt {
      Some(v) => if v == 100i16 { return 0; },
      None => return 1,
    }
  }
use m21_result_option_009.run;
fn main() -> Int { return run(); }
