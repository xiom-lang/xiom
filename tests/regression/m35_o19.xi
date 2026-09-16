// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O19: Result[Int,Str] create -- Ok/Err construction and match
fn main() -> Int {
  var a: Result[Int, Str] = Ok(42);
  var b: Result[Int, Str] = Err("failed");
  match a { Ok(v) => { if v != 42 { return 1; } } Err(_) => { return 2; } }
  match b { Ok(_) => { return 3; } Err(e) => { if e != "failed" { return 4; } } }
  var c = Ok(-7);
  match c { Ok(v) => { if v != -7 { return 5; } } Err(_) => { return 6; } }
  var d = Err("oops");
  match d { Ok(_) => { return 7; } Err(_) => {} }
  return 0;
}
