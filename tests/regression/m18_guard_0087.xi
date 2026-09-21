// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0087

fn main() -> Int {
  var r: Result[Int, Str] = Ok(99);
  match r {
    x if x is Ok(v) && v > 0 => { return 0; }
    _ => { return 1; }
  }
}
