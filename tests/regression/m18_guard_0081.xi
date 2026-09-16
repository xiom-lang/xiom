// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0081

fn main() -> Int {
  var x: Int = 5;
  match 10 {
    x if x > 5 => { return 0; }
    _ => { return 1; }
  }
}
