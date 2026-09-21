// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0099

fn main() -> Int {
  var v: Int = 6;
  match v {
    v if v << 1 > 10 => { return 0; }
    _ => { return 1; }
  }
}
