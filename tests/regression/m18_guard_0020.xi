// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0020

fn main() -> Int {
  var opt: Option[Int] = None;
  match opt {
    Some(v) if v > 10 => { return 1; }
    Some(v) => { return 2; }
    None => { return 0; }
  }
}
