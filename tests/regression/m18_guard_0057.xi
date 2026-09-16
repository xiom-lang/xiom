// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0057

fn main() -> Int {
  var r: Result[Int, Str] = Ok(42);
  match r {
    Ok(v) if v > 0 => {
      match v {
        n if n > 40 => { return 0; }
        _ => { return 1; }
      }
    }
    _ => { return 2; }
  }
}
