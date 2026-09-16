// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0002

fn main() -> Int {
  var x: Int = 5;
  match x {
    v if v > 10 => { return 1; }
    _ => { return 0; }
  }
}
