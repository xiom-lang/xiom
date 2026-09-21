// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0107

fn main() -> Int {
  var ch: Char = 'x';
  match ch {
    c if c == 'x' => { return 0; }
    _ => { return 1; }
  }
}
