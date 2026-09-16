// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

enum Sign { Neg, Zero, Pos }
fn classify(n: Int) -> Sign {
  if n < 0 { return Sign.Neg; }
  if n == 0 { return Sign.Zero; }
  return Sign.Pos;
}
fn main() -> Int {
  match classify(-5) { Neg => {} _ => { return 1; } }
  match classify(0) { Zero => {} _ => { return 2; } }
  match classify(50) { Pos => {} _ => { return 3; } }
  return 0;
}