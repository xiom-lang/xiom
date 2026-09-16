// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

enum Token { A, B, C, D, E, F, G, H, I, J }
fn kind_str(t: Token) -> Int {
  match t {
    A => { return 0; }
    B => { return 1; }
    C => { return 2; }
    D => { return 3; }
    E => { return 4; }
    F => { return 5; }
    G => { return 6; }
    H => { return 7; }
    I => { return 8; }
    J => { return 9; }
  }
}
fn main() -> Int {
  if kind_str(Token.A) != 0 { return 1; }
  if kind_str(Token.J) != 9 { return 2; }
  if kind_str(Token.E) != 4 { return 3; }
  return 0;
}