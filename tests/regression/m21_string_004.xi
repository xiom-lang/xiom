// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_004
pub fn run() -> Int {
    var prefix = "pre_";
    var suffix = "_post";
    var combined = prefix.concat(suffix);
    if combined.len() == 9 { return 0; }
    return 1;
  }
use m21_string_004.run;
fn main() -> Int { return run(); }
