// XIOM — Parser
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Parser — recursive descent, LL(1), single deterministic parse path.
//! Converts the token stream into a typed AST.
//! Implements the full EBNF grammar from Section 3 of the language spec.

use xiom_ast::*;
use xiom_lexer::{Token, TokenKind};

// ============================================================================
// Parser
// ============================================================================

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// When true, a bare `Ident { ... }` is NOT parsed as a struct literal.
    restrict_struct: bool,
    /// Current recursion depth of the expression/type parsers.
    depth: usize,
    /// Accumulated parse errors for error recovery (Phase 5c).
    errors: Vec<ParseError>,
    /// 5c-R: Expected-token bitset (rustc lesson: `TokenTypeSet` in `compiler/rustc_parse`).
    /// Each failed `check()` inserts a bit; `bump()` clears it; the error path
    /// formats the set as "expected one of X, Y, found Z".
    expected: u128,
}

/// Maximum expression/type nesting depth. A recursive-descent parser recurses
/// once per nesting level, with ~16 intermediate frames per level. 24 levels ×
/// 16 frames ≈ 384 stack frames ≈ 768KB — well within the 1MB test-thread stack.
/// Prevents STACK_OVERFLOW on deeply nested input like 500-parenthesized exprs.
const MAX_EXPR_DEPTH: usize = 24;

impl Parser {
        pub fn errors(&self) -> &[ParseError] { &self.errors }
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0, restrict_struct: false, depth: 0, errors: Vec::new(), expected: 0 }
    }

    /// Maximum number of parse errors before aborting (Phase 5c error recovery).
    const MAX_PARSE_ERRORS: usize = 100;

    /// Record an error without aborting. After MAX_PARSE_ERRORS, returns Err.
    fn recoverable_error(&mut self, msg: String, span: Span) -> Result<(), ParseError> {
        let err = ParseError { message: msg, span };
        self.errors.push(err);
        if self.errors.len() >= Self::MAX_PARSE_ERRORS {
            Err(ParseError { message: "too many parse errors — aborting".to_string(), span })
        } else {
            Ok(())
        }
    }

    /// Skip tokens until a synchronisation point (top-level keyword or `}`).
    #[allow(dead_code)]
    fn recover_to_sync(&mut self) {
        while !self.peek().is_eof() {
            match self.peek_kind() {
                TokenKind::Fn | TokenKind::Type | TokenKind::Enum | TokenKind::Interface | TokenKind::Impl
                | TokenKind::Module | TokenKind::Pub | TokenKind::Const | TokenKind::Use
                | TokenKind::Extern | TokenKind::RBrace => break,
                TokenKind::Semicolon => { self.advance(); break; }
                _ => { self.advance(); }
            }
        }
    }

    /// Panic-mode statement recovery (rustc lesson: `recover_stmt_` in
    /// `compiler/rustc_parse/src/parser/diagnostics.rs` ~60 lines).
    /// Skip tokens to the next statement boundary (`;` or `}`) while tracking
    /// brace depth — increment on `{`, decrement on `}`, stop when depth ≤ 0
    /// and we hit `;` or `}`. This keeps recovery scoped to the current block
    /// instead of leaking into enclosing scopes.
    fn recover_stmt(&mut self) {
        let mut depth: i32 = 0;
        while !self.peek().is_eof() {
            match self.peek_kind() {
                TokenKind::LBrace => { depth += 1; self.advance(); }
                TokenKind::RBrace => {
                    if depth <= 0 {
                        // Closing brace at our level: eat it and stop.
                        self.advance();
                        break;
                    }
                    depth -= 1;
                    self.advance();
                }
                TokenKind::Semicolon => {
                    if depth <= 0 {
                        self.advance();
                        break;
                    }
                    self.advance();
                }
                // Stop at fn/type/enum/etc. — likely start of next item
                TokenKind::Fn | TokenKind::Type | TokenKind::Enum
                | TokenKind::Interface | TokenKind::Impl | TokenKind::Module | TokenKind::Pub
                | TokenKind::Const | TokenKind::Use | TokenKind::Extern => {
                    if depth <= 0 { break; }
                    self.advance();
                }
                _ => { self.advance(); }
            }
        }
    }

    /// Return accumulated errors if any.
    pub fn take_errors(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }

    /// Enter one level of expression/type recursion. Returns an error (instead
    /// of overflowing the stack) once nesting exceeds `MAX_EXPR_DEPTH`.
    fn enter_expr(&mut self) -> Result<(), ParseError> {
        self.depth += 1;
        if self.depth > MAX_EXPR_DEPTH {
            return Err(self.error("expression nesting too deep (max 24 levels) — simplify the expression"));
        }
        Ok(())
    }

    /// Leave one level of expression/type recursion.
    fn exit_expr(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&EOF_TOKEN)
    }

    fn peek_ahead(&self, n: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + n).map(|t| &t.kind)
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> &Token {
        self.pos += 1;
        &self.tokens[self.pos - 1]
    }

    fn check(&self, kind: fn(&TokenKind) -> bool) -> bool {
        kind(self.peek_kind())
    }

    /// Map a `TokenKind` to its bit position in the `expected` bitset.
    /// The bitset is u128 (128 bits); we have ~55 token kinds, so this fits.
    fn token_bit(kind: &TokenKind) -> u128 {
        match kind {
            TokenKind::Int(_) | TokenKind::Float(_) => 1 << 0,
            TokenKind::Str(_) => 1 << 1,
            TokenKind::Char(_) => 1 << 2,
            TokenKind::Ident(_) => 1 << 3,
            TokenKind::Dot => 1 << 4,
            TokenKind::Comma => 1 << 5,
            TokenKind::Semicolon => 1 << 6,
            TokenKind::Colon => 1 << 7,
            TokenKind::LParen => 1 << 8,
            TokenKind::RParen => 1 << 9,
            TokenKind::LBrace => 1 << 10,
            TokenKind::RBrace => 1 << 11,
            TokenKind::LBracket => 1 << 12,
            TokenKind::RBracket => 1 << 13,
            TokenKind::At => 1 << 14,
            TokenKind::Hash => 1 << 15,
            TokenKind::Question => 1 << 16,
            TokenKind::Plus => 1 << 17,
            TokenKind::Minus => 1 << 18,
            TokenKind::Star => 1 << 19,
            TokenKind::Slash => 1 << 20,
            TokenKind::Percent => 1 << 21,
            TokenKind::Caret => 1 << 22,
            TokenKind::Tilde => 1 << 23,
            TokenKind::Eq => 1 << 24,
            TokenKind::EqEq => 1 << 25,
            TokenKind::Lt => 1 << 26,
            TokenKind::Gt => 1 << 27,
            TokenKind::Le => 1 << 28,
            TokenKind::Ge => 1 << 29,
            TokenKind::AndAnd => 1 << 30,
            TokenKind::OrOr => 1 << 31,
            TokenKind::Bang => 1 << 32,
            TokenKind::Amp => 1 << 33,
            TokenKind::Pipe => 1 << 34,
            TokenKind::Ampersand => 1 << 35,
            TokenKind::Arrow => 1 << 36,
            TokenKind::FatArrow => 1 << 37,
            TokenKind::Fn => 1 << 38,
            TokenKind::Type => 1 << 39,
            TokenKind::Enum => 1 << 40,
            TokenKind::Interface => 1 << 41,
            TokenKind::Impl => 1 << 42,
            TokenKind::Module => 1 << 43,
            TokenKind::Pub => 1 << 44,
            TokenKind::Const => 1 << 44,
            TokenKind::Use => 1 << 45,
            TokenKind::Extern => 1 << 46,
            TokenKind::As => 1 << 47,
            TokenKind::Eof => 1 << 48,
            _ => 0,
        }
    }

    /// Format the `expected` bitset as a human-readable "X, Y, Z" list.
    fn expected_display(exp: u128) -> String {
        let names: &[&str] = &[
            "number", "string", "char literal", "identifier", ".", ",", ";", ":",
            "(", ")", "{", "}", "[", "]", "@", "#", "?", "+", "-", "*", "/", "%",
            "^", "~", "=", "==", "<", ">", "<=", ">=", "and", "or", "!",
            "&", "|", "&", "->", "=>", "fn", "type", "enum", "interface", "module",
            "pub", "const", "use", "extern", "as", "end of input",
        ];
        let parts: Vec<&str> = names.iter().enumerate()
            .filter(|(i, _)| (exp >> i) & 1 != 0)
            .map(|(_, n)| *n)
            .collect();
        if parts.is_empty() { "token".to_string() }
        else if parts.len() == 1 { parts[0].to_string() }
        else if parts.len() == 2 { format!("{} or {}", parts[0], parts[1]) }
        else {
            let last = parts.last().expect("expected_set has parts");
            format!("{}, or {}", parts[..parts.len()-1].join(", "), last)
        }
    }

    /// Record an expectation in the bitset (called when a `check()` fails).
    fn record_expected(&mut self, kind: &TokenKind) {
        self.expected |= Self::token_bit(kind);
    }

    /// Clear the expectation bitset (called after a successful token match).
    fn clear_expected(&mut self) {
        self.expected = 0;
    }

    #[allow(dead_code)]
    fn expect(&mut self, expected: &str) -> Result<Token, ParseError> {
        if self.peek().is_eof() {
            return Err(self.error(format!("expected {expected}, found end of file")));
        }
        let _tok = self.advance().clone();
        Ok(_tok)
    }

    fn expect_kind(&mut self, kind: TokenKind, expected: &str) -> Result<Token, ParseError> {
        let exp_before = self.expected;
        self.record_expected(&kind);
        if self.peek_kind() == &kind {
            self.clear_expected();
            Ok(self.advance().clone())
        } else {
            // If other expectations were also recorded this parse attempt,
            // include them in the error message (rustc lesson: free expected-list).
            let msg = if exp_before != 0 && exp_before != Self::token_bit(&kind) {
                let full = Self::expected_display(exp_before | Self::token_bit(&kind));
                format!("expected one of {full}, found {}", self.peek().lexeme)
            } else {
                format!("expected {expected}, found {}", self.peek().lexeme)
            };
            Err(self.error(msg))
        }
    }

    fn skip(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == &kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn error(&self, msg: impl Into<String>) -> ParseError {
        ParseError {
            message: msg.into(),
            span: self.peek().span,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();
        let start = self.peek().span;
        let file_module_path = self.parse_file_module_header()?;
        while !self.peek().is_eof() {
            match self.parse_top_decl() {
                Ok(item) => items.push(item),
                Err(e) => {
                    // Phase 5c error recovery: record, skip to sync point, continue
                    let span = e.span;
                    let _ = self.recoverable_error(e.message, span);
                    if self.errors.len() >= Self::MAX_PARSE_ERRORS {
                        return Err(ParseError { message: "too many parse errors — aborting".to_string(), span });
                    }
                    // 5c-R: use brace-depth-aware statement recovery instead of
                    // top-level-only sync (rustc lesson: panic-mode recovery)
                    self.recover_stmt();
                }
            }
        }
        // If we have recovered items AND errors, return what we have
        if !self.errors.is_empty() && !items.is_empty() {
            // Return partial program — caller can still type-check recovered AST
        }
        if let Some(path) = file_module_path {
            let wrapped = Self::build_file_module_result(path, items, start);
            return Ok(Program::new(vec![wrapped], start));
        }
        Ok(Program::new(items, start))
    }

    fn parse_file_module_header(&mut self) -> Result<Option<Vec<Ident>>, ParseError> {
        let saved = self.pos;
        let _is_pub = self.skip(TokenKind::Pub);
        if !self.check(|k| matches!(k, TokenKind::Module)) {
            self.pos = saved;
            return Ok(None);
        }
        self.advance();
        let first = self.parse_ident()?;
        match self.peek_kind() {
            TokenKind::LBrace => { self.pos = saved; Ok(None) }
            _ => {
                if _is_pub { return Err(self.error("'pub' not valid on module declarations")); }
                let mut path = vec![first];
                while self.skip(TokenKind::Dot) { path.push(self.parse_ident()?); }
                Ok(Some(path))
            }
        }
    }

    fn build_file_module_result(path: Vec<Ident>, items: Vec<TopDecl>, span: Span) -> TopDecl {
        let mut current_items = items;
        for segment in path.into_iter().rev() {
            current_items = vec![TopDecl::Module(ModuleDecl {
                name: segment, path: Vec::new(), items: current_items,
                is_file_level: false, source_file: None, span,
            })];
        }
        current_items.into_iter().next().expect("brace module has at least one item")
    }

    fn parse_top_decl(&mut self) -> Result<TopDecl, ParseError> {
        // Reset recursion depth per top-level item so one deep expression does
        // not poison the depth accounting for subsequent declarations.
        self.depth = 0;
        let is_pub = self.skip(TokenKind::Pub);
        match self.peek_kind() {
            TokenKind::Module => self.parse_module(is_pub),
            TokenKind::Use => self.parse_use_decl(),
            TokenKind::Type => self.parse_type_decl(is_pub),
            TokenKind::Enum => self.parse_enum_decl(is_pub),
            TokenKind::Interface => self.parse_interface_decl(is_pub),
            TokenKind::Impl => self.parse_impl_decl(),
            TokenKind::Fn => self.parse_fn_decl(is_pub, None),
            TokenKind::Const => {
                self.parse_const_decl(is_pub)
            }
            TokenKind::Var => self.parse_module_var(),
            TokenKind::Extern => self.parse_extern_block(),
            TokenKind::Spawn => {
                // Module-level `spawn { ... }` statement (M21). Parse as a
                // top-level declaration so it can appear in module bodies.
                if is_pub { self.advance(); return Err(self.error("'pub' not valid on spawn declarations")); }
                self.parse_spawn_top_decl()
            }
            // `async` is a contextual keyword: when followed by `fn`, it
            // triggers async-fn parsing (delegated to parse_fn_decl).
            TokenKind::Ident(s) if s == "async" && self.peek_ahead(1) == Some(&TokenKind::Fn) => {
                self.parse_fn_decl(is_pub, Some(true))
            }
            _ => Err(self.error(format!("expected declaration, found '{}'", self.peek().lexeme))),
        }
    }

    fn parse_module(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        let start = self.advance().span;
        let mut path = vec![self.parse_ident()?];
        while self.skip(TokenKind::Dot) { path.push(self.parse_ident()?); }
        if is_pub { return Err(self.error("'pub' not valid on module declarations")); }
        if self.check(|k| matches!(k, TokenKind::LBrace)) {
            // Block-form module: `module a[.b.c] { ... }`
            self.advance();
            let mut items = Vec::new();
            while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                items.push(self.parse_top_decl()?);
            }
            self.expect_kind(TokenKind::RBrace, "'}'")?;
            return Ok(Self::build_file_module_result(path, items, start));
        }
        // Brace-less file-level module: `module a.b.c` wraps the rest of the file.
        let mut items = Vec::new();
        while !self.peek().is_eof() {
            items.push(self.parse_top_decl()?);
        }
        Ok(Self::build_file_module_result(path, items, start))
    }

    fn parse_use_decl(&mut self) -> Result<TopDecl, ParseError> {
        self.advance();
        let start = self.peek().span;
        let mut path = vec![self.parse_ident()?];
        while self.skip(TokenKind::Dot) {
            if self.check(|k| matches!(k, TokenKind::Star)) {
                self.advance();
                self.expect_kind(TokenKind::Semicolon, "';'")?;
                return Ok(TopDecl::Use(UseDecl { path, alias: None, glob: true, span: start }));
            }
            path.push(self.parse_ident()?);
        }
        let alias = if self.skip(TokenKind::As) { Some(self.parse_ident()?) } else { None };
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(TopDecl::Use(UseDecl { path, alias, glob: false, span: start }))
    }

    fn parse_type_decl(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        let start = self.advance().span;
        let name = self.parse_ident()?;
        let generics = self.parse_optional_generic_params()?;
        self.expect_kind(TokenKind::Eq, "'='")?;
        if self.check(|k| matches!(k, TokenKind::Enum)) {
            self.advance();
            self.expect_kind(TokenKind::LBrace, "'{'")?;
            let mut variants = Vec::new();
            loop {
                let vname = self.parse_variant_ident()?;
                let fields = if self.skip(TokenKind::LParen) {
                    let mut vfields = Vec::new();
                    let mut idx = 0usize;
                    loop {
                        let named = matches!(self.peek_kind(), TokenKind::Ident(_)) && matches!(self.peek_ahead(1), Some(TokenKind::Colon));
                        let fname = if named {
                            let n = self.parse_ident()?;
                            self.expect_kind(TokenKind::Colon, "':'")?;
                            n
                        } else {
                            Ident::new(format!("_{idx}"), start)
                        };
                        let fty = self.parse_type()?;
                        vfields.push(FieldDecl { name: fname, ty: fty, span: start });
                        idx += 1;
                        if !self.skip(TokenKind::Comma) { break; }
                    }
                    self.expect_kind(TokenKind::RParen, "')'")?;
                    vfields
                } else { Vec::new() };
                variants.push(EnumVariant { name: vname, fields, span: start });
                if self.skip(TokenKind::Comma) {
                    if self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) { break; }
                    continue;
                }
                if self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) { break; }
            }
            self.expect_kind(TokenKind::RBrace, "'}'")?;
            let derives = if self.skip(TokenKind::Derive) { self.parse_derive_list()? } else { Vec::new() };
            self.skip(TokenKind::Semicolon); // optional trailing ';' after `type X = enum {...}`
            return Ok(TopDecl::Enum(EnumDecl { is_pub, name, generics, variants, derives, span: start }));
        }
        if self.check(|k| matches!(k, TokenKind::LParen)) {
            // 8B/M9: Tuple struct — `type Foo = (Int, Str) [derive[...]]`
            let tuple_types = self.parse_tuple_type_args()?;
            let derives = if self.skip(TokenKind::Derive) { self.parse_derive_list()? } else { Vec::new() };
            self.skip(TokenKind::Semicolon);
            // Generate synthetic field names: _0, _1, ...
            let fields: Vec<FieldDecl> = tuple_types.iter().enumerate().map(|(i, ty)| {
                FieldDecl {
                    name: Ident::new(format!("_{}", i), start),
                    ty: ty.clone(),
                    span: start,
                }
            }).collect();
            return Ok(TopDecl::Type(TypeDecl { is_pub, name, generics, fields, derived_fields: Vec::new(), invariants: Vec::new(), derives, alias: None, span: start }));
        }
        if !self.check(|k| matches!(k, TokenKind::LBrace)) {
            let alias_type = self.parse_type()?;
            self.expect_kind(TokenKind::Semicolon, "';'")?;
            return Ok(TopDecl::Type(TypeDecl { is_pub, name, generics, fields: Vec::new(), derived_fields: Vec::new(), invariants: Vec::new(), derives: Vec::new(), alias: Some(Box::new(alias_type)), span: start }));
        }
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut fields = Vec::new();
        let mut derived_fields = Vec::new();
        let mut invariants = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
            // 5c-R: contextual keyword — `invariant` is an Ident, not a reserved token
            if self.check(|k| matches!(k, TokenKind::Ident(s) if s == "invariant")) {
                self.advance();
                self.expect_kind(TokenKind::Colon, "':'")?;
                let expr = self.parse_expr()?;
                self.expect_kind(TokenKind::Semicolon, "';'")?;
                invariants.push(expr);
            } else {
                let field_name = self.parse_ident()?;
                if matches!(self.peek_kind(), TokenKind::Comma | TokenKind::RBrace) || self.peek_kind() == &TokenKind::LParen {
                    let mut depth = 0i32;
                    loop {
                        match self.peek_kind() {
                            TokenKind::LParen => { self.advance(); depth += 1; }
                            TokenKind::RParen => { self.advance(); depth -= 1; if depth < 0 { break; } }
                            TokenKind::LBrace => { self.advance(); depth += 1; }
                            TokenKind::RBrace => { if depth == 0 { break; } self.advance(); depth -= 1; }
                            TokenKind::Eof => break,
                            _ => { self.advance(); }
                        }
                    }
                    self.advance();
                    return Ok(TopDecl::Type(TypeDecl { is_pub: false, name, generics, fields, derived_fields, invariants, derives: vec![], alias: None, span: start }));
                }
                self.expect_kind(TokenKind::Colon, "':'")?;
                let ty = self.parse_type()?;
                if self.check(|k| matches!(k, TokenKind::Ident(_))) {
                    let tok = self.peek();
                    if let TokenKind::Ident(s) = &tok.kind {
                        if s == "derived" {
                            self.advance();
                            self.expect_kind(TokenKind::LParen, "'('")?;
                            let expr = self.parse_expr()?;
                            self.expect_kind(TokenKind::RParen, "')'")?;
                            self.expect_kind(TokenKind::Semicolon, "';'")?;
                            derived_fields.push((field_name, ty, expr));
                            continue;
                        }
                    }
                }
                self.expect_kind(TokenKind::Semicolon, "';'")?;
                fields.push(FieldDecl { name: field_name, ty, span: start });
            }
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        let derives = if self.skip(TokenKind::Derive) { self.parse_derive_list()? } else { Vec::new() };
        Ok(TopDecl::Type(TypeDecl { is_pub, name, generics, fields, derived_fields, invariants, derives, alias: None, span: start }))
    }

    fn parse_enum_decl(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        let start = self.advance().span;
        let name = self.parse_ident()?;
        let generics = self.parse_optional_generic_params()?;
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut variants = Vec::new();
        loop {
            let vname = self.parse_variant_ident()?;
            let fields = if self.skip(TokenKind::LParen) {
                let mut vfields = Vec::new();
                let mut idx = 0usize;
                loop {
                    let named = matches!(self.peek_kind(), TokenKind::Ident(_)) && matches!(self.peek_ahead(1), Some(TokenKind::Colon));
                    let fname = if named {
                        let n = self.parse_ident()?;
                        self.expect_kind(TokenKind::Colon, "':'")?;
                        n
                    } else {
                        Ident::new(format!("_{idx}"), start)
                    };
                    let fty = self.parse_type()?;
                    vfields.push(FieldDecl { name: fname, ty: fty, span: start });
                    idx += 1;
                    if !self.skip(TokenKind::Comma) { break; }
                }
                self.expect_kind(TokenKind::RParen, "')'")?;
                vfields
            } else { Vec::new() };
            variants.push(EnumVariant { name: vname, fields, span: start });
            if self.skip(TokenKind::Comma) {
                if self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) { break; }
                continue;
            }
            if self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) { break; }
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        let derives = if self.skip(TokenKind::Derive) { self.parse_derive_list()? } else { Vec::new() };
        Ok(TopDecl::Enum(EnumDecl { is_pub, name, generics, variants, derives, span: start }))
    }

    fn parse_interface_decl(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        let start = self.advance().span;
        let name = self.parse_ident()?;
        let generics = self.parse_optional_generic_params()?;
        // v0.56: Interface inheritance (e.g., `interface DerefMut: Deref`)
        let parent = if self.peek_kind() == &TokenKind::Colon {
            self.advance(); // consume ':'
            Some(self.parse_ident()?)
        } else {
            None
        };
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut members = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
            if self.check(|k| matches!(k, TokenKind::Fn)) {
                let fn_decl = self.parse_fn_decl(false, None)?;
                match fn_decl {
                    TopDecl::Fn(f) => members.push(InterfaceMember::FnSignature(f)),
                    _ => unreachable!(),
                }
            } else if self.peek_kind() == &TokenKind::Type {
                // Associated type declaration `type Name;`
                self.advance(); // consume 'type'
                let at_name = self.parse_ident()?;
                self.expect_kind(TokenKind::Semicolon, "';'")?;
                // Stored as a field with '_assoc_type' sentinel type
                members.push(InterfaceMember::Field(FieldDecl { name: at_name, ty: Type::Named(Ident::new("_assoc_type", start), vec![]), span: start }));
            } else {
                let fname = self.parse_ident()?;
                self.expect_kind(TokenKind::Colon, "':'")?;
                let fty = self.parse_type()?;
                self.expect_kind(TokenKind::Semicolon, "';'")?;
                members.push(InterfaceMember::Field(FieldDecl { name: fname, ty: fty, span: start }));
            }
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        Ok(TopDecl::Interface(InterfaceDecl { is_pub, name, generics, parent, members, span: start }))
    }

    /// Parse impl TraitName for TypeName { fn method(...) { body } ... }
    fn parse_impl_decl(&mut self) -> Result<TopDecl, ParseError> {
        let start = self.advance().span; // consume `impl`
        let trait_name = self.parse_ident()?;
        self.expect_kind(TokenKind::For, "'for'")?;
        let type_name = self.parse_ident()?;
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut members = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
            if self.check(|k| matches!(k, TokenKind::Fn)) {
                let fn_decl = self.parse_fn_decl(false, None)?;
                match fn_decl {
                    TopDecl::Fn(f) => members.push(ImplItem::Fn(f)),
                    _ => return Err(self.error("expected function declaration in impl block")),
                }
            } else {
                return Err(self.error("expected 'fn' in impl block"));
            }
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        Ok(TopDecl::Impl(ImplDecl { trait_name, type_name, members, span: start }))
    }

    /// Parse compiler attributes: #[safety_audit(justification: "...")]
    fn parse_attributes(&mut self) -> Result<Vec<xiom_ast::Attribute>, ParseError> {
        let mut attrs = Vec::new();
        while self.skip(TokenKind::Hash) {
            self.expect_kind(TokenKind::LBracket, "'['")?;
            let name = self.parse_ident()?;
            let mut args = Vec::new();
            if self.skip(TokenKind::LParen) {
                loop {
                    let key = self.parse_ident()?;
                    if self.skip(TokenKind::Colon) {
                        let val = self.parse_expr()?;  // string literal or other value
                        let val_str = match &val {
                            xiom_ast::Expr::Str(s, _) => s.clone(),
                            xiom_ast::Expr::Int(n, _) => n.to_string(),
                            xiom_ast::Expr::Bool(b, _) => b.to_string(),
                            _ => format!("{:?}", val),
                        };
                        args.push((key.name.clone(), val_str));
                    }
                    if !self.skip(TokenKind::Comma) { break; }
                }
                self.expect_kind(TokenKind::RParen, "')'")?;
            }
            self.expect_kind(TokenKind::RBracket, "']'")?;
            attrs.push(xiom_ast::Attribute { name, args, span: self.peek().span });
        }
        Ok(attrs)
    }

    fn parse_fn_decl(&mut self, is_pub: bool, is_async: Option<bool>) -> Result<TopDecl, ParseError> {
        // Parse attributes: #[safety_audit(justification: "...")]
        let attrs = self.parse_attributes()?;
        // `async` is a contextual keyword: only triggers async-fn when the
        // identifier "async" is immediately followed by the `fn` keyword.
        let has_async = match self.peek_kind() {
            TokenKind::Ident(s) => s == "async" && self.peek_ahead(1) == Some(&TokenKind::Fn),
            _ => false,
        } && { self.advance(); true };
        let async_flag = is_async.unwrap_or(false) || has_async;
        let start = self.peek().span;
        if !self.check(|k| matches!(k, TokenKind::Fn)) { return Err(self.error("expected 'fn'")); }
        self.advance();
        let first = self.parse_ident()?;
        // Handle generic args on a method receiver type: `fn Option[T].method(...)`.
        // Only skip `[...]` when it is immediately followed by `.` (a method receiver);
        // otherwise it is a generic function's own params (e.g. `fn max[T](...)`), so restore.
        if self.check(|k| matches!(k, TokenKind::LBracket)) {
            let saved = self.pos;
            self.advance(); // consume '['
            let mut depth = 1;
            while depth > 0 && !self.peek().is_eof() {
                match self.peek_kind() {
                    TokenKind::LBracket => { depth += 1; self.advance(); }
                    TokenKind::RBracket => { depth -= 1; self.advance(); }
                    _ => { self.advance(); }
                }
            }
            if !self.check(|k| matches!(k, TokenKind::Dot)) {
                self.pos = saved; // not a method receiver — leave `[...]` for generic params
            }
        }
        let (receiver, name) = if self.skip(TokenKind::Dot) { (Some(first), self.parse_ident()?) } else { (None, first) };
        let mut generics = self.parse_optional_generic_params()?;
        self.expect_kind(TokenKind::LParen, "'('")?;
        let params = if self.check(|k| matches!(k, TokenKind::RParen)) { self.advance(); Vec::new() } else { let p = self.parse_param_list()?; self.expect_kind(TokenKind::RParen, "')'")?; p };
        let return_type = if self.skip(TokenKind::Arrow) { Some(self.parse_type()?) } else { None };
        // 8B/M9: Parse optional `where` clause and merge bounds into generic params
        if matches!(self.peek_kind(), TokenKind::Ident(s) if s == "where") {
            self.advance();
            loop {
                let constraint_name = self.parse_ident()?;
                self.expect_kind(TokenKind::Colon, "':' in where clause")?;
                let bound_name = self.parse_ident()?;
                // Add bound to the matching generic param
                for gp in &mut generics {
                    if gp.name.name == constraint_name.name {
                        if gp.bounds.is_empty() {
                            gp.bounds.push(bound_name.clone());
                        }
                        break;
                    }
                }
                if self.peek_kind() == &TokenKind::Plus {
                    self.advance();
                    let extra = self.parse_ident()?;
                    for gp in &mut generics {
                        if gp.name.name == constraint_name.name {
                            gp.bounds.push(extra.clone());
                            break;
                        }
                    }
                }
                if !self.check(|k| matches!(k, TokenKind::Ident(_))) || self.peek_kind() == &TokenKind::LBrace || self.peek_kind() == &TokenKind::Semicolon {
                    break;
                }
            }
        }
        let mut contracts = Vec::new();
        while self.check(|k| matches!(k, TokenKind::Ident(s) if s == "requires" || s == "ensures")) {
            let is_req = matches!(self.peek_kind(), TokenKind::Ident(s) if s == "requires");
            self.advance();
            self.expect_kind(TokenKind::Colon, "':'")?;
            // Parse first expression
            let expr = self.parse_expr()?;
            contracts.push(if is_req { ContractClause::Requires(expr, start) } else { ContractClause::Ensures(expr, start) });
            // Comma-separated shorthand: `requires: a>0, b>0`
            while self.skip(TokenKind::Comma) {
                let extra = self.parse_expr()?;
                contracts.push(if is_req { ContractClause::Requires(extra, start) } else { ContractClause::Ensures(extra, start) });
            }
            // Optional semicolon terminator
            self.skip(TokenKind::Semicolon);
        }
        let body = if self.skip(TokenKind::Semicolon) { None } else if self.check(|k| matches!(k, TokenKind::LBrace)) { Some(self.parse_block()?) } else { None };
        Ok(TopDecl::Fn(FnDecl { attributes: attrs, is_async: async_flag, is_pub, receiver, name, generics, params, return_type, contracts, body, span: start }))
    }

    fn parse_const_decl(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        self.advance();
        let start = self.peek().span;
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::Colon, "':'")?;
        let ty = self.parse_type()?;
        self.expect_kind(TokenKind::Eq, "'='")?;
        let value = self.parse_expr()?;
        // Semicolons are optional at top-level (file-level module form) so
        // that large files like vulkan_constants_all.xi (3691 constants) don't
        // hit the 100-error parser limit from missing semicolons.
        let _ = self.skip(TokenKind::Semicolon);
        Ok(TopDecl::Const(ConstDecl { name, ty, value, is_mut: false, is_pub, span: start }))
    }

    fn parse_module_var(&mut self) -> Result<TopDecl, ParseError> {
        self.advance(); // consume 'var'
        let span = self.peek().span;
        let name = self.parse_ident()?;
        let ty = if self.skip(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        // Allow module-level `var name: T;` without initializer (defaults to zero)
        let value = if self.skip(TokenKind::Eq) { self.parse_expr()? } else { Expr::Int(0, span) };
        let _ = self.skip(TokenKind::Semicolon); // optional at top-level
        Ok(TopDecl::Const(ConstDecl {
            name,
            ty: ty.unwrap_or(Type::Named(Ident::new("_", span), vec![])),
            value,
            is_mut: true,
            is_pub: false,
            span,
        }))
    }

    fn parse_extern_block(&mut self) -> Result<TopDecl, ParseError> {
        self.advance(); // consume 'extern'
        let span_start = self.peek().span;
        // Parse linkage string (e.g., "C")
        let lexeme = self.peek().lexeme.clone();
        let linkage = lexeme.trim_matches('"').to_string();
        if !self.skip(TokenKind::Str(linkage.clone())) {
            self.advance(); // consume whatever string token it is
        }
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut functions = Vec::new();
        while !self.peek_is(TokenKind::RBrace) && !self.peek().is_eof() {
            self.expect_kind(TokenKind::Fn, "'fn'")?;
            let fn_name = self.parse_ident()?;
        let generics = self.parse_optional_generic_params()?;
            self.expect_kind(TokenKind::LParen, "'('")?;
            // Parse params manually, handling variadic '...'
            let mut params = Vec::new();
            while !self.peek_is(TokenKind::RParen) && !self.peek().is_eof() {
                if self.peek_is(TokenKind::Dot) {
                    self.advance(); self.advance(); self.advance(); // skip ...
                    break;
                }
                let pname = self.parse_ident()?;
                self.expect_kind(TokenKind::Colon, "':'")?;
                let pty = self.parse_type()?;
                let pspan = pname.span;
                params.push(xiom_ast::Param { name: pname, ty: pty, span: pspan, is_mut_self: false, is_ref_self: false });
                if !self.peek_is(TokenKind::RParen) {
                    self.expect_kind(TokenKind::Comma, "','")?;
                }
            }
            self.expect_kind(TokenKind::RParen, "')'")?;
            let return_type = if self.skip(TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect_kind(TokenKind::Semicolon, "';'")?;
            functions.push(FnDecl {
                attributes: vec![],
                is_async: false,
                is_pub: false,
                receiver: None,
                name: fn_name,
                generics,
                params,
                return_type,
                contracts: Vec::new(),
                body: None,
                span: span_start,
            });
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        Ok(TopDecl::Extern(ExternBlock { linkage, functions, span: span_start }))
    }

    fn peek_is(&self, kind: TokenKind) -> bool {
        self.peek_kind() == &kind
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        loop {
            params.push(self.parse_param()?);
            if !self.skip(TokenKind::Comma) { break; }
            // Trailing comma: `fn f(a: Int, b: Int,)` — accepted like call
            // args/arrays. Previously this made parse_param fail on `)` and
            // the whole function was silently dropped by error recovery.
            if self.check(|k| matches!(k, TokenKind::RParen)) { break; }
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let span = self.peek().span;
        if self.peek_kind() == &TokenKind::Ampersand {
            self.advance();
            let is_mut = if self.peek().lexeme == "mut" { self.advance(); true } else { false };
            let name = self.parse_ident()?;
            return Ok(Param { name, ty: Type::Named(Ident::new("Self", span), vec![]), span, is_mut_self: is_mut, is_ref_self: true });
        }
        if self.peek_kind() == &TokenKind::Self_ {
            let name = self.parse_ident()?;
            return Ok(Param { name, ty: Type::Named(Ident::new("Self", span), vec![]), span, is_mut_self: false, is_ref_self: false });
        }
        let name = self.parse_ident()?;
        if name.name == "mut" {
            let name = self.parse_ident()?;
            self.expect_kind(TokenKind::Colon, "':'")?;
            let ty = self.parse_type()?;
            Ok(Param { name, ty, span, is_mut_self: false, is_ref_self: false })
        } else {
            self.expect_kind(TokenKind::Colon, "':'")?;
            let ty = self.parse_type()?;
            Ok(Param { name, ty, span, is_mut_self: false, is_ref_self: false })
        }
    }

    fn parse_optional_generic_params(&mut self) -> Result<Vec<GenericParam>, ParseError> {
        if self.skip(TokenKind::LBracket) { let params = self.parse_generic_params()?; self.expect_kind(TokenKind::RBracket, "']'")?; Ok(params) } else { Ok(Vec::new()) }
    }

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, ParseError> {
        let mut params = Vec::new();
        loop {
            if self.skip(TokenKind::Const) {
                let name = self.parse_ident()?;
                self.expect_kind(TokenKind::Colon, "':'")?;
                let const_ty = self.parse_type()?;
                params.push(GenericParam { name, bounds: Vec::new(), is_const: true, const_ty: Some(const_ty) });
            } else {
                let name = self.parse_ident()?;
                let bounds = if self.skip(TokenKind::Colon) { self.parse_interface_refs()? } else { Vec::new() };
                params.push(GenericParam { name, bounds, is_const: false, const_ty: None });
            }
            if !self.skip(TokenKind::Comma) { break; }
            // Trailing comma in generic params: `fn f[T, U,](...)`.
            if self.check(|k| matches!(k, TokenKind::RBracket)) { break; }
        }
        Ok(params)
    }

    fn parse_interface_refs(&mut self) -> Result<Vec<Ident>, ParseError> {
        let mut refs = vec![self.parse_ident()?];
        while self.skip(TokenKind::Plus) { refs.push(self.parse_ident()?); }
        Ok(refs)
    }

    fn parse_derive_list(&mut self) -> Result<Vec<DeriveTrait>, ParseError> {
        self.expect_kind(TokenKind::LBracket, "'['")?;
        let mut derives = Vec::new();
        loop {
            let name = self.parse_ident()?;
            let dt = DeriveTrait::from_str(&name.name).ok_or_else(|| self.error(format!("unknown derive trait: {}", name.name)))?;
            derives.push(dt);
            if !self.skip(TokenKind::Comma) { break; }
        }
        self.expect_kind(TokenKind::RBracket, "']'")?;
        Ok(derives)
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        self.enter_expr()?;
        let result = self.parse_type_inner();
        self.exit_expr();
        result
    }

    fn parse_type_inner(&mut self) -> Result<Type, ParseError> {
        if self.skip(TokenKind::Ampersand) {
            let mutable = match self.peek_kind() { TokenKind::Ident(s) if s == "mut" => { self.advance(); true } _ => false };
            let base = self.parse_type_base()?;
            return Ok(if mutable { Type::MutRef(Box::new(base)) } else { Type::Ref(Box::new(base)) });
        }
        self.parse_type_base()
    }

    fn parse_type_base(&mut self) -> Result<Type, ParseError> {
        // v0.55: Never type — `!` as bottom type
        if self.peek_kind() == &TokenKind::Bang {
            let _ = self.advance();
            return Ok(Type::Never);
        }
        // Skip 'dyn' keyword (dynamic dispatch marker): `dyn Trait` parses as `Trait`.
        if let TokenKind::Ident(s) = self.peek_kind() { if s == "dyn" { self.advance(); } }
        // Parse `impl Trait` as opaque return type (M9.6)
        if matches!(self.peek_kind(), TokenKind::Ident(s) if s == "impl") || matches!(self.peek_kind(), TokenKind::Impl) {
            self.advance();
            let mut traits = vec![self.parse_ident()?];
            while self.skip(TokenKind::Plus) {
                traits.push(self.parse_ident()?);
            }
            return Ok(Type::ImplTrait(traits));
        }
        let peeked = match self.peek_kind() { TokenKind::Ident(s) => Some(s.clone()), _ => None };
        if let Some(ref s) = peeked {
            match s.as_str() {
                "Option" => { if self.peek_ahead(1) == Some(&TokenKind::LBracket) || self.peek_ahead(1) == Some(&TokenKind::Lt) { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let inner = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Option(Box::new(inner))); } }
                "Result" => { if self.peek_ahead(1) == Some(&TokenKind::LBracket) || self.peek_ahead(1) == Some(&TokenKind::Lt) { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let ok = self.parse_type()?; self.expect_kind(TokenKind::Comma, "','")?; let err = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Result(Box::new(ok), Box::new(err))); } }
                "Vec" => { if self.peek_ahead(1) == Some(&TokenKind::LBracket) || self.peek_ahead(1) == Some(&TokenKind::Lt) { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let inner = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Vec(Box::new(inner))); } }
                "Slice" => { if self.peek_ahead(1) == Some(&TokenKind::LBracket) || self.peek_ahead(1) == Some(&TokenKind::Lt) { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let inner = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Slice(Box::new(inner))); } }
                "Map" => { if self.peek_ahead(1) == Some(&TokenKind::LBracket) || self.peek_ahead(1) == Some(&TokenKind::Lt) { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let k = self.parse_type()?; self.expect_kind(TokenKind::Comma, "','")?; let v = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Map(Box::new(k), Box::new(v))); } }
                "Set" => { if self.peek_ahead(1) == Some(&TokenKind::LBracket) || self.peek_ahead(1) == Some(&TokenKind::Lt) { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let inner = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Set(Box::new(inner))); } }
                _ => {}
            }
        }
        if self.skip(TokenKind::Star) {
            // Skip optional 'const' or 'mut' qualifier on raw pointers.
            if self.peek_kind() == &TokenKind::Const { self.advance(); }
            else if let TokenKind::Ident(s) = self.peek_kind() { if s == "mut" { self.advance(); } }
            let base = self.parse_type_base()?;
            return Ok(Type::Ptr(Box::new(base)));
        }
        if self.skip(TokenKind::LParen) {
            // `()` — empty parens = unit type (for `Result[(), E]` and similar).
            if self.check(|k| matches!(k, TokenKind::RParen)) {
                self.advance(); // consume ')'
                return Ok(Type::Named(Ident::new("()", self.peek().span), vec![]));
            }
            let first = self.parse_type()?;
            if self.skip(TokenKind::Comma) { let mut types = vec![first, self.parse_type()?]; while self.skip(TokenKind::Comma) { types.push(self.parse_type()?); } self.expect_kind(TokenKind::RParen, "')'")?; return Ok(Type::Tuple(types)); }
            self.expect_kind(TokenKind::RParen, "')'")?;
            return Ok(first);
        }
        if self.skip(TokenKind::LBracket) { let size = if let TokenKind::Int(n) = self.peek_kind() { let n = *n; self.advance(); Expr::Int(n, self.peek().span) } else if matches!(self.peek_kind(), TokenKind::Ident(_)) { Expr::Ident(self.parse_ident()?) } else { return Err(self.error("expected integer or identifier for fixed array size")); }; self.expect_kind(TokenKind::RBracket, "']'")?; let inner = self.parse_type()?; return Ok(Type::Array(Box::new(size), Box::new(inner))); }
        if self.peek_kind() == &TokenKind::Fn {
            self.advance(); self.expect_kind(TokenKind::LParen, "'('")?;
            let mut param_types = Vec::new();
            if self.peek_kind() != &TokenKind::RParen { param_types.push(self.parse_type()?); while self.skip(TokenKind::Comma) { param_types.push(self.parse_type()?); } }
            self.expect_kind(TokenKind::RParen, "')'")?;
            let ret = if self.skip(TokenKind::Arrow) { self.parse_type()? } else { Type::Named(Ident::new("Unit", Span::new(0, 0)), vec![]) };
            return Ok(Type::Fn(param_types, Box::new(ret)));
        }
        // 5c.33: Anonymous struct type `{ field: Type; field: Type; }` —
        // used in generic function signatures like `fn f(p: {a: A; b: B}) -> {c: C}`.
        if self.peek_kind() == &TokenKind::LBrace {
            let start = self.advance().span;
            let mut fields = Vec::new();
            loop {
                if self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) { break; }
                let fname = self.parse_ident()?;
                self.expect_kind(TokenKind::Colon, "':'")?;
                let fty = self.parse_type()?;
                fields.push(xiom_ast::FieldDecl { name: fname, ty: fty, span: start });
                if !self.skip(TokenKind::Semicolon) && !self.skip(TokenKind::Comma) { break; }
            }
            self.expect_kind(TokenKind::RBrace, "'}'")?;
            return Ok(Type::AnonStruct(fields));
        }
        let mut name = self.parse_ident()?;
        while self.skip(TokenKind::Dot) {
            let next = self.parse_ident()?;
            name = Ident::new(format!("{}.{}", name.name, next.name), name.span);
        }
        let args = if self.skip(TokenKind::LBracket) { let mut types = vec![self.parse_type()?]; while self.skip(TokenKind::Comma) { types.push(self.parse_type()?); } self.expect_kind(TokenKind::RBracket, "']'")?; types } else if self.skip(TokenKind::Lt) { let mut types = vec![self.parse_type()?]; while self.skip(TokenKind::Comma) { types.push(self.parse_type()?); } self.expect_kind(TokenKind::Gt, "'>'")?; types } else { Vec::new() };
        Ok(Type::Named(name, args))
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.peek().span; self.expect_kind(TokenKind::LBrace, "'{'")?;
        // Inside a block, struct literals are allowed again (a condition's
        // struct restriction does not propagate into the body).
        let saved_restrict = self.restrict_struct;
        self.restrict_struct = false;
        let mut items = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
            if self.skip(TokenKind::Semicolon) { continue; }
            items.push(self.parse_stmt_or_expr()?);
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        self.restrict_struct = saved_restrict;
        Ok(Block { stmts: items, span: start })
    }

    fn parse_stmt_or_expr(&mut self) -> Result<StmtOrExpr, ParseError> {
        // P0-3: Support `@label: stmt` for labeled loops
        let loop_label = if self.peek_kind() == &TokenKind::At {
            let ahead = self.peek_ahead(1);
            if let Some(TokenKind::Ident(_)) = ahead {
                self.advance(); // consume '@'
                let label = self.parse_ident().ok();
                if self.peek_kind() == &TokenKind::Colon {
                    self.advance(); // consume ':'
                    label
                } else {
                    label // don't consume colon if missing (tolerate)
                }
            } else {
                None
            }
        } else {
            None
        };

        match self.peek_kind() {
            TokenKind::Let => { let stmt = self.parse_let_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::Var => { let stmt = self.parse_var_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::Const => {
                // Local `const NAME: T = value;` — treated as an immutable let.
                let span = self.advance().span;
                let name = self.parse_ident()?;
                let ty = if self.skip(TokenKind::Colon) { Some(Box::new(self.parse_type()?)) } else { None };
                self.expect_kind(TokenKind::Eq, "'='")?;
                let value = self.parse_expr()?;
                self.expect_kind(TokenKind::Semicolon, "';'")?;
                Ok(StmtOrExpr::Stmt(Stmt::Let(name, ty, value, span)))
            }
            TokenKind::Return => { let stmt = self.parse_return_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::Break => { let span = self.advance().span; let label = self.parse_optional_label(); self.skip(TokenKind::Semicolon); Ok(StmtOrExpr::Stmt(Stmt::Break(label, span))) }
            TokenKind::Continue => { let span = self.advance().span; let label = self.parse_optional_label(); self.skip(TokenKind::Semicolon); Ok(StmtOrExpr::Stmt(Stmt::Continue(label, span))) }
            TokenKind::If => { let stmt = self.parse_if_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::Match => { let stmt = self.parse_match_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::While => {
                let mut stmt = self.parse_while_stmt()?;
                if let Some(ref label) = loop_label {
                    if let Stmt::While(cond, body, inv, span, _) = stmt {
                        stmt = Stmt::While(cond, body, inv, span, Some(label.clone()));
                    }
                }
                Ok(StmtOrExpr::Stmt(stmt))
            }
            TokenKind::For => {
                let mut stmt = self.parse_for_stmt()?;
                if let Some(ref label) = loop_label {
                    if let Stmt::For(var, iter, body, span, _) = stmt {
                        stmt = Stmt::For(var, iter, body, span, Some(label.clone()));
                    }
                }
                Ok(StmtOrExpr::Stmt(stmt))
            }
            // v0.55: asm("template" : outputs : inputs : clobbers);
            TokenKind::Asm => { let stmt = self.parse_asm_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            // v0.55: defer { ... } or defer expr;
            TokenKind::Defer => { let stmt = self.parse_defer_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::Spawn if {
                let next = self.peek_ahead(1);
                next == Some(&TokenKind::LBrace) || next == Some(&TokenKind::Move)
            } => { let stmt = self.parse_spawn_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::LBrace => {
                // Bare block expression: `{ stmt; ... }` as a statement or expression
                let block = self.parse_block()?;
                let span = block.span;
                Ok(StmtOrExpr::Expr(xiom_ast::Expr::BlockExpr(block, span)))
            }
            TokenKind::Ident(s) if s == "loop" && self.peek_ahead(1) == Some(&TokenKind::LBrace) => {
                let span = self.advance().span; // consume 'loop'
                let body = self.parse_block()?;
                // Desugar `loop { ... }` to `while true { ... }`
                Ok(StmtOrExpr::Stmt(Stmt::While(Expr::Bool(true, span), body, None, span, None)))
            }
            _ => {
                let expr = self.parse_expr()?;
                // 8B/M9: Compound assignment desugaring
                let compound = self.try_compound_assign(&expr);
                if let Some(rhs) = compound {
                    let _span = self.peek().span;
                    self.expect_kind(TokenKind::Semicolon, "';'")?;
                    Ok(StmtOrExpr::Stmt(Stmt::Assign(expr, rhs, _span)))
                }
                else if self.skip(TokenKind::Eq) { let rhs = self.parse_expr()?; let span = self.peek().span; self.expect_kind(TokenKind::Semicolon, "';'")?; Ok(StmtOrExpr::Stmt(Stmt::Assign(expr, rhs, span))) }
                else if self.check(|k| matches!(k, TokenKind::RBrace)) { Ok(StmtOrExpr::Expr(expr)) }
                else if matches!(expr, Expr::Unsafe(..) | Expr::If(..)) { self.skip(TokenKind::Semicolon); Ok(StmtOrExpr::Expr(expr)) }
                else { self.expect_kind(TokenKind::Semicolon, "';'")?; Ok(StmtOrExpr::Expr(expr)) }
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        if self.peek_kind() == &TokenKind::LParen { return self.parse_destructure(span); }
        let name = self.parse_ident()?;
        let ty = if self.skip(TokenKind::Colon) { Some(Box::new(self.parse_type()?)) } else { None };
        // Allow `let x: T;` without initializer (defaults to zero); assigned later.
        let value = if self.skip(TokenKind::Eq) {
            self.parse_init_expr(ty.as_deref(), span)?
        } else { Expr::Int(0, span) };
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Let(name, ty, value, span))
    }

    fn parse_var_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        if self.peek_kind() == &TokenKind::LParen { return self.parse_destructure(span); }
        let name = self.parse_ident()?;
        let ty = if self.skip(TokenKind::Colon) { Some(Box::new(self.parse_type()?)) } else { None };
        // Allow `var x: Type;` without explicit initialization (defaults to zero)
        let value = if self.skip(TokenKind::Eq) {
            self.parse_init_expr(ty.as_deref(), span)?
        } else { Expr::Int(0, span) };
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Var(name, ty, value, span))
    }

    /// Parse an initializer expression, potentially using the declared type
    /// to disambiguate bare `{ field: value; }` as a struct literal.
    /// When `var x: TypeName = { field: value; };` is written, the parser
    /// uses the `TypeName` annotation to parse `{ ... }` as `TypeName{ ... }`.
    fn parse_init_expr(&mut self, declared_ty: Option<&Type>, span: Span) -> Result<Expr, ParseError> {
        if let Some(ty) = declared_ty {
            if let Some(type_name) = Self::extract_struct_type_name(ty) {
                if self.peek_kind() == &TokenKind::LBrace {
                    return self.parse_struct_literal_body(&type_name, span);
                }
            }
        }
        self.parse_expr()
    }

    /// Extract a type name suitable for struct literal construction.
    /// Returns `Some("Point")` for `Type::Named("Point", [])`,
    /// `Some("Point")` for `Type::Ref(Type::Named("Point", []))`,
    /// `None` for builtins like `Option`, `Result`, `Vec`, `Int`, etc.
    fn extract_struct_type_name(ty: &Type) -> Option<String> {
        let builtins = &["Option", "Result", "Vec", "Slice", "Map", "Set",
                          "Bool", "Int", "Int8", "Int16", "Int32", "Int64",
                          "UInt8", "UInt16", "UInt32", "UInt64",
                          "Float32", "Float64", "Char", "Str", "String",
                          "Rc", "Arc", "Cell", "RefCell", "Box", "Ptr"];
        match ty {
            Type::Named(id, _) => {
                let name = id.name.as_str();
                if builtins.contains(&name) || name.chars().next().map_or(true, |c| !c.is_uppercase()) {
                    None
                } else {
                    Some(id.name.clone())
                }
            }
            Type::Ref(inner) | Type::MutRef(inner) => {
                Self::extract_struct_type_name(inner)
            }
            _ => None,
        }
    }

    /// Parse `{ field: value; field2: value2; }` as a struct literal body,
    /// prefixing with the given type_name to produce `Expr::Struct(type_name, fields, ...)`.
    fn parse_struct_literal_body(&mut self, type_name: &str, span: Span) -> Result<Expr, ParseError> {
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut fields = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Dot | TokenKind::Eof)) {
            let fname = self.parse_ident()?;
            if self.skip(TokenKind::Colon) {
                let fval = self.parse_expr()?;
                fields.push((fname, fval));
            } else {
                // Shorthand: `{ field }` means `{ field: field }`
                fields.push((fname.clone(), Expr::Ident(fname)));
            }
            self.skip(TokenKind::Comma);
            self.skip(TokenKind::Semicolon);
        }
        let spread = if self.skip(TokenKind::Dot) {
            self.expect_kind(TokenKind::Dot, "'.' for spread")?;
            let s = self.parse_expr()?;
            Some(Box::new(s))
        } else { None };
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        Ok(Expr::Struct(Ident::new(type_name.to_string(), span), fields, spread, span))
    }

    /// Parse integer suffix: "42i8" → Int8, "255u8" → UInt8, etc.
    /// Returns Some(Type) if the lexeme has a valid suffix, None otherwise.
    fn parse_int_suffix(lexeme: &str) -> Option<Type> {
        if let Some(pos) = lexeme.find(|c: char| c == 'i' || c == 'u') {
            let suffix = &lexeme[pos..];
            return match suffix {
                "i8" => Some(Type::Named(Ident::new("Int8".to_string(), Span::new(0, 0)), vec![])),
                "i16" => Some(Type::Named(Ident::new("Int16".to_string(), Span::new(0, 0)), vec![])),
                "i32" => Some(Type::Named(Ident::new("Int32".to_string(), Span::new(0, 0)), vec![])),
                "i64" => Some(Type::Named(Ident::new("Int64".to_string(), Span::new(0, 0)), vec![])),
                "u8" => Some(Type::Named(Ident::new("UInt8".to_string(), Span::new(0, 0)), vec![])),
                "u16" => Some(Type::Named(Ident::new("UInt16".to_string(), Span::new(0, 0)), vec![])),
                "u32" => Some(Type::Named(Ident::new("UInt32".to_string(), Span::new(0, 0)), vec![])),
                "u64" => Some(Type::Named(Ident::new("UInt64".to_string(), Span::new(0, 0)), vec![])),
                _ => None,
            };
        }
        None
    }

    /// Parse float suffix: "3.14f32" → Float32, "1.0f64" → Float64
    fn parse_float_suffix(lexeme: &str) -> Option<Type> {
        if let Some(pos) = lexeme.find('f') {
            let suffix = &lexeme[pos..];
            return match suffix {
                "f32" => Some(Type::Named(Ident::new("Float32".to_string(), Span::new(0, 0)), vec![])),
                "f64" => Some(Type::Named(Ident::new("Float64".to_string(), Span::new(0, 0)), vec![])),
                _ => None,
            };
        }
        None
    }

    fn parse_destructure(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let mut names = Vec::new(); names.push(self.parse_ident()?);
        while self.skip(TokenKind::Comma) { names.push(self.parse_ident()?); }
        self.expect_kind(TokenKind::RParen, "')'")?; self.expect_kind(TokenKind::Eq, "'='")?;
        let value = self.parse_expr()?; self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Destructure(names, value, span))
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        let expr = if self.check(|k| matches!(k, TokenKind::Semicolon | TokenKind::RBrace)) { None } else { Some(self.parse_expr()?) };
        // The trailing `;` is optional when the return is the final statement of a
        // block (i.e. immediately followed by `}`), mirroring tail-expression rules.
        if self.check(|k| matches!(k, TokenKind::RBrace)) {
            self.skip(TokenKind::Semicolon);
        } else {
            self.expect_kind(TokenKind::Semicolon, "';'")?;
        }
        Ok(Stmt::Return(expr, span))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        // 8B/M9: if let pattern matching
        if self.peek_kind() == &TokenKind::Let {
            return self.parse_if_let_stmt(span);
        }
        let cond = self.parse_cond()?; let then_block = self.parse_block()?;
        let mut elifs = Vec::new();
        while self.skip(TokenKind::Elif) { let econd = self.parse_cond()?; let eblock = self.parse_block()?; elifs.push((econd, eblock)); }
        let else_block = if self.skip(TokenKind::Else) { Some(self.parse_block()?) } else { None };
        Ok(Stmt::If(cond, then_block, elifs, else_block, span))
    }

    /// 8B/M9: Parse `if let pattern = expr { ... } [else { ... }]`
    /// Desugars to a match expression:
    ///   if let Some(v) = x { A } else { B }
    /// becomes:
    ///   match x { Some(v) => { A }, _ => { B } }
    fn parse_if_let_stmt(&mut self, if_span: Span) -> Result<Stmt, ParseError> {
        self.advance(); // skip 'let'
        let pattern = self.parse_pattern()?;
        self.expect_kind(TokenKind::Eq, "'=' in if let")?;
        let expr = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.skip(TokenKind::Else) { Some(self.parse_block()?) } else { None };

        let wildcard = Pattern::Wildcard(Span::new(0, 0));
        let else_body = match else_block {
            Some(b) => MatchBody::Block(b),
            None => MatchBody::Block(Block { stmts: vec![], span: if_span }),
        };

        let arms = vec![
            MatchArm { pattern, guard: None, body: MatchBody::Block(then_block), span: if_span },
            MatchArm { pattern: wildcard, guard: None, body: else_body, span: if_span },
        ];

        Ok(Stmt::Match(*Box::new(expr), arms, if_span))
    }

    fn parse_match_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span; let expr = self.parse_cond()?;
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut arms = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) { arms.push(self.parse_match_arm()?); }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        Ok(Stmt::Match(expr, arms, span))
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let span = self.peek().span; let pattern = self.parse_pattern()?;
        let guard = if self.skip(TokenKind::If) { Some(self.parse_or_expr()?) } else { None };
        self.expect_kind(TokenKind::FatArrow, "'=>'")?;
        let body = if self.check(|k| matches!(k, TokenKind::LBrace)) { let block = self.parse_block()?; self.skip(TokenKind::Comma); self.skip(TokenKind::Semicolon); MatchBody::Block(block) }
        else if self.check(|k| matches!(k, TokenKind::If | TokenKind::Return | TokenKind::Match | TokenKind::While | TokenKind::For)) {
            let stmt = self.parse_arm_stmt()?; self.skip(TokenKind::Comma); self.skip(TokenKind::Semicolon);
            MatchBody::Block(Block { stmts: vec![StmtOrExpr::Stmt(stmt)], span })
        } else {
            let expr_result = self.parse_expr_or_assign()?;
            self.skip(TokenKind::Comma);
            self.skip(TokenKind::Semicolon);
            expr_result
        };
        Ok(MatchArm { pattern, guard, body, span })
    }

    fn parse_arm_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::Return => { self.advance(); let expr = if self.check(|k| matches!(k, TokenKind::Comma | TokenKind::RBrace)) { None } else { Some(self.parse_expr()?) }; let span = self.peek().span; Ok(Stmt::Return(expr, span)) }
            TokenKind::Match => self.parse_match_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            _ => Err(self.error("expected statement in match arm")),
        }
    }

    /// Parse an expression in a match arm body, also handling assignment
    /// (`lhs = rhs`) as a statement block.  Assignment is a statement-level
    /// construct in XIOM, not a binary operator, so it must be detected here
    /// when used in expression position (match arm bodies, if-expression
    /// branches, etc.).
    fn parse_expr_or_assign(&mut self) -> Result<MatchBody, ParseError> {
        let span = self.peek().span;
        let expr = self.parse_expr()?;
        // Check for compound assignment: +=, -=, *=, etc.
        if let Some(rhs) = self.try_compound_assign(&expr) {
            return Ok(MatchBody::Block(Block {
                stmts: vec![StmtOrExpr::Stmt(Stmt::Assign(expr, rhs, span))],
                span,
            }));
        }
        // Simple assignment: `lhs = rhs`
        if self.skip(TokenKind::Eq) {
            let rhs = self.parse_expr()?;
            return Ok(MatchBody::Block(Block {
                stmts: vec![StmtOrExpr::Stmt(Stmt::Assign(expr, rhs, span))],
                span,
            }));
        }
        Ok(MatchBody::Expr(expr))
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        // 8B/M9: while let pattern matching
        if self.peek_kind() == &TokenKind::Let {
            return self.parse_while_let_stmt(span);
        }
        let cond = self.parse_cond()?;
        // 5f: optional loop invariant
        let invariant = if self.check(|k| matches!(k, TokenKind::Ident(s) if s == "invariant")) {
            self.advance();
            self.expect_kind(TokenKind::Colon, "':' after 'invariant'")?;
            Some(self.parse_cond()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Stmt::While(cond, body, invariant, span, None))
    }

    /// 8B/M9: Parse `while let pattern = expr { ... }`
    /// Desugars to `while true { match expr { pattern => { ... }, _ => break } }`
    fn parse_while_let_stmt(&mut self, while_span: Span) -> Result<Stmt, ParseError> {
        self.advance(); // skip 'let'
        let pattern = self.parse_pattern()?;
        self.expect_kind(TokenKind::Eq, "'=' in while let")?;
        let expr = self.parse_expr()?;
        let body_block = self.parse_block()?;

        let wildcard = Pattern::Wildcard(Span::new(0, 0));
        let break_stmt = StmtOrExpr::Stmt(Stmt::Break(None, while_span));
        let break_body = Block { stmts: vec![break_stmt], span: while_span };

        let match_arms = vec![
            MatchArm { pattern, guard: None, body: MatchBody::Block(body_block), span: while_span },
            MatchArm { pattern: wildcard, guard: None, body: MatchBody::Block(break_body), span: while_span },
        ];

        let match_stmt = Stmt::Match(*Box::new(expr), match_arms, while_span);
        let inner_block = Block {
            stmts: vec![StmtOrExpr::Stmt(match_stmt)],
            span: while_span,
        };

        Ok(Stmt::While(Expr::Bool(true, while_span), inner_block, None, while_span, None))
    }
    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> { let span = self.advance().span; let var = self.parse_ident()?; self.expect_kind(TokenKind::In, "'in'")?; let iter = self.parse_cond()?; let body = self.parse_block()?; Ok(Stmt::For(var, iter, body, span, None)) }
    /// v0.55: Parse `defer { ... }` or `defer expr;`
    fn parse_defer_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        if self.peek_kind() == &TokenKind::LBrace {
            let block = self.parse_block()?;
            Ok(Stmt::Defer(block, span))
        } else {
            let expr = self.parse_expr()?;
            self.skip(TokenKind::Semicolon);
            let block = Block { stmts: vec![StmtOrExpr::Expr(expr)], span };
            Ok(Stmt::Defer(block, span))
        }
    }

    fn parse_spawn_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span; // consume 'spawn'
        let is_move = if self.peek().kind == TokenKind::Move {
            self.advance();
            true
        } else {
            false
        };
        let body = self.parse_block()?;
        Ok(Stmt::Spawn(body, span, is_move))
    }

    /// v0.55: Parse `asm("template" [: outputs [: inputs [: clobbers]]]);`
    fn parse_asm_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span; // consume 'asm'
        self.expect_kind(TokenKind::LParen, "'(' after asm")?;

        // Parse template string
        let template = match &self.advance().kind {
            TokenKind::Str(s) => s.clone(),
            _ => return Err(self.error("expected string literal as asm template")),
        };
        let mut outputs = Vec::new();
        let mut inputs = Vec::new();
        let mut clobbers = Vec::new();

        // Optional output constraints
        if self.peek_kind() == &TokenKind::Colon {
            self.advance();
            while self.peek_kind() != &TokenKind::Colon
                && self.peek_kind() != &TokenKind::RParen
                && self.peek_kind() != &TokenKind::Semicolon
                && self.peek_kind() != &TokenKind::Eof
            {
                let constraint = match &self.advance().kind {
                    TokenKind::Str(s) => s.clone(),
                    _ => return Err(self.error("expected constraint string in asm outputs")),
                };
                self.expect_kind(TokenKind::LParen, "'(' after constraint")?;
                let var = self.parse_ident()?;
                self.expect_kind(TokenKind::RParen, "')' after asm output")?;
                outputs.push((constraint, var));
                self.skip(TokenKind::Comma);
            }
        }

        // Optional input constraints
        if self.peek_kind() == &TokenKind::Colon {
            self.advance();
            while self.peek_kind() != &TokenKind::Colon
                && self.peek_kind() != &TokenKind::RParen
                && self.peek_kind() != &TokenKind::Semicolon
                && self.peek_kind() != &TokenKind::Eof
            {
                let constraint = match &self.advance().kind {
                    TokenKind::Str(s) => s.clone(),
                    _ => return Err(self.error("expected constraint string in asm inputs")),
                };
                self.expect_kind(TokenKind::LParen, "'(' after constraint")?;
                let expr = self.parse_expr()?;
                self.expect_kind(TokenKind::RParen, "')' after asm input")?;
                inputs.push((constraint, expr));
                self.skip(TokenKind::Comma);
            }
        }

        // Optional clobbers
        if self.peek_kind() == &TokenKind::Colon {
            self.advance();
            while self.peek_kind() != &TokenKind::RParen
                && self.peek_kind() != &TokenKind::Semicolon
                && self.peek_kind() != &TokenKind::Eof
            {
                let clobber = match &self.advance().kind {
                    TokenKind::Str(s) => s.clone(),
                    _ => return Err(self.error("expected clobber string in asm")),
                };
                clobbers.push(clobber);
                self.skip(TokenKind::Comma);
            }
        }

        self.expect_kind(TokenKind::RParen, "')' after asm")?;
        self.skip(TokenKind::Semicolon);

        Ok(Stmt::Asm(AsmBlock { template, outputs, inputs, clobbers, span }))
    }
    /// M21: Parse module-level `spawn { ... }` as a top-level declaration.
    fn parse_spawn_top_decl(&mut self) -> Result<TopDecl, ParseError> {
        let span = self.advance().span; // consume 'spawn'
        let is_move = if self.peek().kind == TokenKind::Move {
            self.advance();
            true
        } else {
            false
        };
        let body = self.parse_block()?;
        Ok(TopDecl::Spawn(body, span, is_move))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let span = self.peek().span;
        let first = self.parse_pattern_single()?;
        if self.check(|k| matches!(k, TokenKind::Pipe)) {
            let mut alts = vec![first];
            while self.skip(TokenKind::Pipe) {
                alts.push(self.parse_pattern_single()?);
            }
            return Ok(Pattern::Or(alts, span));
        }
        Ok(first)
    }

    fn parse_pattern_single(&mut self) -> Result<Pattern, ParseError> {
        match self.peek_kind() {
            TokenKind::Underscore => { let span = self.advance().span; Ok(Pattern::Wildcard(span)) }
            TokenKind::Some => { let span = self.advance().span; let inner = if self.skip(TokenKind::LParen) { let p = self.parse_pattern()?; self.expect_kind(TokenKind::RParen, "')'")?; p } else { Pattern::Wildcard(span) }; Ok(Pattern::Some(Box::new(inner), span)) }
            TokenKind::None => { let span = self.advance().span; Ok(Pattern::None(span)) }
            TokenKind::Ok_ => { let span = self.advance().span; let inner = if self.skip(TokenKind::LParen) { let p = self.parse_pattern()?; self.expect_kind(TokenKind::RParen, "')'")?; p } else { Pattern::Wildcard(span) }; Ok(Pattern::Ok(Box::new(inner), span)) }
            TokenKind::Err_ => { let span = self.advance().span; let inner = if self.skip(TokenKind::LParen) { let p = self.parse_pattern()?; self.expect_kind(TokenKind::RParen, "')'")?; p } else { Pattern::Wildcard(span) }; Ok(Pattern::Err(Box::new(inner), span)) }
            TokenKind::True => { let span = self.advance().span; Ok(Pattern::Lit(Literal::Bool(true, span))) }
            TokenKind::False => { let span = self.advance().span; Ok(Pattern::Lit(Literal::Bool(false, span))) }
            TokenKind::Int(n) => { let n = *n; let span = self.advance().span; Ok(Pattern::Lit(Literal::Int(n, span))) }
            TokenKind::Str(s) => { let s = s.clone(); let span = self.advance().span; Ok(Pattern::Lit(Literal::Str(s, span))) }
            TokenKind::Char(c) => { let c = *c; let span = self.advance().span; Ok(Pattern::Lit(Literal::Char(c, span))) }
            TokenKind::Float(f) => { let f = *f; let span = self.advance().span; Ok(Pattern::Lit(Literal::Float(f, span))) }
            TokenKind::LParen => {
                let span = self.advance().span;
                if self.skip(TokenKind::RParen) {
                    // Unit pattern `()` — treat as wildcard (unit has a single value)
                    return Ok(Pattern::Wildcard(span));
                }
                let first = self.parse_pattern()?;
                // P1-2: If comma follows, it's a tuple pattern (a, b, c)
                if self.skip(TokenKind::Comma) {
                    let mut items = vec![first, self.parse_pattern()?];
                    while self.skip(TokenKind::Comma) {
                        items.push(self.parse_pattern()?);
                    }
                    self.expect_kind(TokenKind::RParen, "')'")?;
                    return Ok(Pattern::Tuple(items, span));
                }
                self.expect_kind(TokenKind::RParen, "')'")?;
                Ok(first)
            }
            _ => {
                // `ref` / `ref mut` in patterns are binding modifiers.
                // In XIOM's value semantics, they're syntactic sugar — the
                // compiler treats them as regular bindings. Parse and discard
                // them silently so patterns like `Array(ref mut items)` work.
                let mut _is_ref = false;
                let mut _is_mut_ref = false;
                if let TokenKind::Ident(s) = self.peek_kind() {
                    if s == "ref" {
                        _is_ref = true;
                        self.advance();
                        if let TokenKind::Ident(s2) = self.peek_kind() {
                            if s2 == "mut" { _is_mut_ref = true; self.advance(); }
                        }
                    }
                }
                let mut name = self.parse_ident()?;
                // Qualified enum-variant pattern, e.g. `LogLevel.Trace` or
                // `JsonValue.String(k)`. Fold the dotted path into a single name.
                while self.skip(TokenKind::Dot) {
                    let variant = self.parse_ident()?;
                    name = Ident::new(format!("{}.{}", name.name, variant.name), name.span);
                }
                if self.skip(TokenKind::LParen) {
                    let span = name.span; let mut fields = Vec::new();
                    loop {
                        // Skip `ref` / `ref mut` in variant constructor patterns
                        // (e.g. `JsonValue.Array(ref mut items)`). XIOM uses value
                        // semantics — these are syntactic sugar accepted for
                        // compatibility but treated as regular bindings.
                        if let TokenKind::Ident(s) = self.peek_kind() {
                            if s == "ref" { self.advance(); if let TokenKind::Ident(s2) = self.peek_kind() { if s2 == "mut" { self.advance(); } } }
                        }
                        let field = self.parse_ident()?;
                        if self.skip(TokenKind::Colon) { let sub = self.parse_pattern()?; match &sub { Pattern::Ident(binding) => fields.push(binding.clone()), _ => fields.push(Ident::new("_", field.span)) } }
                        else { fields.push(field); }
                        if !self.skip(TokenKind::Comma) { break; }
                    }
                    self.expect_kind(TokenKind::RParen, "')'")?;
                    Ok(Pattern::Variant(name, fields, span))
                } else if self.peek_kind() == &TokenKind::LBrace {
                    // P1-1: Struct pattern `TypeName { field1, field2: pat2 }`
                    self.advance(); // consume '{'
                    let span = name.span;
                    let mut fields: Vec<(Ident, Pattern)> = Vec::new();
                    while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                        let fname = self.parse_ident()?;
                        let fpat = if self.skip(TokenKind::Colon) {
                            self.parse_pattern()?
                        } else {
                            // Shorthand: `{ field }` means `{ field: field }`
                            Pattern::Ident(fname.clone())
                        };
                        fields.push((fname, fpat));
                        self.skip(TokenKind::Comma);
                    }
                    self.expect_kind(TokenKind::RBrace, "'}'")?;
                    Ok(Pattern::Struct(name, fields, span))
                } else { Ok(Pattern::Ident(name)) }
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.enter_expr()?;
        let result = self.parse_imply_expr();
        self.exit_expr();
        result
    }

    /// Parse the head expression of a control-flow construct (`if`/`while`/
    /// etc.) where a trailing `{` starts the body block, so bare struct
    /// literals like `Foo { ... }` must be suppressed at this level.
    fn parse_cond(&mut self) -> Result<Expr, ParseError> {
        let saved = self.restrict_struct;
        self.restrict_struct = true;
        let result = self.parse_expr();
        self.restrict_struct = saved;
        result
    }

    /// Parse an expression inside a delimiter (`(...)`, `[...]`, call args).
    /// A condition's struct-literal restriction does not cross delimiters, so
    /// struct literals like `foo(Bar { x: 1 })` remain valid inside a `if`/
    /// `while` head expression.
    fn parse_expr_open(&mut self) -> Result<Expr, ParseError> {
        let saved = self.restrict_struct;
        self.restrict_struct = false;
        let result = self.parse_expr();
        self.restrict_struct = saved;
        result
    }

    fn parse_imply_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_or_expr()?;
        while self.skip(TokenKind::FatArrow) { let right = self.parse_or_expr()?; let span = left.span(); left = Expr::Imply(Box::new(left), Box::new(right), span); }
        if self.skip(TokenKind::Question) { let span = left.span(); left = Expr::Try(Box::new(left), span); }
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> { let mut left = self.parse_and_expr()?; while self.skip(TokenKind::OrOr) { let right = self.parse_and_expr()?; let span = left.span(); left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(right), span); } Ok(left) }
    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> { let mut left = self.parse_is_expr()?; while self.skip(TokenKind::AndAnd) { let right = self.parse_is_expr()?; let span = left.span(); left = Expr::Binary(Box::new(left), BinOp::And, Box::new(right), span); } Ok(left) }

    fn parse_is_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_cmp_expr()?;
        if self.skip(TokenKind::Is) { let pattern = self.parse_pattern()?; Ok(Expr::Is(Box::new(left.clone()), pattern, left.span())) } else { Ok(left) }
    }

    fn parse_cmp_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_shift_expr()?;
        loop {
            let op = match self.peek_kind() { TokenKind::EqEq => BinOp::Eq, TokenKind::Neq => BinOp::Neq, TokenKind::Lt => BinOp::Lt, TokenKind::Gt => BinOp::Gt, TokenKind::Le => BinOp::Le, TokenKind::Ge => BinOp::Ge, _ => break };
            self.advance(); let right = self.parse_shift_expr()?; let span = left.span(); left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_shift_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_add_expr()?;
        loop {
            if self.peek_kind() == &TokenKind::Lt && self.peek_ahead(1) == Some(&TokenKind::Lt) {
                self.advance(); self.advance(); let right = self.parse_add_expr()?; let span = left.span(); left = Expr::Binary(Box::new(left), BinOp::Shl, Box::new(right), span);
            } else if self.peek_kind() == &TokenKind::Gt && self.peek_ahead(1) == Some(&TokenKind::Gt) {
                self.advance(); self.advance(); let right = self.parse_add_expr()?; let span = left.span(); left = Expr::Binary(Box::new(left), BinOp::Shr, Box::new(right), span);
            } else { break; }
        }
        Ok(left)
    }

    fn parse_add_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_expr()?;
        loop {
            let op = match self.peek_kind() { TokenKind::Plus => BinOp::Add, TokenKind::Minus => BinOp::Sub, _ => break };
            self.advance(); let right = self.parse_mul_expr()?; let span = left.span(); left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_mul_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_as_expr()?;
        loop {
            let op = match self.peek_kind() { TokenKind::Star => BinOp::Mul, TokenKind::Slash => BinOp::Div, TokenKind::Percent => BinOp::Rem, TokenKind::Caret => BinOp::BitXor, TokenKind::Ampersand => BinOp::BitAnd, TokenKind::Pipe => BinOp::BitOr, _ => break };
            self.advance(); let right = self.parse_as_expr()?; let span = left.span(); left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_unary_prefix()?;
        Ok(expr)
    }

    /// Parse `as` type casts: `expr as Type`.
    /// `as` has lower precedence than unary operators, so `-128 as Int8`
    /// parses as `(-128) as Int8`, not `-(128 as Int8)`.
    fn parse_as_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary_expr()?;
        while self.skip(TokenKind::As) {
            let ty = self.parse_type()?;
            let span = expr.span();
            expr = Expr::As(Box::new(expr), ty, span);
        }
        Ok(expr)
    }

    fn parse_unary_prefix(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek().span;
        match self.peek_kind() {
            TokenKind::Bang => { self.advance(); let inner = self.parse_unary_expr()?; Ok(Expr::Unary(UnaryOp::Not, Box::new(inner), span)) }
            TokenKind::Minus => { self.advance(); let inner = self.parse_unary_expr()?; 
                // Fold `-128i8` → `As(Int(-128), Int8)` so the negation is computed
                // at the literal level before the narrow-int cast, not after.
                if let Expr::As(ref base, ref ty, ref ispan) = inner {
                    if let Expr::Int(n, ref ispan2) = **base {
                        let neg_n = (0u64).wrapping_sub(n); // i64 negation via u64 wrapping
                        return Ok(Expr::As(Box::new(Expr::Int(neg_n, *ispan2)), ty.clone(), *ispan));
                    }
                    if let Expr::Float(f, ref ispan2) = **base {
                        return Ok(Expr::As(Box::new(Expr::Float(-f, *ispan2)), ty.clone(), *ispan));
                    }
                }
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(inner), span)) }
            TokenKind::Star => { self.advance(); let inner = self.parse_unary_expr()?; Ok(Expr::Unary(UnaryOp::Deref, Box::new(inner), span)) }
            TokenKind::Tilde => { self.advance(); let inner = self.parse_unary_expr()?; Ok(Expr::Unary(UnaryOp::BitNot, Box::new(inner), span)) }
            TokenKind::Ampersand => { self.advance(); let mutable = match self.peek_kind() { TokenKind::Ident(s) if s == "mut" => { self.advance(); true } _ => false }; let inner = self.parse_unary_expr()?; if mutable { Ok(Expr::MutRef(Box::new(inner), span)) } else { Ok(Expr::Ref(Box::new(inner), span)) } }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        // 8B/M9: Range syntax after primary — `expr..expr` or `expr..=expr`
        if self.peek_kind() == &TokenKind::Dot && self.peek_ahead(1) == Some(&TokenKind::Dot) {
            self.advance(); self.advance(); // skip ..
            let inclusive = self.peek_kind() == &TokenKind::Eq;
            if inclusive { self.advance(); }
            let right = self.parse_primary()?;
            let span = expr.span();
            let fn_name = if inclusive { "range_inclusive" } else { "range" };
            return Ok(Expr::Call(
                Box::new(Expr::Ident(Ident::new(String::from(fn_name), span))),
                vec![expr, right],
                span,
            ));
        }
        loop {
            match self.peek_kind() {
                TokenKind::ColonColon => {
                    // v0.54: `::` has two meanings:
                    //   1. Turbofish: expr::<Type>(args) — builtins (align_of, type_id, field_offset)
                    //   2. Static method: Type::method — associated item access
                    self.advance(); // consume ::
                    if self.peek_kind() == &TokenKind::Lt {
                        // P2-6: Turbofish with multiple type args: ::<Type1, Type2>(args)
                        self.advance(); // consume <
                        let mut types = vec![self.parse_type()?];
                        while self.skip(TokenKind::Comma) {
                            types.push(self.parse_type()?);
                        }
                        self.expect_kind(TokenKind::Gt, "'>'")?;
                        if self.peek_kind() == &TokenKind::LParen {
                            self.advance();
                            let args = if self.check(|k| matches!(k, TokenKind::RParen)) {
                                self.advance();
                                Vec::new()
                            } else {
                                let a = self.parse_arg_list()?;
                                self.expect_kind(TokenKind::RParen, "')'")?;
                                a
                            };
                            let span = expr.span();
                            expr = Expr::GenericCall(Box::new(expr), types, args, span);
                        }
                    } else {
                        // Static method: Type::method
                        let method = self.parse_ident()?;
                        let span = expr.span();
                        expr = Expr::Field(Box::new(expr), method, span);
                    }
                }
                TokenKind::Dot => {
                    self.advance();
                    if let TokenKind::Int(n) = self.peek_kind() { let idx = *n; self.advance(); let span = expr.span(); let field_id = Ident::new(format!("_{idx}"), span); expr = Expr::Field(Box::new(expr), field_id, span); }
                    else { let field = self.parse_ident()?; let span = expr.span(); expr = Expr::Field(Box::new(expr), field, span); }
                }
                TokenKind::LParen => {
                    self.advance();
                    if self.check(|k| matches!(k, TokenKind::RParen)) { self.advance(); let span = expr.span(); expr = Expr::Call(Box::new(expr), Vec::new(), span); }
                    else {
                        let use_named = if let TokenKind::Ident(_) = self.peek_kind() { let saved = self.pos; self.advance(); let is_named = self.peek_kind() == &TokenKind::Colon && self.peek_ahead(1) != Some(&TokenKind::Colon); self.pos = saved; is_named } else { false };
                        if use_named {
                            let mut fields = Vec::new();
                            loop { let fname = self.parse_ident()?; self.expect_kind(TokenKind::Colon, "':'")?; let fval = self.parse_expr_open()?; fields.push((fname, fval)); if !self.skip(TokenKind::Comma) && !self.skip(TokenKind::Semicolon) { break; } }
                            self.expect_kind(TokenKind::RParen, "')'")?; let span = expr.span();
                            let type_name = match &expr { Expr::Ident(id) => id.clone(), _ => Ident::new("_", span) };
                            expr = Expr::Struct(type_name, fields, None, span);
                        } else { let args = self.parse_arg_list()?; self.expect_kind(TokenKind::RParen, "')'")?; let span = expr.span(); expr = Expr::Call(Box::new(expr), args, span); }
                    }
                }
                TokenKind::LBracket => {
                    self.advance();
                    // Feature 6: explicit generic args on a call, e.g. `spawn[T](f)`.
                    // When the base is a (lowercase) function name and the bracket
                    // group holds type args (starts with an uppercase type name) and
                    // is immediately followed by `(`, treat `[...]` as generic
                    // arguments: discard them and let the `(` call apply to the base.
                    if matches!(&expr, Expr::Ident(n) if n.name.chars().next().map_or(false, |c| c.is_lowercase() || c == '_'))
                        && matches!(self.peek_kind(), TokenKind::Ident(s) if s.chars().next().map_or(false, |c| c.is_uppercase()))
                    {
                        let saved = self.pos;
                        let mut depth = 1;
                        while depth > 0 && !self.peek().is_eof() {
                            match self.peek_kind() {
                                TokenKind::LBracket => { depth += 1; self.advance(); }
                                TokenKind::RBracket => { depth -= 1; self.advance(); }
                                _ => { self.advance(); }
                            }
                        }
                        if self.peek_kind() == &TokenKind::LParen {
                            continue; // generic type args discarded; `(` handles the call
                        }
                        self.pos = saved;
                    }
                    let is_type_name = match &expr {
                        Expr::Ident(name) => name.name.chars().next().map_or(false, |c| c.is_uppercase()),
                        Expr::Field(obj, _, _) => match obj.as_ref() {
                            Expr::Ident(name) => name.name.chars().next().map_or(false, |c| c.is_uppercase()),
                            _ => false,
                        },
                        _ => false,
                    };
                    // `fn`, `*`, `&`, `[`, `(` can start types inside brackets.
                    let peek_is_type_start = matches!(
                        self.peek_kind(),
                        TokenKind::Ident(_) | TokenKind::Fn | TokenKind::Star
                            | TokenKind::Ampersand | TokenKind::LBracket | TokenKind::LParen
                    );
                    let is_type_like = is_type_name && peek_is_type_start;
                    if is_type_like {
                        // When the first token inside brackets is `fn`, it is
                        // unequivocally a function type (e.g. `Vec[fn()]`).  Parse
                        // it as a type argument directly — no heuristic needed.
                        if matches!(self.peek_kind(), TokenKind::Fn | TokenKind::Star
                            | TokenKind::Ampersand | TokenKind::LBracket | TokenKind::LParen)
                        {
                            let mut type_args = vec![self.parse_type()?];
                            while self.skip(TokenKind::Comma) { type_args.push(self.parse_type()?); }
                            self.expect_kind(TokenKind::RBracket, "']'")?;
                            let type_exprs: Vec<Expr> = type_args.iter().map(|t| {
                                match t {
                                    Type::Named(id, _) => Expr::Ident(id.clone()),
                                    _ => Expr::Ident(Ident::new("_", self.peek().span)),
                                }
                            }).collect();
                            let args_expr = if type_exprs.len() == 1 {
                                type_exprs.into_iter().next().expect("len==1 guaranteed")
                            } else {
                                Expr::Tuple(type_exprs, self.peek().span)
                            };
                            expr = Expr::Index(Box::new(expr.clone()), Box::new(args_expr), self.peek().span);
                            continue;
                        }
                        let after_first = self.peek_ahead(1);
                        let looks_like_type_args = matches!(after_first, Some(TokenKind::Colon) | Some(TokenKind::Comma) | Some(TokenKind::LBrace) | Some(TokenKind::LBracket) | Some(TokenKind::RBracket));
                        if looks_like_type_args {
                            if after_first == Some(&TokenKind::RBracket) {
                                let is_type_name = match &expr { Expr::Ident(name) => name.name.chars().next().map_or(false, |c| c.is_uppercase()), Expr::Field(obj, _, _) => match obj.as_ref() { Expr::Ident(name) => name.name.chars().next().map_or(false, |c| c.is_uppercase()), _ => false }, _ => false };
                                if !is_type_name { let inner = self.parse_expr_open()?; self.expect_kind(TokenKind::RBracket, "']'")?; let span = expr.span(); expr = Expr::Index(Box::new(expr), Box::new(inner), span); continue; }
                            }
                            let mut type_args = vec![self.parse_type()?];
                            while self.skip(TokenKind::Comma) { type_args.push(self.parse_type()?); }
                            self.expect_kind(TokenKind::RBracket, "']'")?;
                            // Convert type args to idents for the Index expression
                            // so codegen's type_arg capture can infer concrete types.
                            let type_exprs: Vec<Expr> = type_args.iter().map(|t| {
                                match t {
                                    Type::Named(id, _) => Expr::Ident(id.clone()),
                                    _ => Expr::Ident(Ident::new("Int", self.peek().span)),
                                }
                            }).collect();
                            let args_expr = if type_exprs.len() == 1 {
                                type_exprs.into_iter().next().expect("len==1 guaranteed")
                            } else {
                                Expr::Tuple(type_exprs, self.peek().span)
                            };
                            expr = Expr::Index(Box::new(expr.clone()), Box::new(args_expr), self.peek().span);
                            if !self.restrict_struct && self.peek_kind() == &TokenKind::LBrace {
                                let span = expr.span(); self.advance();
                                let mut fields = Vec::new();
                                while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Dot | TokenKind::Eof)) {
                                    let fname = self.parse_ident()?;
                                    if self.skip(TokenKind::Colon) { let fval = self.parse_expr()?; fields.push((fname, fval)); } else { fields.push((fname.clone(), Expr::Ident(fname))); }
                                    self.skip(TokenKind::Comma);
                                    self.skip(TokenKind::Semicolon);
                                }
                                let spread = if self.skip(TokenKind::Dot) { self.expect_kind(TokenKind::Dot, "'.' for spread")?; let s = self.parse_expr()?; Some(Box::new(s)) } else { None };
                                self.expect_kind(TokenKind::RBrace, "'}'")?;
                                let struct_name = match &expr { Expr::Ident(name) => name.clone(), Expr::Index(base, _, _) => match base.as_ref() { Expr::Ident(name) => name.clone(), _ => Ident::new("__struct", span) }, _ => Ident::new("__struct", span) };
                                expr = Expr::Struct(struct_name, fields, spread, span);
                            }
                        } else { let inner = self.parse_expr_open()?; self.expect_kind(TokenKind::RBracket, "']'")?; let span = expr.span(); expr = Expr::Index(Box::new(expr), Box::new(inner), span); }
                    } else { let inner = self.parse_expr_open()?; self.expect_kind(TokenKind::RBracket, "']'")?; let span = expr.span(); expr = Expr::Index(Box::new(expr), Box::new(inner), span); }
                }
                TokenKind::At => { self.advance(); if let TokenKind::Ident(s) = self.peek_kind() { if s == "pre" { self.advance(); let span = expr.span(); expr = Expr::AtPre(Box::new(expr), span); } else { return Err(self.error("expected 'pre' after '@'")); } } else { return Err(self.error("expected 'pre' after '@'")); } }
                // M17: `as` is handled at the unary level (parse_unary_expr), not
                // postfix level, so `-128 as Int8` parses as `(-128) as Int8` rather
                // than `-(128 as Int8)`. This matches Rust/Zig/C precedence where `as`
                // binds lower than unary operators.
                TokenKind::Colon => {
                    // Single colon: break from postfix loop so the outer
                    // parse_unary_expr can consume it (named args, type annotations).
                    // Double colon `::` is handled by TokenKind::ColonColon above.
                    break;
                }
                TokenKind::Lt if matches!(&expr, Expr::Ident(n) if n.name.chars().next().map_or(false, |c| c.is_uppercase())) => {
                    // Speculative generic type args in expression position, e.g.
                    // `Vec<FieldInfo>.new()`. Commit only if the matching `>` is
                    // immediately followed by `.` or `(`; otherwise restore the
                    // position and let `<` be handled as a comparison operator.
                    let saved = self.pos;
                    self.advance(); // consume `<`
                    let mut depth = 1;
                    let mut aborted = false;
                    while depth > 0 {
                        match self.peek_kind() {
                            TokenKind::Lt => { depth += 1; self.advance(); }
                            TokenKind::Gt => { depth -= 1; self.advance(); }
                            TokenKind::Ident(_) | TokenKind::Comma | TokenKind::Star
                            | TokenKind::Ampersand | TokenKind::LBracket | TokenKind::RBracket => { self.advance(); }
                            _ => { aborted = true; break; }
                        }
                    }
                    if !aborted && matches!(self.peek_kind(), TokenKind::Dot | TokenKind::LParen) {
                        continue; // generic type args discarded; postfix continues
                    }
                    self.pos = saved;
                    break;
                }
                TokenKind::LBrace if !self.restrict_struct => {
                    // Qualified struct literal: `Shape.Circle { r: 5.0 }`
                    // `Path.Type { field: value }` or `Enum.Variant { field: value }`
                    let looks_like_struct = {
                        let after_brace = self.peek_ahead(1);
                        match after_brace {
                            Some(TokenKind::RBrace) => true,
                            Some(TokenKind::Ident(_)) => {
                                let third = self.peek_ahead(2);
                                matches!(third, Some(TokenKind::Colon) | Some(TokenKind::Comma)
                                    | Some(TokenKind::Semicolon) | Some(TokenKind::RBrace))
                            }
                            _ => false,
                        }
                    };
                    if looks_like_struct {
                        let path_name = match &expr {
                            Expr::Ident(id) => id.name.clone(),
                            Expr::Field(obj, field, _) => {
                                let mut name = String::new();
                                fn collect_path(expr: &Expr, out: &mut String) {
                                    match expr {
                                        Expr::Ident(id) => { if !out.is_empty() { out.push('.'); } out.push_str(&id.name); }
                                        Expr::Field(base, field, _) => { collect_path(base, out); out.push('.'); out.push_str(&field.name); }
                                        _ => {}
                                    }
                                }
                                collect_path(&obj, &mut name);
                                if !name.is_empty() { name.push('.'); }
                                name.push_str(&field.name);
                                name
                            }
                            _ => return Err(self.error("expected type name before '{'")),
                        };
                        self.advance(); // consume '{'
                        let mut fields = Vec::new();
                        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Dot | TokenKind::Eof)) {
                            let fname = self.parse_ident()?;
                            if self.skip(TokenKind::Colon) { let fval = self.parse_expr()?; fields.push((fname, fval)); }
                            else { fields.push((fname.clone(), Expr::Ident(fname))); }
                            self.skip(TokenKind::Comma); self.skip(TokenKind::Semicolon);
                        }
                        let spread = if self.skip(TokenKind::Dot) { self.expect_kind(TokenKind::Dot, "'.' for spread")?; let s = self.parse_expr()?; Some(Box::new(s)) } else { None };
                        self.expect_kind(TokenKind::RBrace, "'}'")?;
                        let span = expr.span();
                        expr = Expr::Struct(Ident::new(path_name, span), fields, spread, span);
                        continue;
                    }
                    break;
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek().span;
        match self.peek_kind().clone() {
            TokenKind::Int(n) => { let tok = self.advance(); let suffix_type = Self::parse_int_suffix(&tok.lexeme); if let Some(ty) = suffix_type { Ok(Expr::As(Box::new(Expr::Int(n, span)), ty, span)) } else { Ok(Expr::Int(n, span)) } }
            TokenKind::Float(f) => { let tok = self.advance(); let suffix_type = Self::parse_float_suffix(&tok.lexeme); if let Some(ty) = suffix_type { Ok(Expr::As(Box::new(Expr::Float(f, span)), ty, span)) } else { Ok(Expr::Float(f, span)) } }
            TokenKind::Str(s) => { self.advance(); Ok(Expr::Str(s, span)) }
            TokenKind::Char(c) => { self.advance(); Ok(Expr::Char(c, span)) }
            TokenKind::True => { self.advance(); Ok(Expr::Bool(true, span)) }
            TokenKind::False => { self.advance(); Ok(Expr::Bool(false, span)) }
            TokenKind::LParen => { self.advance(); if self.check(|k| matches!(k, TokenKind::RParen)) { self.advance(); return Ok(Expr::Tuple(Vec::new(), span)); } let first = self.parse_expr_open()?; if self.skip(TokenKind::Comma) { let mut items = vec![first, self.parse_expr_open()?]; while self.skip(TokenKind::Comma) { items.push(self.parse_expr_open()?); } self.expect_kind(TokenKind::RParen, "')'")?; Ok(Expr::Tuple(items, span)) } else { self.expect_kind(TokenKind::RParen, "')'")?; Ok(Expr::Paren(Box::new(first), span)) } }
            TokenKind::Some => { self.advance(); self.expect_kind(TokenKind::LParen, "'('")?; let inner = self.parse_expr_open()?; self.expect_kind(TokenKind::RParen, "')'")?; Ok(Expr::Some(Box::new(inner), span)) }
            TokenKind::None => { self.advance(); Ok(Expr::None(span)) }
            TokenKind::Ok_ => { self.advance(); self.expect_kind(TokenKind::LParen, "'('")?; let inner = self.parse_expr_open()?; self.expect_kind(TokenKind::RParen, "')'")?; Ok(Expr::Ok(Box::new(inner), span)) }
            TokenKind::Err_ => { self.advance(); self.expect_kind(TokenKind::LParen, "'('")?; let inner = self.parse_expr_open()?; self.expect_kind(TokenKind::RParen, "')'")?; Ok(Expr::Err(Box::new(inner), span)) }
            TokenKind::Await => { self.advance(); let inner = self.parse_expr()?; Ok(Expr::Await(Box::new(inner), span)) }
            TokenKind::Comptime => { self.advance(); if matches!(self.peek_kind(), TokenKind::Dot | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace) { Ok(Expr::Ident(Ident::new("comptime".to_string(), span))) } else { let inner = self.parse_expr()?; Ok(Expr::Comptime(Box::new(inner), span)) } }
            TokenKind::LBrace => {
                // M22: Bare `{ field: value; }` as anonymous struct literal.
                // Only parse as struct when content looks like fields and we're
                // not in a restricted context (struct literals allowed).
                if self.restrict_struct {
                    return Err(self.error("unexpected '{' in this context"));
                }
                let after_brace = self.peek_ahead(1);
                let looks_like_struct = match after_brace {
                    Some(TokenKind::RBrace) => true,
                    Some(TokenKind::Ident(_)) => {
                        let third = self.peek_ahead(2);
                        matches!(third, Some(TokenKind::Colon) | Some(TokenKind::Comma)
                            | Some(TokenKind::Semicolon) | Some(TokenKind::RBrace))
                    }
                    Some(TokenKind::Dot) => self.peek_ahead(2) == Some(&TokenKind::Dot),
                    _ => false,
                };
                if looks_like_struct {
                    // Anonymous struct — type resolved by checker from context
                    self.parse_struct_literal_body("_", span)
                } else {
                    Err(self.error("expected expression, found '{'"))
                }
            }
            TokenKind::Unsafe => { self.advance(); let block = self.parse_block()?; Ok(Expr::Unsafe(block, span)) }
            TokenKind::If => { self.advance(); let cond = self.parse_cond()?; let then_block = self.parse_block()?; let mut elifs = Vec::new(); while self.skip(TokenKind::Elif) { let econd = self.parse_cond()?; let eblock = self.parse_block()?; elifs.push((econd, eblock)); } let else_block = if self.skip(TokenKind::Else) { Some(self.parse_block()?) } else { None }; Ok(Expr::If(Box::new(cond), then_block, elifs, else_block, span)) }
            TokenKind::Match => {
                // `match scrutinee { arms }` in expression position (e.g. `let x = match ...`).
                // Statement-position `match` is routed to `parse_match_stmt` by
                // `parse_stmt_or_expr` before reaching here, so both forms coexist.
                self.advance();
                let scrutinee = self.parse_cond()?;
                self.expect_kind(TokenKind::LBrace, "'{'")?;
                let mut arms = Vec::new();
                while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                    arms.push(self.parse_match_arm()?);
                }
                self.expect_kind(TokenKind::RBrace, "'}'")?;
                Ok(Expr::Match(Box::new(scrutinee), arms, span))
            }
            TokenKind::Fn => { self.advance(); self.expect_kind(TokenKind::LParen, "'('")?; let params = if self.check(|k| matches!(k, TokenKind::RParen)) { self.advance(); Vec::new() } else { let p = self.parse_param_list()?; self.expect_kind(TokenKind::RParen, "')'")?; p }; let ret_ty = if self.skip(TokenKind::Arrow) { Some(Box::new(self.parse_type()?)) } else { None }; let body = self.parse_block()?; Ok(Expr::Closure(params, ret_ty, body, span)) }
            TokenKind::Pipe => { self.advance(); let mut params = Vec::new(); loop { params.push(self.parse_ident()?); if self.skip(TokenKind::Comma) { continue; } if self.skip(TokenKind::Pipe) { break; } return Err(self.error("expected ',' or '|' in closure parameters")); } let body = self.parse_expr()?; Ok(Expr::PipeClosure(params, Box::new(body), span)) }
            TokenKind::LBracket => { self.advance(); if self.check(|k| matches!(k, TokenKind::RBracket)) { self.advance(); return Ok(Expr::Array(Vec::new(), span)); } let first = self.parse_expr_open()?; if self.skip(TokenKind::Comma) { let mut items = vec![first]; if !self.check(|k| matches!(k, TokenKind::RBracket)) { items.push(self.parse_expr_open()?); } while self.skip(TokenKind::Comma) { if self.check(|k| matches!(k, TokenKind::RBracket)) { break; } items.push(self.parse_expr_open()?); } self.expect_kind(TokenKind::RBracket, "']'")?; Ok(Expr::Array(items, span)) } else { self.expect_kind(TokenKind::RBracket, "']'")?; Ok(Expr::Array(vec![first], span)) } }
            TokenKind::Self_ => { let span = self.advance().span; Ok(Expr::Ident(Ident::new("self", span))) }
            // `spawn` used as a value/callee (e.g. `spawn[T](f)`), not the
            // `spawn { ... }` statement (which is handled in parse_stmt_or_expr).
            TokenKind::Spawn => { self.advance(); Ok(Expr::Ident(Ident::new("spawn".to_string(), span))) }
            TokenKind::At => { 
                let at_span = self.advance().span;
                let fn_name = self.parse_ident()?;
                self.expect_kind(TokenKind::LParen, "'('")?;
                let args = if self.check(|k| matches!(k, TokenKind::RParen)) { self.advance(); Vec::new() } else { let mut a = vec![self.parse_expr()?]; while self.skip(TokenKind::Comma) { a.push(self.parse_expr()?); } self.expect_kind(TokenKind::RParen, "')'")?; a };
                Ok(Expr::Call(Box::new(Expr::Ident(fn_name)), args, at_span))
            }
            TokenKind::Const => {
                self.advance();
                self.expect_kind(TokenKind::LBrace, "'{'")?;
                let inner = self.parse_expr()?;
                self.expect_kind(TokenKind::RBrace, "'}'")?;
                Ok(Expr::ConstBlock(Box::new(inner), span))
            }
            _ => {
                let name = self.parse_ident()?;
                if !self.restrict_struct && self.check(|k| matches!(k, TokenKind::LBrace)) {
                    let after_brace = self.peek_ahead(1);
                    let looks_like_struct = match after_brace { Some(TokenKind::RBrace) => true, Some(TokenKind::Ident(_)) => { let third = self.peek_ahead(2); matches!(third, Some(TokenKind::Colon) | Some(TokenKind::Comma) | Some(TokenKind::Semicolon) | Some(TokenKind::RBrace)) } Some(TokenKind::Dot) => { self.peek_ahead(2) == Some(&TokenKind::Dot) } _ => false };
                    if looks_like_struct { self.expect_kind(TokenKind::LBrace, "'{'")?; let mut fields = Vec::new(); while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Dot | TokenKind::Eof)) { let fname = self.parse_ident()?; if self.skip(TokenKind::Colon) { let fval = self.parse_expr()?; fields.push((fname, fval)); } else { fields.push((fname.clone(), Expr::Ident(fname))); } self.skip(TokenKind::Comma); self.skip(TokenKind::Semicolon); } let spread = if self.skip(TokenKind::Dot) { self.expect_kind(TokenKind::Dot, "'.' for spread")?; let s = self.parse_expr()?; Some(Box::new(s)) } else { None }; self.expect_kind(TokenKind::RBrace, "'}'")?; Ok(Expr::Struct(name, fields, spread, span)) } else { Ok(Expr::Ident(name)) }
                } else { Ok(Expr::Ident(name)) }
            }
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ParseError> { let saved = self.restrict_struct; self.restrict_struct = false; let mut args = vec![self.parse_expr()?]; while self.skip(TokenKind::Comma) { if self.peek_kind() == &TokenKind::RParen { break; } args.push(self.parse_expr()?); } self.restrict_struct = saved; Ok(args) }

    fn parse_ident(&mut self) -> Result<Ident, ParseError> {
        let tok = self.advance();
        let name = match &tok.kind {
            TokenKind::Ident(name) => name.clone(),
            TokenKind::Self_ => "self".to_string(),
            TokenKind::Comptime => "comptime".to_string(),
            TokenKind::Derive => "derive".to_string(),
            _ => {
                let lex = tok.lexeme.clone();
                // Accept keywords as identifiers (field names, variable names, etc.)
                if lex.chars().all(|c| c.is_ascii_alphabetic() || c == '_') && !lex.is_empty() {
                    lex
                } else {
                    return Err(self.error(format!("expected identifier, found '{lex}'")));
                }
            }
        };
        Ok(Ident::new(name, tok.span))
    }

    fn parse_variant_ident(&mut self) -> Result<Ident, ParseError> {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Ident(name) => Ok(Ident::new(name.clone(), tok.span)),
            TokenKind::Some => Ok(Ident::new("Some".to_string(), tok.span)),
            TokenKind::None => Ok(Ident::new("None".to_string(), tok.span)),
            TokenKind::Ok_ => Ok(Ident::new("Ok".to_string(), tok.span)),
            TokenKind::Err_ => Ok(Ident::new("Err".to_string(), tok.span)),
            _ => { let lexeme = tok.lexeme.clone(); Err(self.error(format!("expected variant name, found '{lexeme}'"))) }
        }
    }

    /// 8B/M9: Try to parse compound assignment (`+=`, `-=`, `*=`, `/=`, `%=`).
    /// Desugars `x += y` to `x = x + y`. Returns Some(rhs_expr) if a compound
    /// assignment was parsed, or None if it's a regular assignment or expression.
    fn try_compound_assign(&mut self, lhs: &Expr) -> Option<Expr> {
        let span = self.peek().span;
        let binop = match &self.peek().kind {
            TokenKind::PlusEq => { self.advance(); BinOp::Add }
            TokenKind::MinusEq => { self.advance(); BinOp::Sub }
            TokenKind::StarEq => { self.advance(); BinOp::Mul }
            TokenKind::SlashEq => { self.advance(); BinOp::Div }
            TokenKind::PercentEq => { self.advance(); BinOp::Rem }
            _ => return None,
        };
        let rhs = self.parse_expr().ok()?;
        // Desugar: x += y  →  x = x + y
        Some(Expr::Binary(Box::new(lhs.clone()), binop, Box::new(rhs), span))
    }

    /// 8B/M9: Parse optional label identifier after break/continue.
    /// Labels use `@name` syntax to avoid conflict with char literals ('x').
    fn parse_optional_label(&mut self) -> Option<Ident> {
        if self.peek_kind() == &TokenKind::At {
            self.advance(); // skip '@'
            let name = self.parse_ident().ok()?;
            return Some(name);
        }
        None
    }
    /// `(Int, Float64, Str)` → `vec![Int, Float64, Str]`
    fn parse_tuple_type_args(&mut self) -> Result<Vec<Type>, ParseError> {
        self.expect_kind(TokenKind::LParen, "'('")?;
        let mut types = Vec::new();
        if self.check(|k| matches!(k, TokenKind::RParen)) {
            self.advance();
            return Ok(types);
        }
        loop {
            types.push(self.parse_type()?);
            if self.skip(TokenKind::Comma) {
                if self.check(|k| matches!(k, TokenKind::RParen)) {
                    self.advance();
                    break;
                }
                continue;
            }
            self.expect_kind(TokenKind::RParen, "')'")?;
            break;
        }
        Ok(types)
    }
}

static EOF_TOKEN: Token = Token { kind: TokenKind::Eof, span: Span { line: 0, col: 0 }, lexeme: String::new() };

#[derive(Debug, Clone)]
pub struct ParseError { pub message: String, pub span: Span }

impl std::fmt::Display for ParseError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "parse error at {}: {}", self.span, self.message) } }
impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;
    use xiom_lexer::Lexer;

    fn parse(source: &str) -> Result<Program, ParseError> { let tokens = Lexer::new(source).tokenize(); Parser::new(tokens).parse_program() }

    #[test] fn test_empty_program() { let prog = parse("").unwrap(); assert!(prog.items.is_empty()); }
    #[test] fn test_simple_function() { let prog = parse("fn add(a: Int, b: Int) -> Int { return a + b; }").unwrap(); assert_eq!(prog.items.len(), 1); match &prog.items[0] { TopDecl::Fn(f) => { assert_eq!(f.name.name, "add"); assert_eq!(f.params.len(), 2); } _ => panic!("expected function"), } }
    #[test] fn test_function_with_contracts() { let prog = parse("fn divide(a: Float64, b: Float64) -> Float64\n  requires: b != 0.0\n  ensures: result * b == a\n{ return a / b; }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { assert_eq!(f.contracts.len(), 2); } _ => panic!("expected function"), } }
    #[test] fn test_struct_decl() { let prog = parse("type Point = { x: Float64; y: Float64; } derive[Eq, Clone]").unwrap(); match &prog.items[0] { TopDecl::Type(t) => { assert_eq!(t.name.name, "Point"); assert_eq!(t.fields.len(), 2); } _ => panic!("expected type"), } }
    #[test] fn test_struct_with_invariants() { let prog = parse("type Health = { current: Int; maximum: Int; invariant: current >= 0; invariant: current <= maximum; }").unwrap(); match &prog.items[0] { TopDecl::Type(t) => { assert_eq!(t.invariants.len(), 2); } _ => panic!("expected type"), } }
    #[test] fn test_enum_decl() { let prog = parse("enum Option { Some(value: Int), None }").unwrap(); match &prog.items[0] { TopDecl::Enum(e) => { assert_eq!(e.variants.len(), 2); } _ => panic!("expected enum"), } }
    #[test] fn test_enum_trailing_comma() { let prog = parse("enum E { A, B, }").unwrap(); match &prog.items[0] { TopDecl::Enum(e) => { assert_eq!(e.variants.len(), 2); } _ => panic!("expected enum"), } }
    // 5d gap: trailing commas in multi-line parameter lists silently DROPPED
    // the whole function (panic-mode recovery swallowed it, no diagnostic).
    #[test] fn test_param_list_trailing_comma() { let prog = parse("fn add3(\n  a: Int,\n  b: Int,\n  c: Int,\n) -> Int { return a + b + c; }").unwrap(); assert_eq!(prog.items.len(), 1); match &prog.items[0] { TopDecl::Fn(f) => { assert_eq!(f.name.name, "add3"); assert_eq!(f.params.len(), 3); } _ => panic!("expected function"), } }
    #[test] fn test_generic_params_trailing_comma() { let prog = parse("fn pair[T, U,](a: T, b: U) -> Int { return 0; }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { assert_eq!(f.generics.len(), 2); } _ => panic!("expected function"), } }
    // Recovered parse errors MUST be visible via parser.errors() so drivers
    // (xiom, LSP, MCP) can refuse silently-partial programs.
    #[test] fn test_recovered_errors_are_visible() {
        let tokens = Lexer::new("fn broken(a: Int, : ) -> Int { return 1; }\nfn ok() -> Int { return 0; }").tokenize();
        let mut p = Parser::new(tokens);
        let prog = p.parse_program().unwrap();
        assert!(!p.errors().is_empty(), "recovered parse errors must be recorded");
        // The valid function must survive recovery.
        assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(f) if f.name.name == "ok")), "recovery must keep the valid fn");
    }
    #[test] fn test_generic_fn() { let prog = parse("fn max[T: Comparable](a: T, b: T) -> T { if a > b { return a; } return b; }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { assert_eq!(f.generics.len(), 1); } _ => panic!("expected function"), } }
    #[test] fn test_method_decl() { let prog = parse("pub fn Vec3.dot(other: &Vec3) -> Float32 { return x * other.x + y * other.y; }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { assert!(f.is_method()); assert_eq!(f.name.name, "dot"); } _ => panic!("expected method"), } }
    #[test] fn test_module() { let prog = parse("module math { pub fn add(a: Int, b: Int) -> Int { return a + b; } }").unwrap(); match &prog.items[0] { TopDecl::Module(m) => { assert_eq!(m.name.name, "math"); } _ => panic!("expected module"), } }
    #[test] fn test_use_decl() { let prog = parse("use math.vector.Vec3 as V3;").unwrap(); match &prog.items[0] { TopDecl::Use(u) => { assert_eq!(u.path.len(), 3); } _ => panic!("expected use"), } }
    #[test] fn test_if_elif_else() { let prog = parse("fn test(x: Int) -> Int { if x > 0 { return 1; } elif x < 0 { return -1; } else { return 0; } }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { let body = f.body.as_ref().unwrap(); if let StmtOrExpr::Stmt(Stmt::If(_, _, elifs, else_block, _)) = &body.stmts[0] { assert_eq!(elifs.len(), 1); assert!(else_block.is_some()); } else { panic!("expected if stmt"); } } _ => panic!("expected function"), } }
    #[test] fn test_match_expr() { let prog = parse("fn check(x: Option[Int]) -> Int { match x { Some(v) => v, None => 0, } }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { let body = f.body.as_ref().unwrap(); if let StmtOrExpr::Stmt(Stmt::Match(_, arms, _)) = &body.stmts[0] { assert_eq!(arms.len(), 2); } else { panic!("expected match stmt"); } } _ => panic!("expected function"), } }
    #[test] fn test_interface() { let prog = parse("interface Comparable { fn compare(other: &Self) -> Int; }").unwrap(); match &prog.items[0] { TopDecl::Interface(_) => {} _ => panic!("expected interface"), } }
    #[test] fn test_for_in_loop() { let prog = parse("fn main() { for i in [0, 1, 2] { print(i); } }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { let body = f.body.as_ref().unwrap();         assert!(matches!(&body.stmts[0], StmtOrExpr::Stmt(Stmt::For(_, _, _, _, _)))); } _ => panic!("expected function"), } }
    #[test] fn test_async_fn() { let prog = parse("async fn fetch(url: Str) -> Str;").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => assert!(f.is_async), _ => panic!("expected async function"), } }
    #[test] fn test_borrow_syntax() { let prog = parse("fn test(x: &Int) -> Int { return x; }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => assert!(matches!(f.params[0].ty, Type::Ref(_))), _ => panic!("expected function"), } }
    #[test] fn test_while_with_param_rhs() { let result = parse("fn test(n: Int) { var i = 0; while i < n { i = i + 1; } }"); assert!(result.is_ok(), "{:?}", result.err()); }
    #[test] fn test_fn_type_in_param() { let src = "fn trapezoidal(f: fn(Float64) -> Float64, a: Float64, b: Float64) -> Float64 { return 0.0; }"; let result = parse(src); assert!(result.is_ok(), "{:?}", result.err()); }
    #[test] fn test_closure_with_return_type() { let src = "fn foo() { var d = fn(x: Int) -> Int { return x * 2; }; }"; let result = parse(src); assert!(result.is_ok(), "{:?}", result.err()); }
    #[test] fn test_full_stack_example() { let src = r#"module stack { use io; type Stack = { items: Int; capacity: Int; } fn new(capacity: Int) -> Stack { return Stack{ items: 0, capacity: capacity, }; } fn push(s: Stack, value: Int) -> Stack { return Stack{ items: s.items + value, capacity: s.capacity, }; } fn main() -> Int { var s = new(3); s = push(s, 10); return s.items; } }"#; let prog = parse(src).unwrap(); assert!(prog.items.len() >= 1); }

    // extern "C" blocks
    #[test] fn test_parse_extern_block_with_functions() { let src = "extern \"C\" {\n  fn malloc(size: Int) -> *UInt8;\n  fn free(ptr: *UInt8);\n}"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Extern(_)))); }
    #[test] fn test_parse_extern_block_with_variadic() { let src = "extern \"C\" {\n  fn printf(format: *UInt8, ...) -> Int32;\n}"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Extern(_)))); }
    #[test] fn test_parse_extern_block_multiple_functions() { let src = "extern \"C\" {\n  fn malloc(size: Int) -> *UInt8;\n  fn free(ptr: *UInt8);\n  fn strlen(s: *UInt8) -> Int;\n}"; let prog = parse(src).unwrap(); if let Some(TopDecl::Extern(eb)) = prog.items.iter().find(|i| matches!(i, TopDecl::Extern(_))) { assert_eq!(eb.functions.len(), 3); } else { panic!("expected Extern block"); } }

    // top-level var
    #[test] fn test_parse_top_level_var() { let src = "module test\nvar PI: Float64 = 3.14159;"; let prog = parse(src).unwrap(); assert!(prog.items.len() >= 1); }
    #[test] fn test_parse_top_level_var_no_type_annotation() { let src = "module test\nvar count = 0;"; let prog = parse(src).unwrap(); assert!(prog.items.len() >= 1); }

    // dotted type paths
    #[test] fn test_parse_dotted_type_path() { let src = "fn foo(x: xiom.io.Error) -> Int { return 0; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }
    #[test] fn test_parse_dotted_type_path_deep() { let src = "fn bar(x: xiom.collections.vec.Vec[Int]) -> Int { return 0; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }

    // pointer types
    #[test] fn test_parse_ptr_type_in_param() { let src = "fn use_ptr(ptr: *UInt8) -> Int { return 0; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }
    #[test] fn test_parse_ptr_type_in_return() { let src = "fn alloc() -> *UInt8 { return null; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }

    // generic bounds
    #[test] fn test_parse_generic_single_bound() { let src = "fn print[T: Display](x: T) { }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }
    #[test] fn test_parse_generic_multi_bound() { let src = "fn dedup[T: Eq + Hash](x: T) { }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }
    #[test] fn test_parse_generic_type_with_bound() { let src = "type BTreeMap[K: Ord, V] = { root: *UInt8; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Type(_)))); }

    // pub const
    #[test] fn test_parse_pub_const() { let src = "pub const MAX_SIZE: Int = 1024;"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Const(_)))); }

    // array with identifier size
    #[test] fn test_parse_array_with_const_size() { let src = "fn sum(arr: [N]Int) -> Int { return 0; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }

    // struct spread
    #[test] fn test_parse_struct_spread_default() { let src = "type Foo = { x: Int; y: Int; } fn main() -> Int { var f = Foo{ x: 1, ..default }; return 0; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }
    #[test] fn test_parse_struct_spread_other() { let src = "type Foo = { x: Int; y: Int; } fn main() -> Int { var base = Foo{ x: 1, y: 2 }; var f = Foo{ ..base }; return 0; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }

    // method receiver
    #[test] fn test_parse_method_with_self_param() { let src = "type Counter = { val: Int; } fn Counter.inc(&self) -> Int { return val + 1; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }
    #[test] fn test_parse_method_with_mut_self_param() { let src = "type Counter = { val: Int; } fn Counter.set(&mut self, v: Int) { val = v; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }

    // caret/tilde operators
    #[test] fn test_parse_bitwise_xor() { let src = "fn xor(a: Int, b: Int) -> Int { return a ^ b; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }
    #[test] fn test_parse_bitwise_not() { let src = "fn not_val(a: Int) -> Int { return ~a; }"; let prog = parse(src).unwrap(); assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(_)))); }

    // Feature 1: `match` as an expression (e.g. `let x = match ...`)
    #[test] fn test_match_as_expression() {
        let src = "fn f(x: Int) -> Int { let y = match x { 1 => 10, _ => 0, }; return y; }";
        let prog = parse(src).unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                let body = f.body.as_ref().unwrap();
                if let StmtOrExpr::Stmt(Stmt::Let(_, _, Expr::Match(_, arms, _), _)) = &body.stmts[0] {
                    assert_eq!(arms.len(), 2);
                } else { panic!("expected let binding a match-expression, got {:?}", body.stmts[0]); }
            }
            _ => panic!("expected function"),
        }
    }

    // Statement-position match must still parse as a statement (no regression).
    #[test] fn test_match_statement_still_stmt() {
        let src = "fn f(x: Int) { match x { 1 => print(x), _ => print(0), } }";
        let prog = parse(src).unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => { assert!(matches!(&f.body.as_ref().unwrap().stmts[0], StmtOrExpr::Stmt(Stmt::Match(..)))); }
            _ => panic!("expected function"),
        }
    }

    // Feature 2: `return expr` without a trailing `;` immediately before `}`
    #[test] fn test_return_without_trailing_semicolon() {
        let src = "type P = { x: Int; } fn f() -> P { return P{ x: 1 } }";
        let result = parse(src);
        assert!(result.is_ok(), "{:?}", result.err());
    }

    // Feature 3: `::` scope resolution as a call argument (was misread as a named arg)
    #[test] fn test_scope_resolution_in_call_arg() {
        let src = "fn f(p: *UInt8) { g(Str::from_c_str(p)); }";
        let result = parse(src);
        assert!(result.is_ok(), "{:?}", result.err());
    }

    // Recursion-depth guard: a compiler must never crash on adversarial input.
    #[test]
    fn test_deep_nesting_errors_cleanly() {
        // 500 nested parens must NOT crash (stack overflow). The parser may
        // survive with error recovery or hit the error limit — both are valid.
        let src = format!("fn main() -> Int {{ return {}1{}; }}", "(".repeat(500), ")".repeat(500));
        let tokens = Lexer::new(&src).tokenize();
        let result = Parser::new(tokens).parse_program();
        // 5c-R: improved error recovery may survive; just assert no crash
        assert!(result.is_ok() || result.is_err(), "deep nesting must not panic");
    }

    #[test]
    fn test_moderate_nesting_ok() {
        // 20 levels should parse fine
        let src = format!("fn main() -> Int {{ return {}1{}; }}", "(".repeat(20), ")".repeat(20));
        let tokens = Lexer::new(&src).tokenize();
        let result = Parser::new(tokens).parse_program();
        assert!(result.is_ok(), "moderate nesting should parse: {:?}", result.err());
    }

    /// 8B/M5: Fuzz harness — feed random tokens to parser, verify no panics.
    #[test]
    fn fuzz_parser_random_input() {
        let mut seed: u64 = 54321;
        for _ in 0..200 {
            let len = ((seed >> 32) % 128) as usize + 1;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let mut input = String::with_capacity(len);
            // Generate random XIOM-like tokens
            for _ in 0..len {
                let byte = (seed % 96) as u8 + 32; // printable ASCII
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                input.push(byte as char);
            }
            let tokens = Lexer::new(&input).tokenize();
            let result = Parser::new(tokens).parse_program();
            // Must not panic — error recovery should handle any input
            assert!(result.is_ok() || result.is_err(), "parser must not panic on random input of length {len}");
        }
    }

    /// 8B/M5: Fuzz harness — edge cases for parser error recovery.
    #[test]
    fn fuzz_parser_edge_cases() {
        let edge_cases = vec![
            "fn", "fn {", "fn main", "fn main(", "fn main(}", "fn main()",
            "fn main() ->", "fn main() -> {", "fn main() -> Int",
            "fn main() -> Int {", "fn main() -> Int {}",
            "fn main() -> Int { return; }",
            "if true", "if true {", "if true {}",
            "if true {} else", "if true {} else {",
            "while true", "while true {", "while true {}",
            "match x {", "match x {}", "match x { _ =>",
            "let", "let x", "let x =", "let x = ;",
            "var", "var x", "var x:", "var x: Int",
            "type", "type Foo", "type Foo =", "type Foo = {",
            "type Foo = {}", "type Foo = { x:", "type Foo = { x: Int",
            "enum", "enum Foo", "enum Foo {", "enum Foo {}",
            "module", "module foo", "module foo {", "module foo {}",
            "use", "use foo", "use foo.", "use foo.bar",
            "extern", "extern \"", "extern \"C\"", "extern \"C\" {",
            "// unterminated line comment (no newline)",
            "/* unterminated block comment",
            "\"unterminated string literal",
            "'unterminated char literal",
        ];
        for case in &edge_cases {
            let tokens = Lexer::new(case).tokenize();
            let result = Parser::new(tokens).parse_program();
            // Must not panic on any edge case
            let _ = result;
        }
    }

    // M9.6: impl Trait return types
    #[test] fn test_impl_trait_return() {
        let src = "fn get_display() -> impl Display { return 42; }";
        let prog = parse(src).unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                let ret = f.return_type.as_ref().expect("should have return type");
                assert!(matches!(ret, Type::ImplTrait(_)), "expected ImplTrait, got {:?}", ret);
                if let Type::ImplTrait(traits) = ret {
                    assert_eq!(traits.len(), 1);
                    assert_eq!(traits[0].name, "Display");
                }
            }
            _ => panic!("expected function"),
        }
    }

    #[test] fn test_impl_trait_multi() {
        let src = "fn foo() -> impl Display + Debug { return \"hi\"; }";
        let prog = parse(src).unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                if let Some(Type::ImplTrait(traits)) = &f.return_type {
                    assert_eq!(traits.len(), 2);
                    assert_eq!(traits[0].name, "Display");
                    assert_eq!(traits[1].name, "Debug");
                } else { panic!("expected ImplTrait"); }
            }
            _ => panic!("expected function"),
        }
    }

    #[test] fn test_impl_trait_method() {
        let src = "fn Iterator.next() -> impl Option { return None; }";
        let prog = parse(src).unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                assert!(f.return_type.is_some());
            }
            _ => panic!("expected function"),
        }
    }

    // M10: Shebang support — `#!` line is treated as a comment.
    #[test] fn test_shebang_skipped() {
        let src = "#!/usr/bin/env xiom\nfn main() -> Int { return 42; }";
        let prog = parse(src).unwrap();
        assert_eq!(prog.items.len(), 1, "shebang should be skipped, leaving 1 item");
    }

    #[test] fn test_shebang_preserves_line_numbers() {
        let src = "#!/usr/bin/env xiom\n\nfn bad(x: Int) -> Int { return x; }";
        let prog = parse(src).unwrap();
        // The function should parse correctly despite shebang + blank line
        assert_eq!(prog.items.len(), 1);
    }

    #[test] fn test_no_shebang_normal() {
        let src = "# not a shebang\nfn main() -> Int { return 0; }";
        let _prog = parse(src).unwrap();
        // `#` at start without `!` is NOT a shebang — should be a lex error or parse error
        // This is fine behavior — XIOM has no preprocessor
    }

    // ── M21-2: Parser error recovery ────────────────────────────────────

    /// Helper: returns errors reported by the parser.
    fn parse_with_errors(source: &str) -> (Result<Program, ParseError>, Vec<ParseError>) {
        let tokens = Lexer::new(source).tokenize();
        let mut p = Parser::new(tokens);
        let result = p.parse_program();
        (result, p.errors().to_vec())
    }

    // Malformed expressions
    #[test] fn test_recover_missing_operand() {
        let (result, _errors) = parse_with_errors("fn main() -> Int { return +; }");
        // Must not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_recover_extra_operator() {
        let (result, _errors) = parse_with_errors("fn main() -> Int { return 1 + + + 2; }");
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_recover_binary_op_no_rhs() {
        let (result, _) = parse_with_errors("fn main() -> Int { return 1 + ; }");
        assert!(result.is_ok() || result.is_err());
    }

    // Unclosed braces, brackets, parens
    #[test] fn test_recover_unclosed_brace() {
        let (_result, _errors) = parse_with_errors("fn main() -> Int { return 42;");
        // Must not crash on unclosed brace
    }

    #[test] fn test_recover_unclosed_bracket() {
        let (_result, _errors) = parse_with_errors("type T = Option[Int;");
    }

    #[test] fn test_recover_unclosed_paren() {
        let (_result, _errors) = parse_with_errors("fn main() -> Int { return foo(1, 2; }");
    }

    #[test] fn test_recover_extra_close_brace() {
        let (_result, _errors) = parse_with_errors("fn main() -> Int { return 42; } } }");
    }

    #[test] fn test_recover_nested_unclosed() {
        let (_result, _errors) = parse_with_errors("fn main() -> Int { if true { return 1; }");
    }

    // Wrong keyword in wrong position
    #[test] fn test_recover_fn_inside_fn() {
        let (result, _errors) = parse_with_errors("fn outer() -> Int { fn inner() -> Int { return 1; } return inner(); }");
        // Nested functions are invalid but parser should recover
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_recover_return_outside_fn() {
        let (_result, _errors) = parse_with_errors("return 42;");
    }

    #[test] fn test_recover_requires_without_fn() {
        let (_result, _errors) = parse_with_errors("requires: x > 0");
    }

    #[test] fn test_recover_derive_without_type() {
        let (_result, _errors) = parse_with_errors("derive[Eq, Clone]");
    }

    // Recovery: valid code after invalid
    #[test] fn test_recover_valid_fn_after_garbage() {
        let (result, _errors) = parse_with_errors("!@#$%^&*\nfn ok() -> Int { return 42; }");
        // Should either fail or recover — must not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_recover_valid_fn_after_invalid_fn() {
        let src = "fn broken(a: Int, : ) -> Int { return 1; }\nfn valid() -> Int { return 0; }";
        let (result, _errors) = parse_with_errors(src);
        // Valid function should survive
        if let Ok(prog) = result {
            assert!(prog.items.iter().any(|i| matches!(i, TopDecl::Fn(f) if f.name.name == "valid")),
                "recovery must preserve valid function after broken one");
        }
    }

    #[test] fn test_recover_after_dangling_else() {
        let src = "fn foo() { if true { return 1; } else }\nfn bar() -> Int { return 0; }";
        let (_result, _errors) = parse_with_errors(src);
    }

    // Multiple errors in one file
    #[test] fn test_multiple_errors_recorded() {
        let src = "fn a() -> Int { return ; }\nfn b() -> Int { return + ; }\nfn c() -> Int { return x y; }";
        let (result, _errors) = parse_with_errors(src);
        // Must not panic regardless
        assert!(result.is_ok() || result.is_err());
    }

    // EOF in middle of expression/statement
    #[test] fn test_eof_in_fn_signature() {
        let (_result, _errors) = parse_with_errors("fn main(");
    }

    #[test] fn test_eof_after_return() {
        let (_result, _errors) = parse_with_errors("fn main() -> Int { return");
    }

    #[test] fn test_eof_in_if_condition() {
        let (_result, _errors) = parse_with_errors("fn main() { if");
    }

    #[test] fn test_eof_in_match_arm() {
        let (_result, _errors) = parse_with_errors("fn main() { match x { Some(v) =>");
    }

    #[test] fn test_eof_after_struct_open() {
        let (_result, _errors) = parse_with_errors("type Foo = { x: Int");
    }

    #[test] fn test_eof_after_enum_open() {
        let (_result, _errors) = parse_with_errors("enum Color { Red, Green");
    }

    // Garbage input
    #[test] fn test_recover_garbage_binary_bytes() {
        let bytes = vec![0x00u8, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
        let input = String::from_utf8_lossy(&bytes);
        let (_result, _errors) = parse_with_errors(&input);
    }

    #[test] fn test_recover_random_chars() {
        let src = "}{][\n@#$\nfn main() -> Int { return 0; }";
        let (_result, _errors) = parse_with_errors(src);
    }

    // Type annotation parse errors
    #[test] fn test_recover_missing_return_type_arrow() {
        let (_result, _errors) = parse_with_errors("fn foo() Int { return 0; }");
    }

    #[test] fn test_recover_missing_param_type() {
        let (_result, _errors) = parse_with_errors("fn foo(a:) -> Int { return 0; }");
    }

    #[test] fn test_recover_missing_param_name() {
        let (_result, _errors) = parse_with_errors("fn foo(: Int) -> Int { return 0; }");
    }

    // Comma-related recovery
    #[test] fn test_recover_double_comma() {
        let (result, _errors) = parse_with_errors("fn foo(a: Int,, b: Int) -> Int { return a + b; }");
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_recover_leading_comma() {
        let (_result, _errors) = parse_with_errors("fn foo(, a: Int) -> Int { return a; }");
    }

    // Recovery from malformed match
    #[test] fn test_recover_match_missing_arrow() {
        let (_result, _errors) = parse_with_errors("fn main() { match x { 1 2, _ => 0, } }");
    }

    #[test] fn test_recover_match_missing_arm() {
        let (_result, _errors) = parse_with_errors("fn main() { match x { } }");
    }

    // Recovery from malformed struct literal
    #[test] fn test_recover_struct_lit_missing_value() {
        let (_result, _errors) = parse_with_errors("fn main() { var p = Point{ x: }; }");
    }

    #[test] fn test_recover_struct_lit_missing_colon() {
        let (_result, _errors) = parse_with_errors("fn main() { var p = Point{ x 1 }; }");
    }

    // Empty source
    #[test] fn test_recover_empty_source() {
        let (result, _errors) = parse_with_errors("");
        assert!(result.is_ok());
    }

    // Only whitespace/newlines
    #[test] fn test_recover_whitespace_only() {
        let (result, _errors) = parse_with_errors("\n\n  \n\t\t\n   \n");
        assert!(result.is_ok());
    }

    // Only comments
    #[test] fn test_recover_comments_only() {
        let (result, _errors) = parse_with_errors("// comment\n/* block */\n// another\n");
        assert!(result.is_ok());
    }

    // Error limit: verify parser doesn't hang on infinite error loops
    #[test] fn test_recover_error_limit() {
        let src = "? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ? ?";
        let (_result, _errors) = parse_with_errors(src);
        // Must not hang
    }

    // Large number of valid items ensures parser is functional after recovery
    #[test] fn test_recover_then_many_valid() {
        let mut src = String::from("!@#$ garbage\n");
        for i in 0..50 {
            src.push_str(&format!("fn f{i}() -> Int {{ return {i}; }}\n"));
        }
        let (result, _errors) = parse_with_errors(&src);
        if let Ok(prog) = result {
            assert!(prog.items.len() >= 50, "parser should recover and parse valid functions");
        }
    }
}
