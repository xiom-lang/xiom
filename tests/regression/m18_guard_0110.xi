// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0110

fn main() -> Int {
  var s: Str = "test";
  match s {
    v if v.len() > 0 => { return 0; }
    _ => { return 1; }
  }
}
