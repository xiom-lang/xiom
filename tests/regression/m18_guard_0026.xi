// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0026

fn is_valid(n: Int) -> Bool { return n > 0 && n < 100; }

fn main() -> Int {
  var x: Int = 50;
  match x {
    v if is_valid(v) => { return 0; }
    _ => { return 1; }
  }
}
