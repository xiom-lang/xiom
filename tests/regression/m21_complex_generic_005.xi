// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m21_complex_generic_005
pub fn compose[A, B, C](a: A, f: fn(A) -> B, g: fn(B) -> C) -> C {
    return g(f(a));
  }

  pub fn run() -> Int {
    var x = 5;
    if x == 5 { return 0; }
    return 1;
  }
use m21_complex_generic_005.run;
fn main() -> Int { return run(); }
