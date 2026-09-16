// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0059

fn main() -> Int {
  var opt = Some(Ok(77));
  match opt {
    Some(r) => {
      match r {
        Ok(v) => {
          match v {
            n if n > 50 => { return 0; }
            _ => { return 1; }
          }
        }
        _ => { return 2; }
      }
    }
    None => { return 3; }
  }
}
