// XIOM — phase1_stress
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

module types {
  pub type Token = { kind: Int; lexeme: Str; line: Int; col: Int; } derive[Eq, Clone]
  pub type AstNode = { tag: Int; left: Int; right: Int; } derive[Eq, Clone]
}

module utils {
  pub fn max(a: Int, b: Int) -> Int { if a > b { return a; } return b; }
  pub fn min(a: Int, b: Int) -> Int { if a < b { return a; } return b; }
}

use types.Token;

fn make_token(kind: Int, text: Str, line: Int, col: Int) -> Token {
  return Token{ kind: kind, lexeme: text, line: line, col: col };
}

fn main() -> Int {
  var position = 0;
  let tok = make_token(1, "fn", 1, 1);
  let m = utils.max(10, 20);
  let n = utils.min(30, 5);
  position = position + 1;
  return tok.kind + m + n + position;
}
