// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0111

type Limits = { min: Int; max: Int; }

fn main() -> Int {
  var v: Int = 75;
  match v {
    v if v > Limits{ min: 0; max: 10; }.max => { return 0; }
    _ => { return 1; }
  }
}
