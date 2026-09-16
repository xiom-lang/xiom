// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0061

fn main() -> Int {
  var v: Int = 42;
  match v {
    v if v < 0 => { return 1; }
    v if v == 0 => { return 2; }
    v if v < 10 => { return 3; }
    v if v < 50 => { return 0; }
    _ => { return 4; }
  }
}
