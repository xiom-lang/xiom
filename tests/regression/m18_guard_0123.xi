// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0123

fn main() -> Int {
  var v: Int = -1;
  match v {
    v if v > 0 && v < 100 || v == -1 => { return 0; }
    _ => { return 1; }
  }
}
