// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E08: Exhaustive match on enum with 5+ variants
enum Token { A, B, C, D, E }
fn kind_val(t: Token) -> Int {
  match t {
    A => { return 0; }
    B => { return 1; }
    C => { return 2; }
    D => { return 3; }
    E => { return 4; }
  }
}
fn main() -> Int {
  if kind_val(Token.A) != 0 { return 1; }
  if kind_val(Token.B) != 1 { return 2; }
  if kind_val(Token.C) != 2 { return 3; }
  if kind_val(Token.D) != 3 { return 4; }
  if kind_val(Token.E) != 4 { return 5; }
  return 0;
}
