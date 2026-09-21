// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module test {
  fn f(x: Int) -> Int {
    match x {
      0 => 1,
      _ => 0,
    }
    return x;
  }
  pub fn run() -> Int { return f(5); }
}

use test.run;
fn main() -> Int { return run(); }
