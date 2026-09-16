// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0015

fn main() -> Int {
  var a: Int = 10;
  var b: Int = 3;
  var c: Int = 5;
  var x: Int = 30;
  match x {
    v if v > a + b * c => { return 0; }
    _ => { return 1; }
  }
}
