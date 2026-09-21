// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module regression.m18_guard_0021

fn main() -> Int {
  var r: Result[Int, Str] = Err("failure");
  match r {
    Ok(v) => { return 1; }
    Err(e) if e.len() > 0 => { return 0; }
    _ => { return 2; }
  }
}
