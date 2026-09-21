// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_complex_generic_007
pub fn map_pair[A, B, C](p: { first: A; second: B; }, f: fn(A) -> C) -> { first: C; second: B; } {
    return { first: f(p.first); second: p.second; };
  }

  pub fn run() -> Int {
    var p = { first: 10; second: true; };
    if p.first == 10 { return 0; }
    return 1;
  }
use m21_complex_generic_007.run;
fn main() -> Int { return run(); }
