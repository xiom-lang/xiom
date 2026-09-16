// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E06: Match with variable binding and condition testing
enum Sign { Neg, Zero, Pos }
fn classify(n: Int) -> Sign {
  if n < 0 { return Sign.Neg; }
  if n == 0 { return Sign.Zero; }
  return Sign.Pos;
}
fn main() -> Int {
  var s1 = classify(-5);
  match s1 {
    Neg => {}
    Zero => { return 1; }
    Pos => { return 1; }
  }
  var s2 = classify(0);
  match s2 {
    Neg => { return 1; }
    Zero => {}
    Pos => { return 1; }
  }
  var s3 = classify(10);
  match s3 {
    Neg => { return 1; }
    Zero => { return 1; }
    Pos => {}
  }
  return 0;
}
