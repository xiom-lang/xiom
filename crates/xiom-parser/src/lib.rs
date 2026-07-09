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
    /// Set while parsing the head expression of `if`/`elif`/`while`/`for`/
    /// `match`, where `{` must begin the body block rather than a struct.
    restrict_struct: bool,
    /// Current recursion depth of the expression/type parsers. Guarded against
    /// unbounded recursion (stack overflow) on adversarial deeply-nested input.
    depth: usize,
}

/// Maximum expression/type nesting depth. A recursive-descent parser recurses
/// once per nesting level, so this bounds native stack usage. 200 is safe for
/// the ~2MB stacks used by test threads while allowing any realistic program.
const MAX_EXPR_DEPTH: usize = 32;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0, restrict_struct: false, depth: 0 }
    }

    /// Enter one level of expression/type recursion. Returns an error (instead
    /// of overflowing the stack) once nesting exceeds `MAX_EXPR_DEPTH`.
    fn enter_expr(&mut self) -> Result<(), ParseError> {
        self.depth += 1;
        if self.depth > MAX_EXPR_DEPTH {
            return Err(self.error("expression nesting too deep (max 200 levels) — simplify the expression"));
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

    #[allow(dead_code)]
    fn expect(&mut self, expected: &str) -> Result<Token, ParseError> {
        if self.peek().is_eof() {
            return Err(self.error(format!("expected {expected}, found end of file")));
        }
        let _tok = self.advance().clone();
        Ok(_tok)
    }

    fn expect_kind(&mut self, kind: TokenKind, expected: &str) -> Result<Token, ParseError> {
        if self.peek_kind() == &kind {
            Ok(self.advance().clone())
        } else {
            Err(self.error(format!(
                "expected {expected}, found {}",
                self.peek().lexeme
            )))
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
            items.push(self.parse_top_decl()?);
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
        current_items.into_iter().next().unwrap()
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
            TokenKind::Fn => self.parse_fn_decl(is_pub, None),
            TokenKind::Async => self.parse_fn_decl(is_pub, Some(true)),
            TokenKind::Const => {
                let _ = is_pub;
                self.parse_const_decl()
            }
            TokenKind::Var => self.parse_module_var(),
            TokenKind::Extern => self.parse_extern_block(),
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
            if self.check(|k| matches!(k, TokenKind::Invariant)) {
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
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut members = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
            if self.check(|k| matches!(k, TokenKind::Fn)) {
                let fn_decl = self.parse_fn_decl(false, None)?;
                match fn_decl {
                    TopDecl::Fn(f) => members.push(InterfaceMember::FnSignature(f)),
                    _ => unreachable!(),
                }
            } else {
                let fname = self.parse_ident()?;
                self.expect_kind(TokenKind::Colon, "':'")?;
                let fty = self.parse_type()?;
                self.expect_kind(TokenKind::Semicolon, "';'")?;
                members.push(InterfaceMember::Field(FieldDecl { name: fname, ty: fty, span: start }));
            }
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        Ok(TopDecl::Interface(InterfaceDecl { is_pub, name, generics, members, span: start }))
    }

    fn parse_fn_decl(&mut self, is_pub: bool, is_async: Option<bool>) -> Result<TopDecl, ParseError> {
        let has_async = self.skip(TokenKind::Async);
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
        let generics = self.parse_optional_generic_params()?;
        self.expect_kind(TokenKind::LParen, "'('")?;
        let params = if self.check(|k| matches!(k, TokenKind::RParen)) { self.advance(); Vec::new() } else { let p = self.parse_param_list()?; self.expect_kind(TokenKind::RParen, "')'")?; p };
        let return_type = if self.skip(TokenKind::Arrow) { Some(self.parse_type()?) } else { None };
        let mut contracts = Vec::new();
        while self.check(|k| matches!(k, TokenKind::Requires | TokenKind::Ensures)) {
            let is_req = matches!(self.peek_kind(), TokenKind::Requires);
            self.advance();
            self.expect_kind(TokenKind::Colon, "':'")?;
            let expr = self.parse_expr()?;
            contracts.push(if is_req { ContractClause::Requires(expr, start) } else { ContractClause::Ensures(expr, start) });
        }
        let body = if self.skip(TokenKind::Semicolon) { None } else if self.check(|k| matches!(k, TokenKind::LBrace)) { Some(self.parse_block()?) } else { None };
        Ok(TopDecl::Fn(FnDecl { is_async: async_flag, is_pub, receiver, name, generics, params, return_type, contracts, body, span: start }))
    }

    fn parse_const_decl(&mut self) -> Result<TopDecl, ParseError> {
        self.advance();
        let start = self.peek().span;
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::Colon, "':'")?;
        let ty = self.parse_type()?;
        self.expect_kind(TokenKind::Eq, "'='")?;
        let value = self.parse_expr()?;
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(TopDecl::Const(ConstDecl { name, ty, value, is_mut: false, span: start }))
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
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(TopDecl::Const(ConstDecl {
            name,
            ty: ty.unwrap_or(Type::Named(Ident::new("_", span), vec![])),
            value,
            is_mut: true,
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
                params.push(xiom_ast::Param { name: pname, ty: pty, span: pspan });
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
        loop { params.push(self.parse_param()?); if !self.skip(TokenKind::Comma) { break; } }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let span = self.peek().span;
        if self.peek_kind() == &TokenKind::Ampersand {
            self.advance();
            if self.peek().lexeme == "mut" {
                self.advance();
            }
            let name = self.parse_ident()?;
            return Ok(Param { name, ty: Type::Named(Ident::new("Self", span), vec![]), span });
        }
        if self.peek_kind() == &TokenKind::Self_ {
            let name = self.parse_ident()?;
            return Ok(Param { name, ty: Type::Named(Ident::new("Self", span), vec![]), span });
        }
        let name = self.parse_ident()?;
        if name.name == "mut" {
            let name = self.parse_ident()?;
            self.expect_kind(TokenKind::Colon, "':'")?;
            let ty = self.parse_type()?;
            Ok(Param { name, ty, span })
        } else {
            self.expect_kind(TokenKind::Colon, "':'")?;
            let ty = self.parse_type()?;
            Ok(Param { name, ty, span })
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
        // Skip 'dyn' keyword (dynamic dispatch marker): `dyn Trait` parses as `Trait`.
        if let TokenKind::Ident(s) = self.peek_kind() { if s == "dyn" { self.advance(); } }
        let peeked = match self.peek_kind() { TokenKind::Ident(s) => Some(s.clone()), _ => None };
        if let Some(ref s) = peeked {
            match s.as_str() {
                "Option" => { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let inner = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Option(Box::new(inner))); }
                "Result" => { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let ok = self.parse_type()?; self.expect_kind(TokenKind::Comma, "','")?; let err = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Result(Box::new(ok), Box::new(err))); }
                "Vec" => { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let inner = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Vec(Box::new(inner))); }
                "Slice" => { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let inner = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Slice(Box::new(inner))); }
                "Map" => { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let k = self.parse_type()?; self.expect_kind(TokenKind::Comma, "','")?; let v = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Map(Box::new(k), Box::new(v))); }
                "Set" => { self.advance(); if !self.skip(TokenKind::LBracket) { self.expect_kind(TokenKind::Lt, "'<'")?; } let inner = self.parse_type()?; if !self.skip(TokenKind::RBracket) { self.expect_kind(TokenKind::Gt, "'>'")?; } return Ok(Type::Set(Box::new(inner))); }
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
            TokenKind::Break => { let span = self.advance().span; self.skip(TokenKind::Semicolon); Ok(StmtOrExpr::Stmt(Stmt::Break(span))) }
            TokenKind::Continue => { let span = self.advance().span; self.skip(TokenKind::Semicolon); Ok(StmtOrExpr::Stmt(Stmt::Continue(span))) }
            TokenKind::If => { let stmt = self.parse_if_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::Match => { let stmt = self.parse_match_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::While => { let stmt = self.parse_while_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::For => { let stmt = self.parse_for_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::Spawn if self.peek_ahead(1) == Some(&TokenKind::LBrace) => { let stmt = self.parse_spawn_stmt()?; Ok(StmtOrExpr::Stmt(stmt)) }
            TokenKind::Ident(s) if s == "loop" && self.peek_ahead(1) == Some(&TokenKind::LBrace) => {
                let span = self.advance().span; // consume 'loop'
                let body = self.parse_block()?;
                // Desugar `loop { ... }` to `while true { ... }`
                Ok(StmtOrExpr::Stmt(Stmt::While(Expr::Bool(true, span), body, span)))
            }
            _ => {
                let expr = self.parse_expr()?;
                if self.skip(TokenKind::Eq) { let rhs = self.parse_expr()?; let span = self.peek().span; self.expect_kind(TokenKind::Semicolon, "';'")?; Ok(StmtOrExpr::Stmt(Stmt::Assign(expr, rhs, span))) }
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
        let value = if self.skip(TokenKind::Eq) { self.parse_expr()? } else { Expr::Int(0, span) };
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Let(name, ty, value, span))
    }

    fn parse_var_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        if self.peek_kind() == &TokenKind::LParen { return self.parse_destructure(span); }
        let name = self.parse_ident()?;
        let ty = if self.skip(TokenKind::Colon) { Some(Box::new(self.parse_type()?)) } else { None };
        // Allow `var x: Type;` without explicit initialization (defaults to zero)
        let value = if self.skip(TokenKind::Eq) { self.parse_expr()? } else { Expr::Int(0, span) };
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Var(name, ty, value, span))
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
        let span = self.advance().span; let cond = self.parse_cond()?; let then_block = self.parse_block()?;
        let mut elifs = Vec::new();
        while self.skip(TokenKind::Elif) { let econd = self.parse_cond()?; let eblock = self.parse_block()?; elifs.push((econd, eblock)); }
        let else_block = if self.skip(TokenKind::Else) { Some(self.parse_block()?) } else { None };
        Ok(Stmt::If(cond, then_block, elifs, else_block, span))
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
        } else { let expr = self.parse_expr()?; self.skip(TokenKind::Comma); self.skip(TokenKind::Semicolon); MatchBody::Expr(expr) };
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

    fn parse_while_stmt(&mut self) -> Result<Stmt, ParseError> { let span = self.advance().span; let cond = self.parse_cond()?; let body = self.parse_block()?; Ok(Stmt::While(cond, body, span)) }
    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> { let span = self.advance().span; let var = self.parse_ident()?; self.expect_kind(TokenKind::In, "'in'")?; let iter = self.parse_cond()?; let body = self.parse_block()?; Ok(Stmt::For(var, iter, body, span)) }
    fn parse_spawn_stmt(&mut self) -> Result<Stmt, ParseError> { let span = self.advance().span; let body = self.parse_block()?; Ok(Stmt::Spawn(body, span)) }

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
            TokenKind::LParen => {
                let span = self.advance().span;
                if self.skip(TokenKind::RParen) {
                    // Unit pattern `()` — treat as wildcard (unit has a single value)
                    return Ok(Pattern::Wildcard(span));
                }
                let inner = self.parse_pattern()?;
                self.expect_kind(TokenKind::RParen, "')'")?;
                Ok(inner)
            }
            _ => {
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
                        let field = self.parse_ident()?;
                        if self.skip(TokenKind::Colon) { let sub = self.parse_pattern()?; match &sub { Pattern::Ident(binding) => fields.push(binding.clone()), _ => fields.push(Ident::new("_", field.span)) } }
                        else { fields.push(field); }
                        if !self.skip(TokenKind::Comma) { break; }
                    }
                    self.expect_kind(TokenKind::RParen, "')'")?;
                    Ok(Pattern::Variant(name, fields, span))
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
        let mut left = self.parse_unary_expr()?;
        loop {
            let op = match self.peek_kind() { TokenKind::Star => BinOp::Mul, TokenKind::Slash => BinOp::Div, TokenKind::Percent => BinOp::Rem, TokenKind::Caret => BinOp::BitXor, TokenKind::Ampersand => BinOp::BitAnd, TokenKind::Pipe => BinOp::BitOr, _ => break };
            self.advance(); let right = self.parse_unary_expr()?; let span = left.span(); left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary_prefix()?;
        // Postfix: as Type cast
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
            TokenKind::Minus => { self.advance(); let inner = self.parse_unary_expr()?; Ok(Expr::Unary(UnaryOp::Neg, Box::new(inner), span)) }
            TokenKind::Star => { self.advance(); let inner = self.parse_unary_expr()?; Ok(Expr::Unary(UnaryOp::Deref, Box::new(inner), span)) }
            TokenKind::Tilde => { self.advance(); let inner = self.parse_unary_expr()?; Ok(Expr::Unary(UnaryOp::BitNot, Box::new(inner), span)) }
            TokenKind::Ampersand => { self.advance(); let mutable = match self.peek_kind() { TokenKind::Ident(s) if s == "mut" => { self.advance(); true } _ => false }; let inner = self.parse_unary_expr()?; if mutable { Ok(Expr::MutRef(Box::new(inner), span)) } else { Ok(Expr::Ref(Box::new(inner), span)) } }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_kind() {
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
                    let is_type_like = is_type_name && matches!(self.peek_kind(), TokenKind::Ident(_));
                    if is_type_like {
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
                                let struct_name = match &expr { Expr::Ident(name) => name.clone(), _ => Ident::new("__struct", span) };
                                expr = Expr::Struct(struct_name, fields, spread, span);
                            }
                        } else { let inner = self.parse_expr_open()?; self.expect_kind(TokenKind::RBracket, "']'")?; let span = expr.span(); expr = Expr::Index(Box::new(expr), Box::new(inner), span); }
                    } else { let inner = self.parse_expr_open()?; self.expect_kind(TokenKind::RBracket, "']'")?; let span = expr.span(); expr = Expr::Index(Box::new(expr), Box::new(inner), span); }
                }
                TokenKind::At => { self.advance(); if let TokenKind::Ident(s) = self.peek_kind() { if s == "pre" { self.advance(); let span = expr.span(); expr = Expr::AtPre(Box::new(expr), span); } else { return Err(self.error("expected 'pre' after '@'")); } } else { return Err(self.error("expected 'pre' after '@'")); } }
                TokenKind::As => { self.advance(); let ty = self.parse_type()?; let span = expr.span(); expr = Expr::As(Box::new(expr), ty, span); }
                TokenKind::Colon if self.peek_ahead(1) == Some(&TokenKind::Colon) => {
                    self.advance(); self.advance(); // consume `::`
                    let method = self.parse_ident()?;
                    // Treat `Type::method` as a static-method / associated-item access.
                    // The following `(` call (if any) is handled by the LParen arm.
                    let span = expr.span();
                    expr = Expr::Field(Box::new(expr), method, span);
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
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek().span;
        match self.peek_kind().clone() {
            TokenKind::Int(n) => { self.advance(); Ok(Expr::Int(n, span)) }
            TokenKind::Float(f) => { self.advance(); Ok(Expr::Float(f, span)) }
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
    #[test] fn test_generic_fn() { let prog = parse("fn max[T: Comparable](a: T, b: T) -> T { if a > b { return a; } return b; }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { assert_eq!(f.generics.len(), 1); } _ => panic!("expected function"), } }
    #[test] fn test_method_decl() { let prog = parse("pub fn Vec3.dot(other: &Vec3) -> Float32 { return x * other.x + y * other.y; }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { assert!(f.is_method()); assert_eq!(f.name.name, "dot"); } _ => panic!("expected method"), } }
    #[test] fn test_module() { let prog = parse("module math { pub fn add(a: Int, b: Int) -> Int { return a + b; } }").unwrap(); match &prog.items[0] { TopDecl::Module(m) => { assert_eq!(m.name.name, "math"); } _ => panic!("expected module"), } }
    #[test] fn test_use_decl() { let prog = parse("use math.vector.Vec3 as V3;").unwrap(); match &prog.items[0] { TopDecl::Use(u) => { assert_eq!(u.path.len(), 3); } _ => panic!("expected use"), } }
    #[test] fn test_if_elif_else() { let prog = parse("fn test(x: Int) -> Int { if x > 0 { return 1; } elif x < 0 { return -1; } else { return 0; } }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { let body = f.body.as_ref().unwrap(); if let StmtOrExpr::Stmt(Stmt::If(_, _, elifs, else_block, _)) = &body.stmts[0] { assert_eq!(elifs.len(), 1); assert!(else_block.is_some()); } else { panic!("expected if stmt"); } } _ => panic!("expected function"), } }
    #[test] fn test_match_expr() { let prog = parse("fn check(x: Option[Int]) -> Int { match x { Some(v) => v, None => 0, } }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { let body = f.body.as_ref().unwrap(); if let StmtOrExpr::Stmt(Stmt::Match(_, arms, _)) = &body.stmts[0] { assert_eq!(arms.len(), 2); } else { panic!("expected match stmt"); } } _ => panic!("expected function"), } }
    #[test] fn test_interface() { let prog = parse("interface Comparable { fn compare(other: &Self) -> Int; }").unwrap(); match &prog.items[0] { TopDecl::Interface(_) => {} _ => panic!("expected interface"), } }
    #[test] fn test_for_in_loop() { let prog = parse("fn main() { for i in [0, 1, 2] { print(i); } }").unwrap(); match &prog.items[0] { TopDecl::Fn(f) => { let body = f.body.as_ref().unwrap(); assert!(matches!(&body.stmts[0], StmtOrExpr::Stmt(Stmt::For(_, _, _, _)))); } _ => panic!("expected function"), } }
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
        // 500 nested parens must return Err, NOT stack overflow
        let src = format!("fn main() -> Int {{ return {}1{}; }}", "(".repeat(500), ")".repeat(500));
        let tokens = Lexer::new(&src).tokenize();
        let result = Parser::new(tokens).parse_program();
        assert!(result.is_err(), "deep nesting should error cleanly, not crash");
    }

    #[test]
    fn test_moderate_nesting_ok() {
        // 20 levels should parse fine
        let src = format!("fn main() -> Int {{ return {}1{}; }}", "(".repeat(20), ")".repeat(20));
        let tokens = Lexer::new(&src).tokenize();
        let result = Parser::new(tokens).parse_program();
        assert!(result.is_ok(), "moderate nesting should parse: {:?}", result.err());
    }
}
