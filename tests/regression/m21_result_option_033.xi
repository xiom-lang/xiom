// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_033
pub fn run() -> Int {
    var opt: Option[Int] = Some(10);
    var result = 0;
    match opt {
      Some(v) => result = v * 2,
      None => result = -1,
    }
    if result == 20 { return 0; }
    return 1;
  }
use m21_result_option_033.run;
fn main() -> Int { return run(); }
