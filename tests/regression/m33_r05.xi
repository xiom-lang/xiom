// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn safe_div(a: Int, b: Int) -> Result[Int, Str] {
  if b == 0 { return Err("div0"); }
  return Ok(a / b);
}
fn bind_div(r: Result[Int, Str], by: Int) -> Result[Int, Str] {
  match r { Ok(v) => safe_div(v, by), Err(e) => Err(e) }
}
fn main() -> Int {
  var a = bind_div(safe_div(20, 4), 2);
  if a.is_ok() { var v = a.unwrap(); if v != 2 { return 1; } } else { return 2; }
  var b = bind_div(safe_div(10, 0), 1);
  if !b.is_err() { return 3; }
  return 0;
}
