// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0113

fn main() -> Int {
  var x: Int = 10;
  var y: Int = 25;
  match x {
    v if v > 5 => { }
    _ => { return 1; }
  }
  match y {
    v if v > 20 => { return 0; }
    _ => { return 2; }
  }
}
