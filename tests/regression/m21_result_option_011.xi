// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_011
pub fn run() -> Int {
    var opt: Option[Int32] = Some(-500000i32);
    match opt {
      Some(v) => if v == -500000i32 { return 0; },
      None => return 1,
    }
  }
use m21_result_option_011.run;
fn main() -> Int { return run(); }
