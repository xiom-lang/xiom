// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0116

fn main() -> Int {
  var opt: Option[Int] = Some(50);
  match opt {
    Some(v) if v > 10 => {
      match v {
        n if n < 100 => { return 0; }
        _ => { return 1; }
      }
    }
    _ => { return 2; }
  }
}
