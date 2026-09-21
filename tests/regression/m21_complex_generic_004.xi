// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_complex_generic_004
type Triple[A, B, C] = { a: A; b: B; c: C; }

  pub fn run() -> Int {
    var t: Triple[Int, Bool, Str] = { a: 1; b: true; c: "hello"; };
    if t.a == 1 && t.b && t.c == "hello" { return 0; }
    return 1;
  }
use m21_complex_generic_004.run;
fn main() -> Int { return run(); }
