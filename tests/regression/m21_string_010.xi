// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_010
pub fn run() -> Int {
    var status = "ok";
    match status {
      "ok" => return 0,
      "err" => return 1,
      _ => return 2,
    }
  }
use m21_string_010.run;
fn main() -> Int { return run(); }
