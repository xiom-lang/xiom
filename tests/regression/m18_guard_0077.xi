// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0077

fn main() -> Int {
  var x: Int = 5;
  match x {
    1 | 2 | 3 if false => { return 1; }
    _ => { return 0; }
  }
}
