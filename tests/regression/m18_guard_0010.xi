// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0010

fn main() -> Int {
  var a: Int = 10;
  var b: Int = 5;
  var x: Int = 20;
  match x {
    v if v > a + b => { return 0; }
    _ => { return 1; }
  }
}
