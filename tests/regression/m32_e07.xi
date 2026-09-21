// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E07: Enum return from function, match on result
enum DivResult { Success(val: Int), Fail }
fn safe_div(a: Int, b: Int) -> DivResult {
  if b == 0 { return DivResult.Fail; }
  return DivResult.Success(a / b);
}
fn main() -> Int {
  match safe_div(10, 2) {
    Success(v) => { if v != 5 { return 1; } }
    Fail => { return 2; }
  }
  match safe_div(10, 0) {
    Success(_) => { return 3; }
    Fail => {}
  }
  return 0;
}
