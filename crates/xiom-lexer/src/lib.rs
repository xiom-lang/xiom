// XIOM -- Lexer
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Lexer -- converts UTF-8 source to a flat token stream.
//! No whitespace significance except within string literals.
//!
//! AUDIT FIXES (readiness Stage 2):
//! - Byte-offset spans: every token carries a half-open BYTE range
//!   [byte_start, byte_end) into the source (post-BOM), alongside the
//!   1-based line/column. This unlocks precise diagnostics, LSP UTF-16
//!   math, incremental change detection and DWARF line tables downstream.
//!   Multi-byte characters advance bytes faster than columns -- both stay
//!   correct by construction.
//! - Comment/sheaang TRIVIA PRESERVATION: comments are no longer destroyed;
//!   `tokenize_with_trivia()` returns them alongside the tokens so fmt can
//!   preserve them and docgen can render doc-comments (previously lost).
//! - Numeric literals that overflow u128 and malformed float text now emit
//!   ERROR TOKENS instead of silently lexing as 0 / 0.0 (audited silent
//!   value corruption). `\xNN` escapes >= 0x80 are rejected (use `\u{...}`
//!   for non-ASCII); previously they smuggled raw bytes into `char`.

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
    Spawn, Await, Comptime, Asm, Defer,
    Move,
    Module, Use, Pub, As,
    Type, Enum, Interface, Derive, Impl,
    // requires/ensures/invariant are CONTEXTUAL keywords (5c-R: interned symbols,
    // rustc lesson). They are tokenized as regular Ident and only recognized
    // as keywords at specific parser positions -- so `var requires = 5;` works.
    True, False, Self_,
    Some, None, Ok_, Err_,
    Unsafe, Extern, Is,

    // --- Literals ---
    Ident(String),
    Int(u64),
    /// D1 (2026-08-08): integer literal that overflows u64 (fits u128/i128).
    /// Emitted when a decimal or hex literal exceeds u64::MAX so native
    /// Int128/UInt128 literals are representable end-to-end.
    BigInt(u128),
    Float(f64),
    Str(String),
    Char(char),

    // --- Operators / Punctuation ---
    Dot, Comma, Semicolon, Colon, ColonColon,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    At, Arrow, FatArrow, Question,
    Plus, Minus, Star, Slash, Percent, Caret, Tilde,
    Bang, Amp, Pipe, Ampersand,
    Eq, EqEq, Neq, Lt, Gt, Le, Ge,
    AndAnd, OrOr,
    // 8B/M9: Compound assignment operators
    PlusEq, MinusEq, StarEq, SlashEq, PercentEq,
    // 8B/M9: Range operators
    DotDot, DotDotEq,
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
// Trivia (comment/shebang preservation -- audit Stage 2)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriviaKind {
    /// `// ...` line comment (text excludes the trailing newline)
    LineComment,
    /// `/* ... */` block comment (text includes delimiters)
    BlockComment,
    /// `#!...` first-line shebang (text excludes the trailing newline)
    Shebang,
}

/// Preserved non-token source text: comments and the shebang line.
/// Downstream consumers (fmt round-trips, docgen /// rendering) opt in via
/// `tokenize_with_trivia()`; plain `tokenize()` behavior is unchanged.
#[derive(Debug, Clone, PartialEq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub text: String,
    pub span: Span,
}

// ============================================================================
// Lexer
// ============================================================================

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: u32,
    col: u32,
    /// BYTE offset into the (post-BOM) source -- advances by len_utf8().
    byte: usize,
    trivia: Vec<Trivia>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        // BUG 23 #6 fix: a UTF-8 BOM (U+FEFF) at the start of a source file must
        // be stripped -- previously it surfaced as `unexpected character` at 1:1
        // and silently broke module registration for catalog files saved with a
        // BOM (writer tools / Windows editors). All leading BOMs are dropped
        // (a double-BOM file is malformed but should still parse).
        //
        // NOTE: byte offsets in spans are relative to the POST-BOM source.
        let chars: Vec<char> = source.chars().collect();
        let mut start = 0;
        while chars.get(start) == Some(&'\u{FEFF}') { start += 1; }
        let src = &chars[start..];
        Self {
            source: src.to_vec(),
            pos: 0,
            line: 1,
            col: 1,
            byte: 0,
            trivia: Vec::new(),
        }
    }

    /// Current position as a ZERO-WIDTH span (used for EOF and errors at pos).
    fn span(&self) -> Span {
        Span::range(self.line, self.col, self.byte as u32, self.byte as u32)
    }

    /// Span from a recorded START (line/col/byte) to the CURRENT position.
    /// Every real token is built through this so its byte range covers the
    /// exact consumed text.
    fn end_span(&self, start_line: u32, start_col: u32, start_byte: usize) -> Span {
        Span::range(
            start_line,
            start_col,
            start_byte as u32,
            self.byte as u32,
        )
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
            self.byte += c.len_utf8();
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
                s.push(self.advance().expect("peek guaranteed character available"));
            } else {
                break;
            }
        }
        s
    }

    /// Parse a numeric literal suffix: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64.
    /// Returns the suffix string if found, empty string otherwise.
    fn parse_numeric_suffix(&mut self) -> String {
        let saved = self.pos;
        let prefix = match self.peek() {
            Some('i') | Some('u') => self.advance().unwrap().to_string(),
            Some('f') => { self.advance(); "f".to_string() }
            _ => return String::new(),
        };
        let width = self.advance_while(|c| c.is_ascii_digit());
        let suffix = format!("{prefix}{width}");
        let valid = matches!(suffix.as_str(), "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64");
        if valid { suffix } else { self.pos = saved; String::new() }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let (tokens, _) = self.tokenize_with_trivia();
        tokens
    }

    /// Tokenize AND preserve comments/shebang as structured trivia.
    /// The token stream itself is identical to `tokenize()`.
    pub fn tokenize_with_trivia(&mut self) -> (Vec<Token>, Vec<Trivia>) {
        // M10: Shebang support -- skip `#!/usr/bin/env xiom` on line 1.
        // The shebang line is treated as a comment for line-number preservation,
        // and is PRESERVED as trivia (fmt/docgen need it -- audited loss).
        if self.pos == 0 && self.peek() == Some('#') && self.peek_n(1) == Some('!') {
            let sl = self.line; let sc = self.col; let sb = self.byte;
            let text = self.advance_while(|c| c != '\n');
            // Advance past the newline if present
            if self.peek() == Some('\n') {
                self.advance();
            }
            // Re-sync line/col after skipping shebang (byte already advanced
            // through the newline via advance(); no manual adjustment).
            self.line = 1;
            self.col = 1;
            self.trivia.push(Trivia {
                kind: TriviaKind::Shebang,
                text,
                span: Span::range(sl, sc, sb as u32, (self.byte - 1) as u32),
            });
        }
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.is_eof();
            tokens.push(tok);
            if is_eof { break; }
        }
        let trivia = std::mem::take(&mut self.trivia);
        (tokens, trivia)
    }

    fn next_token(&mut self) -> Token {
        // Skip whitespace and comments (PRESERVED as trivia).
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => { self.advance(); }
                Some('/') if self.peek_n(1) == Some('/') => {
                    let sl = self.line; let sc = self.col; let sb = self.byte;
                    let text = self.advance_while(|c| c != '\n');
                    self.trivia.push(Trivia {
                        kind: TriviaKind::LineComment,
                        text,
                        span: Span::range(sl, sc, sb as u32, self.byte as u32),
                    });
                }
                Some('/') if self.peek_n(1) == Some('*') => {
                    let sl = self.line; let sc = self.col; let sb = self.byte;
                    self.advance(); self.advance(); // skip /*
                    let mut text = String::from("/*");
                    loop {
                        if self.peek() == Some('*') && self.peek_n(1) == Some('/') {
                            self.advance(); self.advance(); // skip */
                            text.push_str("*/");
                            break;
                        }
                        match self.advance() {
                            Some(c) => text.push(c),
                            None => {
                                return self.error_at(sl, sc, sb, "unterminated block comment");
                            }
                        }
                    }
                    self.trivia.push(Trivia {
                        kind: TriviaKind::BlockComment,
                        text,
                        span: Span::range(sl, sc, sb as u32, self.byte as u32),
                    });
                }
                _ => break,
            }
        }

        let sl = self.line; let sc = self.col; let sb = self.byte;
        let ch = match self.peek() {
            Some(c) => c,
            None => return Token::new(TokenKind::Eof, self.span(), ""),
        };

        match ch {
            // --- Identifiers and keywords ---
            c if c.is_ascii_alphabetic() || c == '_' => {
                let ident = self.advance_while(|c| c.is_ascii_alphanumeric() || c == '_');
                let kind = Self::keyword_or_ident(&ident);
                Token::new(kind, self.end_span(sl, sc, sb), ident)
            }

            // --- Numbers ---
            c if c.is_ascii_digit() => {
                // Hex literal: 0xABCD or 0XABCD
                if c == '0' && matches!(self.peek_n(1), Some(next) if next == 'x' || next == 'X') {
                    self.advance(); // consume '0'
                    self.advance(); // consume 'x' or 'X'
                    let hex = self.advance_while(|c| c.is_ascii_hexdigit() || c == '_');
                    let clean = hex.replace('_', "");
                    // D1: prefer u128 so literals beyond u64 work for Int128/UInt128.
                    // AUDIT #15 FIX: overflow beyond u128 used to silently lex
                    // as 0 (unwrap_or(0)) -- now an explicit error token.
                    let num_u128 = match u128::from_str_radix(&clean, 16) {
                        Ok(n) => n,
                        Err(_) => {
                            let lexeme = format!("0x{hex}");
                            return Token::new(
                                TokenKind::Error(format!(
                                    "hex integer literal 0x{hex} does not fit in 128 bits")),
                                self.end_span(sl, sc, sb),
                                lexeme,
                            );
                        }
                    };
                    let suffix = self.parse_numeric_suffix();
                    let lexeme = if suffix.is_empty() { format!("0x{hex}") } else { format!("0x{hex}{suffix}") };
                    if num_u128 > u64::MAX as u128 {
                        return Token::new(TokenKind::BigInt(num_u128), self.end_span(sl, sc, sb), lexeme);
                    }
                    return Token::new(TokenKind::Int(num_u128 as u64), self.end_span(sl, sc, sb), lexeme);
                }
                let int_part = self.advance_while(|c| c.is_ascii_digit() || c == '_');
                if self.peek() == Some('.') && self.peek_n(1).map_or(false, |c| c.is_ascii_digit()) {
                    self.advance(); // skip '.'
                    let frac = self.advance_while(|c| c.is_ascii_digit());
                    // Handle scientific notation exponent: e308, e-308, E+10
                    let mut exp = String::new();
                    if matches!(self.peek(), Some('e' | 'E')) {
                        exp.push(self.advance().expect("peek guaranteed character available")); // 'e' or 'E'
                        if matches!(self.peek(), Some('+' | '-')) {
                            exp.push(self.advance().expect("peek guaranteed character available"));
                        }
                        let exp_digits = self.advance_while(|c| c.is_ascii_digit());
                        exp.push_str(&exp_digits);
                    }
                    let full = format!("{int_part}.{frac}{exp}");
                    // AUDIT #15 FIX: malformed float text used to lex as 0.0.
                    let num: f64 = match full.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            return Token::new(
                                TokenKind::Error(format!("malformed float literal {full}")),
                                self.end_span(sl, sc, sb),
                                full,
                            );
                        }
                    };
                    let suffix = self.parse_numeric_suffix();
                    let lexeme = if suffix.is_empty() { full } else { format!("{full}{suffix}") };
                    Token::new(TokenKind::Float(num), self.end_span(sl, sc, sb), lexeme)
                } else {
                    // Also handle integer scientific notation: 1e10
                    if matches!(self.peek(), Some('e' | 'E')) {
                        let mut exp = String::from(&int_part);
                        exp.push(self.advance().expect("peek guaranteed character available")); // 'e' or 'E'
                        if matches!(self.peek(), Some('+' | '-')) {
                            exp.push(self.advance().expect("peek guaranteed character available"));
                        }
                        let exp_digits = self.advance_while(|c| c.is_ascii_digit());
                        exp.push_str(&exp_digits);
                        let num: f64 = match exp.parse() {
                            Ok(v) => v,
                            Err(_) => {
                                return Token::new(
                                    TokenKind::Error(format!("malformed float literal {exp}")),
                                    self.end_span(sl, sc, sb),
                                    exp,
                                );
                            }
                        };
                        let suffix = self.parse_numeric_suffix();
                        let lexeme = if suffix.is_empty() { exp } else { format!("{exp}{suffix}") };
                        return Token::new(TokenKind::Float(num), self.end_span(sl, sc, sb), lexeme);
                    }
                    let clean = int_part.replace('_', "");
                    // AUDIT #15 FIX: decimal overflow beyond u128 used to
                    // silently lex as 0 -- now an explicit error token.
                    let num_u128: u128 = match clean.parse() {
                        Ok(n) => n,
                        Err(_) => {
                            return Token::new(
                                TokenKind::Error(format!(
                                    "integer literal {int_part} does not fit in 128 bits")),
                                self.end_span(sl, sc, sb),
                                int_part.clone(),
                            );
                        }
                    };
                    let suffix = self.parse_numeric_suffix();
                    let lexeme = if suffix.is_empty() { int_part } else { format!("{int_part}{suffix}") };
                    // D1: overflow u64 -> BigInt token for native Int128 literals.
                    if num_u128 > u64::MAX as u128 {
                        return Token::new(TokenKind::BigInt(num_u128), self.end_span(sl, sc, sb), lexeme);
                    }
                    Token::new(TokenKind::Int(num_u128 as u64), self.end_span(sl, sc, sb), lexeme)
                }
            }

            // --- Strings ---
            '"' => {
                self.advance(); // skip opening "
                let mut s = String::new();
                loop {
                    match self.peek() {
                        None => return self.error_at(sl, sc, sb, "unterminated string literal"),
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
                                let val = (d1 << 4) | d2;
                                // AUDIT hygiene fix: values >= 0x80 were pushed
                                // into a `char` unchecked (raw byte smuggling).
                                // Non-ASCII needs the explicit \u{...} form.
                                if val >= 0x80 {
                                    return self.error_at(sl, sc, sb,
                                        "hex escape \\xNN must be < 0x80 -- use \\u{...} for non-ASCII");
                                }
                                s.push(val as char);
                            }
                            Some('u') => {
                                if self.advance() != Some('{') {
                                    return self.error_at(sl, sc, sb, "expected '{' after \\u");
                                }
                                let hex = self.advance_while(|c| c.is_ascii_hexdigit());
                                if self.advance() != Some('}') {
                                    return self.error_at(sl, sc, sb, "expected '}' after \\u hex digits");
                                }
                                let codepoint = u32::from_str_radix(&hex, 16).unwrap_or(0xFFFD);
                                s.push(char::from_u32(codepoint).unwrap_or('\u{FFFD}'));
                            }
                            _ => return self.error_at(sl, sc, sb, "invalid escape sequence"),
                        }
                    }
                        Some(c) => { self.advance(); s.push(c); }
                    }
                }
                Token::new(TokenKind::Str(s.clone()), self.end_span(sl, sc, sb), format!("\"{s}\""))
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
                            let val = (d1 << 4) | d2;
                            if val >= 0x80 {
                                return self.error_at(sl, sc, sb,
                                    "hex escape \\xNN must be < 0x80 -- use \\u{...} for non-ASCII");
                            }
                            val as char
                        }
                        _ => return self.error_at(sl, sc, sb, "invalid escape in char literal"),
                    },
                    Some(c) if c != '\'' => c,
                    _ => return self.error_at(sl, sc, sb, "empty char literal"),
                };
                if self.advance() != Some('\'') {
                    return self.error_at(sl, sc, sb, "unterminated char literal");
                }
                Token::new(TokenKind::Char(c), self.end_span(sl, sc, sb), format!("'{c}'"))
            }

            // --- Operators and punctuation ---
            '.' => { self.advance(); Token::new(TokenKind::Dot, self.end_span(sl, sc, sb), ".") }
            ',' => { self.advance(); Token::new(TokenKind::Comma, self.end_span(sl, sc, sb), ",") }
            ';' => { self.advance(); Token::new(TokenKind::Semicolon, self.end_span(sl, sc, sb), ";") }
            ':' => {
                self.advance();
                if self.peek() == Some(':') {
                    self.advance();
                    Token::new(TokenKind::ColonColon, self.end_span(sl, sc, sb), "::")
                } else {
                    Token::new(TokenKind::Colon, self.end_span(sl, sc, sb), ":")
                }
            }
            '(' => { self.advance(); Token::new(TokenKind::LParen, self.end_span(sl, sc, sb), "(") }
            ')' => { self.advance(); Token::new(TokenKind::RParen, self.end_span(sl, sc, sb), ")") }
            '{' => { self.advance(); Token::new(TokenKind::LBrace, self.end_span(sl, sc, sb), "{") }
            '}' => { self.advance(); Token::new(TokenKind::RBrace, self.end_span(sl, sc, sb), "}") }
            '[' => { self.advance(); Token::new(TokenKind::LBracket, self.end_span(sl, sc, sb), "[") }
            ']' => { self.advance(); Token::new(TokenKind::RBracket, self.end_span(sl, sc, sb), "]") }
            '@' => { self.advance(); Token::new(TokenKind::At, self.end_span(sl, sc, sb), "@") }
            '#' => { self.advance(); Token::new(TokenKind::Hash, self.end_span(sl, sc, sb), "#") }
            '?' => { self.advance(); Token::new(TokenKind::Question, self.end_span(sl, sc, sb), "?") }
            '+' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::PlusEq, self.end_span(sl, sc, sb), "+=")
                } else {
                    Token::new(TokenKind::Plus, self.end_span(sl, sc, sb), "+")
                }
            }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    Token::new(TokenKind::Arrow, self.end_span(sl, sc, sb), "->")
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::MinusEq, self.end_span(sl, sc, sb), "-=")
                } else {
                    Token::new(TokenKind::Minus, self.end_span(sl, sc, sb), "-")
                }
            }
            '*' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::StarEq, self.end_span(sl, sc, sb), "*=")
                } else {
                    Token::new(TokenKind::Star, self.end_span(sl, sc, sb), "*")
                }
            }
            '/' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::SlashEq, self.end_span(sl, sc, sb), "/=")
                } else {
                    Token::new(TokenKind::Slash, self.end_span(sl, sc, sb), "/")
                }
            }
            '%' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::PercentEq, self.end_span(sl, sc, sb), "%=")
                } else {
                    Token::new(TokenKind::Percent, self.end_span(sl, sc, sb), "%")
                }
            }
            '^' => { self.advance(); Token::new(TokenKind::Caret, self.end_span(sl, sc, sb), "^") }
            '~' => { self.advance(); Token::new(TokenKind::Tilde, self.end_span(sl, sc, sb), "~") }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Neq, self.end_span(sl, sc, sb), "!=")
                } else {
                    Token::new(TokenKind::Bang, self.end_span(sl, sc, sb), "!")
                }
            }
            '&' => {
                self.advance();
                if self.peek() == Some('&') {
                    self.advance();
                    Token::new(TokenKind::AndAnd, self.end_span(sl, sc, sb), "&&")
                } else {
                    Token::new(TokenKind::Ampersand, self.end_span(sl, sc, sb), "&")
                }
            }
            '|' => {
                self.advance();
                if self.peek() == Some('|') {
                    self.advance();
                    Token::new(TokenKind::OrOr, self.end_span(sl, sc, sb), "||")
                } else {
                    Token::new(TokenKind::Pipe, self.end_span(sl, sc, sb), "|")
                }
            }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::EqEq, self.end_span(sl, sc, sb), "==")
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token::new(TokenKind::FatArrow, self.end_span(sl, sc, sb), "=>")
                } else {
                    Token::new(TokenKind::Eq, self.end_span(sl, sc, sb), "=")
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Le, self.end_span(sl, sc, sb), "<=")
                } else {
                    Token::new(TokenKind::Lt, self.end_span(sl, sc, sb), "<")
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Token::new(TokenKind::Ge, self.end_span(sl, sc, sb), ">=")
                } else {
                    Token::new(TokenKind::Gt, self.end_span(sl, sc, sb), ">")
                }
            }

            _ => {
                self.advance();
                Token::new(TokenKind::Error(format!("unexpected character: '{ch}'")), self.end_span(sl, sc, sb), ch.to_string())
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
            "asm"       => TokenKind::Asm,
            "defer"     => TokenKind::Defer,
            "move"      => TokenKind::Move,
            "module"    => TokenKind::Module,
            "use"       => TokenKind::Use,
            "and"       => TokenKind::AndAnd,   // 8B/M9: boolean operator keyword aliases
            "or"        => TokenKind::OrOr,
            "not"       => TokenKind::Bang,
            "pub"       => TokenKind::Pub,
            "as"        => TokenKind::As,
            "type"      => TokenKind::Type,
            "enum"      => TokenKind::Enum,
            "interface" => TokenKind::Interface,
            "derive"    => TokenKind::Derive,
            "impl"      => TokenKind::Impl,
            // requires/ensures/invariant -- contextuel keywords (5c-R).
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

    /// Error token whose span covers everything consumed since the recorded
    /// start (so diagnostics underline the whole malformed literal).
    fn error_at(&self, sl: u32, sc: u32, sb: usize, msg: impl Into<String>) -> Token {
        Token::new(TokenKind::Error(msg.into()), self.end_span(sl, sc, sb), "")
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
    fn test_utf8_bom_stripped() {
        // BUG 23 #6: a leading UTF-8 BOM must be stripped (module files saved
        // by Windows editors/tools previously failed with "unexpected character").
        let tokens = lex("\u{FEFF}module foo\nfn main() -> Int { return 0; }");
        assert!(matches!(&tokens[0], TokenKind::Module));
        // Double BOM (mangled writer output) is also tolerated.
        let tokens2 = lex("\u{FEFF}\u{FEFF}module foo");
        assert!(matches!(&tokens2[0], TokenKind::Module));
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

    // =====================================================================
    // Byte-offset spans (readiness Stage 2)
    // =====================================================================

    #[test]
    fn test_byte_offsets_ascii() {
        let src = "let x = 42;";
        let toks = Lexer::new(src).tokenize();
        // "let" [0,3), "x" [4,5), "=" [6,7), "42" [8,10), ";" [10,11)
        assert_eq!(toks[0].span.byte_range(), Some((0, 3)));
        assert_eq!(toks[1].span.byte_range(), Some((4, 5)));
        assert_eq!(toks[2].span.byte_range(), Some((6, 7)));
        assert_eq!(toks[3].span.byte_range(), Some((8, 10)));
        assert_eq!(toks[4].span.byte_range(), Some((10, 11)));
        // Lexemes agree with the ranges.
        for t in &toks {
            if let Some((s, e)) = t.span.byte_range() {
                assert_eq!(&src[s as usize..e as usize], t.lexeme, "range must cover the lexeme");
            }
        }
    }

    #[test]
    fn test_byte_offsets_multibyte() {
        // Multi-byte chars: columns count CHARS, bytes count BYTES.
        let src = "var s = \"h\u{00E9}\"; var n = 1;"; // é is 2 bytes
        let toks = Lexer::new(src).tokenize();
        // The string token starts at byte 8 ("var s = "), and its lexeme
        // "\"h\u{00E9}\"" is 4 chars but 5 bytes -> range (8, 13).
        let stok = toks.iter().find(|t| matches!(t.kind, TokenKind::Str(_))).expect("string");
        let (ss, se) = stok.span.byte_range().unwrap();
        assert_eq!((ss, se), (8, 13), "string byte length must include the 2-byte e-acute");
        assert_eq!(stok.lexeme.chars().count(), 4);
        assert_eq!(&src[ss as usize..se as usize], stok.lexeme);
        // The first semicolon sits right after the string at byte 13.
        let semi = &toks[toks.iter().position(|t| t.kind == TokenKind::Semicolon).unwrap()];
        assert_eq!(semi.span.byte_range(), Some((13, 14)));
    }

    #[test]
    fn test_offset_lines_track_newlines() {
        let src = "let a = 1;\nlet b = 2;";
        let toks = Lexer::new(src).tokenize();
        let second_let = &toks[5];
        assert_eq!(second_let.span.line, 2);
        assert_eq!(second_let.span.col, 1);
        assert_eq!(second_let.span.byte_range(), Some((11, 14)));
    }

    // =====================================================================
    // Trivia preservation (audit Stage 2)
    // =====================================================================

    #[test]
    fn test_trivia_preserved() {
        let src = "// leading\nlet x; /* mid */ fn f() {}";
        let (_, trivia) = Lexer::new(src).tokenize_with_trivia();
        assert_eq!(trivia.len(), 2);
        assert_eq!(trivia[0].kind, TriviaKind::LineComment);
        assert_eq!(trivia[0].text, "// leading");
        assert_eq!(trivia[0].span.line, 1);
        assert_eq!(trivia[1].kind, TriviaKind::BlockComment);
        assert_eq!(trivia[1].text, "/* mid */");
        // Ranges align with the source text exactly.
        for t in &trivia {
            let (s, e) = t.span.byte_range().unwrap();
            assert_eq!(&src[s as usize..e as usize], t.text);
        }
    }

    #[test]
    fn test_shebang_preserved_as_trivia() {
        let src = "#!/usr/bin/env xiom\nfn main() {}";
        let (_, trivia) = Lexer::new(src).tokenize_with_trivia();
        assert_eq!(trivia.len(), 1);
        assert_eq!(trivia[0].kind, TriviaKind::Shebang);
        assert_eq!(trivia[0].text, "#!/usr/bin/env xiom");
        let (s, e) = trivia[0].span.byte_range().unwrap();
        assert_eq!(&src[s as usize..e as usize], trivia[0].text);
    }

    #[test]
    fn test_plain_tokenize_still_works_with_comments() {
        // tokenize() behavior unchanged: same token kinds with or without comments.
        let with_c = lex("let x /* c */ = 1;");
        let without_c = lex("let x = 1;");
        assert_eq!(with_c, without_c);
    }

    // =====================================================================
    // AUDIT #15 -- numeric overflow and escape honesty
    // =====================================================================

    #[test]
    fn test_decimal_overflow_beyond_u128_is_error_not_zero() {
        let toks = Lexer::new("340282366920938463463374607431768211456").tokenize(); // 2^128 > u128::MAX
        assert!(matches!(&toks[0].kind, TokenKind::Error(m) if m.contains("does not fit")),
            "overflow must be an error token, got {:?}", toks[0].kind);
    }

    #[test]
    fn test_hex_overflow_beyond_u128_is_error_not_zero() {
        // 33 hex digits = >128 bits.
        let toks = Lexer::new("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF").tokenize();
        assert!(matches!(&toks[0].kind, TokenKind::Error(m) if m.contains("does not fit")),
            "got {:?}", toks[0].kind);
    }

    #[test]
    fn test_u128_max_boundary_still_lexes() {
        let toks = Lexer::new("340282366920938463463374607431768211455").tokenize(); // u128::MAX
        assert!(matches!(&toks[0].kind, TokenKind::BigInt(_)), "got {:?}", toks[0].kind);
    }

    #[test]
    fn test_hex_escape_above_7f_rejected() {
        let toks = lex("\"\\x80\"");
        assert!(matches!(&toks[0], TokenKind::Error(m) if m.contains("\\xNN")),
            "\\x80 must be rejected (use \\u{{...}}), got {:?}", toks[0]);
        let ctoks = lex("'\\xFF'");
        assert!(matches!(&ctoks[0], TokenKind::Error(_)));
    }

    #[test]
    fn test_ascii_hex_escape_still_works() {
        let tokens = lex("\"\\x41\\x00\\x7F\"");
        assert_eq!(tokens[0], TokenKind::Str("A\u{0}\u{7F}".into()));
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

    /// 8B/M5: Fuzz harness -- feed random bytes to lexer, verify no panics.
    /// Uses a simple LCG for deterministic randomness.
    #[test]
    fn fuzz_lexer_random_input() {
        let mut seed: u64 = 12345;
        for _ in 0..500 {
            let len = ((seed >> 32) % 256) as usize;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let mut input = String::with_capacity(len);
            for _ in 0..len {
                let byte = (seed % 256) as u8;
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                input.push(byte as char);
            }
            // Must not panic on any random input
            let mut lexer = Lexer::new(&input);
            let _ = lexer.tokenize();
        }
    }

    /// 8B/M5: Fuzz harness -- edge cases (unterminated strings, nested comments, binary)
    #[test]
    fn fuzz_lexer_edge_cases() {
        let edge_cases = vec![
            "\"", // unterminated string
            "\"\\", // unterminated escape
            "'", // unterminated char
            "'\\", // unterminated char escape
            "/*", // unterminated block comment
            "/*/", // tricky comment
            "/**/", // empty block comment
            "0x", // incomplete hex
            "0b", // incomplete binary
            ".", // lone dot
            "..", // double dot
            "...", // triple dot
            "1.", // trailing dot
            "1e", // incomplete exponent
            "//\n", // line comment
            "\"\\x", // incomplete hex escape
            "\"\\u{", // incomplete unicode escape
            "\"\\u{}\"", // empty unicode escape
            "\"\\u{FFFFFFFF}\"", // overflow unicode escape
            "fn let var if else match while for", // keyword sequence
            "0__123_456__", // weird number underscores
            "_valid_ident", // underscore-prefixed ident
            "0xDEAD_BEEF", // hex with underscore
        ];
        for case in &edge_cases {
            let mut lexer = Lexer::new(case);
            let _ = lexer.tokenize(); // must not panic
        }
    }
}
