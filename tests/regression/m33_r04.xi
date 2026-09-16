// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn map_err(r: Result[Int, Str], f: fn(Str) -> Str) -> Result[Int, Str] {
  match r { Ok(v) => Ok(v), Err(e) => Err(f(e)) }
}
fn append_bang(s: Str) -> Str { return s + "!"; }
fn main() -> Int {
  match map_err(Err("fail"), append_bang) { Err(msg) => { if msg != "fail!" { return 1; } } Ok(_) => { return 2; } }
  if map_err(Ok(42), append_bang).is_ok() { return 0; }
  return 3;
}
