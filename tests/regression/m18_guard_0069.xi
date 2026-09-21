// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0069

fn main() -> Int {
  var x: Int64 = 999999i64;
  match x {
    v if v > 500000i64 => { return 0; }
    _ => { return 1; }
  }
}
