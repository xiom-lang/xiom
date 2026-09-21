// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0122

fn main() -> Int {
  var v: Int = 0;
  match v {
    v if v > 0 && v > 1 && v > 2 && v > 3 && v > 4 && v > 5 && v > 6 && v > 7 && v > 8 && v > 9 && v > 10 && v > 11 && v > 12 && v > 13 && v > 14 && v > 15 && v > 16 && v > 17 && v > 18 && v > 19 && v > 20 && v > 21 && v > 22 && v > 23 && v > 24 && v > 25 && v > 26 && v > 27 => { return 1; }
    _ => { return 0; }
  }
}
