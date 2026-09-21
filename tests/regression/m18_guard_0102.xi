// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0102

fn main() -> Int {
  var x: Int = 15;
  var pred = |n| n > 5 && n < 20;
  match x {
    v if pred(v) => { return 0; }
    _ => { return 1; }
  }
}
