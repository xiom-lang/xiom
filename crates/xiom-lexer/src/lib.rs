// XIOM — Lexer
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Lexer — converts UTF-8 source to a flat token stream.
//! No whitespace significance except within string literals.

use xiom_ast::Span;

// ============================================================================
// Token definitions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // --- Keywords ---
    Let, Var, Const, Fn, Return,
    Break, Continue,
    If, Elif, Else, Match, While, For, In,
    Spawn, Await, Comptime,
    Module, Use, Pub, As,
    Type, Enum, Interface, Derive,
    // requires/ensures/invariant are CONTEXTUAL keywords (5c-R: interned symbols,
    // rustc lesson). They are tokenized as regular Ident and only recognized
    // as keywords at specific parser positions — so `var requires = 5;` works.
    True, False, Self_,
    Some, None, Ok_, Err_,
    Unsafe, Extern, Is,

    // --- Literals ---
    Ident(String),
    Int(u64),
    Float(f64),
    Str(String),
    Char(char),

    // --- Operators / Punctuation ---
    Dot, Comma, Semicolon, Colon,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    At, Arrow, FatArrow, Question,
    Plus, Minus, Star, Slash, Percent, Caret, Tilde,
    Bang, Amp, Pipe, Ampersand,
    Eq, EqEq, Neq, Lt, Gt, Le, Ge,
    AndAnd, OrOr,
    Underscore, Hash,

    // --- Special ---
    Eof,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, lexeme: impl Into<String>) -> Self {
        Self { kind, span, lexeme: lexeme.into() }
    }

    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }
}

// ============================================================================
// Lexer
// ============================================================================

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn span(&self) -> Span {
        Span::new(self.line, self.col)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.source.get(self.pos + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn advance_while<F: Fn(char) -> bool>(&mut self, pred: F) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if pred(c) {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        s
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.is_eof();
            tokens.push(tok);
            if is_eof { break; }
        }
        tokens
    }

    fn next_token(&mut self) -> Token {
        // Skip whitespace and comments
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => { self.advance(); }
                Some('/') if self.peek_n(1) == Some('/') => {
                    self.advance_while(|c| c != '\n');
                }
                Some('/') if self.peek_n(1) == Some('*') => {
                    self.advance(); self.advance(); // skip /*
                    loop {
                        if self.peek() == Some('*') && self.peek_n(1) == Some('/') {
                            self.advance(); self.advance(); // skip */
                            break;
                        }
                        if self.advance().is_none() {
                            return self.error("unterminated block comment");
                        }
                    }
                }
                _ => break,
            }
        }

        let start = self.span();
        let ch = match self.peek() {
            Some(c) => c,
            None => return Token::new(TokenKind::Eof, self.span(), ""),
        };

        match ch {
            // --- Identifiers and keywords ---
            c if c.is_ascii_alphabetic() || c == '_' => {
                let ident = self.advance_while(|c| c.is_ascii_alphanumeric() || c == '_');
                let kind = Self::keyword_or_ident(&ident);
                Token::new(kind, start, ident)
            }

            // --- Numbers ---
            c if c.is_ascii_digit() => {
                // Hex literal: 0xABCD or 0XABCD
                if c == '0' && matches!(self.peek_n(1), Some(next) if next == 'x' || next == 'X') {
                    self.advance(); // consume '0'
                    self.advance(); // consume 'x' or 'X'
                    let hex = self.advance_while(|c| c.is_ascii_hexdigit() || c == '_');
                    let num: u64 = u64::from_str_radix(&hex.replace('_', ""), 16).unwrap_or(0);
                    return Token::new(TokenKind::Int(num), start, format!("0x{hex}"));
                }
                let int_part = self.advance_while(|c| c.is_ascii_digit() || c == '_');
                if self.peek() == Some('.') && self.peek_n(1).map_or(false, |c| c.is_ascii_digit()) {
                    self.advance(); // skip '.'
                    let frac = self.advance_while(|c| c.is_ascii_digit());
                    // Handle scientific notation exponent: e308, e-308, E+10
                    let mut exp = String::new();
                    if matches!(self.peek(), Some('e' | 'E')) {
                        exp.push(self.advance().unwrap()); // 'e' or 'E'
                        if matches!(self.peek(), Some('+' | '-')) {
                            exp.push(self.advance().unwrap());
                        }
                        let exp_digits = self.advance_while(|c| c.is_ascii_digit());
                        exp.push_str(&exp_digits);
                    }
                    let full = format!("{int_part}.{frac}{exp}");
                    let num: f64 = full.parse().unwrap_or(0.0);
                    Token::new(TokenKind::Float(num), start, full)
                } else {
                    // Also handle integer scientific notation: 1e10
                    if matches!(self.peek(), Some('e' | 'E')) {
                        let mut exp = String::from(&int_part);
                        exp.push(self.advance().unwrap()); // 'e' or 'E'
                        if matches!(self.peek(), Some('+' | '-')) {
                            exp.push(self.advance().unwrap());
                        }
                        let exp_digits = self.advance_while(|c| c.is_ascii_digit());
                        exp.push_str(&exp_digits);
                        let num: f64 = exp.parse().unwrap_or(0.0);
                        return Token::new(TokenKind::Float(num), start, exp);
                    }
                    let num: u64 = int_part.replace('_', "").parse().unwrap_or(0);
                    Token::new(TokenKind::Int(num), start, int_part)
                }
            }

            // --- Strings ---
            '"' => {
                self.advance(); // skip opening "
                let mut s = String::new();
                loop {
                    match self.peek() {
                        None => return self.error_at(start, "unterminated string literal"),
                        Some('"') => { self.advance(); break; }
                    Some('\\') => {
                        self.advance();
                        match self.advance() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some('r') => s.push('\r'),
                            Some('\\') => s.push('\\'),
                            Some('"') => s.push('"'),
                            Some('\'') => s.push('\''),
                            Some('0') => s.push('\0'),
                            Some('b') => s.push('\x08'),
                            Some('f') => s.push('\x0C'),
                            Some('x') => {
                                let h1 = self.advance().unwrap_or('0');
                                let h2 = self.advance().unwrap_or('0');
                                let d1 = h1.to_digit(16).unwrap_or(0) as u8;
                                let d2 = h2.to_digit(16).unwrap_or(0) as u8;
                                s.push(((d1 << 4) | d2) as char);
                            }
                            Some('u') => {
                                if self.advance() != Some('{') {
                                    return self.error("expected '{' after \\u");
                                }
                                let hex = self.advance_while(|c| c.is_ascii_hexdigit());
                                if self.advance() != Some('}') {
                                    return self.error("expected '}' after \\u hex digits");
                                }
                                let codepoint = u32::from_str_radix(&hex, 16).unwrap_or(0xFFFD);
                                s.push(char::from_u32(codepoint).unwrap_or('\u{FFFD}'));
                            }
                            _ => return self.error("invalid escape sequence"),
                        }
                    }
                        Some(c) => { self.advance(); s.push(c); }
                    }
                }
                Token::new(TokenKind::Str(s.clone()), start, format!("\"{s}\""))
            }

            // --- Char literals ---
            '\'' => {
                self.advance(); // skip opening '
                let c = match self.advance() {
                    Some('\\') => match self.advance() {
                        Some('n') => '\n',
                        Some('t') => '\t',
                        Some('r') => '\r',
                        Some('\\') => '\\',
                        Some('\'') => '\'',
                        Some('"') => '"',
                        Some('0') => '\0',
                        Some('b') => '\x08',
                        Some('f') => '\x0C',
                        Some('x') => {
                            // Hex escape: \xNN (2 hex digits)
                            let h1 = self.advance().unwrap_or('\0');
                            let h2 = self.advance().unwrap_or('\0');
                            let d1 = h1.to_digit(16).unwrap_or(0) as u8;
                            let d2 = h2.to_digit(16).unwrap_or(0) as u8;
                            ((d1 << 4) | d2) as char
                        }
                        _ => return self.error("invalid escape in char literal"),
                    },
                    Some(c) if c != '\'' => c,
                    _ => return self.error("empty char literal"),
                };
                if self.advance() != Some('\'') {
                    return self.error("unterminated char literal");
                }
                Token::new(TokenKind::Char(c), start, format!("'{c}'"))
            }

            // --- Operators and punctuation ---
            '.' => { self.advance(); Token::new(TokenKind::Dot, start, ".") }
            ',' => { self.advance(); Token::new(TokenKind::Comma, start, ",") }
            ';' => { self.advance(); Token::new(TokenKind::Semicolon, start, ";") }
            ':' => { self.advance(); Token::new(TokenKind::Colon, start, ":") }
            '(' => { self.advance(); Token::new(TokenKind::LParen, start, "(") }
            ')' => { self.advance(); Token::new(TokenKind::RParen, start, ")") }
            '{' => { self.advance(); Token::new(TokenKind::LBrace, start, "{") }
            '}' => { self.advance(); Token::new(TokenKind::RBrace, start, "}") }
            '[' => { self.advance(); Token::new(TokenKind::LBracket, start, "[") }
            ']' => { self.advance(); Token::new(TokenKind::RBracket, start, "]") }
            '@' => { self.advance(); Token::new(TokenKind::At, start, "@") }
            '#' => { self.advance(); Token::new(TokenKind::Hash, start, "#") }
            '?' => { self.advance(); Token::new(TokenKind::Question, start, "?") }
            '+' => { self.advance(); Token::new(TokenKind::Plus, start, "+") }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    Token::new(TokenKind::Arrow, start, "->")
                } else {
                    Token::new(TokenKind::Minus, start, "-")
                }
            }
            '*' => { self.advance(); Token::new(TokenKind::Star, start, "*") }
            '/' => { self.advance(); Token::new(TokenKind::Slash, start, "/") }
            '%' => { self.advance(); Token::new(TokenKind::Percent, start, "%") }
            '^' => { self.advance(); Token::new(TokenKind::Caret, start, "^") }
            '~' => { self.advance(); Token::new(TokenKind::Tilde, start, "~") }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Neq, start, "!=")
                } else {
                    Token::new(TokenKind::Bang, start, "!")
                }
            }
            '&' => {
                self.advance();
                if self.peek() == Some('&') {
                    self.advance();
                    Token::new(TokenKind::AndAnd, start, "&&")
                } else {
                    Token::new(TokenKind::Ampersand, start, "&")
                }
            }
            '|' => {
                self.advance();
                if self.peek() == Some('|') {
                    self.advance();
                    Token::new(TokenKind::OrOr, start, "||")
                } else {
                    Token::new(TokenKind::Pipe, start, "|")
                }
            }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::EqEq, start, "==")
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token::new(TokenKind::FatArrow, start, "=>")
                } else {
                    Token::new(TokenKind::Eq, start, "=")
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Le, start, "<=")
                } else {
                    Token::new(TokenKind::Lt, start, "<")
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Ge, start, ">=")
                } else {
                    Token::new(TokenKind::Gt, start, ">")
                }
            }

            _ => {
                self.advance();
                Token::new(TokenKind::Error(format!("unexpected character: '{ch}'")), start, ch.to_string())
            }
        }
    }

    fn keyword_or_ident(s: &str) -> TokenKind {
        match s {
            "let"       => TokenKind::Let,
            "var"       => TokenKind::Var,
            "const"     => TokenKind::Const,
            "fn"        => TokenKind::Fn,
            "return"    => TokenKind::Return,
            "break"     => TokenKind::Break,
            "continue"  => TokenKind::Continue,
            "if"        => TokenKind::If,
            "elif"      => TokenKind::Elif,
            "else"      => TokenKind::Else,
            "match"     => TokenKind::Match,
            "while"     => TokenKind::While,
            "for"       => TokenKind::For,
            "in"        => TokenKind::In,
            "spawn"     => TokenKind::Spawn,
            "await"     => TokenKind::Await,
            "comptime"  => TokenKind::Comptime,
            "module"    => TokenKind::Module,
            "use"       => TokenKind::Use,
            "pub"       => TokenKind::Pub,
            "as"        => TokenKind::As,
            "type"      => TokenKind::Type,
            "enum"      => TokenKind::Enum,
            "interface" => TokenKind::Interface,
            "derive"    => TokenKind::Derive,
            // requires/ensures/invariant — contextuel keywords (5c-R).
            // Removed from reserved set; tokenized as regular Ident.
            "true"      => TokenKind::True,
            "false"     => TokenKind::False,
            "self"      => TokenKind::Self_,

            "Some"      => TokenKind::Some,
            "None"      => TokenKind::None,
            "Ok"        => TokenKind::Ok_,
            "Err"       => TokenKind::Err_,
            "unsafe"    => TokenKind::Unsafe,
            "extern"    => TokenKind::Extern,
            "is"        => TokenKind::Is,
            "_"         => TokenKind::Underscore,
            _           => TokenKind::Ident(s.to_string()),
        }
    }

    fn error(&self, msg: impl Into<String>) -> Token {
        Token::new(TokenKind::Error(msg.into()), self.span(), "")
    }

    fn error_at(&self, span: Span, msg: impl Into<String>) -> Token {
        Token::new(TokenKind::Error(msg.into()), span, "")
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<TokenKind> {
        Lexer::new(source).tokenize().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_empty() {
        assert_eq!(lex(""), vec![TokenKind::Eof]);
    }

    #[test]
    fn test_keywords() {
        let tokens = lex("let var const fn return if elif else match while for in spawn async await comptime module use pub as type enum interface derive requires ensures invariant true false self");
        assert!(tokens.iter().any(|t| matches!(t, TokenKind::Let)));
        assert!(tokens.iter().any(|t| matches!(t, TokenKind::Fn)));
        assert!(tokens.iter().any(|t| matches!(t, TokenKind::Interface)));
        assert!(tokens.iter().any(|t| matches!(t, TokenKind::Derive)));
        // requires/ensures/invariant are contextual keywords (5c-R): tokenized as Ident
        assert!(tokens.iter().any(|t| matches!(t, TokenKind::Ident(s) if s == "invariant")));
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex("foo bar_baz Point Vec3 new result");
        assert!(matches!(&tokens[0], TokenKind::Ident(s) if s == "foo"));
        assert!(matches!(&tokens[4], TokenKind::Ident(s) if s == "new"));
        assert!(matches!(&tokens[5], TokenKind::Ident(s) if s == "result"));
    }

    #[test]
    fn test_numbers() {
        let tokens = lex("42 100_000 3.14 0.5");
        assert_eq!(tokens[0], TokenKind::Int(42));
        assert_eq!(tokens[1], TokenKind::Int(100000));
        assert_eq!(tokens[2], TokenKind::Float(3.14));
        assert_eq!(tokens[3], TokenKind::Float(0.5));
    }

    #[test]
    fn test_strings() {
        let tokens = lex(r#""hello" "escaped\n\tworld""#);
        assert_eq!(tokens[0], TokenKind::Str("hello".into()));
        assert_eq!(tokens[1], TokenKind::Str("escaped\n\tworld".into()));
    }

    #[test]
    fn test_chars() {
        let tokens = lex("'a' '\\n' '\\''");
        assert_eq!(tokens[0], TokenKind::Char('a'));
        assert_eq!(tokens[1], TokenKind::Char('\n'));
        assert_eq!(tokens[2], TokenKind::Char('\''));
    }

    #[test]
    fn test_operators() {
        let tokens = lex("+ - * / % == != < > <= >= && || ! = -> =>");
        assert_eq!(tokens[0], TokenKind::Plus);
        assert_eq!(tokens[4], TokenKind::Percent);
        assert_eq!(tokens[5], TokenKind::EqEq);
        assert_eq!(tokens[6], TokenKind::Neq);
        assert_eq!(tokens[15], TokenKind::Arrow);
        assert_eq!(tokens[16], TokenKind::FatArrow);
    }

    #[test]
    fn test_punctuation() {
        let tokens = lex(". , ; : ( ) { } [ ] @ ? & |");
        assert_eq!(tokens[0], TokenKind::Dot);
        assert_eq!(tokens[4], TokenKind::LParen);
        assert_eq!(tokens[10], TokenKind::At);
        assert_eq!(tokens[11], TokenKind::Question);
    }

    #[test]
    fn test_comments_skipped() {
        let tokens = lex("let x // this is a comment\nlet y /* block */ let z");
        let idents: Vec<_> = tokens.iter().filter_map(|t| match t {
            TokenKind::Ident(s) => Some(s.clone()),
            _ => None,
        }).collect();
        assert_eq!(idents, vec!["x", "y", "z"]);
    }

    #[test]
    fn test_generic_syntax() {
        let tokens = lex("fn max[T: Comparable](a: T, b: T) -> T");
        assert!(tokens.contains(&TokenKind::Fn));
        assert!(tokens.contains(&TokenKind::LBracket));
        assert!(tokens.contains(&TokenKind::RBracket));
        assert!(tokens.contains(&TokenKind::Colon));
        // Check Comparable appears
        let has_comparable = tokens.iter().any(|t| matches!(t, TokenKind::Ident(s) if s == "Comparable"));
        assert!(has_comparable);
    }

    #[test]
    fn test_derive_syntax() {
        let tokens = lex("type Point = { x: Float64; y: Float64; } derive[Eq, Clone]");
        assert!(tokens.contains(&TokenKind::Derive));
        assert!(tokens.contains(&TokenKind::LBracket));
        // Check Eq and Clone identifiers appear after derive[
        let has_eq = tokens.iter().any(|t| matches!(t, TokenKind::Ident(s) if s == "Eq"));
        let has_clone = tokens.iter().any(|t| matches!(t, TokenKind::Ident(s) if s == "Clone"));
        assert!(has_eq);
        assert!(has_clone);
    }

    #[test]
    fn test_caret_token() {
        let tokens = lex("a ^ b");
        assert!(tokens.iter().any(|t| matches!(t, TokenKind::Caret)));
    }

    #[test]
    fn test_tilde_token() {
        let tokens = lex("~a");
        assert!(tokens.iter().any(|t| matches!(t, TokenKind::Tilde)));
    }

    #[test]
    fn test_char_escapes() {
        let tokens = lex("'\\n' '\\t' '\\r' '\\\\' '\\'' '\\\"' '\\0' '\\b' '\\f'");
        assert_eq!(tokens[0], TokenKind::Char('\n'));
        assert_eq!(tokens[1], TokenKind::Char('\t'));
        assert_eq!(tokens[2], TokenKind::Char('\r'));
        assert_eq!(tokens[3], TokenKind::Char('\\'));
        assert_eq!(tokens[4], TokenKind::Char('\''));
        assert_eq!(tokens[5], TokenKind::Char('"'));
        assert_eq!(tokens[6], TokenKind::Char('\0'));
        assert_eq!(tokens[7], TokenKind::Char('\x08'));
        assert_eq!(tokens[8], TokenKind::Char('\x0C'));
    }

    #[test]
    fn test_string_escapes() {
        let tokens = lex("\"\\n\" \"\\t\" \"\\r\" \"\\\\\" \"\\\"\" \"\\'\" \"\\0\" \"\\b\" \"\\f\"");
        assert_eq!(tokens[0], TokenKind::Str("\n".into()));
        assert_eq!(tokens[1], TokenKind::Str("\t".into()));
        assert_eq!(tokens[2], TokenKind::Str("\r".into()));
        assert_eq!(tokens[3], TokenKind::Str("\\".into()));
        assert_eq!(tokens[4], TokenKind::Str("\"".into()));
        assert_eq!(tokens[5], TokenKind::Str("'".into()));
        assert_eq!(tokens[6], TokenKind::Str("\0".into()));
        assert_eq!(tokens[7], TokenKind::Str("\x08".into()));
        assert_eq!(tokens[8], TokenKind::Str("\x0C".into()));
    }
}
