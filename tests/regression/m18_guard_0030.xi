// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0030

fn is_even_wrapper(n: Int) -> Bool { return n % 2 == 0; }

fn main() -> Int {
  var x: Int = 15;
  match x {
    v if !is_even_wrapper(v) => { return 0; }
    _ => { return 1; }
  }
}
