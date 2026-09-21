// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0016

fn main() -> Int {
  var r: Result[Int, Str] = Ok(42);
  match r {
    Ok(v) if v > 10 => { return 0; }
    _ => { return 1; }
  }
}
