// m188 lock (packages relay): const values as match arms never matched --
// bare `CODE_A` patterns parse as Pattern::Ident and were treated as
// catch-all bindings, so every input fell to the wildcard arm
// (status_to_str returned "UNKNOWN" for every code). Const idents must be
// value-checked.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m188_const_match_arms

const CODE_A: Int = 1;
const CODE_B: Int = 2;

fn status_to_str(code: Int) -> Str {
  match code {
    CODE_A => { return "A"; }
    CODE_B => { return "B"; }
    _ => { return "UNKNOWN"; }
  }
}

fn main() -> Int {
  if status_to_str(1) != "A" { return 1; }
  if status_to_str(2) != "B" { return 2; }
  if status_to_str(9) != "UNKNOWN" { return 3; }
  return 0;
}
