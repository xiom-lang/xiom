// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0096

fn main() -> Int {
  var v: Int = 3;
  match v {
    v if v & 1 == 1 => { return 0; }
    _ => { return 1; }
  }
}
