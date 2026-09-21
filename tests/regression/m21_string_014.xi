// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_014
fn check(s: Str) -> Bool {
    return s.len() > 0;
  }

  pub fn run() -> Int {
    var name = "test";
    if check(name) { return 0; }
    return 1;
  }
use m21_string_014.run;
fn main() -> Int { return run(); }
