// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

fn make_opt(flag: Bool) -> Result[Int, Str] {
  if flag { return Ok(42); }
  return Err("disabled");
}
fn opt_to_result(o: Option[Int]) -> Result[Int, Str] {
  match o { Some(v) => Ok(v), None => Err("missing") }
}
fn main() -> Int {
  match make_opt(true) { Ok(v) => { if v != 42 { return 1; } } Err(_) => { return 2; } }
  var p: Option[Int] = Some(99);
  match opt_to_result(p) { Ok(v) => { if v != 99 { return 3; } } Err(_) => { return 4; } }
  var q: Option[Int] = None;
  match opt_to_result(q) { Ok(_) => { return 5; } Err(_) => {} }
  return 0;
}
