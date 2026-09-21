// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0024

fn main() -> Int {
  var opt: Option[Int] = None;
  match opt {
    None => { return 0; }
    Some(v) if v > 0 => { return 1; }
    _ => { return 2; }
  }
}
