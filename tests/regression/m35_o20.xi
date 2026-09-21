// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O20: Result map -- match-based map over Ok value
fn result_map_int(r: Result[Int, Str], f: fn(Int) -> Int) -> Result[Int, Str] {
  match r { Ok(v) => Ok(f(v)), Err(e) => Err(e) }
}
fn add_ten(x: Int) -> Int { return x + 10; }
fn main() -> Int {
  match result_map_int(Ok(5), add_ten) { Ok(v) => { if v != 15 { return 1; } } Err(_) => { return 2; } }
  match result_map_int(Err("bad"), add_ten) { Ok(_) => { return 3; } Err(e) => { if e != "bad" { return 4; } } }
  match result_map_int(Ok(0), add_ten) { Ok(v) => { if v != 10 { return 5; } } Err(_) => { return 6; } }
  return 0;
}
