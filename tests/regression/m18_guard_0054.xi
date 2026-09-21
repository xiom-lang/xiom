// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0054

fn main() -> Int {
  var opt = Some(50);
  match opt {
    Some(v) if v > 100 => { return 1; }
    Some(v) => { return 0; }
    None => { return 2; }
  }
}
