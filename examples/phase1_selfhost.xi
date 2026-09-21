// XIOM -- phase1_selfhost
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module lexer {
    pub type Token = { kind: Int; line: Int; col: Int; }
    pub fn is_alpha(c: Int) -> Bool { return (c >= 65 && c <= 90) || (c >= 97 && c <= 122); }
    pub fn is_digit(c: Int) -> Bool { return c >= 48 && c <= 57; }
    pub fn make_token(kind: Int, line: Int, col: Int) -> Token { return Token{ kind: kind, line: line, col: col }; }
}

module parser {
    use lexer.Token;
    pub fn parse_number(tok: Token) -> Int { return tok.kind; }
    pub fn parse_ident(tok: Token) -> Int { return tok.kind + 1; }
}

module compiler {
    use lexer.make_token; use lexer.is_alpha; use lexer.is_digit;
    use parser.parse_number; use parser.parse_ident;
    pub fn compile(source: Int) -> Int {
        let tok = make_token(1, 1, 1); let num = parse_number(tok);
        let alpha_check = is_alpha(65); let digit_check = is_digit(48);
        if alpha_check && digit_check { return num; } return 0;
    }
}

use compiler.compile;
fn main() -> Int { return compile(0); }
