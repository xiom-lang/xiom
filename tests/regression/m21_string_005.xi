// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_string_005
fn greet(name: Str) -> Str {
    return "Hello, ".concat(name);
  }

  pub fn run() -> Int {
    var msg = greet("World");
    if msg == "Hello, World" { return 0; }
    return 1;
  }
use m21_string_005.run;
fn main() -> Int { return run(); }
