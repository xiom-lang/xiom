// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_015
pub fn run() -> Int {
    var s = "line1\nline2";
    if s.len() == 11 { return 0; }
    return 1;
  }
use m21_string_015.run;
fn main() -> Int { return run(); }
