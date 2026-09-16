// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0009

fn main() -> Int {
  var x: Int = 7;
  match x {
    v if v < 0 => { return 1; }
    v if v > 10 => { return 2; }
    v if v > 0 && v < 10 => { return 0; }
    _ => { return 3; }
  }
}
