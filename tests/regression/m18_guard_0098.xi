// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0098

fn main() -> Int {
  var v: Int = 5;
  match v {
    v if v ^ v == 0 => { return 0; }
    _ => { return 1; }
  }
}
