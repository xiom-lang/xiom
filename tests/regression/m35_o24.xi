// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O24: Result unwrap_or -- match-based unwrap_or implementation
fn unwrap_or_result(r: Result[Int, Str], default: Int) -> Int {
  match r { Ok(v) => v, Err(_) => default }
}
fn main() -> Int {
  if unwrap_or_result(Ok(42), 0) != 42 { return 1; }
  if unwrap_or_result(Err("bad"), 99) != 99 { return 2; }
  if unwrap_or_result(Ok(-5), 10) != -5 { return 3; }
  if unwrap_or_result(Ok(0), 7) != 0 { return 4; }
  return 0;
}
