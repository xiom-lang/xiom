// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0018

fn main() -> Int {
  var opt: Option[Int] = Some(99);
  match opt {
    Some(v) if v > 50 => { return 0; }
    _ => { return 1; }
  }
}
