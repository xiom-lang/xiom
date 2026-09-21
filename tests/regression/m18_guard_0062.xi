// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0062

fn main() -> Int {
  var v: Int = 75;
  match v {
    v if v >= 80 => { return 0; }
    v if v >= 50 => { return 0; }
    _ => { return 1; }
  }
}
