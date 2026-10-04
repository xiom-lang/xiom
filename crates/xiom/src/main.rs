// XIOM Programming Language
// -----------------------------------------------------------------------
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
// -----------------------------------------------------------------------

//! XIOM Compiler CLI
//! Usage:
//!   xiom <source.xi>                         print LLVM IR to stdout
//!   xiom --emit-ir <source.xi>               print LLVM IR to stdout
//!   xiom -o <output> <source.xi>             compile to native binary
//!   xiom --target wasm <source.xi>           compile to WASM
//!   xiom --target wasm -o out.wasm <src.xi>  compile to WASM with name
//!   xiom --run <source.xi>                   compile and run, print exit code
//!   xiom --diagnostics=json <source.xi>      JSON-structured compiler output
//!   xiom --dump-contracts <source.xi>        emit contract index as JSON
//!   xiom --dump-tokens <source.xi>           canonical token dump (selfhost parity gate)
//!   xiom --dump-ast <source.xi>              canonical AST dump (selfhost parity gate)
//!   xiom --dump-check <source.xi>            canonical checker dump (selfhost parity gate)
//!   xiom --sandbox <source.xi>                safety audit report (text)
//!   xiom --sandbox=strict <source.xi>         block compilation on HIGH findings
//!   xiom --sandbox-report=json <source.xi>    safety audit as JSON

use std::env;
use std::process;
use std::time::Duration;

use xiom::{self, compile, CompileConfig, Target, resolve_source_files};

mod cli;

/// Call compile() and exit on failure -- all process::exit calls are confined to this binary.
fn compile_or_exit(config: &CompileConfig, sources: &[String]) {
    if let Err(errors) = compile(config, sources) {
        for e in &errors {
            eprintln!("error: {e}");
        }
        process::exit(1);
    }
}

/// M10: Watch mode for `xiom run --watch <file>`.
/// C22 (playground relay): source dirs for `xiom run` / `--watch`. The run
/// path compiles a %TEMP%/xiom_run copy, so the checker's catalog would
/// otherwise never see the script's own directory (sibling modules/packages
/// failed with undefined-variable errors while `xiom --check` worked).
/// Mirrors compile()'s parent + grandparent-with-.xi-guard walk.
fn run_script_source_dirs(script: &str) -> Vec<String> {
    let mut dirs = Vec::new();
    let path = std::path::Path::new(script);
    let Some(parent) = path.parent() else { return dirs; };
    let parent = if parent.as_os_str().is_empty() {
        std::path::Path::new(".")
    } else {
        parent
    };
    dirs.push(parent.to_string_lossy().to_string());
    if let Some(grandparent) = parent.parent() {
        if !grandparent.as_os_str().is_empty()
            && std::fs::read_dir(grandparent).map_or(false, |entries| {
                entries.flatten().any(|e| {
                    e.path().extension().map_or(false, |ext| ext == "xi")
                })
            })
        {
            dirs.push(grandparent.to_string_lossy().to_string());
        }
    }
    dirs
}

/// Polls the source file every 500ms and re-runs on changes.
fn run_script_watch(path: &str) {
    let get_mtime = || std::fs::metadata(path).ok().and_then(|m| m.modified().ok());
    let mut last_mtime = get_mtime();

    eprintln!("[WATCH] Monitoring '{path}' -- press Ctrl+C to stop");
    // Unique per watch session: parallel watchers must not collide on a
    // shared `_script_watch.xi` in the temp dir.
    let watch_suffix = xiom::jit::rand_suffix();

    loop {
        let current_mtime = get_mtime();
        if current_mtime != last_mtime {
            if current_mtime.is_some() {
                // Debounce: wait 200ms for the file write to complete
                std::thread::sleep(std::time::Duration::from_millis(200));
                eprintln!("\n[WATCH] File changed, re-running...");
                let source = match std::fs::read_to_string(path) {
                    Ok(s) => s,
                    Err(e) => { eprintln!("error: cannot read '{path}': {e}"); process::exit(1); }
                };
                let wrapped = xiom::implicit_main::wrap_implicit_main(&source);
                let tmp_dir = std::env::temp_dir().join("xiom_run");
                let _ = std::fs::create_dir_all(&tmp_dir);
                let tmp_src = tmp_dir.join(format!("_script_watch_{watch_suffix:x}.xi"));
                let tmp_out = tmp_dir.join(format!("_script_watch_{watch_suffix:x}.exe"));
                std::fs::write(&tmp_src, &wrapped).unwrap_or_else(|e| {
                    eprintln!("error: write temp: {e}"); process::exit(1);
                });
                let config = CompileConfig {
                    output_file: Some(tmp_out.to_string_lossy().to_string()),
                    do_run: true,
                    // C22: same source-dir hint as the non-watch run path.
                    extra_source_dirs: run_script_source_dirs(path),
                    ..CompileConfig::default()
                };
                if let Err(errors) = compile(&config, &[tmp_src.to_string_lossy().to_string()]) {
                    for e in &errors { eprintln!("error: {e}"); }
                }
            }
            last_mtime = current_mtime;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

use xiom_lexer::{Lexer, Token, TokenKind};
use xiom_parser::Parser;
use xiom_codegen::sandbox::SafetyAuditor;

// ============================================================================
// --dump-tokens: canonical token dump for the selfhost Phase 1 parity gate
// ============================================================================
//
// Line format (byte-stable; mirrored by selfhost/src/lexer.xi dump_tokens):
//
//   {line}:{col}:{byte_start}:{byte_end} {TAG}[ {PAYLOAD}]
//
// TAG is the TokenKind variant name. PAYLOAD is lowercase hex, exact:
//   Ident:  hex of the identifier text bytes
//   Int:    16 hex digits (u64 value, zero-padded)
//   BigInt: 32 hex digits (u128 value: high u64 then low u64)
//   Float:  hex of the token lexeme bytes (float VALUE parity is deferred:
//           the selfhost side has no correctly-rounded decimal->f64 parser
//           or i64<->f64 bitcast intrinsic yet; documented in the Phase 1
//           checklist)
//   Str:    hex of the DECODED string bytes
//   Char:   hex codepoint, no padding
//   Error:  hex of the message bytes
//   keywords/operators/Eof: no payload.

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0F) as usize] as char);
    }
    out
}

/// Variant name for payload-less token kinds (exhaustive over the enum).
fn bare_token_kind(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Let => "Let",
        TokenKind::Var => "Var",
        TokenKind::Const => "Const",
        TokenKind::Fn => "Fn",
        TokenKind::Return => "Return",
        TokenKind::Break => "Break",
        TokenKind::Continue => "Continue",
        TokenKind::If => "If",
        TokenKind::Elif => "Elif",
        TokenKind::Else => "Else",
        TokenKind::Match => "Match",
        TokenKind::While => "While",
        TokenKind::For => "For",
        TokenKind::In => "In",
        TokenKind::Spawn => "Spawn",
        TokenKind::Await => "Await",
        TokenKind::Comptime => "Comptime",
        TokenKind::Asm => "Asm",
        TokenKind::Defer => "Defer",
        TokenKind::Move => "Move",
        TokenKind::Module => "Module",
        TokenKind::Use => "Use",
        TokenKind::Pub => "Pub",
        TokenKind::As => "As",
        TokenKind::Type => "Type",
        TokenKind::Enum => "Enum",
        TokenKind::Interface => "Interface",
        TokenKind::Derive => "Derive",
        TokenKind::Impl => "Impl",
        TokenKind::True => "True",
        TokenKind::False => "False",
        TokenKind::Self_ => "Self_",
        TokenKind::Some => "Some",
        TokenKind::None => "None",
        TokenKind::Ok_ => "Ok_",
        TokenKind::Err_ => "Err_",
        TokenKind::Unsafe => "Unsafe",
        TokenKind::Extern => "Extern",
        TokenKind::Is => "Is",
        TokenKind::Dot => "Dot",
        TokenKind::Comma => "Comma",
        TokenKind::Semicolon => "Semicolon",
        TokenKind::Colon => "Colon",
        TokenKind::ColonColon => "ColonColon",
        TokenKind::LParen => "LParen",
        TokenKind::RParen => "RParen",
        TokenKind::LBrace => "LBrace",
        TokenKind::RBrace => "RBrace",
        TokenKind::LBracket => "LBracket",
        TokenKind::RBracket => "RBracket",
        TokenKind::At => "At",
        TokenKind::Arrow => "Arrow",
        TokenKind::FatArrow => "FatArrow",
        TokenKind::Question => "Question",
        TokenKind::Plus => "Plus",
        TokenKind::Minus => "Minus",
        TokenKind::Star => "Star",
        TokenKind::Slash => "Slash",
        TokenKind::Percent => "Percent",
        TokenKind::Caret => "Caret",
        TokenKind::Tilde => "Tilde",
        TokenKind::Bang => "Bang",
        TokenKind::Amp => "Amp",
        TokenKind::Pipe => "Pipe",
        TokenKind::Ampersand => "Ampersand",
        TokenKind::Eq => "Eq",
        TokenKind::EqEq => "EqEq",
        TokenKind::Neq => "Neq",
        TokenKind::Lt => "Lt",
        TokenKind::Gt => "Gt",
        TokenKind::Le => "Le",
        TokenKind::Ge => "Ge",
        TokenKind::AndAnd => "AndAnd",
        TokenKind::OrOr => "OrOr",
        TokenKind::PlusEq => "PlusEq",
        TokenKind::MinusEq => "MinusEq",
        TokenKind::StarEq => "StarEq",
        TokenKind::SlashEq => "SlashEq",
        TokenKind::PercentEq => "PercentEq",
        TokenKind::DotDot => "DotDot",
        TokenKind::DotDotEq => "DotDotEq",
        TokenKind::Underscore => "Underscore",
        TokenKind::Hash => "Hash",
        TokenKind::Eof => "Eof",
        // Payload variants are formatted by dump_token; unreachable here.
        TokenKind::Ident(_)
        | TokenKind::Int(_)
        | TokenKind::BigInt(_)
        | TokenKind::Float(_)
        | TokenKind::Str(_)
        | TokenKind::Char(_)
        | TokenKind::Error(_) => "?",
    }
}

fn dump_token(tok: &Token) -> String {
    let s = &tok.span;
    let mut out = format!("{}:{}:{}:{} ", s.line, s.col, s.byte_start, s.byte_end);
    match &tok.kind {
        TokenKind::Ident(v) => {
            out.push_str("Ident ");
            out.push_str(&hex_bytes(v.as_bytes()));
        }
        TokenKind::Int(v) => out.push_str(&format!("Int {:016x}", v)),
        TokenKind::BigInt(v) => out.push_str(&format!(
            "BigInt {:016x}{:016x}",
            (v >> 64) as u64,
            *v as u64
        )),
        TokenKind::Float(_) => {
            out.push_str("Float ");
            out.push_str(&hex_bytes(tok.lexeme.as_bytes()));
        }
        TokenKind::Str(v) => {
            out.push_str("Str ");
            out.push_str(&hex_bytes(v.as_bytes()));
        }
        TokenKind::Char(c) => out.push_str(&format!("Char {:x}", *c as u32)),
        TokenKind::Error(m) => {
            out.push_str("Error ");
            out.push_str(&hex_bytes(m.as_bytes()));
        }
        other => out.push_str(bare_token_kind(other)),
    }
    out
}

/// Canonical dump of the whole token stream (including the final Eof line).
fn dump_tokens(source: &str) -> String {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut out = String::new();
    for tok in &tokens {
        out.push_str(&dump_token(tok));
        out.push('\n');
    }
    out
}

// ============================================================================
// --dump-ast: canonical AST dump for the selfhost Phase 2 parity gate
// ============================================================================
//
// Line format (byte-stable; mirrored by selfhost/src/ast_dump.xi):
//
//   {indent}{Kind}[ key=value]... [span=l:c:bs:be]
//
// `indent` is two spaces per depth level; child nodes follow their parent and
// are indented one level. Every text payload is lowercase hex. Integer
// payloads are zero-padded hex (u64: 16 digits, u128: 32 digits). Float
// literal payloads dump the SOURCE LEXEME bytes at the node's byte span
// (float VALUE parity is deferred: the selfhost has no correctly rounded
// decimal->f64 parser; the lexeme is Phase 1-gated). Spans are POST-BOM
// `line:col:byte_start:byte_end`, exactly as in --dump-tokens. A file that
// fails to parse yields a single `PARSE-ERROR` line.
//
// Field order per node is defined by this walker; the selfhost mirror must
// emit the identical order.

use xiom_ast::{
    AsmBlock, Attribute, Block, ConstDecl, ContractClause, EnumDecl, EnumVariant, Expr, ExternBlock,
    FieldDecl, FnDecl, GenericParam, Ident, ImplDecl, ImplItem, InterfaceDecl, InterfaceMember,
    Literal, MatchArm, MatchBody, Param, Pattern, Program, Span, Stmt, StmtOrExpr,
    TopDecl, Type, TypeDecl, UseDecl,
};

fn ast_span(s: &Span) -> String {
    format!("{}:{}:{}:{}", s.line, s.col, s.byte_start, s.byte_end)
}

/// BOM-stripped source + output buffer for the canonical AST dump.
struct AstDump<'a> {
    out: String,
    src: &'a str,
}

impl<'a> AstDump<'a> {
    fn new(src: &'a str) -> Self {
        Self { out: String::new(), src }
    }

    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str("  ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn hx(&self, s: &str) -> String {
        hex_bytes(s.as_bytes())
    }

    fn id_hex(&self, id: &Ident) -> String {
        hex_bytes(id.name.as_bytes())
    }

    fn id_name(&mut self, id: &Ident, depth: usize) {
        self.line(depth, &format!("Name name={} span={}", self.id_hex(id), ast_span(&id.span)));
    }

    /// Source lexeme bytes at a span (used for Float literals only).
    fn lexeme(&self, s: &Span) -> String {
        let b = self.src.as_bytes();
        let (a, z) = (s.byte_start as usize, s.byte_end as usize);
        if a < z && z <= b.len() {
            hex_bytes(&b[a..z])
        } else {
            "-".to_string()
        }
    }

    fn opt_label(&self, label: &Option<Ident>) -> String {
        match label {
            Some(id) => self.id_hex(id),
            None => "-".to_string(),
        }
    }

    fn program(&mut self, p: &Program) {
        self.line(0, &format!("Program items={} span={}", p.items.len(), ast_span(&p.span)));
        for item in &p.items {
            self.top_decl(item, 1);
        }
    }

    fn top_decl(&mut self, d: &TopDecl, depth: usize) {
        match d {
            TopDecl::Module(m) => {
                let path = if m.path.is_empty() {
                    "-".to_string()
                } else {
                    m.path.iter().map(|i| self.id_hex(i)).collect::<Vec<_>>().join(".")
                };
                let source = match &m.source_file {
                    Some(s) => self.hx(s),
                    None => "-".to_string(),
                };
                self.line(
                    depth,
                    &format!(
                        "Module name={} path={} filelevel={} source={} span={}",
                        self.id_hex(&m.name),
                        path,
                        m.is_file_level as u8,
                        source,
                        ast_span(&m.span)
                    ),
                );
                for item in &m.items {
                    self.top_decl(item, depth + 1);
                }
            }
            TopDecl::Use(u) => self.use_decl(u, depth),
            TopDecl::Type(t) => self.type_decl(t, depth),
            TopDecl::Enum(e) => self.enum_decl(e, depth),
            TopDecl::Interface(i) => self.interface_decl(i, depth),
            TopDecl::Fn(f) => self.fn_decl(f, depth),
            TopDecl::Const(c) => self.const_decl(c, depth),
            TopDecl::Extern(e) => self.extern_block(e, depth),
            TopDecl::Impl(i) => self.impl_decl(i, depth),
            TopDecl::Spawn(b, s, mv) => {
                self.line(depth, &format!("Spawn move={} span={}", *mv as u8, ast_span(s)));
                self.block(b, depth + 1);
            }
        }
    }

    fn use_decl(&mut self, u: &UseDecl, depth: usize) {
        let path = u.path.iter().map(|i| self.id_hex(i)).collect::<Vec<_>>().join(".");
        let alias = match &u.alias {
            Some(a) => self.id_hex(a),
            None => "-".to_string(),
        };
        self.line(
            depth,
            &format!("Use path={} glob={} alias={} span={}", path, u.glob as u8, alias, ast_span(&u.span)),
        );
    }

    fn type_decl(&mut self, t: &TypeDecl, depth: usize) {
        self.line(
            depth,
            &format!(
                "Type pub={} name={} generic={} fields={} derived={} invariants={} derives={} alias={} span={}",
                t.is_pub as u8,
                self.id_hex(&t.name),
                t.generics.len(),
                t.fields.len(),
                t.derived_fields.len(),
                t.invariants.len(),
                t.derives.len(),
                t.alias.is_some() as u8,
                ast_span(&t.span)
            ),
        );
        for g in &t.generics {
            self.generic_param(g, depth + 1);
        }
        for f in &t.fields {
            self.field_decl(f, depth + 1);
        }
        for (name, ty, expr) in &t.derived_fields {
            self.line(depth + 1, &format!("Derived name={} span={}", self.id_hex(name), ast_span(&name.span)));
            self.ty(ty, depth + 2);
            self.expr(expr, depth + 2);
        }
        for inv in &t.invariants {
            self.expr(inv, depth + 1);
        }
        for dr in &t.derives {
            self.line(depth + 1, &format!("Derive {}", derive_name(dr)));
        }
        if let Some(a) = &t.alias {
            self.ty(a, depth + 1);
        }
    }

    fn enum_decl(&mut self, e: &EnumDecl, depth: usize) {
        self.line(
            depth,
            &format!(
                "Enum pub={} name={} generic={} variants={} derives={} span={}",
                e.is_pub as u8,
                self.id_hex(&e.name),
                e.generics.len(),
                e.variants.len(),
                e.derives.len(),
                ast_span(&e.span)
            ),
        );
        for g in &e.generics {
            self.generic_param(g, depth + 1);
        }
        for v in &e.variants {
            self.enum_variant(v, depth + 1);
        }
        for dr in &e.derives {
            self.line(depth + 1, &format!("Derive {}", derive_name(dr)));
        }
    }

    fn enum_variant(&mut self, v: &EnumVariant, depth: usize) {
        self.line(
            depth,
            &format!("Variant name={} fields={} span={}", self.id_hex(&v.name), v.fields.len(), ast_span(&v.span)),
        );
        for f in &v.fields {
            self.field_decl(f, depth + 1);
        }
    }

    fn interface_decl(&mut self, i: &InterfaceDecl, depth: usize) {
        let parent = match &i.parent {
            Some(p) => self.id_hex(p),
            None => "-".to_string(),
        };
        self.line(
            depth,
            &format!(
                "Interface pub={} name={} generic={} parent={} members={} span={}",
                i.is_pub as u8,
                self.id_hex(&i.name),
                i.generics.len(),
                parent,
                i.members.len(),
                ast_span(&i.span)
            ),
        );
        for g in &i.generics {
            self.generic_param(g, depth + 1);
        }
        for m in &i.members {
            match m {
                InterfaceMember::Field(f) => self.field_decl(f, depth + 1),
                InterfaceMember::FnSignature(f) => self.fn_decl(f, depth + 1),
            }
        }
    }

    fn impl_decl(&mut self, i: &ImplDecl, depth: usize) {
        self.line(
            depth,
            &format!(
                "Impl trait={} traitargs={} type={} span={}",
                self.id_hex(&i.trait_name),
                i.trait_args.len(),
                self.id_hex(&i.type_name),
                ast_span(&i.span)
            ),
        );
        for t in &i.trait_args {
            self.ty(t, depth + 1);
        }
        for m in &i.members {
            match m {
                ImplItem::Fn(f) => self.fn_decl(f, depth + 1),
                ImplItem::Const(c) => self.const_decl(c, depth + 1),
            }
        }
    }

    fn fn_decl(&mut self, f: &FnDecl, depth: usize) {
        let recv = match &f.receiver {
            Some(r) => self.id_hex(r),
            None => "-".to_string(),
        };
        self.line(
            depth,
            &format!(
                "Fn pub={} async={} recv={} name={} generics={} params={} ret={} contracts={} body={} attrs={} span={}",
                f.is_pub as u8,
                f.is_async as u8,
                recv,
                self.id_hex(&f.name),
                f.generics.len(),
                f.params.len(),
                f.return_type.is_some() as u8,
                f.contracts.len(),
                f.body.is_some() as u8,
                f.attributes.len(),
                ast_span(&f.span)
            ),
        );
        for a in &f.attributes {
            self.attribute(a, depth + 1);
        }
        for g in &f.generics {
            self.generic_param(g, depth + 1);
        }
        for p in &f.params {
            self.param(p, depth + 1);
        }
        if let Some(r) = &f.return_type {
            self.ty(r, depth + 1);
        }
        for c in &f.contracts {
            self.contract(c, depth + 1);
        }
        if let Some(b) = &f.body {
            self.block(b, depth + 1);
        }
    }

    fn const_decl(&mut self, c: &ConstDecl, depth: usize) {
        self.line(
            depth,
            &format!(
                "Const pub={} mut={} name={} span={}",
                c.is_pub as u8,
                c.is_mut as u8,
                self.id_hex(&c.name),
                ast_span(&c.span)
            ),
        );
        self.ty(&c.ty, depth + 1);
        self.expr(&c.value, depth + 1);
    }

    fn extern_block(&mut self, e: &ExternBlock, depth: usize) {
        self.line(
            depth,
            &format!("Extern linkage={} fns={} span={}", self.hx(&e.linkage), e.functions.len(), ast_span(&e.span)),
        );
        for f in &e.functions {
            self.fn_decl(f, depth + 1);
        }
    }

    fn attribute(&mut self, a: &Attribute, depth: usize) {
        self.line(
            depth,
            &format!("Attr name={} args={} span={}", self.id_hex(&a.name), a.args.len(), ast_span(&a.span)),
        );
        for (k, v) in &a.args {
            self.line(depth + 1, &format!("Arg key={} value={}", self.hx(k), self.hx(v)));
        }
    }

    fn generic_param(&mut self, g: &GenericParam, depth: usize) {
        self.line(
            depth,
            &format!(
                "Generic name={} bounds={} const={} constty={}",
                self.id_hex(&g.name),
                g.bounds.len(),
                g.is_const as u8,
                g.const_ty.is_some() as u8
            ),
        );
        for b in &g.bounds {
            self.line(depth + 1, &format!("Bound name={} span={}", self.id_hex(b), ast_span(&b.span)));
        }
        if let Some(t) = &g.const_ty {
            self.ty(t, depth + 1);
        }
    }

    fn field_decl(&mut self, f: &FieldDecl, depth: usize) {
        self.line(depth, &format!("Field name={} span={}", self.id_hex(&f.name), ast_span(&f.span)));
        self.ty(&f.ty, depth + 1);
    }

    fn param(&mut self, p: &Param, depth: usize) {
        self.line(
            depth,
            &format!(
                "Param name={} mutself={} refself={} span={}",
                self.id_hex(&p.name),
                p.is_mut_self as u8,
                p.is_ref_self as u8,
                ast_span(&p.span)
            ),
        );
        self.ty(&p.ty, depth + 1);
    }

    fn contract(&mut self, c: &ContractClause, depth: usize) {
        match c {
            ContractClause::Requires(e, s) => {
                self.line(depth, &format!("Requires span={}", ast_span(s)));
                self.expr(e, depth + 1);
            }
            ContractClause::Ensures(e, s) => {
                self.line(depth, &format!("Ensures span={}", ast_span(s)));
                self.expr(e, depth + 1);
            }
        }
    }

    fn block(&mut self, b: &Block, depth: usize) {
        self.line(depth, &format!("Block stmts={} span={}", b.stmts.len(), ast_span(&b.span)));
        for s in &b.stmts {
            match s {
                StmtOrExpr::Stmt(st) => {
                    self.line(depth + 1, "Stmt");
                    self.stmt(st, depth + 2);
                }
                StmtOrExpr::Expr(e) => {
                    self.line(depth + 1, "Tail");
                    self.expr(e, depth + 2);
                }
            }
        }
    }

    fn stmt(&mut self, s: &Stmt, depth: usize) {
        match s {
            Stmt::Let(id, ty, e, sp) => {
                self.line(
                    depth,
                    &format!("Let name={} ty={} span={}", self.id_hex(id), ty.is_some() as u8, ast_span(sp)),
                );
                if let Some(t) = ty {
                    self.ty(t, depth + 1);
                }
                self.expr(e, depth + 1);
            }
            Stmt::Var(id, ty, e, sp) => {
                self.line(
                    depth,
                    &format!("Var name={} ty={} span={}", self.id_hex(id), ty.is_some() as u8, ast_span(sp)),
                );
                if let Some(t) = ty {
                    self.ty(t, depth + 1);
                }
                self.expr(e, depth + 1);
            }
            Stmt::Assign(l, r, sp) => {
                self.line(depth, &format!("Assign span={}", ast_span(sp)));
                self.expr(l, depth + 1);
                self.expr(r, depth + 1);
            }
            Stmt::Return(e, sp) => {
                self.line(depth, &format!("Return has={} span={}", e.is_some() as u8, ast_span(sp)));
                if let Some(e) = e {
                    self.expr(e, depth + 1);
                }
            }
            Stmt::Expr(e, sp) => {
                self.line(depth, &format!("ExprStmt span={}", ast_span(sp)));
                self.expr(e, depth + 1);
            }
            Stmt::If(c, t, elifs, els, sp) => {
                self.line(
                    depth,
                    &format!("StmtIf elifs={} else={} span={}", elifs.len(), els.is_some() as u8, ast_span(sp)),
                );
                self.expr(c, depth + 1);
                self.block(t, depth + 1);
                for (ec, eb) in elifs {
                    self.line(depth + 1, "Elif");
                    self.expr(ec, depth + 2);
                    self.block(eb, depth + 2);
                }
                if let Some(b) = els {
                    self.line(depth + 1, "Else");
                    self.block(b, depth + 2);
                }
            }
            Stmt::Match(scrut, arms, sp) => {
                self.line(depth, &format!("StmtMatch arms={} span={}", arms.len(), ast_span(sp)));
                self.expr(scrut, depth + 1);
                for a in arms {
                    self.match_arm(a, depth + 1);
                }
            }
            Stmt::While(c, b, inv, sp, label) => {
                self.line(
                    depth,
                    &format!(
                        "While inv={} label={} span={}",
                        inv.is_some() as u8,
                        self.opt_label(label),
                        ast_span(sp)
                    ),
                );
                self.expr(c, depth + 1);
                self.block(b, depth + 1);
                if let Some(e) = inv {
                    self.expr(e, depth + 1);
                }
            }
            Stmt::For(id, it, b, sp, label) => {
                self.line(
                    depth,
                    &format!("For name={} label={} span={}", self.id_hex(id), self.opt_label(label), ast_span(sp)),
                );
                self.expr(it, depth + 1);
                self.block(b, depth + 1);
            }
            Stmt::Spawn(b, sp, mv) => {
                self.line(depth, &format!("StmtSpawn move={} span={}", *mv as u8, ast_span(sp)));
                self.block(b, depth + 1);
            }
            Stmt::Destructure(names, e, sp) => {
                self.line(depth, &format!("Destructure names={} span={}", names.len(), ast_span(sp)));
                for n in names {
                    self.id_name(n, depth + 1);
                }
                self.expr(e, depth + 1);
            }
            Stmt::Break(label, sp) => {
                self.line(depth, &format!("Break label={} span={}", self.opt_label(label), ast_span(sp)));
            }
            Stmt::Continue(label, sp) => {
                self.line(depth, &format!("Continue label={} span={}", self.opt_label(label), ast_span(sp)));
            }
            Stmt::Asm(a) => self.asm_block(a, depth),
            Stmt::Defer(b, sp) => {
                self.line(depth, &format!("Defer span={}", ast_span(sp)));
                self.block(b, depth + 1);
            }
            Stmt::Assert(c, msg, sp) => {
                self.line(depth, &format!("Assert msg={} span={}", msg.is_some() as u8, ast_span(sp)));
                self.expr(c, depth + 1);
                if let Some(m) = msg {
                    self.expr(m, depth + 1);
                }
            }
            Stmt::Debugger(sp) => {
                self.line(depth, &format!("Debugger span={}", ast_span(sp)));
            }
        }
    }

    fn asm_block(&mut self, a: &AsmBlock, depth: usize) {
        self.line(
            depth,
            &format!(
                "Asm template={} outputs={} inputs={} clobbers={} span={}",
                self.hx(&a.template),
                a.outputs.len(),
                a.inputs.len(),
                a.clobbers.len(),
                ast_span(&a.span)
            ),
        );
        for (constraint, id) in &a.outputs {
            self.line(depth + 1, &format!("Out constraint={} name={} span={}", self.hx(constraint), self.id_hex(id), ast_span(&id.span)));
        }
        for (constraint, e) in &a.inputs {
            self.line(depth + 1, &format!("In constraint={}", self.hx(constraint)));
            self.expr(e, depth + 2);
        }
        for c in &a.clobbers {
            self.line(depth + 1, &format!("Clobber name={}", self.hx(c)));
        }
    }

    fn match_arm(&mut self, a: &MatchArm, depth: usize) {
        let body = match &a.body {
            MatchBody::Block(_) => "block",
            MatchBody::Expr(_) => "expr",
        };
        self.line(
            depth,
            &format!("Arm guard={} body={} span={}", a.guard.is_some() as u8, body, ast_span(&a.span)),
        );
        self.pattern(&a.pattern, depth + 1);
        if let Some(g) = &a.guard {
            self.expr(g, depth + 1);
        }
        match &a.body {
            MatchBody::Block(b) => self.block(b, depth + 1),
            MatchBody::Expr(e) => self.expr(e, depth + 1),
        }
    }

    fn pattern(&mut self, p: &Pattern, depth: usize) {
        match p {
            Pattern::Wildcard(s) => self.line(depth, &format!("Wildcard span={}", ast_span(s))),
            Pattern::Ident(i) => {
                self.line(depth, &format!("PatIdent name={} span={}", self.id_hex(i), ast_span(&i.span)));
            }
            Pattern::Variant(name, fields, s) => {
                self.line(
                    depth,
                    &format!("PatVariant name={} fields={} span={}", self.id_hex(name), fields.len(), ast_span(s)),
                );
                for f in fields {
                    self.id_name(f, depth + 1);
                }
            }
            Pattern::Struct(name, fields, s) => {
                self.line(
                    depth,
                    &format!("PatStruct name={} fields={} span={}", self.id_hex(name), fields.len(), ast_span(s)),
                );
                for (fname, fpat) in fields {
                    self.line(depth + 1, &format!("PatField name={} span={}", self.id_hex(fname), ast_span(&fname.span)));
                    self.pattern(fpat, depth + 2);
                }
            }
            Pattern::Tuple(pats, s) => {
                self.line(depth, &format!("PatTuple n={} span={}", pats.len(), ast_span(s)));
                for p in pats {
                    self.pattern(p, depth + 1);
                }
            }
            Pattern::Lit(l) => self.literal(l, depth),
            Pattern::Some(inner, s) => {
                self.line(depth, &format!("PatSome span={}", ast_span(s)));
                self.pattern(inner, depth + 1);
            }
            Pattern::None(s) => self.line(depth, &format!("PatNone span={}", ast_span(s))),
            Pattern::Ok(inner, s) => {
                self.line(depth, &format!("PatOk span={}", ast_span(s)));
                self.pattern(inner, depth + 1);
            }
            Pattern::Err(inner, s) => {
                self.line(depth, &format!("PatErr span={}", ast_span(s)));
                self.pattern(inner, depth + 1);
            }
            Pattern::Or(pats, s) => {
                self.line(depth, &format!("PatOr n={} span={}", pats.len(), ast_span(s)));
                for p in pats {
                    self.pattern(p, depth + 1);
                }
            }
        }
    }

    fn literal(&mut self, l: &Literal, depth: usize) {
        match l {
            Literal::Int(v, s) => self.line(depth, &format!("LitInt value={:016x} span={}", v, ast_span(s))),
            Literal::Float(_, s) => {
                self.line(depth, &format!("LitFloat lex={} span={}", self.lexeme(s), ast_span(s)));
            }
            Literal::Str(v, s) => {
                self.line(depth, &format!("LitStr data={} span={}", hex_bytes(v.as_bytes()), ast_span(s)));
            }
            Literal::Char(c, s) => self.line(depth, &format!("LitChar cp={:x} span={}", *c as u32, ast_span(s))),
            Literal::Bool(b, s) => self.line(depth, &format!("LitBool value={} span={}", *b as u8, ast_span(s))),
        }
    }

    fn ty(&mut self, t: &Type, depth: usize) {
        match t {
            Type::Named(id, args) => {
                self.line(
                    depth,
                    &format!("Named name={} span={} args={}", self.id_hex(id), ast_span(&id.span), args.len()),
                );
                for a in args {
                    self.ty(a, depth + 1);
                }
            }
            Type::Ref(inner) => {
                self.line(depth, "Ref");
                self.ty(inner, depth + 1);
            }
            Type::MutRef(inner) => {
                self.line(depth, "MutRef");
                self.ty(inner, depth + 1);
            }
            Type::Option(inner) => {
                self.line(depth, "Opt");
                self.ty(inner, depth + 1);
            }
            Type::Result(a, b) => {
                self.line(depth, "Res");
                self.ty(a, depth + 1);
                self.ty(b, depth + 1);
            }
            Type::Vec(inner) => {
                self.line(depth, "Vec");
                self.ty(inner, depth + 1);
            }
            Type::Slice(inner) => {
                self.line(depth, "SliceTy");
                self.ty(inner, depth + 1);
            }
            Type::Map(a, b) => {
                self.line(depth, "MapTy");
                self.ty(a, depth + 1);
                self.ty(b, depth + 1);
            }
            Type::Set(inner) => {
                self.line(depth, "SetTy");
                self.ty(inner, depth + 1);
            }
            Type::Tuple(items) => {
                self.line(depth, &format!("TupleTy n={}", items.len()));
                for i in items {
                    self.ty(i, depth + 1);
                }
            }
            Type::Ptr(inner) => {
                self.line(depth, "Ptr");
                self.ty(inner, depth + 1);
            }
            Type::Array(n, inner) => {
                self.line(depth, "ArrayTy");
                self.expr(n, depth + 1);
                self.ty(inner, depth + 1);
            }
            Type::Fn(params, ret) => {
                self.line(depth, &format!("FnTy params={}", params.len()));
                for p in params {
                    self.ty(p, depth + 1);
                }
                self.ty(ret, depth + 1);
            }
            Type::ImplTrait(ids) => {
                self.line(depth, &format!("ImplTrait n={}", ids.len()));
                for b in ids {
                    self.line(depth + 1, &format!("Bound name={} span={}", self.id_hex(b), ast_span(&b.span)));
                }
            }
            Type::AnonStruct(fields) => {
                self.line(depth, &format!("AnonStruct fields={}", fields.len()));
                for f in fields {
                    self.field_decl(f, depth + 1);
                }
            }
            Type::Never => self.line(depth, "Never"),
        }
    }

    fn expr(&mut self, e: &Expr, depth: usize) {
        match e {
            Expr::Ident(i) => {
                self.line(depth, &format!("ExprIdent name={} span={}", self.id_hex(i), ast_span(&i.span)));
            }
            Expr::Int(v, s) => self.line(depth, &format!("LitInt value={:016x} span={}", v, ast_span(s))),
            Expr::BigInt(v, s) => self.line(depth, &format!("LitBigInt value={:032x} span={}", v, ast_span(s))),
            Expr::Float(_, s) => {
                self.line(depth, &format!("LitFloat lex={} span={}", self.lexeme(s), ast_span(s)));
            }
            Expr::Str(v, s) => {
                self.line(depth, &format!("LitStr data={} span={}", hex_bytes(v.as_bytes()), ast_span(s)));
            }
            Expr::Char(c, s) => self.line(depth, &format!("LitChar cp={:x} span={}", *c as u32, ast_span(s))),
            Expr::Bool(b, s) => self.line(depth, &format!("LitBool value={} span={}", *b as u8, ast_span(s))),
            Expr::Paren(inner, s) => {
                self.line(depth, &format!("Paren span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Unary(op, inner, s) => {
                self.line(depth, &format!("Unary op={} span={}", unary_name(op), ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Binary(l, op, r, s) => {
                self.line(depth, &format!("Binary op={} span={}", binop_name(op), ast_span(s)));
                self.expr(l, depth + 1);
                self.expr(r, depth + 1);
            }
            Expr::Try(inner, s) => {
                self.line(depth, &format!("Try span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Imply(l, r, s) => {
                self.line(depth, &format!("Imply span={}", ast_span(s)));
                self.expr(l, depth + 1);
                self.expr(r, depth + 1);
            }
            Expr::Is(inner, pat, s) => {
                self.line(depth, &format!("Is span={}", ast_span(s)));
                self.expr(inner, depth + 1);
                self.pattern(pat, depth + 1);
            }
            Expr::Field(inner, field, s) => {
                self.line(depth, &format!("Field name={} span={}", self.id_hex(field), ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Call(callee, args, s) => {
                self.line(depth, &format!("Call args={} span={}", args.len(), ast_span(s)));
                self.expr(callee, depth + 1);
                for a in args {
                    self.expr(a, depth + 1);
                }
            }
            Expr::GenericCall(callee, types, args, s) => {
                self.line(
                    depth,
                    &format!("GenericCall types={} args={} span={}", types.len(), args.len(), ast_span(s)),
                );
                self.expr(callee, depth + 1);
                for t in types {
                    self.ty(t, depth + 1);
                }
                for a in args {
                    self.expr(a, depth + 1);
                }
            }
            Expr::Index(base, idx, s) => {
                self.line(depth, &format!("Index span={}", ast_span(s)));
                self.expr(base, depth + 1);
                self.expr(idx, depth + 1);
            }
            Expr::AtPre(inner, s) => {
                self.line(depth, &format!("AtPre span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Ref(inner, s) => {
                self.line(depth, &format!("ExprRef span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::MutRef(inner, s) => {
                self.line(depth, &format!("ExprMutRef span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Some(inner, s) => {
                self.line(depth, &format!("ExprSome span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::None(s) => self.line(depth, &format!("ExprNone span={}", ast_span(s))),
            Expr::Ok(inner, s) => {
                self.line(depth, &format!("ExprOk span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Err(inner, s) => {
                self.line(depth, &format!("ExprErr span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Struct(name, fields, base, s) => {
                self.line(
                    depth,
                    &format!(
                        "StructLit name={} fields={} base={} span={}",
                        self.id_hex(name),
                        fields.len(),
                        base.is_some() as u8,
                        ast_span(s)
                    ),
                );
                for (fname, fval) in fields {
                    self.line(
                        depth + 1,
                        &format!("Init name={} span={}", self.id_hex(fname), ast_span(&fname.span)),
                    );
                    self.expr(fval, depth + 2);
                }
                if let Some(b) = base {
                    self.line(depth + 1, "Base");
                    self.expr(b, depth + 2);
                }
            }
            Expr::Array(items, s) => {
                self.line(depth, &format!("ArrayLit n={} span={}", items.len(), ast_span(s)));
                for i in items {
                    self.expr(i, depth + 1);
                }
            }
            Expr::BlockExpr(b, s) => {
                self.line(depth, &format!("BlockExpr span={}", ast_span(s)));
                self.block(b, depth + 1);
            }
            Expr::ConstBlock(inner, s) => {
                self.line(depth, &format!("ConstBlock span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Closure(params, ret, body, s) => {
                self.line(
                    depth,
                    &format!(
                        "Closure params={} ret={} span={}",
                        params.len(),
                        ret.is_some() as u8,
                        ast_span(s)
                    ),
                );
                for p in params {
                    self.param(p, depth + 1);
                }
                if let Some(r) = ret {
                    self.ty(r, depth + 1);
                }
                self.block(body, depth + 1);
            }
            Expr::PipeClosure(names, body, s) => {
                self.line(depth, &format!("PipeClosure names={} span={}", names.len(), ast_span(s)));
                for n in names {
                    self.id_name(n, depth + 1);
                }
                self.expr(body, depth + 1);
            }
            Expr::Await(inner, s) => {
                self.line(depth, &format!("Await span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::Comptime(inner, s) => {
                self.line(depth, &format!("Comptime span={}", ast_span(s)));
                self.expr(inner, depth + 1);
            }
            Expr::As(inner, ty, s) => {
                self.line(depth, &format!("As span={}", ast_span(s)));
                self.expr(inner, depth + 1);
                self.ty(ty, depth + 1);
            }
            Expr::Tuple(items, s) => {
                self.line(depth, &format!("TupleLit n={} span={}", items.len(), ast_span(s)));
                for i in items {
                    self.expr(i, depth + 1);
                }
            }
            Expr::If(cond, then, elifs, els, s) => {
                self.line(
                    depth,
                    &format!("ExprIf elifs={} else={} span={}", elifs.len(), els.is_some() as u8, ast_span(s)),
                );
                self.expr(cond, depth + 1);
                self.block(then, depth + 1);
                for (ec, eb) in elifs {
                    self.line(depth + 1, "Elif");
                    self.expr(ec, depth + 2);
                    self.block(eb, depth + 2);
                }
                if let Some(b) = els {
                    self.line(depth + 1, "Else");
                    self.block(b, depth + 2);
                }
            }
            Expr::Match(scrut, arms, s) => {
                self.line(depth, &format!("ExprMatch arms={} span={}", arms.len(), ast_span(s)));
                self.expr(scrut, depth + 1);
                for a in arms {
                    self.match_arm(a, depth + 1);
                }
            }
            Expr::Unsafe(b, s) => {
                self.line(depth, &format!("Unsafe span={}", ast_span(s)));
                self.block(b, depth + 1);
            }
            Expr::Error(_, s) => {
                self.line(depth, &format!("ExprError span={}", ast_span(s)));
            }
        }
    }
}

fn derive_name(d: &xiom_ast::DeriveTrait) -> &'static str {
    use xiom_ast::DeriveTrait::*;
    match d {
        Eq => "Eq",
        Clone => "Clone",
        Display => "Display",
        Hash => "Hash",
        Ord => "Ord",
        Debug => "Debug",
    }
}

fn unary_name(op: &xiom_ast::UnaryOp) -> &'static str {
    use xiom_ast::UnaryOp::*;
    match op {
        Neg => "Neg",
        Not => "Not",
        Ref => "Ref",
        MutRef => "MutRef",
        BitNot => "BitNot",
        Deref => "Deref",
    }
}

fn binop_name(op: &xiom_ast::BinOp) -> &'static str {
    use xiom_ast::BinOp::*;
    match op {
        Add => "Add",
        Sub => "Sub",
        Mul => "Mul",
        Div => "Div",
        Rem => "Rem",
        Eq => "Eq",
        Neq => "Neq",
        Lt => "Lt",
        Gt => "Gt",
        Le => "Le",
        Ge => "Ge",
        Shl => "Shl",
        Shr => "Shr",
        And => "And",
        Or => "Or",
        Assign => "Assign",
        BitXor => "BitXor",
        BitAnd => "BitAnd",
        BitOr => "BitOr",
    }
}

/// Canonical AST dump of a source file (see the format notes above).
fn dump_ast(source: &str) -> String {
    let src = source.strip_prefix('\u{feff}').unwrap_or(source);
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => {
            let mut d = AstDump::new(src);
            d.program(&program);
            d.out
        }
        Err(_) => "PARSE-ERROR\n".to_string(),
    }
}

// ============================================================================
// --dump-check: canonical checker diagnostics for the selfhost Phase 3 gate
// ============================================================================
//
// Line format (byte-stable; mirrored by selfhost/src/checker_dump.xi):
//
//   {kind} {code} {line}:{col} {escaped-message}    one line per diagnostic
//   CHECK-OK                                        no diagnostics
//   PARSE-ERROR                                     input/lex/parse failed
//
// Diagnostics are emitted in exactly the order `CompileResult.diagnostics`
// carries them: catalog module collisions (W001), then checker warnings
// (`take_warnings` order), then type errors (checker push order). The message
// escapes `\`, LF, CR and TAB as `\\`, `\n`, `\r`, `\t`; every other byte is
// emitted verbatim (UTF-8). `kind`/`code` are the structured diagnostic
// fields, so the gate compares severity and code as well as text.
//
// The command stops after the checker plus the non-strict borrow pass
// (`CompileConfig::dump_check` mirrors `compile()`'s E001 warnings and never
// runs codegen), so a clean program is just `CHECK-OK`.
fn dump_check_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn dump_check(source_path: &str) -> String {
    let config = CompileConfig { dump_check: true, ..CompileConfig::default() };
    let result = xiom::compile_with_diagnostics(&config, &[source_path.to_string()]);
    if result
        .diagnostics
        .iter()
        .any(|d| d.kind == "lex_error" || d.kind == "parse_error" || d.kind == "io_error")
    {
        return "PARSE-ERROR\n".to_string();
    }
    if result.diagnostics.is_empty() {
        return "CHECK-OK\n".to_string();
    }
    let mut out = String::new();
    for d in &result.diagnostics {
        out.push_str(&d.kind);
        out.push(' ');
        out.push_str(&d.code);
        out.push(' ');
        out.push_str(&d.line.to_string());
        out.push(':');
        out.push_str(&d.col.to_string());
        out.push(' ');
        out.push_str(&dump_check_escape(&d.message));
        out.push('\n');
    }
    out
}


/// M10.4: Interactive REPL -- compile and execute each line as a script.
/// State (let/var declarations) persists across lines.
fn run_repl() {
    use std::io::{self, Write};
    eprintln!("XIOM REPL v{} -- type :help for commands, :quit to exit", env!("CARGO_PKG_VERSION"));
    let mut line_num = 0u64;
    let mut state: Vec<String> = Vec::new(); // accumulated let/var declarations
    // Unique per REPL session: two REPLs must not collide on `_repl_1.xi`.
    let repl_suffix = xiom::jit::rand_suffix();

    loop {
        line_num += 1;
        print!("xiom> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() || line.is_empty() {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        // Special commands
        if trimmed.starts_with(':') {
            match trimmed {
                ":quit" | ":q" => break,
                ":help" | ":h" => {
                    eprintln!("  :quit, :q    Exit the REPL");
                    eprintln!("  :help, :h    Show this help");
                    eprintln!("  :vars        Show accumulated variables");
                    eprintln!("  :reset       Clear accumulated state");
                    eprintln!("  :type <e>    Show the type of an expression (future)");
                    eprintln!("  Any other input is compiled as a script and executed.");
                }
                ":vars" => {
                    if state.is_empty() { eprintln!("  (no variables)"); }
                    else { for v in &state { eprintln!("  {v}"); } }
                }
                ":reset" => { state.clear(); eprintln!("  State cleared."); }
                _ if trimmed.starts_with(":type") => {
                    eprintln!("  (type inspection coming in REPL v2)");
                }
                _ => eprintln!("  Unknown command: {trimmed}. Type :help for commands."),
            }
            continue;
        }

        // Track let/var declarations for state persistence
        let is_binding = trimmed.starts_with("let ") || trimmed.starts_with("var ");
        if is_binding {
            state.push(trimmed.to_string());
        }

        // Build source: accumulated state + current input
        let mut source = String::new();
        for stmt in &state {
            source.push_str(stmt);
            source.push('\n');
        }
        source.push_str(trimmed);
        source.push('\n');

        let source = xiom::implicit_main::wrap_implicit_main(&source);
        let tmp_dir = std::env::temp_dir().join("xiom_repl");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let tmp_src = tmp_dir.join(format!("_repl_{repl_suffix:x}_{line_num}.xi"));
        let tmp_out = tmp_dir.join(format!("_repl_{repl_suffix:x}_{line_num}.exe"));
        std::fs::write(&tmp_src, &source).ok();

        let config = CompileConfig {
            output_file: Some(tmp_out.to_string_lossy().to_string()),
            do_run: true,
            ..CompileConfig::default()
        };
        let sources = vec![tmp_src.to_string_lossy().to_string()];
        if let Err(errors) = compile(&config, &sources) {
            for e in &errors { eprintln!("error: {e}"); }
        }
    }
    eprintln!("Goodbye.");
}

fn main() {
    // Deep recursion (parser depth cap 128, codegen visitors) needs far more
    // than the default 1MB Windows main-thread stack. rustc runs its compiler
    // on a big-stack thread for the same reason. All work happens here;
    // process::exit calls inside real_main still terminate immediately.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(real_main)
        .expect("failed to spawn compiler worker thread");
    if child.join().is_err() {
        // A panic escaped real_main; mirror the standard panic exit code.
        std::process::exit(101);
    }
}

fn real_main() {
    // Audit #12: the timeout / memory watchdogs set this cooperative token
    // instead of calling process::exit from a worker thread.
    xiom_codegen::cancel::reset();
    // Stage 5: clap owns the flag surface (src/cli.rs). It parses for
    // validation and returns argv unchanged, so every legacy scan below is
    // byte-compatible; the reads migrate to the clap matches in follow-up
    // work without touching the surface definition.
    let args = cli::parse(env::args().collect());

    // R48 (playground C2): native tool dispatch FIRST, mirroring the
    // launcher wrapper so `xiom fmt` / `xiom lsp` / `xiom mcp` / `xiom pkg` /
    // ... (including their --version/--help) work on Linux/macOS installs
    // that have no xiom.bat. Execs the sibling binary from bin/.
    if let Some(tool) = args.get(1).and_then(|w| match w.as_str() {
        "fmt" => Some("xiom-fmt"),
        "lsp" => Some("xiom-lsp"),
        "mcp" => Some("xiom-mcp"),
        "pkg" => Some("xiom-pkg"),
        "dbg" => Some("xiom-dbg"),
        "verify" => Some("xiom-verify"),
        "ffigen" => Some("xiom-ffigen"),
        _ => None,
    }) {
        run_tool_dispatch(tool, &args[2..]);
    }

    if args.len() < 2 || args.iter().any(|a| a == "--help") {
        print_usage();
        process::exit(if args.iter().any(|a| a == "--help") { 0 } else { 1 });
    }

    // Show AI help
    if args.iter().any(|a| a == "--help-ai") {
        eprintln!("{}", xiom::ai::ai_help_text());
        process::exit(0);
    }

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("{}", version_line());
        // FE-13: answer "which xiom am I running" from the binary's own path.
        if let Ok(exe) = std::env::current_exe() {
            if let Some(root) = xiom::doctor::install_root_of(&exe) {
                println!("install root: {}", root.display());
            }
        }
        return;
    }

    // 5c-R: --explain EXXXX opens the error code reference
    if let Some(pos) = args.iter().position(|a| a == "--explain") {
        if let Some(code) = args.get(pos + 1) {
            if !xiom::explain_error(code) {
                process::exit(1);
            }
            return;
        }
        eprintln!("usage: xiom --explain <code>  (e.g., xiom --explain T001)");
        process::exit(1);
    }

    // v0.55: xiom build-runtime -- pre-compile C runtime shared library for OrcJIT
    if args.get(1).map_or(false, |a| a == "build-runtime") {
        let output_dir = xiom::jit::jit_cache_dir();
        match xiom_jit::build_runtime_library(&output_dir) {
            Ok(path) => {
                eprintln!("  Runtime library built: {}", path.display());
                eprintln!("  OrcJIT ready: use --jit flag for in-process compilation.");
            }
            Err(e) => {
                eprintln!("error: runtime build failed: {e}");
                process::exit(1);
            }
        }
        return;
    }

    // -- M10.4: xiom repl -- interactive scripting shell --------------
    if args.get(1).map_or(false, |a| a == "repl") {
        run_repl();
        return;
    }

    // -- M10: xiom run -- JIT/scripting execution ---------------------
    // R63 (playground C3): accept global flags BEFORE the `run` subcommand
    // (`xiom --opt-level=0 run f.xi`). The old check required args[1] ==
    // "run", so a leading flag made the driver read "run" as the source file.
    let run_pos = args.iter().position(|a| a == "run");
    let leading_run_flags_ok = run_pos.map_or(false, |p| {
        p > 0
            && args[1..p].iter().all(|a| {
                a.starts_with("--opt-level")
                    || a.starts_with("-O")
                    || a == "--force"
                    || a == "--no-cache"
                    || a == "--jit"
            })
    });
    if run_pos == Some(1) || leading_run_flags_ok {
        let run_pos = run_pos.unwrap();
        // Fold any leading global flags into `remaining` so the existing
        // --opt-level/-O parser below sees them.
        let mut remaining: Vec<&str> = args[1..run_pos].iter().map(|s| s.as_str()).collect();
        remaining.extend(args.iter().skip(run_pos + 1).map(|s| s.as_str()));
        if remaining.is_empty() {
            eprintln!("usage: xiom run <file.xi>     execute a script");
            eprintln!("       xiom run -e \"<code>\"   execute inline code");
            eprintln!("       xiom run -               read script from stdin");
            eprintln!("       xiom run --watch <file>  watch and re-run on changes");
            process::exit(1);
        }

        let watch_mode = remaining.contains(&"--watch");
        // R51 (playground audit S19.1): honor --opt-level on the script-run
        // path. It used to leak into `effective` (so `xiom run --opt-level 0
        // f.xi` tried to read "--opt-level" as the file) AND fall through to
        // the -O2 compile default -- measured 7.8s vs 2.3s at -O0 on the
        // lesson corpus. The effective level is also part of the script-cache
        // key now.
        let mut script_opt_level: Option<u8> = None;
        let mut run_args: Vec<&str> = Vec::new();
        let mut ri = 0;
        while ri < remaining.len() {
            let a = remaining[ri];
            if a == "--opt-level" {
                if let Some(v) = remaining.get(ri + 1) {
                    script_opt_level = v.parse().ok();
                    ri += 2;
                    continue;
                }
            } else if let Some(v) = a.strip_prefix("--opt-level=") {
                script_opt_level = v.parse().ok();
                ri += 1;
                continue;
            } else if let Some(v) = a.strip_prefix("-O") {
                // R63: accept the short spelling the playground driver probes
                // for (`xiom run -O0 file.xi`).
                if !v.is_empty() {
                    script_opt_level = v.parse().ok();
                    ri += 1;
                    continue;
                }
            }
            run_args.push(a);
            ri += 1;
        }
        // C25: read the cache/JIT flags from the RAW arg list -- the
        // `effective` filter below strips them, so checking `effective`
        // made both `--no-cache` and `--jit` dead (a warm cache could
        // never be bypassed).
        let no_cache = run_args.iter().any(|a| *a == "--no-cache");
        let use_jit = run_args.iter().any(|a| *a == "--jit");
        let effective: Vec<&str> = run_args.into_iter()
            .filter(|a| *a != "--watch" && *a != "--cache" && *a != "--no-cache" && *a != "--jit")
            .collect();
        if effective.is_empty() { process::exit(1); }

        let mut script_path: Option<String> = None;
        let source = if effective[0] == "-e" {
            // xiom run -e "expr"
            if effective.len() < 2 {
                eprintln!("error: -e requires an expression");
                process::exit(1);
            }
            effective[1..].join(" ")
        } else if effective[0] == "-" {
            // xiom run -  (read from stdin)
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error reading stdin: {e}"); process::exit(1);
            });
            buf
        } else {
            // xiom run <file.xi>  (possibly with --watch)
            let path = effective[0];
            if watch_mode {
                // Watch mode: compile once, then poll for changes
                run_script_watch(path);
                return;
            }
            script_path = Some(path.to_string());
            match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => { eprintln!("error: cannot read '{path}': {e}"); process::exit(1); }
            }
        };

        // Apply implicit main wrapping for scripting convenience
        let source = xiom::implicit_main::wrap_implicit_main(&source);

        // M10: Check script cache for instant re-run
        let script_cache_level = xiom::jit::effective_opt_level(script_opt_level, false);
        // C25: `--jit` must not execute a cached binary just to discard it.
        if !no_cache && !use_jit {
            if let Some(cached) = xiom::jit::script_cache_get(&source, script_cache_level) {
                // C25: Command::output() defaults the child's stdin to NULL,
                // so a warm cache served `[]` where the cold run read the
                // piped line. Inherit stdin (stdout/stderr stay captured for
                // the success gate and replay below).
                let output = std::process::Command::new(&cached)
                    .stdin(std::process::Stdio::inherit())
                    .output();
                if let Ok(out) = output {
                    if out.status.success() && !use_jit {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        if !stdout.is_empty() { print!("{stdout}"); }
                        return;
                    }
                }
            }
        }

        // M10.1d: True JIT execution if --jit flag is set
        if use_jit {
            match xiom::jit::jit_execute(&source) {
                Ok(code) => { eprintln!("  JIT exit code: {code}"); return; }
                Err(e) => { eprintln!("  JIT error: {e}"); process::exit(1); }
            }
        }

        // Write to temp file, compile, and run
        let tmp_dir = std::env::temp_dir().join("xiom_run");
        let _ = std::fs::create_dir_all(&tmp_dir);
        // Unique per invocation: concurrent `xiom run` processes must not
        // race on a shared `_script.xi` / `_script.exe`.
        let run_suffix = xiom::jit::rand_suffix();
        let tmp_src = tmp_dir.join(format!("_script_{run_suffix:x}.xi"));
        let tmp_out = tmp_dir.join(format!("_script_{run_suffix:x}.exe"));
        std::fs::write(&tmp_src, &source).unwrap_or_else(|e| {
            eprintln!("error: cannot write temp file: {e}"); process::exit(1);
        });

        // Resolve runtime libraries needed for linking using project discovery
        let (_, graph_dirs) = xiom::expand_sources_with_graph(&[tmp_src.to_string_lossy().to_string()]);
        let mut link_paths: Vec<String> = graph_dirs.into_iter().collect();
        // Add CARGO_MANIFEST_DIR-based runtime path as fallback
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let runtime = std::path::Path::new(&manifest).join("..").join("..").join("runtime");
            if runtime.is_dir() {
                link_paths.push(runtime.to_string_lossy().to_string());
            }
        }
        let link_libs: Vec<String> = Vec::new();
        let c_sources: Vec<String> = Vec::new(); // Auto-discovered by compile()

        // M12: Ensure XIOM_STDLIB is set for scripting mode so stdlib types resolve
        if std::env::var("XIOM_STDLIB").is_err() {
            // Try to find stdlib relative to the xiom binary
            let exe = std::env::current_exe().unwrap_or_default();
            let mut search = exe.parent();
            for _ in 0..8 {
                if let Some(dir) = search {
                    let candidate = dir.join("stdlib");
                    if candidate.is_dir() {
                        // SAFETY: set_var is called before any threads are spawned
                        unsafe { std::env::set_var("XIOM_STDLIB", candidate.to_string_lossy().to_string()); }
                        break;
                    }
                    search = dir.parent();
                } else { break; }
            }
        }

        let config = CompileConfig {
            output_file: Some(tmp_out.to_string_lossy().to_string()),
            do_run: true,
            script_mode: true,
            // R51: thread the script path's --opt-level into the compiler.
            opt_level: script_opt_level,
            link_paths,
            link_libs,
            c_sources,
            // C22: the temp copy lives in %TEMP%/xiom_run; hand the checker
            // the script's real directory so sibling modules resolve exactly
            // like `xiom --check`.
            extra_source_dirs: script_path
                .as_deref()
                .map(run_script_source_dirs)
                .unwrap_or_default(),
            ..CompileConfig::default()
        };

        let sources = vec![tmp_src.to_string_lossy().to_string()];
        compile_or_exit(&config, &sources);

        // M10: Cache the compiled script for instant re-run
        if !no_cache {
            xiom::jit::script_cache_put(&source, &tmp_out, script_cache_level);
        }
        return;
    }

    // Stage 5 (clap step 2): flag reads below come from the clap matches;
    // exact-form-sensitive checks and command words use the raw view.
    let emit_ir = args.flag("emit-ir");
    let emit_tokens = args.flag("emit-tokens");
    let dump_tokens_flag = args.flag("dump-tokens");
    let dump_ast_flag = args.flag("dump-ast");
    let dump_check_flag = args.flag("dump-check");
    let do_run = args.flag("run");
    let check_only = args.flag("check");
    let release = args.flag("release");
    // 2026-09-10: explicit optimization level (--opt-level 0..=3). The old
    // -O2 floor existed because pre-CRT-layout IR miscompiled at -O0/-O1
    // (i128 loops + inlined Vec ops); the floor is now overridable.
    let opt_level: Option<u8> = args.value("opt-level")
        .and_then(|v| v.parse::<u8>().ok());
    let target = parse_target(args.value("target").as_deref());
    let check_contracts = !args.flag("no-contracts") && !release;
    let runtime_contracts = args.flag("runtime-contracts");
    // Security review (2026-08-13): release builds strip assert/dbg!/debugger;
    // --keep-debug-checks retains them in release binaries.
    let keep_debug_checks = args.flag("keep-debug-checks");
    // Exact `--diagnostics=json` form only (the space form was never read).
    let diagnostics_json = args.raw_has("--diagnostics=json");
    let strict_mode = args.flag("strict");
    let debug_symbols = args.flag("debug") || args.flag("debug-symbols");
    let shared_lib = args.flag("shared");
    let static_lib = args.flag("static");
    let watch_mode = args.flag("watch");
    let hot_reload = args.flag("hot-reload");
    let hot_reload_contracts = args.flag("hot-reload-contracts");
    // v0.55: OrcJIT -- in-process JIT compilation via clang DLL loading
    let _use_jit = args.flag("jit");
    // v0.56: Lazy JIT -- incremental recompilation (only with --jit)
    let _lazy_jit = args.flag("lazy");
    // v0.56: LTO -- ThinLTO link-time optimization
    let use_lto = args.flag("lto");
    // 7E.1: Sanitizer flags
    let sanitize: Option<String> = args.value("sanitize");
    // 7E.2: Stack protector
    let stack_protector = args.flag("stack-protector");
    let overflow_checks = args.flag("overflow-checks");
    let strict_exhaustive = args.flag("strict-exhaustive");
    // D2.1 (Phase 7): allow `#[unsafe_direct]` (trusted escape hatch) in user code
    let enable_unsafe_direct = args.flag("enable-unsafe-direct");
    if enable_unsafe_direct {
        // Security review (2026-08-13): the escape hatch disables the
        // unsafe-confinement guard -- surface it loudly on every invocation so
        // a release build log cannot silently contain unguarded code.
        eprintln!(
            "warning: --enable-unsafe-direct is set -- #[unsafe_direct] code bypasses \
             the unsafe-confinement fault guard; audit any binary built with this flag"
        );
    }
    // v0.54: Binary cache -- cache compiled binary by SHA-256 source hash
    let use_cache = args.flag("cache");
    // 5e.5f: Incremental compilation flags
    let incremental = args.flag("incremental");
    let force_recompile = args.flag("force");
    // 7C: Parallel compilation flags
    let parallel = args.flag("parallel") && !args.flag("sequential");
    // I2: Parallel codegen -- rayon-based per-function IR emission
    let parallel_codegen = args.flag("parallel-codegen");
    let jobs: usize = args.value("jobs")
        .and_then(|v| v.parse().ok()).unwrap_or(0);
    // 5g AI Pipeline flags
    let ai_mode = args.flag("ai");
    let ai_local = args.flag("ai-local");
    let ai_dry_run = args.flag("ai-dry-run");
    let ai_silent = args.flag("ai-silent");
    let ai_strict = args.flag("ai-strict");
    let _ai_batch = args.flag("ai-batch") || args.flag("batch");
    let ai_model: Option<String> = args.value("ai-model");
    let ai_timeout: Option<u32> = args.value("ai-timeout")
        .and_then(|v| v.parse().ok());
    let test_mode = args.flag("test");
    let clean_mode = args.flag("clean");
    let install_mode = args.raw_has("install");
    let install_pkg = args.iter().position(|a| a == "install")
        .and_then(|i| args.get(i + 1).cloned())
        .filter(|p| !p.starts_with('-'));
    let publish_mode = args.raw_has("publish");
    let update_mode = args.raw_has("update");
    let bench_mode = args.raw_has("bench");
    let bench_count: u32 = args.value("count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let init_mode = args.raw_has("init");
    let new_mode = args.raw_has("new");
    let build_mode = args.raw_has("build");
    let doctor_mode = args.raw_has("doctor") || args.flag("doctor");
    let doc_mode = args.raw_has("doc") || args.flag("doc");
    let graph_mode = args.present("graph");
    // Exact-form checks: only `--graph=mermaid`/`--graph-format=mermaid`
    // select a format; bare `--graph` defaults to dot (legacy contract).
    let graph_format = if args.raw_has("--graph=mermaid") || args.raw_has("--graph-format=mermaid") {
        Some("mermaid")
    } else if args.raw_has("--graph=dot") || args.raw_has("--graph-format=dot") {
        Some("dot")
    } else if graph_mode {
        Some("dot") // default
    } else {
        None
    };
    let new_name: Option<String> = args.iter().position(|a| a == "new")
        .and_then(|i| args.get(i + 1).cloned())
        .filter(|n| !n.starts_with('-'));
    let registry_cmd = args.raw_has("registry");

    if registry_cmd {
        handle_registry(&args);
        return;
    }

    if init_mode {
        scaffold_project(".", None);
        return;
    }
    if new_mode {
        let name = new_name.as_deref().unwrap_or("xiom-project");
        scaffold_project(name, Some(name));
        return;
    }

    if clean_mode {
        let clean_cache = args.flag("cache");
        if clean_cache {
            let _ = xiom::jit::cache_clean();
            return;
        }
        let extensions = ["exe", "ll", "obj", "o", "out", "wasm", "pdb", "ilk", "exp", "lib"];
        let mut cleaned = 0usize;
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        if std::fs::remove_file(&path).is_ok() {
                            cleaned += 1;
                        }
                    }
                }
            }
        }
        eprintln!("  Cleaned {} build artifact(s)", cleaned);
        return;
    }

    // R50 (registry relay): `xiom install` / `xiom update` were a SECOND
    // package channel -- git clone driven by registry /packages.json -- that
    // bypassed the registry's checksum/signature/yank guarantees. Install now
    // delegates to the verified `xiom pkg install` client; update is retired
    // with guidance. Never fetch /packages.json from an install path.
    //
    // The TOOLCHAIN category lives at `xiom toolchain ...` (D-2); its dispatch
    // must run before `update_mode`, because `xiom toolchain update` also
    // carries the bare `update` token.
    if args.raw_has("toolchain") {
        run_toolchain_command(&args);
    }
    if install_mode {
        eprintln!("note: 'xiom install' is deprecated -- delegating to the verified 'xiom pkg install' client.");
        let mut forwarded: Vec<String> = vec!["install".to_string()];
        if let Some(pkg) = install_pkg.clone() {
            forwarded.push(pkg);
        }
        run_tool_dispatch("xiom-pkg", &forwarded);
    }
    if update_mode {
        eprintln!("error: 'xiom update' is retired -- it used the unverified /packages.json git channel.");
        eprintln!("       To update the TOOLCHAIN, re-run the release installer for your platform");
        eprintln!("       (or 'xiom toolchain update' once it ships).");
        eprintln!("       For packages: 'xiom pkg install <package>@<version>'.");
        process::exit(1);
    }

    if bench_mode {
        run_benchmarks(&args, bench_count);
        return;
    }

    // FE-8: `xiom publish` ran a legacy git-tag flow whose closing advice was
    // the retired `xiom install {name}` channel. Mirror `xiom install`:
    // delegate to the verified client with a deprecation note.
    if publish_mode {
        eprintln!("note: 'xiom publish' is deprecated -- delegating to the verified 'xiom pkg publish' client.");
        let mut forwarded: Vec<String> = vec!["publish".to_string()];
        if let Some(pos) = args.iter().position(|a| a == "publish") {
            forwarded.extend(args.iter().skip(pos + 1).cloned());
        }
        run_tool_dispatch("xiom-pkg", &forwarded);
    }

    if doctor_mode {
        run_doctor(&args);
        return;
    }

    if doc_mode {
        run_doc(&args);
        return;
    }

    let verify = args.flag("verify") || args.raw_has("--verify-output");
    let verify_output = args.value("verify-output");

    let output_file = args.value("output");

    let link_libs = args.values("link");
    let link_paths = args.values("link-path");
    let c_sources = args.values("c-source");

    // Resolved early so the timeout watchdog can consult the project
    // manifest ([compiler] timeout-secs) before spawning.
    let source_paths = resolve_source_files(&args);

    let timeout_secs: u64 = args.value("timeout")
        .and_then(|v| v.parse().ok())
        .or_else(|| {
            // AUDIT #18 (completion): xiom.toml [compiler] timeout-secs is a
            // project default (explicit --timeout wins). Honored now that the
            // watchdog uses cooperative cancellation instead of a worker
            // thread process::exit (audit #12).
            source_paths.first().and_then(|p| {
                xiom_graph::manifest::find_and_parse_manifest(std::path::Path::new(p))
                    .ok()
                    .and_then(|m| m.compiler.timeout_secs)
                    .map(|t| t as u64)
            })
        })
        .unwrap_or(300);

    let max_recursion_depth: u32 = args.value("max-depth")
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);
    let max_recursion_depth = std::cmp::min(max_recursion_depth, 10000u32);

    if timeout_secs > 0 {
        let duration = Duration::from_secs(timeout_secs);
        std::thread::spawn(move || {
            std::thread::sleep(duration);
            eprintln!("error: compilation timed out after {} seconds", timeout_secs);
            // Audit #12: cooperative cancellation -- codegen observes this
            // between functions and the clang child is killed; real_main
            // reports the failure and exits from the MAIN thread.
            xiom_codegen::cancel::request_cancel();
        });
    }

    let max_memory_mb: u64 = args.value("max-memory-mb")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if max_memory_mb > 0 {
        let max_bytes = max_memory_mb * 1024 * 1024;
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(2));
                if let Some(used_bytes) = get_process_memory_bytes() {
                    if used_bytes > max_bytes {
                        eprintln!(
                            "error: memory budget exceeded ({} MB used of {} MB limit). \
                             Try --max-memory-mb with a higher value or simplify the input.",
                            used_bytes / 1024 / 1024,
                            max_memory_mb
                        );
                        // Audit #12: cooperative cancellation (see timeout).
                        xiom_codegen::cancel::request_cancel();
                        return;
                    }
                }
            }
        });
    }

    let source_paths = resolve_source_files(&args);

    // -- M10.2: xiom build --standalone -- script-to-binary ----------
    let standalone_mode = args.flag("standalone");
    let scaffold_mode = args.flag("scaffold");

    if standalone_mode && !source_paths.is_empty() {
        let script_path = &source_paths[0];
        let source = match std::fs::read_to_string(script_path) {
            Ok(s) => s,
            Err(e) => { eprintln!("error: cannot read '{script_path}': {e}"); process::exit(1); }
        };
        let wrapped = xiom::implicit_main::wrap_implicit_main(&source);
        let out_name = output_file.clone().unwrap_or_else(|| {
            let stem = std::path::Path::new(script_path)
                .file_stem().and_then(|s| s.to_str()).unwrap_or("script");
            if cfg!(windows) { format!("{stem}.exe") } else { stem.to_string() }
        });

        if scaffold_mode {
            let proj_name = std::path::Path::new(script_path)
                .file_stem().and_then(|s| s.to_str()).unwrap_or("script");
            let proj_dir = std::path::Path::new(&proj_name);
            let src_dir = proj_dir.join("src");
            std::fs::create_dir_all(&src_dir).unwrap_or_else(|e| {
                eprintln!("error: cannot create project dir: {e}"); process::exit(1);
            });
            std::fs::write(src_dir.join("main.xi"), &wrapped).unwrap_or_else(|e| {
                eprintln!("error: cannot write main.xi: {e}"); process::exit(1);
            });
            std::fs::write(proj_dir.join("package.xi"), format!(
                "[package]\nname = \"{proj_name}\"\nversion = \"0.1.0\"\n\n[dependencies]\n"
            )).unwrap_or_else(|e| {
                eprintln!("error: cannot write package.xi: {e}"); process::exit(1);
            });
            eprintln!("  Scaffolded project: {proj_name}/");
        }

        let config = CompileConfig {
            output_file: Some(out_name.clone()),
            release: true,
            ..CompileConfig::default()
        };
        let tmp_src = std::env::temp_dir()
            .join("xiom_standalone")
            .join(format!("_script_{:x}.xi", xiom::jit::rand_suffix()));
        let _ = std::fs::create_dir_all(tmp_src.parent().unwrap());
        std::fs::write(&tmp_src, &wrapped).unwrap_or_else(|e| {
            eprintln!("error: cannot write temp file: {e}"); process::exit(1);
        });
        compile_or_exit(&config, &[tmp_src.to_string_lossy().to_string()]);
        eprintln!("  Standalone binary: {out_name}");
        return;
    }

    if standalone_mode && source_paths.is_empty() {
        eprintln!("usage: xiom build --standalone [--scaffold] <script.xi> [-o output]");
        eprintln!("  Converts a script into a standalone production binary.");
        process::exit(1);
    }

    if source_paths.is_empty() && !test_mode && !build_mode && !graph_mode {
        eprintln!("error: no source file(s) provided");
        process::exit(1);
    }

    if test_mode {
        run_xiom_tests(&args);
        return;
    }

    let config = {
        let mut config = CompileConfig {
            target,
            emit_ir,
            do_run,
            check_only,
            dump_check: false,
            release,
            opt_level,
            check_contracts,
            diagnostics_json,
            strict_mode,
            debug_symbols,
            shared_lib,
            static_lib,
            max_recursion_depth,
            dump_contracts: args.flag("dump-contracts"),
            verify,
            verify_output,
            output_file,
            link_libs,
            link_paths,
            c_sources,
        hot_reload: false,  // set to true by hot reload loop below
        hot_reload_contracts,
        sanitize,
        stack_protector,
        runtime_contracts,
        keep_debug_checks,
        overflow_checks,
        strict_exhaustive,
        incremental,
        force: force_recompile,
        parallel,
        jobs,
        script_mode: false,
        cache: use_cache,
        lto: use_lto,
        parallel_codegen,
        enable_unsafe_direct,
        extra_source_dirs: Vec::new(),
    };

    // AUDIT #18 FIX (second half): the xiom.toml `[compiler]` table was
    // PARSED but IGNORED -- project settings never reached compilation.
    // Manifest values now act as PROJECT DEFAULTS; explicit CLI flags keep
    // precedence (a flag absent = field not chosen by the user).
    if let Some(first_src) = source_paths.first() {
        if let Ok(manifest) = xiom_graph::manifest::find_and_parse_manifest(
            std::path::Path::new(first_src))
        {
            let cc = &manifest.compiler;
            if cc.release && !release {
                config.release = true;
            }
            if cc.incremental && !incremental {
                config.incremental = true;
            }
            if let Some(d) = cc.max_depth {
                if args.value("max-depth").is_none() {
                    config.max_recursion_depth = d;
                }
            }
            if let Some(t) = &cc.target {
                if args.value("target").is_none() {
                    match t.as_str() {
                        "wasm" => config.target = Target::Wasm,
                        "wasi" => config.target = Target::Wasi,
                        "arm" => config.target = Target::Arm,
                        "riscv" => config.target = Target::RisCv,
                        "native" => config.target = Target::Native,
                        other => eprintln!("warning: ignoring unknown [compiler] target '{other}'"),
                    }
                }
            }
            if std::env::var("XIOM_VERBOSE_CONFIG").as_deref() == Ok("1") {
                eprintln!("[config] manifest applied: release={} incremental={} max_depth={:?}",
                    config.release, config.incremental, cc.max_depth);
            }
        }
    }
    config
    };

    // 7F.2: Build graph visualization
    if let Some(fmt) = graph_format {
        let first = if !source_paths.is_empty() {
            std::path::Path::new(&source_paths[0]).to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        };
        match xiom_graph::build_project_graph(&first) {
            Ok(graph) => {
                let format = if fmt == "mermaid" {
                    xiom::graph_viz::GraphFormat::Mermaid
                } else {
                    xiom::graph_viz::GraphFormat::Dot
                };
                let output = xiom::graph_viz::generate_dot_graph(&graph, format);
                println!("{output}");
            }
            Err(e) => {
                eprintln!("error: cannot build dependency graph: {e}");
                process::exit(1);
            }
        }
        return;
    }

    // --emit-tokens: output token stream and exit
    if emit_tokens && !source_paths.is_empty() {
        for path_str in &source_paths {
            let source = match std::fs::read_to_string(path_str) {
                Ok(s) => s,
                Err(e) => { eprintln!("error: {}: {}", path_str, e); continue; }
            };
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();
            for tok in &tokens {
                println!("{}:{}: {:?}", tok.span.line, tok.span.col, tok.kind);
            }
        }
        return;
    }

    // --dump-tokens: canonical token dump and exit (selfhost Phase 1 gate)
    if dump_tokens_flag && !source_paths.is_empty() {
        for path_str in &source_paths {
            let source = match std::fs::read_to_string(path_str) {
                Ok(s) => s,
                Err(e) => { eprintln!("error: {}: {}", path_str, e); continue; }
            };
            print!("{}", dump_tokens(&source));
        }
        return;
    }

    // --dump-ast: canonical AST dump and exit (selfhost Phase 2 gate)
    if dump_ast_flag && !source_paths.is_empty() {
        for path_str in &source_paths {
            let source = match std::fs::read_to_string(path_str) {
                Ok(s) => s,
                Err(e) => { eprintln!("error: {}: {}", path_str, e); continue; }
            };
            print!("{}", dump_ast(&source));
        }
        return;
    }

    // --dump-check: canonical checker diagnostics and exit (Phase 3 gate)
    if dump_check_flag && !source_paths.is_empty() {
        for path_str in &source_paths {
            print!("{}", dump_check(path_str));
        }
        return;
    }

    // 7F.1: Build daemon mode
    if build_mode && !watch_mode {
        if !source_paths.is_empty() {
            let (resolved, _) = xiom::expand_sources_with_graph(&source_paths);
            compile_or_exit(&config, &resolved);
        } else {
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            if let Ok(graph) = xiom_graph::build_project_graph(&cwd) {
                eprintln!("Build: {} ({} modules)", graph.project_name, graph.len());
                match graph.compilation_order() {
                    Ok(order) => {
                        let files: Vec<String> = order.iter()
                            .map(|p| p.to_string_lossy().to_string())
                            .collect();
                        compile_or_exit(&config, &files);
                    }
                    Err(e) => { eprintln!("error: {e}"); process::exit(1); }
                }
            } else {
                eprintln!("error: no xiom.toml or package.xi found. Run 'xiom init' first.");
                process::exit(1);
            }
        }
        return;
    }

    // --sandbox: run safety audit and exit (skips compilation unless --sandbox=strict passes)
    let sandbox_mode = args.iter().any(|a| a == "--sandbox" || a.starts_with("--sandbox="));
    if sandbox_mode {
        let strict = args.iter().any(|a| a == "--sandbox=strict");
        let json_output = args.iter().any(|a| a == "--sandbox-report=json");
        let _text_output = !json_output && !args.iter().any(|a| a.starts_with("--sandbox-report="));
        let output_file = args.iter().position(|a| a == "--sandbox-report")
            .and_then(|i| args.get(i + 1).cloned());
        // Also support --sandbox-report=<path>
        let output_file = output_file.or_else(|| {
            args.iter().find(|a| a.starts_with("--sandbox-report="))
                .and_then(|a| a.strip_prefix("--sandbox-report="))
                .map(|s| if s == "json" || s == "text" || s == "silent" { "" } else { s })
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        });
        let silent = args.iter().any(|a| a == "--sandbox-report=silent");

        let mut overall_exit = 0;
        for source_path in &source_paths {
            // AUDIT #11 FIX: unreadable input used to flow through
            // unwrap_or_default() as an EMPTY program -- the audit then
            // scored nothing and the process exited 0 (false-green CI).
            let source = match std::fs::read_to_string(source_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot read '{source_path}': {e}");
                    overall_exit = std::cmp::max(overall_exit, 3);
                    continue;
                }
            };
            let mut parser = Parser::new(Lexer::new(&source).tokenize());
            // AUDIT #11 FIX (part 2): parse failures were silently skipped
            // via `if let Ok`, and partial-AST errors were never taken.
            let program = match parser.parse_program() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: {e}");
                    overall_exit = std::cmp::max(overall_exit, 3);
                    continue;
                }
            };
            let parse_errors = parser.take_errors();
            if !parse_errors.is_empty() {
                for e in &parse_errors {
                    eprintln!("parse error: {e:?}");
                }
                overall_exit = std::cmp::max(overall_exit, 3);
                continue;
            }
            {
                let mut auditor = SafetyAuditor::new();
                let report = auditor.audit(&program, source_path);

                let output = if json_output {
                    report.to_json()
                } else {
                    report.to_text()
                };

                if let Some(ref path) = output_file {
                    if !path.is_empty() {
                        let _ = std::fs::write(path, &output);
                    }
                }
                if !silent && output_file.is_none() {
                    println!("{output}");
                }

                if strict && (report.summary.safety_score == "HIGH" || report.summary.safety_score == "CRITICAL") {
                    eprintln!("error: --sandbox=strict blocked compilation due to {} HIGH severity findings", report.summary.high_severity);
                    overall_exit = 3;
                } else if report.summary.safety_score == "HIGH" || report.summary.safety_score == "CRITICAL" {
                    overall_exit = std::cmp::max(overall_exit, 2);
                } else if report.summary.safety_score == "MEDIUM" {
                    overall_exit = std::cmp::max(overall_exit, 1);
                }
            }
        }
        process::exit(overall_exit);
    }

    // 5e Hot Reload
    if watch_mode || hot_reload {
        let hot_config = CompileConfig {
            shared_lib: hot_reload || shared_lib,
            hot_reload,
            incremental,
            force: force_recompile,
            ..config
        };
        eprintln!("\n[HOT RELOAD] Watching {} source file(s)...", source_paths.len());
        eprintln!("[HOT RELOAD] Press Ctrl+C to stop.\n");

        let mut last_mod: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for p in &source_paths {
            if let Ok(meta) = std::fs::metadata(p) {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                        last_mod.insert(p.clone(), dur.as_secs());
                    }
                }
            }
        }

        // Initial compile -- exit on failure for hot reload
        if let Err(errors) = compile(&hot_config, &source_paths) {
            for e in &errors { eprintln!("error: {e}"); }
            process::exit(1);
        }

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let mut changed = false;
            for p in &source_paths {
                if let Ok(meta) = std::fs::metadata(p) {
                    if let Ok(mtime) = meta.modified() {
                        if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
                            let s = dur.as_secs();
                            if last_mod.get(p) != Some(&s) { last_mod.insert(p.clone(), s); changed = true; }
                        }
                    }
                }
            }
            if changed { if let Err(errors) = compile(&hot_config, &source_paths) { for e in &errors { eprintln!("error: {e}"); } } }
        }
    }

    // 5g AI Pipeline: run check-only compile first to get diagnostics, then call LLM
    if ai_mode || ai_local || ai_dry_run {
        let ai_config = xiom::ai::load_ai_config(ai_model.clone());
        let mut ai_config = xiom::ai::AiConfig {
            enabled: true, local_only: ai_local, dry_run: ai_dry_run,
            silent: ai_silent, strict: ai_strict,
            timeout_secs: ai_timeout.unwrap_or(ai_config.timeout_secs),
            model: ai_model.unwrap_or(ai_config.model),
            ..ai_config
        };
        // AI-05: enforce the --ai-local privacy contract and refuse plaintext
        // key transmission before any request.
        if let Err(e) = xiom::ai::finalize_config(&mut ai_config) {
            eprintln!("[AI] {e}");
            process::exit(1);
        }
        eprintln!("[AI] Provider: {}, Model: {}",
            ai_config.provider, ai_config.model);

        // 5f.3e: Batch mode -- collect diagnostics from all files into single output
        if _ai_batch {
            let mut all_diagnostics: Vec<xiom::Diagnostic> = Vec::new();
            let mut all_sources: Vec<(String, String)> = Vec::new(); // (path, source)

            for path in &source_paths {
                let check_config = CompileConfig {
                    check_only: true, emit_ir: true, diagnostics_json: true,
                    dump_check: false,
                    target: config.target, release: config.release,
                    do_run: false, check_contracts: config.check_contracts,
                    strict_mode: config.strict_mode, debug_symbols: config.debug_symbols,
                    shared_lib: false, static_lib: false,
                    max_recursion_depth: config.max_recursion_depth,
                    dump_contracts: config.dump_contracts,
                    verify: config.verify,
                    verify_output: config.verify_output.clone(),
                    output_file: None,
                    incremental: false, force: false,
                    parallel: false, jobs: 0,
                    link_libs: vec![],
                    link_paths: vec![],
                    c_sources: vec![],
                    hot_reload: false,
                    hot_reload_contracts: false,
                    sanitize: None,
                    stack_protector: false,
                    runtime_contracts: false,
                    keep_debug_checks: false,
                    overflow_checks: config.overflow_checks,
                    strict_exhaustive: config.strict_exhaustive,
                    script_mode: false,
                    cache: false,
                    lto: false,
                    parallel_codegen: false,
                    enable_unsafe_direct: enable_unsafe_direct,
                    opt_level: config.opt_level,
                    extra_source_dirs: config.extra_source_dirs.clone(),
                };
                let result = xiom::compile_with_diagnostics(&check_config, &[path.clone()]);
                let source = std::fs::read_to_string(path).unwrap_or_default();
                all_diagnostics.extend(result.diagnostics);
                all_sources.push((path.clone(), source));
            }

            if !all_diagnostics.is_empty() {
                // 5f.3f: Run Z3 verification for contract violations to get counterexamples
                let z3_models = xiom::ai::run_z3_for_contract_errors(&all_diagnostics, &all_sources);
                match xiom::ai::run_ai_pipeline_batch(&ai_config, &all_sources, &all_diagnostics, &z3_models) {
                    Ok(output) if !ai_silent => {
                        eprintln!("xiom --ai --batch: {} hints -> .xiom_ai.json ({} API, {} cached, {} Z3 models)",
                            output.total_hints, output.api_calls, output.cached_hints, z3_models.len());
                    }
                    Err(e) => eprintln!("[AI] {e}"),
                    _ => {}
                }
            } else {
                eprintln!("[AI] All sources compile cleanly -- no diagnostics.");
            }
        } else {
            // Single-file mode (existing behavior)
            for path in &source_paths {
                let check_config = CompileConfig {
                    check_only: true, emit_ir: true, diagnostics_json: true,
                    dump_check: false,
                    target: config.target, release: config.release,
                    do_run: false, check_contracts: config.check_contracts,
                    strict_mode: config.strict_mode, debug_symbols: config.debug_symbols,
                    shared_lib: false, static_lib: false,
                    max_recursion_depth: config.max_recursion_depth,
                    dump_contracts: config.dump_contracts,
                    verify: config.verify,
                    verify_output: config.verify_output.clone(),
                    output_file: None,
                    incremental: false, force: false,
                    parallel: false, jobs: 0,
                    link_libs: vec![],
                    link_paths: vec![],
                    c_sources: vec![],
                    hot_reload: false,
                    hot_reload_contracts: false,
                    sanitize: None,
                    stack_protector: false,
                    runtime_contracts: false,
                    keep_debug_checks: false,
                    overflow_checks: config.overflow_checks,
                    strict_exhaustive: config.strict_exhaustive,
                    script_mode: false,
                    cache: false,
                    lto: false,
                    parallel_codegen: false,
                    enable_unsafe_direct: enable_unsafe_direct,
                    opt_level: config.opt_level,
                    extra_source_dirs: config.extra_source_dirs.clone(),
                };
                let result = xiom::compile_with_diagnostics(&check_config, &[path.clone()]);
                let source = std::fs::read_to_string(path).unwrap_or_default();

                // 5f.3f: Run Z3 for contract errors in single-file mode too
                let diags = result.diagnostics.clone();
                let sources = vec![(path.clone(), source.clone())];
                let z3_models = xiom::ai::run_z3_for_contract_errors(&diags, &sources);

                if !result.diagnostics.is_empty() {
                    match xiom::ai::run_ai_pipeline(&ai_config, &source, path, &result.diagnostics, &z3_models) {
                        Ok(output) if !ai_silent => {
                            eprintln!("xiom --ai: {} hints -> .xiom_ai.json ({} API, {} cached, {} Z3 models)",
                                output.total_hints, output.api_calls, output.cached_hints, z3_models.len());
                        }
                        Err(e) => eprintln!("[AI] {e}"),
                        _ => {}
                    }
                } else {
                    eprintln!("[AI] No diagnostics -- source compiles cleanly.");
                }
            }
        }
    }

    compile_or_exit(&config, &source_paths);
}

/// `--version` / `-V` line. The version is ALWAYS the workspace version from
/// Cargo.toml -- release.yml's guard refuses a tag that does not match it, and
/// every staged tool must contain `v<version>` before the archive is sealed.
/// The release tag and stats suffix are optional BUILD-TIME stamps: when the
/// packaging workflow does not set them, the output is just the true version
/// (no hardcoded, stale milestone claims).
fn version_line() -> String {
    let mut line = format!("XIOM Compiler v{}", env!("CARGO_PKG_VERSION"));
    if let Some(tag) = option_env!("XIOM_RELEASE_TAG") {
        if !tag.is_empty() {
            line.push_str(&format!(" \"{tag}\""));
        }
    }
    if let Some(stats) = option_env!("XIOM_RELEASE_STATS") {
        if !stats.is_empty() {
            line.push_str(&format!(" - {stats}"));
        }
    }
    line
}

fn print_usage() {
    eprintln!("{}", version_line());
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  xiom <source.xi>            Compile to a native binary (or print IR)");
    eprintln!("  xiom run <file.xi>          Compile and run a script");
    eprintln!("  xiom build                  Build the project (xiom.toml / package.xi)");
    eprintln!();
    eprintln!("GETTING STARTED:");
    eprintln!("  doctor              Check the toolchain (LLVM/NASM/z3/stdlib; --json for CI)");
    eprintln!("  doctor --deep       Also compile and run a trivial program end-to-end");
    eprintln!("  --version, -V       Print the compiler version and install root");
    eprintln!("  --help              Show this help");
    eprintln!("  --explain <CODE>    Explain a diagnostic code (e.g. --explain T001)");
    eprintln!();
    eprintln!("PROJECT:");
    eprintln!("  build               Build entire project (from xiom.toml / package.xi)");
    eprintln!("  build --watch       Build daemon: watch and rebuild on changes");
    eprintln!("  run <file.xi>       Execute as a script (auto-wraps in fn main())");
    eprintln!("  run -               Read the script from stdin");
    eprintln!("  run -e \"<code>\"     Execute inline code");
    eprintln!("  --run               Compile and run (requires fn main())");
    eprintln!("  --test              Run the example test suite");
    eprintln!("  --test-dir <dir>    Test directory for --test (default: examples)");
    eprintln!("  --graph             Dependency graph (DOT format); --graph=mermaid");
    eprintln!("  --scaffold          With build --standalone: scaffold from a script");
    eprintln!("  --clean             Clean build artifacts and caches");
    eprintln!();
    eprintln!("TOOLS (launcher: xiom.bat / xiom wrapper):");
    eprintln!("  fmt | lsp | mcp | pkg | dbg | verify | ffigen | ai | graph | test");
    eprintln!("  Each runs the matching xiom-* binary from the same bin/ directory.");
    eprintln!("  <tool> --help       Show a tool's own help (e.g. xiom pkg --help)");
    eprintln!();
    eprintln!("OUTPUT:");
    eprintln!("  -o <output>         Output binary path (default: a.exe on Windows, a.out elsewhere)");
    eprintln!("  --target <target>   Target: native (default), wasm, wasi, arm, riscv");
    eprintln!("  --emit-ir           Print LLVM IR to stdout (no compilation)");
    eprintln!("  --emit-tokens       Print the token stream and exit");
    eprintln!("  --dump-tokens       Print the canonical token dump (selfhost parity gate)");
    eprintln!("  --dump-ast          Print the canonical AST dump (selfhost parity gate)");
    eprintln!("  --dump-check        Print the canonical checker dump (selfhost parity gate)");
    eprintln!("  --diagnostics=json  Output diagnostics as JSON");
    eprintln!("  --dump-contracts    Print the contract index as JSON");
    eprintln!("  --shared            Compile as a shared library (DLL)");
    eprintln!("  --static            Compile as a static library");
    eprintln!("  --standalone        Standalone build (no stdlib runtime dependency)");
    eprintln!();
    eprintln!("SAFETY:");
    eprintln!("  --no-contracts      Disable contract runtime checks (faster, less safe)");
    eprintln!("  --runtime-contracts Force runtime contract checks even in release mode");
    eprintln!("  --keep-debug-checks Keep assert/dbg!/debugger in release builds");
    eprintln!("  --sandbox           Run safety audit on unsafe blocks (text report)");
    eprintln!("  --sandbox=strict    Block compilation on HIGH severity findings");
    eprintln!("  --sandbox-report=json  Output the sandbox report as JSON");
    eprintln!("  --verify            Generate SMT-LIB contract verification output");
    eprintln!("  --verify-output <f> Write SMT-LIB to a file");
    eprintln!("  --stack-protector   Enable stack canaries (-fstack-protector)");
    eprintln!("  --overflow-checks   Enable integer overflow runtime checks");
    eprintln!("  --enable-unsafe-direct  Allow #[unsafe_direct] (trusted escape hatch)");
    eprintln!();
    eprintln!("BUILD AND CACHE:");
    eprintln!("  --release           Release build (strips contracts and debug intrinsics)");
    eprintln!("  --debug, -g         Emit debug info");
    eprintln!("  --opt-level <0..3>  Explicit optimization level");
    eprintln!("  --lto               ThinLTO link-time optimization");
    eprintln!("  --incremental       Cache compiled IR, skip unchanged sources");
    eprintln!("  --force             Force recompile, ignore all caches");
    eprintln!("  --cache / --no-cache  Binary cache by source hash (default: on)");
    eprintln!("  --parallel          Enable parallel lex+parse (rayon thread pool)");
    eprintln!("  --sequential        Force sequential compilation");
    eprintln!("  --jobs <N>          Number of parallel compile jobs (default: num CPUs)");
    eprintln!("  --parallel-codegen  Rayon-based per-function IR emission");
    eprintln!("  --jit               In-process JIT execution (clang backend)");
    eprintln!("  --lazy              Incremental recompilation (with --jit)");
    eprintln!("  --watch             Watch source files and recompile on change");
    eprintln!("  --hot-reload        Hot reload: watch + shared library");
    eprintln!("  --hot-reload-contracts  Verify contracts before hot-swapping pointers");
    eprintln!("  --sanitize=<type>   Sanitizer: address, undefined, leak, thread");
    eprintln!("  --strict            Strict mode (alias: --strict-mode)");
    eprintln!("  --strict-exhaustive Strict exhaustiveness checks");
    eprintln!("  --timeout <seconds> Set compilation timeout (default: 300; 0 disables)");
    eprintln!("  --max-memory-mb <N> Set max memory budget in MB (0 = disabled)");
    eprintln!("  --max-depth <N>     Max contract recursion depth (default: 500)");
    eprintln!("  --link <name>       Link a native library (repeatable, e.g. vulkan-1)");
    eprintln!("  --link-path <dir>   Add a library search path (repeatable, -L<dir>)");
    eprintln!("  --c-source <file>   Link an extra C/object file (repeatable)");
    eprintln!("  --check             Type-check only (no code generation)");
    eprintln!();
    eprintln!("PACKAGES, BENCHMARKS, AI:");
    eprintln!("  --registry <url>    Package registry URL (pkg)");
    eprintln!("  --locked            Use the lockfile exactly (pkg)");
    eprintln!("  --frozen            Offline lockfile-only mode (pkg)");
    eprintln!("  --count <N>         Iterations for `xiom bench` (default: 10)");
    eprintln!("  --bench-file <f>    Benchmark a single file");
    eprintln!("  --ai                AI-assisted diagnostics (Ollama or API key)");
    eprintln!("  --ai-local          Local LLM only, never sends code off-machine");
    eprintln!("  --ai-dry-run        Print the prompt, do not call the LLM");
    eprintln!("  --ai-silent         Suppress stdout, write only .xiom_ai.json");
    eprintln!("  --ai-strict         Refuse binary output on any contract violation");
    eprintln!("  --ai-batch, --batch Analyze all sources, single .xiom_ai.json");
    eprintln!("  --ai-model=<name>   Override the AI model (default: codellama)");
    eprintln!("  --ai-timeout=<sec>  LLM call timeout (default: 10s)");
    eprintln!("  --help-ai           AI mode setup and configuration guide");
    eprintln!();
    eprintln!("SUBCOMMANDS:");
    eprintln!("  repl                Interactive REPL");
    eprintln!("  build-runtime       Build the C runtime library");
    eprintln!("  doc <file.xi>       Generate documentation (Markdown/HTML)");
    eprintln!();
    eprintln!("DEPENDENCIES:");
    eprintln!("  Required: clang (LLVM) -- compile IR to a native binary (see 'xiom doctor')");
    eprintln!("  Optional: opt (LLVM) -- IR optimization passes");
    eprintln!("  Optional: nasm -- hardware-accelerated crypto/memcpy (stdlib)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  xiom run examples/demo_float.xi");
    eprintln!("  xiom -o prog.exe source.xi");
    eprintln!("  xiom --emit-ir examples/demo_float.xi");
    eprintln!("  xiom --target wasm -o prog.wasm source.xi");
    eprintln!("  xiom --verify examples/phase1_contracts.xi");
}

fn parse_target(target: Option<&str>) -> Target {
    match target {
        Some("wasm") => Target::Wasm,
        Some("wasi") => Target::Wasi,
        Some("arm") => Target::Arm,
        Some("riscv") => Target::RisCv,
        _ => Target::Native,
    }
}

fn get_process_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "windows")]
    {
        let pid = std::process::id();
        if let Ok(output) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {pid}).WorkingSet64")])
            .output()
        {
            if output.status.success() {
                if let Ok(s) = String::from_utf8(output.stdout) {
                    if let Ok(bytes) = s.trim().parse::<u64>() {
                        return Some(bytes);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let kb: u64 = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return Some(kb * 1024);
                }
            }
        }
        None
    }
}

fn run_xiom_tests(args: &cli::Cli) {
    use std::process::Command as Cmd;

    // Legacy contract: `--test <dir>` used the token after `--test` as the
    // directory (even though `--test` is also a bool flag); `--test-dir` is
    // the explicit spelling. Kept on the raw view to preserve that quirk.
    let test_dir = args.iter().position(|a| a == "--test")
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| args.value("test-dir"))
        .unwrap_or_else(|| "examples".to_string());

    let mut test_files: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&test_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("xi") {
                test_files.push(path.to_string_lossy().to_string());
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir(&test_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub) = std::fs::read_dir(&path) {
                    for e in sub.flatten() {
                        let p = e.path();
                        if p.extension().and_then(|ext| ext.to_str()) == Some("xi") {
                            test_files.push(p.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    if test_files.is_empty() {
        eprintln!("  No .xi test files found in {}", test_dir);
        std::process::exit(1);
    }

    eprintln!("  Running {} test(s)...", test_files.len());
    let mut passed = 0usize;
    let mut failed = 0usize;

    for test_file in &test_files {
        let exe_path = format!("{}.test.exe", test_file);
        let xiompath = std::env::current_exe().unwrap_or_else(|_| "xiom".into());
        let compile = Cmd::new(&xiompath)
            .args(["-o", &exe_path, test_file])
            .output();

        match compile {
            Ok(out) if out.status.success() => {
                match Cmd::new(&exe_path).output() {
                    Ok(run_out) => {
                        if run_out.status.success() {
                            passed += 1;
                            eprintln!("    PASS  {}", test_file);
                        } else {
                            failed += 1;
                            eprintln!("    FAIL  {} (exit code {})", test_file, run_out.status.code().unwrap_or(-1));
                            if !run_out.stderr.is_empty() {
                                eprintln!("      {}", String::from_utf8_lossy(&run_out.stderr).trim());
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        eprintln!("    FAIL  {} (cannot run: {})", test_file, e);
                    }
                }
                let _ = std::fs::remove_file(&exe_path);
            }
            Ok(_) => {
                failed += 1;
                eprintln!("    FAIL  {} (compilation failed)", test_file);
            }
            Err(e) => {
                failed += 1;
                eprintln!("    FAIL  {} (cannot compile: {})", test_file, e);
            }
        }
    }

    eprintln!("  {} passed, {} failed", passed, failed);
    if failed > 0 { std::process::exit(1); }
}

// -- Phase 5d: Package Manager ------------------------------------------

// R50 (registry relay): the legacy `xiom install` / `xiom update` handlers
// (git clone from registry /packages.json, no checksum/signature/yank) were
// removed. `xiom install` now delegates to the verified `xiom pkg install`
// client and `xiom update` is retired with guidance -- see real_main.
// FE-8: the legacy `xiom publish` git-tag flow was removed the same way; it
// now delegates to `xiom pkg publish`.

fn dirs_next() -> Option<String> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
}

fn handle_registry(args: &[String]) {
    // CRB-3c wrinkle: the legacy registry list now lives under XIOM_HOME
    // (`<home>/registry.json`) so `xiom install/registry` and `xiom pkg`
    // share one tree; an existing `~/.xiom/registry.json` is still honored
    // so old setups keep working.
    let home = xiom_graph::paths::xiom_home();
    let new_path = home.join("registry.json");
    let legacy_path = dirs_next()
        .map(|h| std::path::PathBuf::from(format!("{}/.xiom/registry.json", h)));
    // An EXPLICIT XIOM_HOME always wins; otherwise prefer the canonical file
    // and only fall back to the legacy `~/.xiom/registry.json` when it is
    // the one that actually exists.
    let xiom_home_explicit = std::env::var("XIOM_HOME")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let reg_path = if xiom_home_explicit || new_path.exists() {
        new_path
    } else {
        legacy_path.filter(|p| p.exists()).unwrap_or(new_path)
    };
    let reg_path = reg_path.to_string_lossy().to_string();
    let reg_dir = std::path::Path::new(&reg_path)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| home.clone());

    let sub_cmd = args.iter()
        .position(|a| a == "registry")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| "list".to_string());

    match sub_cmd.as_str() {
        "init" => {
            let initial = serde_json::json!({ "packages": {} });
            if let Ok(json) = serde_json::to_string_pretty(&initial) {
                std::fs::create_dir_all(&reg_dir).ok();
                if std::fs::write(&reg_path, &json).is_ok() {
                    eprintln!("  Created local registry: {}", reg_path);
                }
            }
        }
        "add" => {
            let pkg_name = args.iter()
                .position(|a| a == "add")
                .and_then(|i| args.get(i + 1).cloned())
                .or_else(|| args.iter()
                    .position(|a| a == "registry")
                    .and_then(|i| args.get(i + 2).cloned()));
            let repo_url = args.iter()
                .position(|a| a == "add")
                .and_then(|i| args.get(i + 2).cloned())
                .or_else(|| args.iter()
                    .position(|a| a == "registry")
                    .and_then(|i| args.get(i + 3).cloned()));

            match (pkg_name, repo_url) {
                (Some(name), Some(url)) => {
                    let mut registry: serde_json::Value = if let Ok(content) = std::fs::read_to_string(&reg_path) {
                        serde_json::from_str(&content).unwrap_or(serde_json::json!({ "packages": {} }))
                    } else {
                        serde_json::json!({ "packages": {} })
                    };
                    if let Some(pkgs) = registry.get_mut("packages").and_then(|p| p.as_object_mut()) {
                        let entry = serde_json::json!({
                            "repo": url,
                            "description": "",
                            "license": "MIT"
                        });
                        pkgs.insert(name.clone(), entry);
                        if let Ok(json) = serde_json::to_string_pretty(&registry) {
                            std::fs::create_dir_all(&reg_dir).ok();
                            std::fs::write(&reg_path, &json).ok();
                            eprintln!("  Added '{}' to local registry -> {}", name, url);
                        }
                    }
                }
                _ => {
                    eprintln!("  Usage: xiom registry add <name> <repo-url>");
                }
            }
        }
        "list" | _ => {
            if let Ok(content) = std::fs::read_to_string(&reg_path) {
                if let Ok(registry) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(pkgs) = registry["packages"].as_object() {
                        eprintln!("  Local registry ({} packages):", pkgs.len());
                        for (name, pkg) in pkgs {
                            let repo = pkg["repo"].as_str().unwrap_or("-");
                            let desc = pkg["description"].as_str().unwrap_or("");
                            eprintln!("    {name:<20} {repo:<50} {desc}");
                        }
                    } else {
                        eprintln!("  No packages in local registry. Use 'xiom registry add <name> <url>'");
                    }
                }
            } else {
                eprintln!("  No local registry found. Create one with 'xiom registry init'");
            }
        }
    }
}

fn run_benchmarks(args: &cli::Cli, iterations: u32) {
    // `xiom bench [<dir>]` -- the directory is the positional word after
    // `bench`; `--bench-file` selects a single file.
    let bench_dir = args.iter().position(|a| a == "bench")
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| "benches".to_string());

    let bench_file = args.value("bench-file")
        .or_else(|| args.iter().find(|a| a.ends_with(".xi")).cloned());

    let mut bench_files: Vec<String> = Vec::new();
    if let Some(file) = bench_file {
        bench_files.push(file);
    } else {
        if let Ok(entries) = std::fs::read_dir(&bench_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("xi") {
                    bench_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    if bench_files.is_empty() {
        eprintln!("  No benchmark files found. Create a benches/ directory with .xi files.");
        eprintln!("  Or specify a file: xiom bench my_bench.xi");
        return;
    }

    eprintln!("  Running {} benchmark(s) x {} iterations...", bench_files.len(), iterations);

    for bench_file in &bench_files {
        let exe_path = format!("{}.bench.exe", bench_file);
        let xiompath = std::env::current_exe().unwrap_or_else(|_| "xiom".into());

        let compile = std::process::Command::new(&xiompath)
            .args(["-o", &exe_path, "--release", bench_file])
            .output();

        match compile {
            Ok(out) if out.status.success() => {
                let mut times: Vec<f64> = Vec::new();
                for _ in 0..iterations {
                    use std::time::Instant;
                    let start = Instant::now();
                    let _ = std::process::Command::new(&exe_path).output();
                    let elapsed = start.elapsed().as_secs_f64();
                    times.push(elapsed);
                }

                times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let min = times.first().copied().unwrap_or(0.0);
                let max = times.last().copied().unwrap_or(0.0);
                let mean = times.iter().sum::<f64>() / times.len() as f64;
                let median = times[times.len() / 2];

                let name = std::path::Path::new(bench_file)
                    .file_stem().and_then(|n| n.to_str()).unwrap_or(bench_file);
                eprintln!("  {name:<30} {min:>8.4}s  {mean:>8.4}s  {median:>8.4}s  {max:>8.4}s");

                let _ = std::fs::remove_file(&exe_path);
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("  FAIL  {} (compilation failed: {})", bench_file, stderr.lines().next().unwrap_or(""));
            }
            Err(e) => {
                eprintln!("  FAIL  {} (cannot compile: {})", bench_file, e);
            }
        }
    }
    eprintln!("  Benchmark complete.");
}

fn scaffold_project(dir: &str, pkg_name: Option<&str>) {
    let name = pkg_name.unwrap_or_else(|| {
        std::path::Path::new(dir).file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("xiom-project")
    });

    let src_dir = format!("{dir}/src");
    let tests_dir = format!("{dir}/tests");
    for d in &[dir, &src_dir, &tests_dir] {
        if let Err(e) = std::fs::create_dir_all(d) {
            if !std::path::Path::new(d).exists() {
                eprintln!("  Cannot create {}: {}", d, e);
                return;
            }
        }
    }

    let manifest = format!(r#"// XIOM package manifest
name: "{name}"
version: "0.1.0"
authors: ["Your Name"]
license: "MIT"
description: "A new XIOM project"

dependencies: []

sources: [
    "src/main.xi",
]

tests: [
    "tests/test_main.xi",
]
"#);
    let manifest_path = format!("{dir}/package.xi");
    if !std::path::Path::new(&manifest_path).exists() {
        std::fs::write(&manifest_path, &manifest).ok();
    }

    let main_xi = format!(r#"// {name} -- entry point
module {name}

fn main() -> Int {{
    return 0;
}}
"#);
    let main_path = format!("{dir}/src/main.xi");
    if !std::path::Path::new(&main_path).exists() {
        std::fs::write(&main_path, &main_xi).ok();
    }

    let test_xi = format!(r#"// {name} -- tests
module {name}_test

fn test_hello() -> Int {{
    return 0;
}}
"#);
    let test_path = format!("{dir}/tests/test_main.xi");
    if !std::path::Path::new(&test_path).exists() {
        std::fs::write(&test_path, &test_xi).ok();
    }

    let gitignore = "*.exe\n*.ll\n*.obj\n*.o\n*.out\n*.wasm\n*.pdb\n*.ilk\n*.exp\n*.lib\nxiom.lock\n";
    let gitignore_path = format!("{dir}/.gitignore");
    if !std::path::Path::new(&gitignore_path).exists() {
        std::fs::write(&gitignore_path, gitignore).ok();
    }

    eprintln!("  Created project '{name}' in {dir}/");
    eprintln!("  ");
    eprintln!("  {dir}/");
    eprintln!("  |-- package.xi       <- project manifest");
    eprintln!("  |-- src/main.xi      <- entry point");
    eprintln!("  |-- tests/");
    eprintln!("  |   `-- test_main.xi <- tests");
    eprintln!("  `-- .gitignore");
    eprintln!("  ");
    eprintln!("  Next steps:");
    eprintln!("    cd {dir}");
    eprintln!("    xiom check         <- type-check your project");
    eprintln!("    xiom src/main.xi --run   <- compile and run");
    eprintln!("    xiom test          <- run test suite");
}

/// D-2: `xiom toolchain check|update|rollback` (docs/POST_RELEASE_PLAN.md
/// section 1). `check` is read-only against the GitHub Releases API
/// (xiom-lang/xiom only) with the spec's exit codes: 0 up-to-date,
/// 1 update available, 2 verification failed, 3 permission/install-kind.
/// `update` already verifies the archive's SHA256 from SHA256SUMS before it
/// refuses the swap when build-provenance attestation cannot be verified;
/// `rollback` restores the previous bin/lib kept under rollback/.
fn run_toolchain_command(args: &cli::Cli) -> ! {
    let sub = args
        .iter()
        .position(|a| a == "toolchain")
        .and_then(|i| args.iter().skip(i + 1).find(|a| !a.starts_with('-')))
        .map(|s| s.as_str());
    match sub {
        Some("check") => match xiom::toolchain_cmd::check() {
            Ok(report) => {
                if args.flag("json") {
                    println!("{}", xiom::toolchain_cmd::render_check_json(&report));
                } else {
                    print!("{}", xiom::toolchain_cmd::render_check_text(&report));
                }
                process::exit(xiom::toolchain_cmd::check_exit_code(&report));
            }
            Err(e) => {
                if args.flag("json") {
                    println!(
                        "{}",
                        serde_json::json!({"schema": 1, "status": "error", "error": e})
                    );
                }
                eprintln!("error: toolchain check failed: {e}");
                process::exit(2);
            }
        },
        Some("update") => {
            eprintln!("error: 'xiom toolchain update' is not available in this build.");
            eprintln!("       Build-provenance attestation verification is not wired into the");
            eprintln!("       in-process updater yet (dependency procurement). Use:");
            eprintln!("         xiom toolchain check           # current vs latest release");
            eprintln!("         re-run the release installer   # the supported update path");
            process::exit(3);
        }
        Some("rollback") => {
            eprintln!("error: 'xiom toolchain rollback' is not available in this build yet.");
            process::exit(3);
        }
        Some(other) => {
            eprintln!("error: unknown 'xiom toolchain {other}' subcommand (expected: check | update | rollback)");
            process::exit(3);
        }
        None => {
            eprintln!("usage: xiom toolchain check [--json]");
            eprintln!("       xiom toolchain update [--dry-run]");
            eprintln!("       xiom toolchain rollback");
            process::exit(3);
        }
    }
}

/// FE-1..FE-7: `xiom doctor` v2 -- the shared toolchain probe (clang/nasm,
/// PATH then per-OS known locations) and the shared stdlib resolver
/// (`xiom_graph::paths::stdlib_root`), an identity block, OS-specific
/// remediation, and `--json`. Exit codes: 0 all-OK, 1 warnings, 2 errors.
fn run_doctor(args: &cli::Cli) {
    let report = xiom::doctor::build_report(args.flag("deep"));
    if args.flag("json") {
        println!("{}", xiom::doctor::render_json(&report));
    } else {
        print!("{}", xiom::doctor::render_text(&report));
    }
    process::exit(xiom::doctor::exit_code(&report));
}

/// R48 (playground C2): run a sibling tool binary (`fmt`, `lsp`, `mcp`,
/// `pkg`, `dbg`, `verify`, `ffigen`) with the remaining arguments and exit
/// with its status. Mirrors the launcher wrapper's dispatch table so the
/// tools work on installs that have no xiom.bat.
fn run_tool_dispatch(tool: &str, rest: &[String]) -> ! {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let name = if cfg!(windows) { format!("{tool}.exe") } else { tool.to_string() };
    let candidate = exe_dir.join(&name);
    if !candidate.exists() {
        eprintln!(
            "error: {name} not found next to the compiler ({})",
            exe_dir.display()
        );
        eprintln!("       reinstall the toolchain or run `{tool}` from the install launcher");
        process::exit(1);
    }
    match std::process::Command::new(&candidate).args(rest).status() {
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("error: cannot run {}: {e}", candidate.display());
            process::exit(1);
        }
    }
}

/// 9B: xiom doc -- generate documentation for XIOM source files.
/// Delegates to the standalone xiom-doc binary, passing through all args
/// after the `doc` subcommand.
fn run_doc(args: &[String]) {
    // Find the position of "doc" or "--doc" in args, pass everything after it
    let doc_pos = args.iter().position(|a| a == "doc" || a == "--doc").unwrap_or(0);
    let doc_args: Vec<&str> = args[doc_pos + 1..].iter().map(|s| s.as_str()).collect();

    // CRB-3c: same installer-aligned home resolver as `xiom doctor`.
    let home = xiom_graph::paths::xiom_home();
    let doc_bin = home.join("bin")
        .join(if cfg!(windows) { "xiom-doc.exe" } else { "xiom-doc" });

    if doc_bin.exists() {
        let mut cmd = std::process::Command::new(&doc_bin);
        cmd.args(&doc_args);
        let status = cmd.status().unwrap_or_else(|e| {
            eprintln!("xiom doc: failed to run xiom-doc: {e}");
            std::process::exit(1);
        });
        std::process::exit(status.code().unwrap_or(1));
    } else {
        eprintln!("xiom doc: xiom-doc binary not found at {}", doc_bin.display());
        eprintln!("  Build it with: cargo build -p xiom-doc --release");
        eprintln!("  Then copy to: {}", doc_bin.display());
        std::process::exit(1);
    }
}
