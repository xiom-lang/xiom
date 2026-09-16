// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0094

fn main() -> Int {
  var v: Int = 10;
  match v {
    v if v > 0 == true => { return 0; }
    _ => { return 1; }
  }
}
