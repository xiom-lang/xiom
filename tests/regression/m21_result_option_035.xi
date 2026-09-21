// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_result_option_035
fn get_value() -> Option[Int] {
    return Some(42);
  }

  pub fn run() -> Int {
    var opt = get_value();
    match opt {
      Some(v) => if v == 42 { return 0; },
      None => return 1,
    }
  }
use m21_result_option_035.run;
fn main() -> Int { return run(); }
