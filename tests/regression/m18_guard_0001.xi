// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0001

fn main() -> Int {
  var x: Int = 15;
  match x {
    v if v > 10 => { return 0; }
    _ => { return 1; }
  }
}
