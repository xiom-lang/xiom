// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O23: Result or_else -- match-based or_else fallback
fn res_or_else(r: Result[Int, Str], fallback: fn(Str) -> Result[Int, Str]) -> Result[Int, Str] {
  match r { Ok(v) => Ok(v), Err(e) => fallback(e) }
}
fn recover(s: Str) -> Result[Int, Str] {
  if s == "retry" { return Ok(99); }
  return Err("unrecoverable");
}
fn main() -> Int {
  match res_or_else(Ok(42), recover) { Ok(v) => { if v != 42 { return 1; } } Err(_) => { return 2; } }
  match res_or_else(Err("retry"), recover) { Ok(v) => { if v != 99 { return 3; } } Err(_) => { return 4; } }
  match res_or_else(Err("fatal"), recover) { Ok(_) => { return 5; } Err(_) => {} }
  return 0;
}
