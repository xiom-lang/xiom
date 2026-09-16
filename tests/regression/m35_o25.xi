// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O25: Result expect -- .unwrap() with match validation
fn main() -> Int {
  var a: Result[Int, Str] = Ok(42);
  var b: Result[Int, Str] = Err("oops");
  if a.unwrap() != 42 { return 1; }
  if a.is_ok() != true { return 2; }
  if b.is_err() != true { return 3; }
  match b { Ok(_) => { return 4; } Err(e) => { if e != "oops" { return 5; } } }
  return 0;
}
