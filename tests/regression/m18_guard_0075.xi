// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0075

fn main() -> Int {
  var v: Int = 7;
  match v {
    v if v > 0 && (v < 10 || v > 100) && v != 50 => { return 0; }
    _ => { return 1; }
  }
}
