// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0083

fn main() -> Int {
  var val: Int = 10;
  match val {
    v if v > 0 => {
      var v: Int = 99;
      match v {
        99 => { return 0; }
        _ => { return 1; }
      }
    }
    _ => { return 2; }
  }
}
