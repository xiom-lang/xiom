// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_032
pub fn run() -> Int {
    var outer: Option[Result[Int, Int]] = Some(Ok(42));
    match outer {
      Some(inner) => match inner {
        Ok(v) => if v == 42 { return 0; },
        Err(_) => return 1,
      },
      None => return 1,
    }
  }
use m21_result_option_032.run;
fn main() -> Int { return run(); }
