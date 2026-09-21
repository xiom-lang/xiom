// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0097

fn main() -> Int {
  var v: Int = 2;
  match v {
    v if v | 4 > v => { return 0; }
    _ => { return 1; }
  }
}
