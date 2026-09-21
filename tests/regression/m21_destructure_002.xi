// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_destructure_002
fn make_pair() -> { a: Int; b: Str; } {
    return { a: 42; b: "hello"; };
  }

  pub fn run() -> Int {
    var p = make_pair();
    if p.a == 42 && p.b == "hello" { return 0; }
    return 1;
  }
use m21_destructure_002.run;
fn main() -> Int { return run(); }
