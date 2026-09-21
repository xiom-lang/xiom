// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0012

fn main() -> Int {
  var a: Int = 50;
  var b: Int = 20;
  var x: Int = 30;
  match x {
    v if v == a - b => { return 0; }
    _ => { return 1; }
  }
}
