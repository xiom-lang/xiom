// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E12: Enum as return from function, multiple match sites
enum Eval { Pass, Fail(reason: Str) }
fn check_even(n: Int) -> Eval {
  if n % 2 == 0 { return Eval.Pass; }
  return Eval.Fail("odd");
}
fn main() -> Int {
  match check_even(4) {
    Pass => {}
    Fail(_) => { return 1; }
  }
  match check_even(5) {
    Pass => { return 2; }
    Fail(r) => {
      if r != "odd" { return 3; }
    }
  }
  match check_even(0) {
    Pass => {}
    Fail(_) => { return 4; }
  }
  match check_even(7) {
    Pass => { return 5; }
    Fail(_) => {}
  }
  return 0;
}
