// AXIOM — Parser
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! AXIOM Parser — recursive descent, LL(1), single deterministic parse path.
//! Converts the token stream into a typed AST.
//! Implements the full EBNF grammar from Section 3 of the language spec.

use axiom_ast::*;
use axiom_lexer::{Token, TokenKind};

// ============================================================================
// Parser
// ============================================================================

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
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

    // ========================================================================
    // Top-level: Program
    // ========================================================================

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

    /// Try to parse a file-level `module a.b.c` declaration at the start of the file.
    /// If the next tokens are `module ident {`, restores position and returns None
    /// (inline module).  If they are `module ident . ...` or `module ident` followed by
    /// newline/EOF, consumes the declaration and returns the module path.
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
            TokenKind::LBrace => {
                self.pos = saved;
                Ok(None)
            }
            _ => {
                if _is_pub {
                    return Err(self.error("'pub' not valid on module declarations"));
                }
                let mut path = vec![first];
                while self.skip(TokenKind::Dot) {
                    path.push(self.parse_ident()?);
                }
                Ok(Some(path))
            }
        }
    }

    /// Build a nested `ModuleDecl` tree from a dot-path and the collected items.
    /// e.g. path = [benchmark, math], items = [fn f1, fn f2]
    ///   → ModuleDecl { name: "benchmark", items: [ModuleDecl { name: "math", items: [fn f1, fn f2] }] }
    fn build_file_module_result(path: Vec<Ident>, items: Vec<TopDecl>, span: Span) -> TopDecl {
        let mut current_items = items;
        for segment in path.into_iter().rev() {
            current_items = vec![TopDecl::Module(ModuleDecl {
                name: segment,
                path: Vec::new(),
                items: current_items,
                is_file_level: false,
                source_file: None,
                span,
            })];
        }
        current_items.into_iter().next().unwrap()
    }

    fn parse_top_decl(&mut self) -> Result<TopDecl, ParseError> {
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
                if is_pub { return Err(self.error("'pub' not valid on const declarations")); }
                self.parse_const_decl()
            }
            _ => Err(self.error(format!(
                "expected declaration, found '{}'",
                self.peek().lexeme
            ))),
        }
    }

    // ========================================================================
    // Module
    // ========================================================================

    fn parse_module(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        let start = self.advance().span; // skip 'module'
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut items = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
            items.push(self.parse_top_decl()?);
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        if is_pub { return Err(self.error("'pub' not valid on module declarations")); }
        Ok(TopDecl::Module(ModuleDecl {
            name,
            path: Vec::new(),
            items,
            is_file_level: false,
            source_file: None,
            span: start,
        }))
    }

    // ========================================================================
    // Use
    // ========================================================================

    fn parse_use_decl(&mut self) -> Result<TopDecl, ParseError> {
        self.advance(); // skip 'use'
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
        let alias = if self.skip(TokenKind::As) {
            Some(self.parse_ident()?)
        } else {
            None
        };
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(TopDecl::Use(UseDecl { path, alias, glob: false, span: start }))
    }

    // ========================================================================
    // Type declarations
    // ========================================================================

    fn parse_type_decl(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        let start = self.advance().span; // skip 'type'
        let name = self.parse_ident()?;
        let generics = self.parse_optional_generic_params()?;
        self.expect_kind(TokenKind::Eq, "'='")?;

        // Type alias: type Name = ExistingType;
        if !self.check(|k| matches!(k, TokenKind::LBrace)) {
            let alias_type = self.parse_type()?;
            self.expect_kind(TokenKind::Semicolon, "';'")?;
            return Ok(TopDecl::Type(TypeDecl {
                is_pub, name, generics,
                fields: Vec::new(),
                derived_fields: Vec::new(),
                invariants: Vec::new(),
                derives: Vec::new(),
                alias: Some(Box::new(alias_type)),
                span: start,
            }));
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
                // Check if this looks like an enum variant (no colon, followed by , or } or ()
                if matches!(self.peek_kind(), TokenKind::Comma | TokenKind::RBrace) || self.peek_kind() == &TokenKind::LParen {
                    // Enum-like type definition — skip to closing brace
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
                    // Now at }, advance past it and return
                    self.advance();
                    return Ok(TopDecl::Type(TypeDecl {
                        is_pub: false,
                        name, generics, fields, derived_fields, invariants,
                        derives: vec![],
                        alias: None,
                        span: start,
                    }));
                }
                self.expect_kind(TokenKind::Colon, "':'")?;
                let ty = self.parse_type()?;
                if self.check(|k| matches!(k, TokenKind::Ident(_))) {
                    // Check for 'derived' keyword
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

        // Optional derive clause
        let derives = if self.skip(TokenKind::Derive) {
            self.parse_derive_list()?
        } else {
            Vec::new()
        };

        Ok(TopDecl::Type(TypeDecl {
            is_pub, name, generics, fields, derived_fields, invariants, derives, alias: None, span: start,
        }))
    }

    // ========================================================================
    // Enum declarations
    // ========================================================================

    fn parse_enum_decl(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        let start = self.advance().span; // skip 'enum'
        let name = self.parse_ident()?;
        let generics = self.parse_optional_generic_params()?;
        self.expect_kind(TokenKind::LBrace, "'{'")?;

        let mut variants = Vec::new();
        loop {
            let vname = self.parse_variant_ident()?;
            let fields = if self.skip(TokenKind::LParen) {
                let mut vfields = Vec::new();
                loop {
                    let fname = self.parse_ident()?;
                    self.expect_kind(TokenKind::Colon, "':'")?;
                    let fty = self.parse_type()?;
                    vfields.push(FieldDecl { name: fname, ty: fty, span: start });
                    if !self.skip(TokenKind::Comma) { break; }
                }
                self.expect_kind(TokenKind::RParen, "')'")?;
                vfields
            } else {
                Vec::new()
            };
            variants.push(EnumVariant { name: vname, fields, span: start });

            if self.skip(TokenKind::Comma) {
                if self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                    break;
                }
                continue;
            }
            if self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) { break; }
        }

        self.expect_kind(TokenKind::RBrace, "'}'")?;

        let derives = if self.skip(TokenKind::Derive) {
            self.parse_derive_list()?
        } else {
            Vec::new()
        };

        Ok(TopDecl::Enum(EnumDecl { is_pub, name, generics, variants, derives, span: start }))
    }

    // ========================================================================
    // Interface declarations
    // ========================================================================

    fn parse_interface_decl(&mut self, is_pub: bool) -> Result<TopDecl, ParseError> {
        let start = self.advance().span; // skip 'interface'
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

    // ========================================================================
    // Function declarations
    // ========================================================================

    fn parse_fn_decl(&mut self, is_pub: bool, is_async: Option<bool>) -> Result<TopDecl, ParseError> {
        let has_async = self.skip(TokenKind::Async);
        let async_flag = is_async.unwrap_or(false) || has_async;
        let start = self.peek().span;

        if !self.check(|k| matches!(k, TokenKind::Fn)) {
            return Err(self.error("expected 'fn'"));
        }
        self.advance(); // skip 'fn'

        // Check for method syntax: fn TypeName.methodName
        let first = self.parse_ident()?;
        let (receiver, name) = if self.skip(TokenKind::Dot) {
            (Some(first), self.parse_ident()?)
        } else {
            (None, first)
        };

        let generics = self.parse_optional_generic_params()?;

        // Parameters
        self.expect_kind(TokenKind::LParen, "'('")?;
        let params = if self.check(|k| matches!(k, TokenKind::RParen)) {
            self.advance();
            Vec::new()
        } else {
            let p = self.parse_param_list()?;
            self.expect_kind(TokenKind::RParen, "')'")?;
            p
        };

        // Return type
        let return_type = if self.skip(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Contracts
        let mut contracts = Vec::new();
        while self.check(|k| matches!(k, TokenKind::Requires | TokenKind::Ensures)) {
            let is_req = matches!(self.peek_kind(), TokenKind::Requires);
            self.advance();
            self.expect_kind(TokenKind::Colon, "':'")?;
            let expr = self.parse_expr()?;
            contracts.push(if is_req {
                ContractClause::Requires(expr, start)
            } else {
                ContractClause::Ensures(expr, start)
            });
        }

        // Body or signature
        let body = if self.skip(TokenKind::Semicolon) {
            None // just a signature
        } else {
            Some(self.parse_block()?)
        };

        Ok(TopDecl::Fn(FnDecl {
            is_async: async_flag,
            is_pub,
            receiver,
            name,
            generics,
            params,
            return_type,
            contracts,
            body,
            span: start,
        }))
    }

    // ========================================================================
    // Const declarations
    // ========================================================================

    fn parse_const_decl(&mut self) -> Result<TopDecl, ParseError> {
        self.advance(); // skip 'const'
        let start = self.peek().span;
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::Colon, "':'")?;
        let ty = self.parse_type()?;
        self.expect_kind(TokenKind::Eq, "'='")?;
        let value = self.parse_expr()?;
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(TopDecl::Const(ConstDecl { name, ty, value, span: start }))
    }

    // ========================================================================
    // Parameters
    // ========================================================================

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        loop {
            params.push(self.parse_param()?);
            if !self.skip(TokenKind::Comma) { break; }
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let span = self.peek().span;
        // Check for 'mut' keyword on parameters
        let name = self.parse_ident()?;
        if name.name == "mut" {
            // mut parameter — parse the real name
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
        if self.skip(TokenKind::LBracket) {
            let params = self.parse_generic_params()?;
            self.expect_kind(TokenKind::RBracket, "']'")?;
            Ok(params)
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, ParseError> {
        let mut params = Vec::new();
        loop {
            let name = self.parse_ident()?;
            let bounds = if self.skip(TokenKind::Colon) {
                self.parse_interface_refs()?
            } else {
                Vec::new()
            };
            params.push(GenericParam { name, bounds });
            if !self.skip(TokenKind::Comma) { break; }
        }
        Ok(params)
    }

    fn parse_interface_refs(&mut self) -> Result<Vec<Ident>, ParseError> {
        let mut refs = vec![self.parse_ident()?];
        while self.skip(TokenKind::Plus) {
            refs.push(self.parse_ident()?);
        }
        Ok(refs)
    }

    fn parse_derive_list(&mut self) -> Result<Vec<DeriveTrait>, ParseError> {
        self.expect_kind(TokenKind::LBracket, "'['")?;
        let mut derives = Vec::new();
        loop {
            let name = self.parse_ident()?;
            let dt = DeriveTrait::from_str(&name.name)
                .ok_or_else(|| self.error(format!("unknown derive trait: {}", name.name)))?;
            derives.push(dt);
            if !self.skip(TokenKind::Comma) { break; }
        }
        self.expect_kind(TokenKind::RBracket, "']'")?;
        Ok(derives)
    }

    // ========================================================================
    // Type parsing
    // ========================================================================

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        // Reference types: &T or &mut T
        if self.skip(TokenKind::Ampersand) {
            // Check for 'mut' keyword — mut is tokenized as an identifier
            let mutable = match self.peek_kind() {
                TokenKind::Ident(s) if s == "mut" => {
                    self.advance();
                    true
                }
                _ => false,
            };
            let base = self.parse_type_base()?;
            return Ok(if mutable { Type::MutRef(Box::new(base)) } else { Type::Ref(Box::new(base)) });
        }
        self.parse_type_base()
    }

    fn parse_type_base(&mut self) -> Result<Type, ParseError> {
        // Check for special compound types: Option[T], Result[T,E], Vec[T], etc.
        let peeked = match self.peek_kind() {
            TokenKind::Ident(s) => Some(s.clone()),
            _ => None,
        };

        if let Some(ref s) = peeked {
            match s.as_str() {
                "Option" => {
                    self.advance();
                    self.expect_kind(TokenKind::LBracket, "'['")?;
                    let inner = self.parse_type()?;
                    self.expect_kind(TokenKind::RBracket, "']'")?;
                    return Ok(Type::Option(Box::new(inner)));
                }
                "Result" => {
                    self.advance();
                    self.expect_kind(TokenKind::LBracket, "'['")?;
                    let ok = self.parse_type()?;
                    self.expect_kind(TokenKind::Comma, "','")?;
                    let err = self.parse_type()?;
                    self.expect_kind(TokenKind::RBracket, "']'")?;
                    return Ok(Type::Result(Box::new(ok), Box::new(err)));
                }
                "Vec" => {
                    self.advance();
                    self.expect_kind(TokenKind::LBracket, "'['")?;
                    let inner = self.parse_type()?;
                    self.expect_kind(TokenKind::RBracket, "']'")?;
                    return Ok(Type::Vec(Box::new(inner)));
                }
                "Slice" => {
                    self.advance();
                    self.expect_kind(TokenKind::LBracket, "'['")?;
                    let inner = self.parse_type()?;
                    self.expect_kind(TokenKind::RBracket, "']'")?;
                    return Ok(Type::Slice(Box::new(inner)));
                }
                "Map" => {
                    self.advance();
                    self.expect_kind(TokenKind::LBracket, "'['")?;
                    let k = self.parse_type()?;
                    self.expect_kind(TokenKind::Comma, "','")?;
                    let v = self.parse_type()?;
                    self.expect_kind(TokenKind::RBracket, "']'")?;
                    return Ok(Type::Map(Box::new(k), Box::new(v)));
                }
                "Set" => {
                    self.advance();
                    self.expect_kind(TokenKind::LBracket, "'['")?;
                    let inner = self.parse_type()?;
                    self.expect_kind(TokenKind::RBracket, "']'")?;
                    return Ok(Type::Set(Box::new(inner)));
                }
                _ => {}
            }
        }

        // Pointer: *T
        if self.skip(TokenKind::Star) {
            let base = self.parse_type_base()?;
            return Ok(Type::Ptr(Box::new(base)));
        }

        // Tuple: (T, U, ...)
        if self.skip(TokenKind::LParen) {
            let first = self.parse_type()?;
            if self.skip(TokenKind::Comma) {
                let mut types = vec![first, self.parse_type()?];
                while self.skip(TokenKind::Comma) {
                    types.push(self.parse_type()?);
                }
                self.expect_kind(TokenKind::RParen, "')'")?;
                return Ok(Type::Tuple(types));
            }
            self.expect_kind(TokenKind::RParen, "')'")?;
            return Ok(first); // just a parenthesized type
        }

        // Fixed array: [N]T
        if self.skip(TokenKind::LBracket) {
            if let TokenKind::Int(n) = self.peek_kind() {
                let n = *n;
                self.advance();
                self.expect_kind(TokenKind::RBracket, "']'")?;
                let inner = self.parse_type()?;
                return Ok(Type::Array(Box::new(Expr::Int(n, self.peek().span)), Box::new(inner)));
            }
            // Not a fixed array, could be an expression — fall through
            return Err(self.error("expected integer for fixed array size"));
        }

        // Function pointer type: fn(T, U) -> V
        if self.peek_kind() == &TokenKind::Fn {
            self.advance();
            self.expect_kind(TokenKind::LParen, "'('")?;
            let mut param_types = Vec::new();
            if self.peek_kind() != &TokenKind::RParen {
                param_types.push(self.parse_type()?);
                while self.skip(TokenKind::Comma) {
                    param_types.push(self.parse_type()?);
                }
            }
            self.expect_kind(TokenKind::RParen, "')'")?;
            let ret = if self.skip(TokenKind::Arrow) {
                self.parse_type()?
            } else {
                Type::Named(Ident::new("Unit", Span::new(0, 0)), vec![])
            };
            return Ok(Type::Fn(param_types, Box::new(ret)));
        }

        // Named type with optional generics: TypeName or TypeName[T, U]
        let name = self.parse_ident()?;
        let args = if self.skip(TokenKind::LBracket) {
            let mut types = vec![self.parse_type()?];
            while self.skip(TokenKind::Comma) {
                types.push(self.parse_type()?);
            }
            self.expect_kind(TokenKind::RBracket, "']'")?;
            types
        } else {
            Vec::new()
        };

        Ok(Type::Named(name, args))
    }

    // ========================================================================
    // Statements and blocks
    // ========================================================================

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.peek().span;
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut items = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
            items.push(self.parse_stmt_or_expr()?);
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        Ok(Block { stmts: items, span: start })
    }

    fn parse_stmt_or_expr(&mut self) -> Result<StmtOrExpr, ParseError> {
        match self.peek_kind() {
            TokenKind::Let => {
                let stmt = self.parse_let_stmt()?;
                Ok(StmtOrExpr::Stmt(stmt))
            }
            TokenKind::Var => {
                let stmt = self.parse_var_stmt()?;
                Ok(StmtOrExpr::Stmt(stmt))
            }
            TokenKind::Return => {
                let stmt = self.parse_return_stmt()?;
                Ok(StmtOrExpr::Stmt(stmt))
            }
            TokenKind::If => {
                let stmt = self.parse_if_stmt()?;
                Ok(StmtOrExpr::Stmt(stmt))
            }
            TokenKind::Match => {
                let stmt = self.parse_match_stmt()?;
                Ok(StmtOrExpr::Stmt(stmt))
            }
            TokenKind::While => {
                let stmt = self.parse_while_stmt()?;
                Ok(StmtOrExpr::Stmt(stmt))
            }
            TokenKind::For => {
                let stmt = self.parse_for_stmt()?;
                Ok(StmtOrExpr::Stmt(stmt))
            }
            TokenKind::Spawn => {
                let stmt = self.parse_spawn_stmt()?;
                Ok(StmtOrExpr::Stmt(stmt))
            }
            _ => {
                // Try to parse as expression statement — if followed by '=', it's an assignment
                let expr = self.parse_expr()?;
                if self.skip(TokenKind::Eq) {
                    let rhs = self.parse_expr()?;
                    let span = self.peek().span;
                    self.expect_kind(TokenKind::Semicolon, "';'")?;
                    Ok(StmtOrExpr::Stmt(Stmt::Assign(expr, rhs, span)))
                } else {
                    self.expect_kind(TokenKind::Semicolon, "';'")?;
                    Ok(StmtOrExpr::Expr(expr))
                }
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span; // skip 'let'
        if self.peek_kind() == &TokenKind::LParen {
            return self.parse_destructure(span);
        }
        let name = self.parse_ident()?;
        let ty = if self.skip(TokenKind::Colon) {
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };
        self.expect_kind(TokenKind::Eq, "'='")?;
        let value = self.parse_expr()?;
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Let(name, ty, value, span))
    }

    fn parse_var_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span; // skip 'var'
        if self.peek_kind() == &TokenKind::LParen {
            return self.parse_destructure(span);
        }
        let name = self.parse_ident()?;
        let ty = if self.skip(TokenKind::Colon) {
            Some(Box::new(self.parse_type()?))
        } else {
            None
        };
        self.expect_kind(TokenKind::Eq, "'='")?;
        let value = self.parse_expr()?;
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Var(name, ty, value, span))
    }

    fn parse_destructure(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance(); // skip (
        let mut names = Vec::new();
        names.push(self.parse_ident()?);
        while self.skip(TokenKind::Comma) {
            names.push(self.parse_ident()?);
        }
        self.expect_kind(TokenKind::RParen, "')'")?;
        self.expect_kind(TokenKind::Eq, "'='")?;
        let value = self.parse_expr()?;
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Destructure(names, value, span))
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span; // skip 'return'
        let expr = if self.check(|k| matches!(k, TokenKind::Semicolon)) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect_kind(TokenKind::Semicolon, "';'")?;
        Ok(Stmt::Return(expr, span))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span; // skip 'if'
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let mut elifs = Vec::new();
        while self.skip(TokenKind::Elif) {
            let econd = self.parse_expr()?;
            let eblock = self.parse_block()?;
            elifs.push((econd, eblock));
        }
        let else_block = if self.skip(TokenKind::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Stmt::If(cond, then_block, elifs, else_block, span))
    }

    fn parse_match_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span; // skip 'match'
        let expr = self.parse_expr()?;
        self.expect_kind(TokenKind::LBrace, "'{'")?;
        let mut arms = Vec::new();
        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
            arms.push(self.parse_match_arm()?);
        }
        self.expect_kind(TokenKind::RBrace, "'}'")?;
        Ok(Stmt::Match(expr, arms, span))
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let span = self.peek().span;
        let pattern = self.parse_pattern()?;
        let guard = if self.skip(TokenKind::If) {
            // Use parse_or_expr instead of parse_expr to prevent the guard from
            // consuming the => FatArrow via parse_imply_expr.
            Some(self.parse_or_expr()?)
        } else {
            None
        };
        self.expect_kind(TokenKind::FatArrow, "'=>'")?;
        let body = if self.check(|k| matches!(k, TokenKind::LBrace)) {
            MatchBody::Block(self.parse_block()?)
        } else if self.check(|k| matches!(k, TokenKind::If | TokenKind::Return | TokenKind::Match | TokenKind::While | TokenKind::For)) {
            let stmt = self.parse_arm_stmt()?;
            self.skip(TokenKind::Comma);
            MatchBody::Block(Block {
                stmts: vec![StmtOrExpr::Stmt(stmt)],
                span,
            })
        } else {
            let expr = self.parse_expr()?;
            self.skip(TokenKind::Comma);
            MatchBody::Expr(expr)
        };
        Ok(MatchArm { pattern, guard, body, span })
    }

    fn parse_arm_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::Return => {
                self.advance();
                let expr = if self.check(|k| matches!(k, TokenKind::Comma | TokenKind::RBrace)) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                let span = self.peek().span;
                Ok(Stmt::Return(expr, span))
            }
            TokenKind::Match => self.parse_match_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            _ => Err(self.error("expected statement in match arm")),
        }
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While(cond, body, span))
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        let var = self.parse_ident()?;
        self.expect_kind(TokenKind::In, "'in'")?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For(var, iter, body, span))
    }

    fn parse_spawn_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.advance().span;
        let body = self.parse_block()?;
        Ok(Stmt::Spawn(body, span))
    }

    // ========================================================================
    // Patterns
    // ========================================================================

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek_kind() {
            TokenKind::Underscore => {
                let span = self.advance().span;
                Ok(Pattern::Wildcard(span))
            }
            TokenKind::Some => {
                let span = self.advance().span;
                self.expect_kind(TokenKind::LParen, "'('")?;
                let inner = self.parse_pattern()?;
                self.expect_kind(TokenKind::RParen, "')'")?;
                Ok(Pattern::Some(Box::new(inner), span))
            }
            TokenKind::None => {
                let span = self.advance().span;
                Ok(Pattern::None(span))
            }
            TokenKind::Ok_ => {
                let span = self.advance().span;
                self.expect_kind(TokenKind::LParen, "'('")?;
                let inner = self.parse_pattern()?;
                self.expect_kind(TokenKind::RParen, "')'")?;
                Ok(Pattern::Ok(Box::new(inner), span))
            }
            TokenKind::Err_ => {
                let span = self.advance().span;
                self.expect_kind(TokenKind::LParen, "'('")?;
                let inner = self.parse_pattern()?;
                self.expect_kind(TokenKind::RParen, "')'")?;
                Ok(Pattern::Err(Box::new(inner), span))
            }
            TokenKind::True => {
                let span = self.advance().span;
                Ok(Pattern::Lit(Literal::Bool(true, span)))
            }
            TokenKind::False => {
                let span = self.advance().span;
                Ok(Pattern::Lit(Literal::Bool(false, span)))
            }
            TokenKind::Int(n) => {
                let n = *n;
                let span = self.advance().span;
                Ok(Pattern::Lit(Literal::Int(n, span)))
            }
            TokenKind::Str(s) => {
                let s = s.clone();
                let span = self.advance().span;
                Ok(Pattern::Lit(Literal::Str(s, span)))
            }
            _ => {
                // Identifier — could be a binding or a variant pattern
                let name = self.parse_ident()?;
                if self.skip(TokenKind::LParen) {
                    let span = name.span;
                    let mut fields = Vec::new();
                    loop {
                        let field = self.parse_ident()?;
                        // Check for labeled pattern field: name: subpattern
                        if self.skip(TokenKind::Colon) {
                            // Parse sub-pattern (e.g., _, another variant, etc.)
                            let _sub = self.parse_pattern()?;
                            // For now, just push the field name (ignore sub-pattern binding)
                        }
                        fields.push(field);
                        if !self.skip(TokenKind::Comma) { break; }
                    }
                    self.expect_kind(TokenKind::RParen, "')'")?;
                    Ok(Pattern::Variant(name, fields, span))
                } else {
                    Ok(Pattern::Ident(name))
                }
            }
        }
    }

    // ========================================================================
    // Expressions
    // ========================================================================

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_imply_expr()
    }

    fn parse_imply_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_or_expr()?;
        while self.skip(TokenKind::FatArrow) {
            let right = self.parse_or_expr()?;
            let span = left.span();
            left = Expr::Imply(Box::new(left), Box::new(right), span);
        }
        // Postfix: ?
        if self.skip(TokenKind::Question) {
            let span = left.span();
            left = Expr::Try(Box::new(left), span);
        }
        Ok(left)
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        while self.skip(TokenKind::OrOr) {
            let right = self.parse_and_expr()?;
            let span = left.span();
            left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_is_expr()?;
        while self.skip(TokenKind::AndAnd) {
            let right = self.parse_is_expr()?;
            let span = left.span();
            left = Expr::Binary(Box::new(left), BinOp::And, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_is_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_cmp_expr()?;
        if self.skip(TokenKind::Is) {
            let pattern = self.parse_pattern()?;
            Ok(Expr::Is(Box::new(left.clone()), pattern, left.span()))
        } else {
            Ok(left)
        }
    }

    fn parse_cmp_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_add_expr()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::Neq => BinOp::Neq,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_add_expr()?;
            let span = left.span();
            left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_add_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_expr()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul_expr()?;
            let span = left.span();
            left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_mul_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expr()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Rem,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary_expr()?;
            let span = left.span();
            left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek().span;
        match self.peek_kind() {
            TokenKind::Bang => {
                self.advance();
                let inner = self.parse_unary_expr()?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(inner), span))
            }
            TokenKind::Minus => {
                self.advance();
                let inner = self.parse_unary_expr()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(inner), span))
            }
            TokenKind::Ampersand => {
                self.advance();
                let mutable = match self.peek_kind() {
                    TokenKind::Ident(s) if s == "mut" => {
                        self.advance();
                        true
                    }
                    _ => false,
                };
                let inner = self.parse_unary_expr()?;
                if mutable {
                    Ok(Expr::MutRef(Box::new(inner), span))
                } else {
                    Ok(Expr::Ref(Box::new(inner), span))
                }
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_kind() {
                TokenKind::Dot => {
                    self.advance();
                    // Check for tuple index access: expr.0, expr.1, etc.
                    if let TokenKind::Int(n) = self.peek_kind() {
                        let idx = *n;
                        self.advance();
                        let span = expr.span();
                        let field_id = Ident::new(format!("_{idx}"), span);
                        expr = Expr::Field(Box::new(expr), field_id, span);
                    } else {
                        let field = self.parse_ident()?;
                        let span = expr.span();
                        expr = Expr::Field(Box::new(expr), field, span);
                    }
                }
                TokenKind::LParen => {
                    self.advance();
                    if self.check(|k| matches!(k, TokenKind::RParen)) {
                        self.advance();
                        let span = expr.span();
                        expr = Expr::Call(Box::new(expr), Vec::new(), span);
                    } else {
                        // Check for named args: Circle(radius: 2.0)
                        // If the first token after ( is an ident and the next is :, parse as named args
                        let use_named = if let TokenKind::Ident(_) = self.peek_kind() {
                            // Save position, try to peek ahead
                            let saved = self.pos;
                            self.advance(); // skip ident
                            let is_named = self.peek_kind() == &TokenKind::Colon;
                            self.pos = saved;
                            is_named
                        } else {
                            false
                        };
                        if use_named {
                            // Parse named args: field: value, field: value, ...
                            let mut fields = Vec::new();
                            loop {
                                let fname = self.parse_ident()?;
                                self.expect_kind(TokenKind::Colon, "':'")?;
                                let fval = self.parse_expr()?;
                                fields.push((fname, fval));
                                if !self.skip(TokenKind::Comma) { break; }
                            }
                            self.expect_kind(TokenKind::RParen, "')'")?;
                            let span = expr.span();
                            expr = Expr::Struct(Ident::new("_", span), fields, span);
                        } else {
                            let args = self.parse_arg_list()?;
                            self.expect_kind(TokenKind::RParen, "')'")?;
                            let span = expr.span();
                            expr = Expr::Call(Box::new(expr), args, span);
                        }
                    }
                }
                TokenKind::LBracket => {
                    // Could be index, generic args, or struct literal type args
                    self.advance();
                    let is_type_like = matches!(self.peek_kind(), TokenKind::Ident(_));
                    if is_type_like {
                        // Look ahead after the ident to distinguish type args from index expressions
                        // Type args indicators: ident follows by : (bound), , (multiple), . (method),
                        //   { (struct literal), [ (nested generic), or ] (single)
                        // Index indicators: ident followed by an operator (+, -, *, /, %, etc.)
                        let after_first = self.peek_ahead(1);
                        let looks_like_type_args = matches!(after_first,
                            Some(TokenKind::Colon) | Some(TokenKind::Comma) |
                            Some(TokenKind::LBrace) |
                            Some(TokenKind::LBracket) | Some(TokenKind::RBracket)
                        );
                        if looks_like_type_args {
                            // When ident is followed by ] only, distinguish type names (uppercase)
                            // from variable/index access (lowercase) to avoid mis-parsing
                            // v[i] { block } as type args + struct literal.
                            if after_first == Some(&TokenKind::RBracket) {
                                let is_type_name = match &expr {
                                    Expr::Ident(name) => name.name.chars().next().map_or(false, |c| c.is_uppercase()),
                                    Expr::Field(obj, _, _) => match obj.as_ref() {
                                        Expr::Ident(name) => name.name.chars().next().map_or(false, |c| c.is_uppercase()),
                                        _ => false,
                                    }
                                    _ => false,
                                };
                                if !is_type_name {
                                    let inner = self.parse_expr()?;
                                    self.expect_kind(TokenKind::RBracket, "']'")?;
                                    let span = expr.span();
                                    expr = Expr::Index(Box::new(expr), Box::new(inner), span);
                                    continue;
                                }
                            }
                            // Parse as generic type args: Name[T, U, ...] or Name[T, U]{...}
                            let mut type_args = vec![self.parse_type()?];
                            while self.skip(TokenKind::Comma) {
                                type_args.push(self.parse_type()?);
                            }
                            self.expect_kind(TokenKind::RBracket, "']'")?;
                            // Check if followed by { — struct literal with generic args
                            if self.peek_kind() == &TokenKind::LBrace {
                                let span = expr.span();
                                self.advance(); // skip {
                                let mut fields = Vec::new();
                                while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                                    let fname = self.parse_ident()?;
                                    // Support field shorthand: { field } => { field: field }
                                    if self.skip(TokenKind::Colon) {
                                        let fval = self.parse_expr()?;
                                        fields.push((fname, fval));
                                    } else {
                                        fields.push((fname.clone(), Expr::Ident(fname)));
                                    }
                                    self.skip(TokenKind::Comma);
                                }
                                self.expect_kind(TokenKind::RBrace, "'}'")?;
                                // Build Expr::Struct — extract name from preceding expr
                                let struct_name = match &expr {
                                    Expr::Ident(name) => name.clone(),
                                    _ => Ident::new("__struct", span),
                                };
                                expr = Expr::Struct(struct_name, fields, span);
                            }
                        } else {
                            // Looks like an index expression: expr[ident + ...]
                            let inner = self.parse_expr()?;
                            self.expect_kind(TokenKind::RBracket, "']'")?;
                            let span = expr.span();
                            expr = Expr::Index(Box::new(expr), Box::new(inner), span);
                        }
                    } else {
                        let inner = self.parse_expr()?;
                        self.expect_kind(TokenKind::RBracket, "']'")?;
                        let span = expr.span();
                        expr = Expr::Index(Box::new(expr), Box::new(inner), span);
                    }
                }
                TokenKind::At => {
                    self.advance();
                    // '@pre' — special postfix
                    if let TokenKind::Ident(s) = self.peek_kind() {
                        if s == "pre" {
                            self.advance();
                            let span = expr.span();
                            expr = Expr::AtPre(Box::new(expr), span);
                        } else {
                            return Err(self.error("expected 'pre' after '@'"));
                        }
                    } else {
                        return Err(self.error("expected 'pre' after '@'"));
                    }
                }
                TokenKind::As => {
                    self.advance();
                    let ty = self.parse_type()?;
                    let span = expr.span();
                    expr = Expr::As(Box::new(expr), ty, span);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.peek().span;
        match self.peek_kind().clone() {
            TokenKind::Int(n) => {
                self.advance();
                Ok(Expr::Int(n, span))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expr::Float(f, span))
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr::Str(s, span))
            }
            TokenKind::Char(c) => {
                self.advance();
                Ok(Expr::Char(c, span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true, span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false, span))
            }
            TokenKind::LParen => {
                self.advance();
                let first = self.parse_expr()?;
                if self.skip(TokenKind::Comma) {
                    let mut items = vec![first, self.parse_expr()?];
                    while self.skip(TokenKind::Comma) {
                        items.push(self.parse_expr()?);
                    }
                    self.expect_kind(TokenKind::RParen, "')'")?;
                    Ok(Expr::Tuple(items, span))
                } else {
                    self.expect_kind(TokenKind::RParen, "')'")?;
                    Ok(Expr::Paren(Box::new(first), span))
                }
            }
            TokenKind::Some => {
                self.advance();
                self.expect_kind(TokenKind::LParen, "'('")?;
                let inner = self.parse_expr()?;
                self.expect_kind(TokenKind::RParen, "')'")?;
                Ok(Expr::Some(Box::new(inner), span))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::None(span))
            }
            TokenKind::Ok_ => {
                self.advance();
                self.expect_kind(TokenKind::LParen, "'('")?;
                let inner = self.parse_expr()?;
                self.expect_kind(TokenKind::RParen, "')'")?;
                Ok(Expr::Ok(Box::new(inner), span))
            }
            TokenKind::Err_ => {
                self.advance();
                self.expect_kind(TokenKind::LParen, "'('")?;
                let inner = self.parse_expr()?;
                self.expect_kind(TokenKind::RParen, "')'")?;
                Ok(Expr::Err(Box::new(inner), span))
            }
            TokenKind::Await => {
                self.advance();
                let inner = self.parse_expr()?;
                Ok(Expr::Await(Box::new(inner), span))
            }
            TokenKind::Comptime => {
                self.advance();
                // If followed by . ( ) [ { — treat as identifier (module name), not keyword
                if matches!(self.peek_kind(), TokenKind::Dot | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace) {
                    Ok(Expr::Ident(Ident::new("comptime".to_string(), span)))
                } else {
                    let inner = self.parse_expr()?;
                    Ok(Expr::Comptime(Box::new(inner), span))
                }
            }
            TokenKind::If => {
                self.advance();
                let cond = self.parse_expr()?;
                let then_block = self.parse_block()?;
                let mut elifs = Vec::new();
                while self.skip(TokenKind::Elif) {
                    let econd = self.parse_expr()?;
                    let eblock = self.parse_block()?;
                    elifs.push((econd, eblock));
                }
                let else_block = if self.skip(TokenKind::Else) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Expr::If(Box::new(cond), then_block, elifs, else_block, span))
            }
            TokenKind::Fn => {
                self.advance();
                self.expect_kind(TokenKind::LParen, "'('")?;
                let params = if self.check(|k| matches!(k, TokenKind::RParen)) {
                    self.advance();
                    Vec::new()
                } else {
                    let p = self.parse_param_list()?;
                    self.expect_kind(TokenKind::RParen, "')'")?;
                    p
                };
                let ret_ty = if self.skip(TokenKind::Arrow) {
                    Some(Box::new(self.parse_type()?))
                } else {
                    None
                };
                let body = self.parse_block()?;
                Ok(Expr::Closure(params, ret_ty, body, span))
            }
            TokenKind::Pipe => {
                self.advance(); // skip |
                let mut params = Vec::new();
                loop {
                    params.push(self.parse_ident()?);
                    if self.skip(TokenKind::Comma) { continue; }
                    if self.skip(TokenKind::Pipe) { break; }
                    return Err(self.error("expected ',' or '|' in closure parameters"));
                }
                let body = self.parse_expr()?;
                Ok(Expr::PipeClosure(params, Box::new(body), span))
            }
            TokenKind::LBracket => {
                self.advance();
                if self.check(|k| matches!(k, TokenKind::RBracket)) {
                    self.advance();
                    return Ok(Expr::Array(Vec::new(), span));
                }
                let first = self.parse_expr()?;
                if self.skip(TokenKind::Comma) {
                    let mut items = vec![first];
                    if !self.check(|k| matches!(k, TokenKind::RBracket)) {
                        items.push(self.parse_expr()?);
                    }
                    while self.skip(TokenKind::Comma) {
                        if self.check(|k| matches!(k, TokenKind::RBracket)) {
                            break;
                        }
                        items.push(self.parse_expr()?);
                    }
                    self.expect_kind(TokenKind::RBracket, "']'")?;
                    Ok(Expr::Array(items, span))
                } else {
                    self.expect_kind(TokenKind::RBracket, "']'")?;
                    Ok(Expr::Array(vec![first], span))
                }
            }
            TokenKind::Self_ => {
                let span = self.advance().span;
                Ok(Expr::Ident(Ident::new("self", span)))
            }
            _ => {
                // Identifier — could start a struct literal or be a plain ident
                let name = self.parse_ident()?;
                // Only parse as struct literal if { follows AND the next token
                // could be a field name (identifier or }) — not a keyword like return/if/let
                if self.check(|k| matches!(k, TokenKind::LBrace)) {
                    let after_brace = self.peek_ahead(1);
                    let looks_like_struct = match after_brace {
                        Some(TokenKind::RBrace) => true,
                        Some(TokenKind::Ident(_)) => {
                            let third = self.peek_ahead(2);
                            // field: value => normal field, field, => shorthand, field } => shorthand
                            matches!(third, Some(TokenKind::Colon) | Some(TokenKind::Comma) | Some(TokenKind::RBrace))
                        }
                        _ => false,
                    };
                    if looks_like_struct {
                        self.expect_kind(TokenKind::LBrace, "'{'")?;
                        let mut fields = Vec::new();
                        while !self.check(|k| matches!(k, TokenKind::RBrace | TokenKind::Eof)) {
                            let fname = self.parse_ident()?;
                            // Support field shorthand: { field } => { field: field }
                            if self.skip(TokenKind::Colon) {
                                let fval = self.parse_expr()?;
                                fields.push((fname, fval));
                            } else {
                                fields.push((fname.clone(), Expr::Ident(fname)));
                            }
                            self.skip(TokenKind::Comma);
                        }
                        self.expect_kind(TokenKind::RBrace, "'}'")?;
                        Ok(Expr::Struct(name, fields, span))
                    } else {
                        Ok(Expr::Ident(name))
                    }
                } else {
                    Ok(Expr::Ident(name))
                }
            }
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = vec![self.parse_expr()?];
        while self.skip(TokenKind::Comma) {
            // Allow trailing comma before )
            if self.peek_kind() == &TokenKind::RParen {
                break;
            }
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn parse_ident(&mut self) -> Result<Ident, ParseError> {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Ident(name) => Ok(Ident::new(name.clone(), tok.span)),
            // Allow keyword-like identifiers that can appear as variable names, module names, etc.
            TokenKind::Self_ => Ok(Ident::new("self".to_string(), tok.span)),
            TokenKind::Comptime => Ok(Ident::new("comptime".to_string(), tok.span)),
            TokenKind::Derive => Ok(Ident::new("derive".to_string(), tok.span)),
            _ => {
                let lexeme = tok.lexeme.clone();
                Err(self.error(format!("expected identifier, found '{lexeme}'")))
            }
        }
    }

    /// Parse a variant name — accepts identifiers and keyword-like names (Some, None, Ok, Err).
    fn parse_variant_ident(&mut self) -> Result<Ident, ParseError> {
        let tok = self.advance();
        match &tok.kind {
            TokenKind::Ident(name) => Ok(Ident::new(name.clone(), tok.span)),
            TokenKind::Some => Ok(Ident::new("Some".to_string(), tok.span)),
            TokenKind::None => Ok(Ident::new("None".to_string(), tok.span)),
            TokenKind::Ok_ => Ok(Ident::new("Ok".to_string(), tok.span)),
            TokenKind::Err_ => Ok(Ident::new("Err".to_string(), tok.span)),
            _ => {
                let lexeme = tok.lexeme.clone();
                Err(self.error(format!("expected variant name, found '{lexeme}'")))
            }
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

// ============================================================================
// ParseError
// ============================================================================

static EOF_TOKEN: Token = Token {
    kind: TokenKind::Eof,
    span: Span { line: 0, col: 0 },
    lexeme: String::new(),
};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_lexer::Lexer;

    fn parse(source: &str) -> Result<Program, ParseError> {
        let tokens = Lexer::new(source).tokenize();
        Parser::new(tokens).parse_program()
    }



    #[test]
    fn test_empty_program() {
        let prog = parse("").unwrap();
        assert!(prog.items.is_empty());
    }

    #[test]
    fn test_simple_function() {
        let prog = parse("fn add(a: Int, b: Int) -> Int { return a + b; }").unwrap();
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                assert_eq!(f.name.name, "add");
                assert_eq!(f.params.len(), 2);
                assert!(!f.is_async);
                assert!(!f.is_pub);
                assert!(f.receiver.is_none());
                assert!(f.body.is_some());
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_function_with_contracts() {
        let prog = parse(
            "fn divide(a: Float64, b: Float64) -> Float64\n  requires: b != 0.0\n  ensures: result * b == a\n{ return a / b; }"
        ).unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                assert_eq!(f.contracts.len(), 2);
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_struct_decl() {
        let prog = parse("type Point = { x: Float64; y: Float64; } derive[Eq, Clone]").unwrap();
        match &prog.items[0] {
            TopDecl::Type(t) => {
                assert_eq!(t.name.name, "Point");
                assert_eq!(t.fields.len(), 2);
                assert_eq!(t.derives.len(), 2);
            }
            _ => panic!("expected type"),
        }
    }

    #[test]
    fn test_struct_with_invariants() {
        let prog = parse("type Health = { current: Int; maximum: Int; invariant: current >= 0; invariant: current <= maximum; }").unwrap();
        match &prog.items[0] {
            TopDecl::Type(t) => {
                assert_eq!(t.invariants.len(), 2);
            }
            _ => panic!("expected type"),
        }
    }

    #[test]
    fn test_enum_decl() {
        let prog = parse("enum Option { Some(value: Int), None }").unwrap();
        match &prog.items[0] {
            TopDecl::Enum(e) => {
                assert_eq!(e.variants.len(), 2);
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn test_enum_trailing_comma() {
        let prog = parse("enum E { A, B, }").unwrap();
        match &prog.items[0] {
            TopDecl::Enum(e) => {
                assert_eq!(e.variants.len(), 2);
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn test_enum_trailing_comma_single() {
        let prog = parse("enum E { A, }").unwrap();
        match &prog.items[0] {
            TopDecl::Enum(e) => {
                assert_eq!(e.variants.len(), 1);
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn test_generic_fn() {
        let prog = parse("fn max[T: Comparable](a: T, b: T) -> T { if a > b { return a; } return b; }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                assert_eq!(f.generics.len(), 1);
                assert_eq!(f.generics[0].bounds[0].name, "Comparable");
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_method_decl() {
        let prog = parse("pub fn Vec3.dot(other: &Vec3) -> Float32 { return x * other.x + y * other.y; }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                assert!(f.is_method());
                assert!(f.is_pub);
                assert_eq!(f.receiver.as_ref().unwrap().name, "Vec3");
                assert_eq!(f.name.name, "dot");
            }
            _ => panic!("expected method"),
        }
    }

    #[test]
    fn test_let_and_var() {
        let prog = parse("fn main() -> Int { let x: Int = 42; var y = 10; return x + y; }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                let body = f.body.as_ref().unwrap();
                assert_eq!(body.stmts.len(), 3);
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_if_elif_else() {
        let prog = parse("fn test(x: Int) -> Int { if x > 0 { return 1; } elif x < 0 { return -1; } else { return 0; } }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                let body = f.body.as_ref().unwrap();
                if let StmtOrExpr::Stmt(Stmt::If(_, _, elifs, else_block, _)) = &body.stmts[0] {
                    assert_eq!(elifs.len(), 1);
                    assert!(else_block.is_some());
                } else {
                    panic!("expected if stmt");
                }
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_match_expr() {
        let prog = parse("fn check(x: Option[Int]) -> Int { match x { Some(v) => v, None => 0, } }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                let body = f.body.as_ref().unwrap();
                if let StmtOrExpr::Stmt(Stmt::Match(_, arms, _)) = &body.stmts[0] {
                    assert_eq!(arms.len(), 2);
                } else {
                    panic!("expected match stmt");
                }
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_module() {
        let prog = parse("module math { pub fn add(a: Int, b: Int) -> Int { return a + b; } }").unwrap();
        match &prog.items[0] {
            TopDecl::Module(m) => {
                assert_eq!(m.name.name, "math");
                assert_eq!(m.items.len(), 1);
            }
            _ => panic!("expected module"),
        }
    }

    #[test]
    fn test_use_decl() {
        let prog = parse("use math.vector.Vec3 as V3;").unwrap();
        match &prog.items[0] {
            TopDecl::Use(u) => {
                assert_eq!(u.path.len(), 3);
                assert_eq!(u.alias.as_ref().unwrap().name, "V3");
            }
            _ => panic!("expected use"),
        }
    }

    #[test]
    fn test_interface() {
        let prog = parse("interface Comparable { fn compare(other: &Self) -> Int; }").unwrap();
        match &prog.items[0] {
            TopDecl::Interface(_) => {}
            _ => panic!("expected interface"),
        }
    }

    #[test]
    fn test_full_stack_example() {
        let src = r#"module stack { use io; type Stack = { items: Int; capacity: Int; } fn new(capacity: Int) -> Stack { return Stack{ items: 0, capacity: capacity, }; } fn push(s: Stack, value: Int) -> Stack { return Stack{ items: s.items + value, capacity: s.capacity, }; } fn main() -> Int { var s = new(3); s = push(s, 10); return s.items; } }"#;
        let prog = parse(src).unwrap();
        assert!(prog.items.len() >= 1);
    }

    #[test]
    fn test_borrow_syntax() {
        let prog = parse("fn test(x: &Int) -> Int { return x; }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                assert_eq!(f.params.len(), 1);
                assert!(matches!(f.params[0].ty, Type::Ref(_)));
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_mut_borrow_syntax() {
        let prog = parse("fn test(x: &mut Int) { x = 42; }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                assert_eq!(f.params.len(), 1);
                assert!(matches!(f.params[0].ty, Type::MutRef(_)));
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_clone_call() {
        let prog = parse("fn test(x: Int) -> Int { return x.clone(); }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => assert!(f.body.is_some()),
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_result_question() {
        let prog = parse("fn test() -> Result[Int, Str] { let x = compute()?; return Ok(x); }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => assert!(f.body.is_some()),
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_async_fn() {
        let prog = parse("async fn fetch(url: Str) -> Str;").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => assert!(f.is_async),
            _ => panic!("expected async function"),
        }
    }

    #[test]
    fn test_spawn_stmt() {
        let prog = parse("fn main() { spawn { work(); } }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                let body = f.body.as_ref().unwrap();
                assert!(matches!(&body.stmts[0], StmtOrExpr::Stmt(Stmt::Spawn(_, _))));
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_for_in_loop() {
        let prog = parse("fn main() { for i in [0, 1, 2] { print(i); } }").unwrap();
        match &prog.items[0] {
            TopDecl::Fn(f) => {
                let body = f.body.as_ref().unwrap();
                assert!(matches!(&body.stmts[0], StmtOrExpr::Stmt(Stmt::For(_, _, _, _))));
            }
            _ => panic!("expected function"),
        }
    }

    #[test]
    fn test_while_with_param_rhs() {
        let result = parse("fn test(n: Int) { var i = 0; while i < n { i = i + 1; } }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_fn_type_in_param() {
        let src = "fn trapezoidal(f: fn(Float64) -> Float64, a: Float64, b: Float64) -> Float64 { return 0.0; }";
        let result = parse(src);
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_closure_with_return_type() {
        let src = "fn foo() { var d = fn(x: Int) -> Int { return x * 2; }; }";
        let result = parse(src);
        assert!(result.is_ok(), "{:?}", result.err());
    }
}
