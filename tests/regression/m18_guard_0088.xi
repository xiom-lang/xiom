// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0088

fn main() -> Int {
  var opt: Option[Int] = None;
  match opt {
    x if x is Some(n) && n > 10 => { return 1; }
    _ => { return 0; }
  }
}
