// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E10: Generic enum with payload (Result-like), match both variants
enum Result[V, E] { Ok(val: V), Err(err: E) }
fn div(a: Int, b: Int) -> Result[Int, Str] {
  if b == 0 { return Result.Err("divide by zero"); }
  return Result.Ok(a / b);
}
fn main() -> Int {
  match div(10, 2) {
    Ok(v) => { if v != 5 { return 1; } }
    Err(_) => { return 2; }
  }
  match div(10, 0) {
    Ok(_) => { return 3; }
    Err(e) => { if e != "divide by zero" { return 4; } }
  }
  return 0;
}
