// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m178 lock (stdlib wave 57): Result patterns on a Str-returning call must be
// a clean T001. It used to bind `s` as `_` and codegen then loaded a stale
// local slot from an earlier match arm -- invalid IR ("Instruction does not
// dominate all uses!").
fn s() -> Str { return "x"; }

fn main() -> Int {
  match s() {
    Ok(v) => { if v == "y" { return 11; } },
    Err(_) => { return 12; },
  }
  return 0;
}
