// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0013

fn main() -> Int {
  var x: Int = 5;
  var total: Int = 100;
  var divisor: Int = 10;
  match x {
    v if v > total / divisor => { return 1; }
    v if v > 0 => { return 0; }
    _ => { return 2; }
  }
}
