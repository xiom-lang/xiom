// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_006
fn get_greeting() -> Str {
    return "hi";
  }

  pub fn run() -> Int {
    var s = get_greeting();
    if s == "hi" { return 0; }
    return 1;
  }
use m21_string_006.run;
fn main() -> Int { return run(); }
