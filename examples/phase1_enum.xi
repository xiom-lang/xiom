// XIOM -- phase1_enum
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

enum TokenKind {
  Eof,
  Ident(name: Str),
  Int(value: Int),
  Str(text: Str)
}

fn token_name(kind: Int) -> Int {
  match kind {
    0 => 1,
    1 => 2,
    2 => 3,
    _ => 0,
  }
}

fn main() -> Int {
  return token_name(1);
}
