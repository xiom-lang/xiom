// XIOM -- Selfhost parser core (Phase 2: 1:1 port of crates/xiom-parser)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Ports lib.rs lines 347-1105 (parse_program .. parse_type_inner start) and
// 1759-1770 (parse_spawn_top_decl):
//   parse_program, parse_file_module_header, build_file_module_result,
//   parse_top_decl(_with_pending), module/use/type/enum/interface/impl decls,
//   parse_attributes, parse_fn_decl, parse_const_decl, parse_module_var,
//   parse_extern_block, parse_spawn_top_decl.
//
// Control flow, spans, and error-unwind order mirror the Rust parser exactly.
// Fallible calls (`?`) latch via p_err and return -1 (or an empty Vec as the
// sentinel); every caller checks p_failed before using a result. Node pushes
// happen in the same order as Rust value construction; child INDICES are
// stored in the parent kind.
//
// NkFn arena order: (is_pub, is_async, recv, name, generics, params, ret,
// contracts, body, attrs); recv/ret/body are -1 when absent.
//
// The one known non-span deviation: an attribute argument whose value is not
// a Str/Int/Bool literal falls back to a partial rendering instead of Rust
// `format!("{:?}", expr)` (see pc_attr_value_str).

module selfhost_parser_core

use xiom.string;
use selfhost_ast;
use selfhost_ast.Node;
use selfhost_ast.NodeKind;
use selfhost_ast.Span;
use selfhost_lexer.Token;
use selfhost_lexer.TokenKind;
use selfhost_parser_state;
use selfhost_parser_state.Parser;
use selfhost_parser_expr;

// ============================================================================
// Local helpers
// ============================================================================

fn pc_tok_span(t: Token) -> Span {
  return selfhost_ast.span_new(t.line, t.col, t.byte_start, t.byte_end);
}

fn pc_ident_name(p: &Parser, idx: Int) -> Str {
  if idx < 0 { return ""; }
  match p.nodes[idx].kind {
    NkIdent(name) => { return name; }
    _ => { return ""; }
  }
}

/// Non-negative decimal rendering (Rust `usize` Display for `_{idx}` names).
fn pc_dec_str(v: Int) -> Str {
  if v <= 0 { return "0"; }
  var buf = Vec[UInt8].new();
  var digits: [24]UInt8;
  var pos = 24;
  var n = v;
  while n > 0 {
    pos = pos - 1;
    digits[pos] = (48 + n % 10) as UInt8;
    n = n / 10;
  }
  while pos < 24 {
    buf.push(digits[pos]);
    pos = pos + 1;
  }
  return Str::from_utf8(buf);
}

/// Rust `u64::to_string` (attribute Int values).
fn pc_uint_str(v: UInt) -> Str {
  if v == (0 as UInt) { return "0"; }
  var buf = Vec[UInt8].new();
  var digits: [24]UInt8;
  var pos = 24;
  var n = v;
  while n > (0 as UInt) {
    pos = pos - 1;
    digits[pos] = (48 + ((n % (10 as UInt)) as Int)) as UInt8;
    n = n / (10 as UInt);
  }
  while pos < 24 {
    buf.push(digits[pos]);
    pos = pos + 1;
  }
  return Str::from_utf8(buf);
}

/// Rust `lexeme.trim_matches('"')` (extern linkage).
fn pc_trim_quotes(s: Str) -> Str {
  var lo = 0;
  var hi = s.len();
  while lo < hi && string.byte_at(s, lo) == 34 { lo = lo + 1; }
  while hi > lo && string.byte_at(s, hi - 1) == 34 { hi = hi - 1; }
  return string.str_slice(s, lo, hi);
}

/// Rust parse_attributes value rendering. Str/Int/Bool are exact; other
/// expression kinds use a partial fallback (Rust uses `format!("{:?}", val)`).
fn pc_attr_value_str(p: &Parser, idx: Int) -> Str {
  match p.nodes[idx].kind {
    NkLitStr(data) => { return Str::from_utf8(data); }
    NkLitInt(v) => { return pc_uint_str(v); }
    NkLitBool(value) => {
      if value != 0 { return "true"; }
      return "false";
    }
    NkLitFloat(lex) => { return lex; }
    NkLitChar(cp) => { return pc_dec_str(cp); }
    NkLitBigInt(hi, lo) => { return pc_uint_str(hi) + pc_uint_str(lo); }
    NkIdent(name) => { return name; }
    _ => { return ""; }
  }
}

fn pc_extend(dst: &mut Vec[Int], src: Vec[Int]) {
  var i = 0;
  while i < src.len() {
    dst.push(src[i]);
    i = i + 1;
  }
}

fn pc_push_front(v: Vec[Int], x: Int) -> Vec[Int] {
  var nv = Vec[Int].new();
  nv.push(x);
  var i = 0;
  while i < v.len() {
    nv.push(v[i]);
    i = i + 1;
  }
  return nv;
}

fn pc_generic_name(p: &Parser, gidx: Int) -> Str {
  var n = -1;
  match p.nodes[gidx].kind {
    NkGeneric(name, bounds, is_const, const_ty) => { n = name; }
    _ => { return ""; }
  }
  return pc_ident_name(p, n);
}

fn pc_generic_bounds_empty(p: &Parser, gidx: Int) -> Bool {
  match p.nodes[gidx].kind {
    NkGeneric(name, bounds, is_const, const_ty) => { return bounds.len() == 0; }
    _ => { return true; }
  }
}

/// In-place `gp.bounds.push(bound)` (Rust `&mut generics` where clause).
fn pc_generic_push_bound(p: &mut Parser, gidx: Int, bound: Int) {
  var n = -1;
  var old = Vec[Int].new();
  var ic = 0;
  var ct = -1;
  var ok = false;
  match p.nodes[gidx].kind {
    NkGeneric(name, bounds, is_const, const_ty) => {
      n = name;
      old = bounds;
      ic = is_const;
      ct = const_ty;
      ok = true;
    }
    _ => {}
  }
  if !ok { return; }
  var nb = Vec[Int].new();
  var i = 0;
  while i < old.len() {
    nb.push(old[i]);
    i = i + 1;
  }
  nb.push(bound);
  p.nodes[gidx].kind = NodeKind.NkGeneric(n, nb, ic, ct);
}

/// `matches!(peek_kind(), TokenKind::Ident(s) if s == kw)`.
fn pc_kw_is(p: &Parser, kw: Str) -> Bool {
  match selfhost_parser_state.p_peek_kind(p) {
    TkIdent(s) => { return s == kw; }
    _ => { return false; }
  }
}

fn pc_is_contract_kw(p: &Parser) -> Bool {
  match selfhost_parser_state.p_peek_kind(p) {
    TkIdent(s) => { return s == "requires" || s == "ensures"; }
    _ => { return false; }
  }
}

fn pc_is_ident(p: &Parser) -> Bool {
  match selfhost_parser_state.p_peek_kind(p) {
    TkIdent(_) => { return true; }
    _ => { return false; }
  }
}

// --- TokenKind equality workaround -----------------------------------------
// `TokenKind == TokenKind` lowers to `icmp eq %struct.Vec` for this payload
// enum and clang rejects the IR (COMPILER_BUGS 2026-10-02 (g); repro
// tmp/sprintc/phase2_parser/core_agent/repro_enum_eq/probe_pskip.xi). Tag
// codes preserve `==` semantics without the struct compare. This also
// replaces
// parser_state.p_skip / p_peek_is / p_is_eof / p_expect_kind, whose bodies
// use `==` (every module function is codegen'd, so the broken IR is emitted
// even when unused).

fn pc_kind_code(k: TokenKind) -> Int {
  match k {
    TkLet => { return 1; }
    TkVar => { return 2; }
    TkConst => { return 3; }
    TkFn => { return 4; }
    TkReturn => { return 5; }
    TkBreak => { return 6; }
    TkContinue => { return 7; }
    TkIf => { return 8; }
    TkElif => { return 9; }
    TkElse => { return 10; }
    TkMatch => { return 11; }
    TkWhile => { return 12; }
    TkFor => { return 13; }
    TkIn => { return 14; }
    TkSpawn => { return 15; }
    TkAwait => { return 16; }
    TkComptime => { return 17; }
    TkAsm => { return 18; }
    TkDefer => { return 19; }
    TkMove => { return 20; }
    TkModule => { return 21; }
    TkUse => { return 22; }
    TkPub => { return 23; }
    TkAs => { return 24; }
    TkType => { return 25; }
    TkEnum => { return 26; }
    TkInterface => { return 27; }
    TkDerive => { return 28; }
    TkImpl => { return 29; }
    TkTrue => { return 30; }
    TkFalse => { return 31; }
    TkSelf => { return 32; }
    TkSome => { return 33; }
    TkNone => { return 34; }
    TkOkV => { return 35; }
    TkErrV => { return 36; }
    TkUnsafe => { return 37; }
    TkExtern => { return 38; }
    TkIs => { return 39; }
    TkIdent(_) => { return 40; }
    TkInt(_) => { return 41; }
    TkBigInt(_, _) => { return 42; }
    TkFloat(_) => { return 43; }
    TkStr(_) => { return 44; }
    TkChar(_) => { return 45; }
    TkDot => { return 46; }
    TkComma => { return 47; }
    TkSemicolon => { return 48; }
    TkColon => { return 49; }
    TkColonColon => { return 50; }
    TkLParen => { return 51; }
    TkRParen => { return 52; }
    TkLBrace => { return 53; }
    TkRBrace => { return 54; }
    TkLBracket => { return 55; }
    TkRBracket => { return 56; }
    TkAt => { return 57; }
    TkArrow => { return 58; }
    TkFatArrow => { return 59; }
    TkQuestion => { return 60; }
    TkPlus => { return 61; }
    TkMinus => { return 62; }
    TkStar => { return 63; }
    TkSlash => { return 64; }
    TkPercent => { return 65; }
    TkCaret => { return 66; }
    TkTilde => { return 67; }
    TkBang => { return 68; }
    TkAmp => { return 69; }
    TkPipe => { return 70; }
    TkAmpersand => { return 71; }
    TkEq => { return 72; }
    TkEqEq => { return 73; }
    TkNeq => { return 74; }
    TkLt => { return 75; }
    TkGt => { return 76; }
    TkLe => { return 77; }
    TkGe => { return 78; }
    TkAndAnd => { return 79; }
    TkOrOr => { return 80; }
    TkPlusEq => { return 81; }
    TkMinusEq => { return 82; }
    TkStarEq => { return 83; }
    TkSlashEq => { return 84; }
    TkPercentEq => { return 85; }
    TkDotDot => { return 86; }
    TkDotDotEq => { return 87; }
    TkUnderscore => { return 88; }
    TkHash => { return 89; }
    TkEof => { return 90; }
    TkError(_) => { return 91; }
  }
}

fn pc_kind_is(k: TokenKind, want: TokenKind) -> Bool {
  return pc_kind_code(k) == pc_kind_code(want);
}

fn pc_skip(p: &mut Parser, kind: TokenKind) -> Bool {
  if pc_kind_is(selfhost_parser_state.p_peek_kind(p), kind) {
    selfhost_parser_state.p_advance(p);
    return true;
  }
  return false;
}

fn pc_peek_is(p: &Parser, kind: TokenKind) -> Bool {
  return pc_kind_is(selfhost_parser_state.p_peek_kind(p), kind);
}

fn pc_is_eof(p: &Parser) -> Bool {
  return pc_kind_is(selfhost_parser_state.p_peek_kind(p), TkEof);
}

fn pc_expect_kind(p: &mut Parser, kind: TokenKind, label: Str) -> Token {
  if pc_kind_is(selfhost_parser_state.p_peek_kind(p), kind) {
    return selfhost_parser_state.p_advance(p);
  }
  let found = selfhost_parser_state.p_peek(p).lexeme;
  let _ = selfhost_parser_state.p_err(p, "expected " + label + ", found " + found);
  return selfhost_parser_state.p_peek(p);
}

// ============================================================================
// Program / module header (lib.rs parse_program .. build_file_module_result)
// ============================================================================

pub fn pc_parse_program(p: &mut Parser) -> Int {
  var items = Vec[Int].new();
  let start = selfhost_parser_state.p_peek_span(p);
  let path = pc_parse_file_module_header(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  while !pc_is_eof(p) {
    let decls = pc_parse_top_decl_with_pending(p);
    if selfhost_parser_state.p_failed(p) {
      // Phase 5c recovery: clear the parse latch, record the error (which
      // re-latches only when MAX_PARSE_ERRORS is reached), skip to a sync
      // point, and continue. Mirrors Rust's recoverable_error + error-list
      // check: the recorded Err is the only abort signal.
      let msg = p.err_msg;
      let span = p.err_span;
      p.failed = 0;
      p.err_msg = "";
      let _ = selfhost_parser_state.p_recoverable_error(p, msg, span);
      if selfhost_parser_state.p_failed(p) { return -1; }
      selfhost_parser_state.p_recover_stmt(p);
    } else {
      pc_extend(items, decls);
    }
  }
  if path.len() > 0 {
    let wrapped = pc_build_file_module_result(p, path, items, start);
    var one = Vec[Int].new();
    one.push(wrapped);
    return selfhost_parser_state.p_n(p, NodeKind.NkProgram(one), start);
  }
  return selfhost_parser_state.p_n(p, NodeKind.NkProgram(items), start);
}

/// Returns the brace-less file module path, or an empty Vec for "no header"
/// (Rust `Option<Vec<Ident>>::None`). On a hard failure p_failed is set.
fn pc_parse_file_module_header(p: &mut Parser) -> Vec[Int] {
  let saved = p.pos;
  let is_pub = pc_skip(p, TkPub);
  if !pc_peek_is(p, TkModule) {
    p.pos = saved;
    return Vec[Int].new();
  }
  selfhost_parser_state.p_advance(p);
  let first = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
  if pc_peek_is(p, TkLBrace) {
    p.pos = saved;
    return Vec[Int].new();
  }
  if is_pub {
    let _ = selfhost_parser_state.p_err(p, "'pub' not valid on module declarations");
    return Vec[Int].new();
  }
  var path = Vec[Int].new();
  path.push(first);
  while pc_skip(p, TkDot) {
    let seg = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
    path.push(seg);
  }
  // FE-17: `module a.b.c;` trailing semicolon tolerated.
  let _ = pc_skip(p, TkSemicolon);
  return path;
}

/// Wrap `items` bottom-up in one NkModule per path segment (reversed), and
/// return the outermost node index. Rust `build_file_module_result`.
fn pc_build_file_module_result(p: &mut Parser, path: Vec[Int], items: Vec[Int], span: Span) -> Int {
  var current = items;
  var i = path.len();
  while i > 0 {
    i = i - 1;
    let segment = path[i];
    let m = selfhost_parser_state.p_n(p, NodeKind.NkModule(segment, Vec[Int].new(), current, 0, 0, ""), span);
    var nxt = Vec[Int].new();
    nxt.push(m);
    current = nxt;
  }
  return current[0];
}

// ============================================================================
// Top-level dispatch
// ============================================================================

fn pc_parse_top_decl_with_pending(p: &mut Parser) -> Vec[Int] {
  let item = pc_parse_top_decl(p);
  if selfhost_parser_state.p_failed(p) { return Vec[Int].new(); }
  var decls = Vec[Int].new();
  var i = 0;
  while i < p.pending_externs.len() {
    decls.push(p.pending_externs[i]);
    i = i + 1;
  }
  p.pending_externs = Vec[Int].new();
  decls.push(item);
  return decls;
}

fn pc_parse_top_decl(p: &mut Parser) -> Int {
  selfhost_parser_state.p_reset_depth(p);
  var is_pub = 0;
  if pc_skip(p, TkPub) { is_pub = 1; }
  let k = selfhost_parser_state.p_peek_kind(p);
  if pc_kind_is(k, TkModule) { return pc_parse_module(p, is_pub); }
  if pc_kind_is(k, TkUse) { return pc_parse_use_decl(p); }
  if pc_kind_is(k, TkType) { return pc_parse_type_decl(p, is_pub); }
  if pc_kind_is(k, TkEnum) { return pc_parse_enum_decl(p, is_pub); }
  if pc_kind_is(k, TkInterface) { return pc_parse_interface_decl(p, is_pub); }
  if pc_kind_is(k, TkImpl) { return pc_parse_impl_decl(p); }
  if pc_kind_is(k, TkFn) { return pc_parse_fn_decl(p, is_pub, -1); }
  if pc_kind_is(k, TkConst) { return pc_parse_const_decl(p, is_pub); }
  if pc_kind_is(k, TkVar) { return pc_parse_module_var(p); }
  if pc_kind_is(k, TkExtern) { return pc_parse_extern_block(p); }
  if pc_kind_is(k, TkSpawn) {
    // Module-level `spawn { ... }` statement (M21).
    if is_pub != 0 {
      selfhost_parser_state.p_advance(p);
      return selfhost_parser_state.p_err(p, "'pub' not valid on spawn declarations");
    }
    return pc_parse_spawn_top_decl(p);
  }
  var async_kw = false;
  match k {
    TkIdent(s) => {
      if s == "async" && pc_kind_is(selfhost_parser_state.p_peek_ahead_kind(p, 1), TkFn) {
        async_kw = true;
      }
    }
    _ => {}
  }
  if async_kw { return pc_parse_fn_decl(p, is_pub, 1); }
  // D2.1: `unsafe` applies strictly to block expressions.
  if pc_kind_is(k, TkUnsafe) {
    selfhost_parser_state.p_advance(p);
    return selfhost_parser_state.p_err(
      p,
      "`unsafe` applies only to block expressions `unsafe { ... }`; it cannot prefix declarations (fn/module/struct/impl)"
    );
  }
  // D2.1 (Phase 6): fn-level attributes precede the `fn` keyword.
  if pc_kind_is(k, TkHash) { return pc_parse_fn_decl(p, is_pub, -1); }
  let lex = selfhost_parser_state.p_peek(p).lexeme;
  return selfhost_parser_state.p_err(p, "expected declaration, found '" + lex + "'");
}

// ============================================================================
// module / use
// ============================================================================

fn pc_parse_module(p: &mut Parser, is_pub: Int) -> Int {
  let start = pc_tok_span(selfhost_parser_state.p_advance(p));
  let first = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var path = Vec[Int].new();
  path.push(first);
  while pc_skip(p, TkDot) {
    let seg = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    path.push(seg);
  }
  if is_pub != 0 {
    return selfhost_parser_state.p_err(p, "'pub' not valid on module declarations");
  }
  if pc_peek_is(p, TkLBrace) {
    // Block-form module: `module a[.b.c] { ... }`
    selfhost_parser_state.p_advance(p);
    var items = Vec[Int].new();
    while !(pc_peek_is(p, TkRBrace) || pc_is_eof(p)) {
      let decls = pc_parse_top_decl_with_pending(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      pc_extend(items, decls);
    }
    let _ = pc_expect_kind(p, TkRBrace, "'}'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    // FE-17: one trailing semicolon tolerated after the closing brace.
    let _ = pc_skip(p, TkSemicolon);
    return pc_build_file_module_result(p, path, items, start);
  }
  // Brace-less file-level module: wraps the rest of the file.
  let _ = pc_skip(p, TkSemicolon);
  var items = Vec[Int].new();
  while !pc_is_eof(p) {
    let decls = pc_parse_top_decl_with_pending(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    pc_extend(items, decls);
  }
  return pc_build_file_module_result(p, path, items, start);
}

fn pc_parse_use_decl(p: &mut Parser) -> Int {
  selfhost_parser_state.p_advance(p);
  let start = selfhost_parser_state.p_peek_span(p);
  let first = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var path = Vec[Int].new();
  path.push(first);
  while pc_skip(p, TkDot) {
    if pc_peek_is(p, TkStar) {
      selfhost_parser_state.p_advance(p);
      let _ = pc_expect_kind(p, TkSemicolon, "';'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      return selfhost_parser_state.p_n(p, NodeKind.NkUse(path, 1, -1), start);
    }
    let seg = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    path.push(seg);
  }
  var alias = -1;
  if pc_skip(p, TkAs) {
    let a = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    alias = a;
  }
  let _ = pc_expect_kind(p, TkSemicolon, "';'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkUse(path, 0, alias), start);
}

// ============================================================================
// type / enum declarations
// ============================================================================

fn pc_parse_type_decl(p: &mut Parser, is_pub: Int) -> Int {
  let start = pc_tok_span(selfhost_parser_state.p_advance(p));
  let name = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let generics = selfhost_parser_expr.pe_parse_optional_generic_params(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = pc_expect_kind(p, TkEq, "'='");
  if selfhost_parser_state.p_failed(p) { return -1; }

  if pc_peek_is(p, TkEnum) {
    selfhost_parser_state.p_advance(p);
    let _ = pc_expect_kind(p, TkLBrace, "'{'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    var variants = Vec[Int].new();
    var vgo = true;
    while vgo {
      let vname = selfhost_parser_expr.pe_parse_variant_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      var vfields = Vec[Int].new();
      if pc_skip(p, TkLParen) {
        var idx = 0;
        var fgo = true;
        while fgo {
          var named = false;
          match selfhost_parser_state.p_peek_kind(p) {
            TkIdent(_) => {
              if pc_kind_is(selfhost_parser_state.p_peek_ahead_kind(p, 1), TkColon) { named = true; }
            }
            _ => {}
          }
          var fname = -1;
          if named {
            let n = selfhost_parser_expr.pe_parse_ident(p);
            if selfhost_parser_state.p_failed(p) { return -1; }
            let _ = pc_expect_kind(p, TkColon, "':'");
            if selfhost_parser_state.p_failed(p) { return -1; }
            fname = n;
          } else {
            fname = selfhost_parser_state.p_n_ident(p, "_" + pc_dec_str(idx), start);
          }
          let fty = selfhost_parser_expr.pe_parse_type(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          vfields.push(selfhost_parser_state.p_n(p, NodeKind.NkField(fname, fty), start));
          idx = idx + 1;
          if !pc_skip(p, TkComma) { fgo = false; }
        }
        let _ = pc_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
      }
      variants.push(selfhost_parser_state.p_n(p, NodeKind.NkVariant(vname, vfields), start));
      if pc_skip(p, TkComma) {
        if pc_peek_is(p, TkRBrace) || pc_is_eof(p) {
          vgo = false;
        }
      } elif pc_peek_is(p, TkRBrace) || pc_is_eof(p) {
        vgo = false;
      }
    }
    let _ = pc_expect_kind(p, TkRBrace, "'}'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    var derives = Vec[Int].new();
    if pc_skip(p, TkDerive) {
      let d = selfhost_parser_expr.pe_parse_derive_list(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      derives = d;
    }
    let _ = pc_skip(p, TkSemicolon);
    return selfhost_parser_state.p_n(p, NodeKind.NkEnumDecl(is_pub, name, generics, variants, derives), start);
  }

  if pc_peek_is(p, TkLParen) {
    // 8B/M9: Tuple struct -- `type Foo = (Int, Str) [derive[...]]`.
    let tuple_types = selfhost_parser_expr.pe_parse_tuple_type_args(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    var derives2 = Vec[Int].new();
    if pc_skip(p, TkDerive) {
      let d2 = selfhost_parser_expr.pe_parse_derive_list(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      derives2 = d2;
    }
    let _ = pc_skip(p, TkSemicolon);
    var flds = Vec[Int].new();
    var ti = 0;
    while ti < tuple_types.len() {
      let fname2 = selfhost_parser_state.p_n_ident(p, "_" + pc_dec_str(ti), start);
      flds.push(selfhost_parser_state.p_n(p, NodeKind.NkField(fname2, tuple_types[ti]), start));
      ti = ti + 1;
    }
    return selfhost_parser_state.p_n(p, NodeKind.NkTypeDecl(is_pub, name, generics, flds, Vec[Int].new(), Vec[Int].new(), derives2, -1), start);
  }

  if !pc_peek_is(p, TkLBrace) {
    let alias_type = selfhost_parser_expr.pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = pc_expect_kind(p, TkSemicolon, "';'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    return selfhost_parser_state.p_n(p, NodeKind.NkTypeDecl(is_pub, name, generics, Vec[Int].new(), Vec[Int].new(), Vec[Int].new(), Vec[Int].new(), alias_type), start);
  }

  let _ = pc_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var fields = Vec[Int].new();
  var derived_fields = Vec[Int].new();
  var invariants = Vec[Int].new();
  while !(pc_peek_is(p, TkRBrace) || pc_is_eof(p)) {
    // 5c-R: `invariant` is a contextual Ident, not a reserved token.
    var is_inv = false;
    match selfhost_parser_state.p_peek_kind(p) {
      TkIdent(s) => {
        if s == "invariant" { is_inv = true; }
      }
      _ => {}
    }
    if is_inv {
      selfhost_parser_state.p_advance(p);
      let _ = pc_expect_kind(p, TkColon, "':'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let expr = selfhost_parser_expr.pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = pc_expect_kind(p, TkSemicolon, "';'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      invariants.push(expr);
    } else {
      let field_name = selfhost_parser_expr.pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let kf = selfhost_parser_state.p_peek_kind(p);
      if pc_kind_is(kf, TkComma) || pc_kind_is(kf, TkRBrace) || pc_kind_is(kf, TkLParen) {
        // Legacy tuple-ish field list: skip balanced tokens, then return.
        var depth = 0;
        var sgo = true;
        while sgo {
          let k2 = selfhost_parser_state.p_peek_kind(p);
          if pc_kind_is(k2, TkLParen) {
            selfhost_parser_state.p_advance(p);
            depth = depth + 1;
          } elif pc_kind_is(k2, TkRParen) {
            selfhost_parser_state.p_advance(p);
            depth = depth - 1;
            if depth < 0 { sgo = false; }
          } elif pc_kind_is(k2, TkLBrace) {
            selfhost_parser_state.p_advance(p);
            depth = depth + 1;
          } elif pc_kind_is(k2, TkRBrace) {
            if depth == 0 {
              sgo = false;
            } else {
              selfhost_parser_state.p_advance(p);
              depth = depth - 1;
            }
          } elif pc_is_eof(p) {
            sgo = false;
          } else {
            selfhost_parser_state.p_advance(p);
          }
        }
        selfhost_parser_state.p_advance(p);
        return selfhost_parser_state.p_n(p, NodeKind.NkTypeDecl(0, name, generics, fields, derived_fields, invariants, Vec[Int].new(), -1), start);
      }
      let _ = pc_expect_kind(p, TkColon, "':'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let ty = selfhost_parser_expr.pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      var is_derived = false;
      match selfhost_parser_state.p_peek_kind(p) {
        TkIdent(s) => {
          if s == "derived" { is_derived = true; }
        }
        _ => {}
      }
      if is_derived {
        selfhost_parser_state.p_advance(p);
        let _ = pc_expect_kind(p, TkLParen, "'('");
        if selfhost_parser_state.p_failed(p) { return -1; }
        let expr2 = selfhost_parser_expr.pe_parse_expr(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        let _ = pc_expect_kind(p, TkRParen, "')'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        let _ = pc_expect_kind(p, TkSemicolon, "';'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        derived_fields.push(selfhost_parser_state.p_n(p, NodeKind.NkDerivedField(field_name, ty, expr2), selfhost_parser_state.p_span_of(p, field_name)));
      } else {
        let _ = pc_expect_kind(p, TkSemicolon, "';'");
        if selfhost_parser_state.p_failed(p) { return -1; }
        fields.push(selfhost_parser_state.p_n(p, NodeKind.NkField(field_name, ty), start));
      }
    }
  }
  let _ = pc_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var derives3 = Vec[Int].new();
  if pc_skip(p, TkDerive) {
    let d3 = selfhost_parser_expr.pe_parse_derive_list(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    derives3 = d3;
  }
  return selfhost_parser_state.p_n(p, NodeKind.NkTypeDecl(is_pub, name, generics, fields, derived_fields, invariants, derives3, -1), start);
}

fn pc_parse_enum_decl(p: &mut Parser, is_pub: Int) -> Int {
  let start = pc_tok_span(selfhost_parser_state.p_advance(p));
  let name = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let generics = selfhost_parser_expr.pe_parse_optional_generic_params(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = pc_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var variants = Vec[Int].new();
  var vgo = true;
  while vgo {
    let vname = selfhost_parser_expr.pe_parse_variant_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    var vfields = Vec[Int].new();
    if pc_skip(p, TkLParen) {
      var idx = 0;
      var fgo = true;
      while fgo {
        var named = false;
        match selfhost_parser_state.p_peek_kind(p) {
          TkIdent(_) => {
            if pc_kind_is(selfhost_parser_state.p_peek_ahead_kind(p, 1), TkColon) { named = true; }
          }
          _ => {}
        }
        var fname = -1;
        if named {
          let n = selfhost_parser_expr.pe_parse_ident(p);
          if selfhost_parser_state.p_failed(p) { return -1; }
          let _ = pc_expect_kind(p, TkColon, "':'");
          if selfhost_parser_state.p_failed(p) { return -1; }
          fname = n;
        } else {
          fname = selfhost_parser_state.p_n_ident(p, "_" + pc_dec_str(idx), start);
        }
        let fty = selfhost_parser_expr.pe_parse_type(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        vfields.push(selfhost_parser_state.p_n(p, NodeKind.NkField(fname, fty), start));
        idx = idx + 1;
        if !pc_skip(p, TkComma) { fgo = false; }
      }
      let _ = pc_expect_kind(p, TkRParen, "')'");
      if selfhost_parser_state.p_failed(p) { return -1; }
    }
    variants.push(selfhost_parser_state.p_n(p, NodeKind.NkVariant(vname, vfields), start));
    if pc_skip(p, TkComma) {
      if pc_peek_is(p, TkRBrace) || pc_is_eof(p) {
        vgo = false;
      }
    } elif pc_peek_is(p, TkRBrace) || pc_is_eof(p) {
      vgo = false;
    }
  }
  let _ = pc_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var derives = Vec[Int].new();
  if pc_skip(p, TkDerive) {
    let d = selfhost_parser_expr.pe_parse_derive_list(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    derives = d;
  }
  return selfhost_parser_state.p_n(p, NodeKind.NkEnumDecl(is_pub, name, generics, variants, derives), start);
}

// ============================================================================
// interface / impl declarations
// ============================================================================

fn pc_parse_interface_decl(p: &mut Parser, is_pub: Int) -> Int {
  let start = pc_tok_span(selfhost_parser_state.p_advance(p));
  let name = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let generics = selfhost_parser_expr.pe_parse_optional_generic_params(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  // v0.56: interface inheritance (e.g., `interface DerefMut: Deref`).
  var parent = -1;
  if pc_kind_is(selfhost_parser_state.p_peek_kind(p), TkColon) {
    selfhost_parser_state.p_advance(p);
    let par = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    parent = par;
  }
  let _ = pc_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var members = Vec[Int].new();
  while !(pc_peek_is(p, TkRBrace) || pc_is_eof(p)) {
    if pc_peek_is(p, TkFn) {
      let f = pc_parse_fn_decl(p, 0, -1);
      if selfhost_parser_state.p_failed(p) { return -1; }
      members.push(f);
    } elif pc_kind_is(selfhost_parser_state.p_peek_kind(p), TkType) {
      // Associated type declaration `type Name;` (sentinel `_assoc_type`).
      selfhost_parser_state.p_advance(p);
      let at_name = selfhost_parser_expr.pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = pc_expect_kind(p, TkSemicolon, "';'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let sent = selfhost_parser_state.p_n_ident(p, "_assoc_type", start);
      let sent_ty = selfhost_parser_state.p_n(p, NodeKind.NkTyNamed(sent, Vec[Int].new()), start);
      members.push(selfhost_parser_state.p_n(p, NodeKind.NkField(at_name, sent_ty), start));
    } else {
      let fname = selfhost_parser_expr.pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = pc_expect_kind(p, TkColon, "':'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let fty = selfhost_parser_expr.pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = pc_expect_kind(p, TkSemicolon, "';'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      members.push(selfhost_parser_state.p_n(p, NodeKind.NkField(fname, fty), start));
    }
  }
  let _ = pc_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkInterface(is_pub, name, generics, parent, members), start);
}

fn pc_parse_impl_decl(p: &mut Parser) -> Int {
  let start = pc_tok_span(selfhost_parser_state.p_advance(p)); // consume `impl`
  let trait_name = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var trait_args = Vec[Int].new();
  // D1: `impl Num[Int]` -- generic args on the trait.
  if pc_kind_is(selfhost_parser_state.p_peek_kind(p), TkLBracket) {
    selfhost_parser_state.p_advance(p);
    while !(pc_peek_is(p, TkRBracket) || pc_is_eof(p)) {
      let t = selfhost_parser_expr.pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      trait_args.push(t);
      if !pc_skip(p, TkComma) { break; }
    }
    let _ = pc_expect_kind(p, TkRBracket, "']'");
    if selfhost_parser_state.p_failed(p) { return -1; }
  }
  var type_name = selfhost_parser_state.p_n_ident(p, "_", start);
  if pc_skip(p, TkFor) {
    let tn = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    type_name = tn;
  }
  let _ = pc_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var members = Vec[Int].new();
  while !(pc_peek_is(p, TkRBrace) || pc_is_eof(p)) {
    if pc_peek_is(p, TkFn) {
      let f = pc_parse_fn_decl(p, 0, -1);
      if selfhost_parser_state.p_failed(p) { return -1; }
      members.push(f);
    } elif pc_peek_is(p, TkConst) {
      let c = pc_parse_const_decl(p, 0);
      if selfhost_parser_state.p_failed(p) { return -1; }
      members.push(c);
    } else {
      return selfhost_parser_state.p_err(p, "expected 'fn' in impl block");
    }
  }
  let _ = pc_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkImpl(trait_name, trait_args, type_name, members), start);
}

// ============================================================================
// attributes
// ============================================================================

pub fn pc_parse_attributes(p: &mut Parser) -> Vec[Int] {
  var attrs = Vec[Int].new();
  while pc_skip(p, TkHash) {
    let _ = pc_expect_kind(p, TkLBracket, "'['");
    if selfhost_parser_state.p_failed(p) { return attrs; }
    let name = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return attrs; }
    var args = Vec[Int].new();
    if pc_skip(p, TkLParen) {
      var ago = true;
      while ago {
        let key = selfhost_parser_expr.pe_parse_ident(p);
        if selfhost_parser_state.p_failed(p) { return attrs; }
        if pc_skip(p, TkColon) {
          let val = selfhost_parser_expr.pe_parse_expr(p);
          if selfhost_parser_state.p_failed(p) { return attrs; }
          let key_str = pc_ident_name(p, key);
          let val_str = pc_attr_value_str(p, val);
          args.push(selfhost_parser_state.p_n(p, NodeKind.NkAttrArg(key_str, val_str), selfhost_parser_state.p_peek_span(p)));
        }
        if !pc_skip(p, TkComma) { ago = false; }
      }
      let _ = pc_expect_kind(p, TkRParen, "')'");
      if selfhost_parser_state.p_failed(p) { return attrs; }
    }
    let _ = pc_expect_kind(p, TkRBracket, "']'");
    if selfhost_parser_state.p_failed(p) { return attrs; }
    // Rust quirk: the attribute span is the peek AFTER `]`.
    let aspan = selfhost_parser_state.p_peek_span(p);
    attrs.push(selfhost_parser_state.p_n(p, NodeKind.NkAttr(name, args), aspan));
  }
  return attrs;
}

// ============================================================================
// fn declaration (+ where clauses, contracts, receiver generics)
// ============================================================================

fn pc_parse_fn_decl(p: &mut Parser, is_pub_in: Int, is_async_opt: Int) -> Int {
  let attrs = pc_parse_attributes(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  // Attributes may be written BEFORE or AFTER `pub` (m166).
  var is_pub = is_pub_in;
  if is_pub == 0 {
    if pc_skip(p, TkPub) { is_pub = 1; }
  }
  // `async` is contextual: only when immediately followed by `fn`.
  var has_async = false;
  match selfhost_parser_state.p_peek_kind(p) {
    TkIdent(s) => {
      if s == "async" && pc_kind_is(selfhost_parser_state.p_peek_ahead_kind(p, 1), TkFn) {
        has_async = true;
        selfhost_parser_state.p_advance(p);
      }
    }
    _ => {}
  }
  var async_flag = 0;
  if is_async_opt != -1 { async_flag = is_async_opt; }
  if has_async { async_flag = 1; }
  let start = selfhost_parser_state.p_peek_span(p);
  if !pc_peek_is(p, TkFn) {
    return selfhost_parser_state.p_err(p, "expected 'fn'");
  }
  selfhost_parser_state.p_advance(p);
  let first = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  // BUG 38b: capture single-uppercase receiver generic names when the
  // `[...]` is immediately followed by `.` (method receiver form).
  var receiver_generics = Vec[Str].new();
  if pc_peek_is(p, TkLBracket) {
    let saved = p.pos;
    selfhost_parser_state.p_advance(p);
    var depth = 1;
    while depth > 0 && !pc_is_eof(p) {
      let kr = selfhost_parser_state.p_peek_kind(p);
      if pc_kind_is(kr, TkLBracket) {
        depth = depth + 1;
        selfhost_parser_state.p_advance(p);
      } elif pc_kind_is(kr, TkRBracket) {
        depth = depth - 1;
        selfhost_parser_state.p_advance(p);
      } else {
        match kr {
          TkIdent(s) => {
            if depth == 1 && s.len() == 1 {
              let c0 = string.byte_at(s, 0) as Int;
              if c0 >= 65 && c0 <= 90 {
                receiver_generics.push(s);
              }
            }
          }
          _ => {}
        }
        selfhost_parser_state.p_advance(p);
      }
    }
    if !pc_peek_is(p, TkDot) {
      // Not a method receiver -- leave `[...]` for generic params.
      p.pos = saved;
      receiver_generics = Vec[Str].new();
    }
  }
  var recv = -1;
  var name = first;
  if pc_skip(p, TkDot) {
    recv = first;
    let n = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    name = n;
  }
  var generics = selfhost_parser_expr.pe_parse_optional_generic_params(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  // Receiver generics come FIRST (receiver-first), deduped by name.
  var ri = receiver_generics.len();
  while ri > 0 {
    ri = ri - 1;
    let rg = receiver_generics[ri];
    var found = false;
    var gi = 0;
    while gi < generics.len() {
      if pc_generic_name(p, generics[gi]) == rg { found = true; }
      gi = gi + 1;
    }
    if !found {
      let gname = selfhost_parser_state.p_n_ident(p, rg, start);
      let gnode = selfhost_parser_state.p_n(p, NodeKind.NkGeneric(gname, Vec[Int].new(), 0, -1), start);
      generics = pc_push_front(generics, gnode);
    }
  }
  let _ = pc_expect_kind(p, TkLParen, "'('");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var params = Vec[Int].new();
  if pc_peek_is(p, TkRParen) {
    selfhost_parser_state.p_advance(p);
  } else {
    let plist = selfhost_parser_expr.pe_parse_param_list(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = pc_expect_kind(p, TkRParen, "')'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    params = plist;
  }
  var ret = -1;
  if pc_skip(p, TkArrow) {
    let t = selfhost_parser_expr.pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    ret = t;
  }
  // 8B/M9: optional `where` clause merges bounds into generic params.
  if pc_kw_is(p, "where") {
    selfhost_parser_state.p_advance(p);
    var wgo = true;
    while wgo {
      let cname = selfhost_parser_expr.pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = pc_expect_kind(p, TkColon, "':' in where clause");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let bname = selfhost_parser_expr.pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let cname_str = pc_ident_name(p, cname);
      var gi2 = 0;
      while gi2 < generics.len() {
        if pc_generic_name(p, generics[gi2]) == cname_str {
          // First bound only when the param has no bounds yet.
          if pc_generic_bounds_empty(p, generics[gi2]) {
            pc_generic_push_bound(p, generics[gi2], bname);
          }
          gi2 = generics.len();
        } else {
          gi2 = gi2 + 1;
        }
      }
      if pc_kind_is(selfhost_parser_state.p_peek_kind(p), TkPlus) {
        selfhost_parser_state.p_advance(p);
        let extra = selfhost_parser_expr.pe_parse_ident(p);
        if selfhost_parser_state.p_failed(p) { return -1; }
        var gi3 = 0;
        while gi3 < generics.len() {
          if pc_generic_name(p, generics[gi3]) == cname_str {
            pc_generic_push_bound(p, generics[gi3], extra);
            gi3 = generics.len();
          } else {
            gi3 = gi3 + 1;
          }
        }
      }
      let pk = selfhost_parser_state.p_peek_kind(p);
      if !pc_is_ident(p) || pc_kind_is(pk, TkLBrace) || pc_kind_is(pk, TkSemicolon) { wgo = false; }
    }
  }
  var contracts = Vec[Int].new();
  while pc_is_contract_kw(p) {
    let is_req = pc_kw_is(p, "requires");
    selfhost_parser_state.p_advance(p);
    let _ = pc_expect_kind(p, TkColon, "':'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    // BUG 56: a trailing `{` after a contract expr starts the body, so the
    // expr is parsed with struct-literal restriction (parse_cond pattern).
    let saved_restrict = p.restrict_struct;
    p.restrict_struct = 1;
    let expr = selfhost_parser_expr.pe_parse_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    p.restrict_struct = saved_restrict;
    if is_req {
      contracts.push(selfhost_parser_state.p_n(p, NodeKind.NkRequires(expr), start));
    } else {
      contracts.push(selfhost_parser_state.p_n(p, NodeKind.NkEnsures(expr), start));
    }
    // Comma-separated shorthand: `requires: a>0, b>0`.
    while pc_skip(p, TkComma) {
      let sr = p.restrict_struct;
      p.restrict_struct = 1;
      let extra = selfhost_parser_expr.pe_parse_expr(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      p.restrict_struct = sr;
      if is_req {
        contracts.push(selfhost_parser_state.p_n(p, NodeKind.NkRequires(extra), start));
      } else {
        contracts.push(selfhost_parser_state.p_n(p, NodeKind.NkEnsures(extra), start));
      }
    }
    let _ = pc_skip(p, TkSemicolon);
  }
  var body = -1;
  if pc_skip(p, TkSemicolon) {
    body = -1;
  } elif pc_peek_is(p, TkLBrace) {
    let b = selfhost_parser_expr.pe_parse_block(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    body = b;
  }
  return selfhost_parser_state.p_n(p, NodeKind.NkFn(is_pub, async_flag, recv, name, generics, params, ret, contracts, body, attrs), start);
}

// ============================================================================
// const / module var / extern block / spawn top decl
// ============================================================================

fn pc_parse_const_decl(p: &mut Parser, is_pub: Int) -> Int {
  selfhost_parser_state.p_advance(p);
  let start = selfhost_parser_state.p_peek_span(p);
  let name = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = pc_expect_kind(p, TkColon, "':'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let ty = selfhost_parser_expr.pe_parse_type(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  let _ = pc_expect_kind(p, TkEq, "'='");
  if selfhost_parser_state.p_failed(p) { return -1; }
  let value = selfhost_parser_expr.pe_parse_expr(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  // Semicolons are optional at top-level (file-level module form).
  let _ = pc_skip(p, TkSemicolon);
  return selfhost_parser_state.p_n(p, NodeKind.NkConst(is_pub, 0, name, ty, value), start);
}

fn pc_parse_module_var(p: &mut Parser) -> Int {
  selfhost_parser_state.p_advance(p); // consume 'var'
  let span = selfhost_parser_state.p_peek_span(p);
  let name = selfhost_parser_expr.pe_parse_ident(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  var ty = -1;
  if pc_skip(p, TkColon) {
    let t = selfhost_parser_expr.pe_parse_type(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    ty = t;
  } else {
    let id = selfhost_parser_state.p_n_ident(p, "_", span);
    ty = selfhost_parser_state.p_n(p, NodeKind.NkTyNamed(id, Vec[Int].new()), span);
  }
  var value = -1;
  if pc_skip(p, TkEq) {
    let v = selfhost_parser_expr.pe_parse_expr(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    value = v;
  } else {
    value = selfhost_parser_state.p_n(p, NodeKind.NkLitInt(0 as UInt), span);
  }
  let _ = pc_skip(p, TkSemicolon);
  return selfhost_parser_state.p_n(p, NodeKind.NkConst(0, 1, name, ty, value), span);
}

fn pc_parse_extern_block(p: &mut Parser) -> Int {
  selfhost_parser_state.p_advance(p); // consume 'extern'
  let span_start = selfhost_parser_state.p_peek_span(p);
  // Rust trims quotes from the lexeme and skips `Str(linkage)`; both paths
  // consume exactly one token, so one advance mirrors it.
  let linkage = pc_trim_quotes(selfhost_parser_state.p_peek(p).lexeme);
  selfhost_parser_state.p_advance(p);
  let _ = pc_expect_kind(p, TkLBrace, "'{'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  var fns = Vec[Int].new();
  while !(pc_peek_is(p, TkRBrace) || pc_is_eof(p)) {
    let _ = pc_expect_kind(p, TkFn, "'fn'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    let fn_name = selfhost_parser_expr.pe_parse_ident(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let generics = selfhost_parser_expr.pe_parse_optional_generic_params(p);
    if selfhost_parser_state.p_failed(p) { return -1; }
    let _ = pc_expect_kind(p, TkLParen, "'('");
    if selfhost_parser_state.p_failed(p) { return -1; }
    // Params handled manually for variadic '...'.
    var params = Vec[Int].new();
    while !(pc_peek_is(p, TkRParen) || pc_is_eof(p)) {
      if pc_peek_is(p, TkDot) {
        selfhost_parser_state.p_advance(p);
        selfhost_parser_state.p_advance(p);
        selfhost_parser_state.p_advance(p);
        break;
      }
      let pname = selfhost_parser_expr.pe_parse_ident(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let _ = pc_expect_kind(p, TkColon, "':'");
      if selfhost_parser_state.p_failed(p) { return -1; }
      let pty = selfhost_parser_expr.pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      let pspan = selfhost_parser_state.p_span_of(p, pname);
      params.push(selfhost_parser_state.p_n(p, NodeKind.NkParam(pname, pty, 0, 0), pspan));
      if !pc_peek_is(p, TkRParen) {
        let _ = pc_expect_kind(p, TkComma, "','");
        if selfhost_parser_state.p_failed(p) { return -1; }
      }
    }
    let _ = pc_expect_kind(p, TkRParen, "')'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    var fret = -1;
    if pc_skip(p, TkArrow) {
      let rt = selfhost_parser_expr.pe_parse_type(p);
      if selfhost_parser_state.p_failed(p) { return -1; }
      fret = rt;
    }
    let _ = pc_expect_kind(p, TkSemicolon, "';'");
    if selfhost_parser_state.p_failed(p) { return -1; }
    fns.push(selfhost_parser_state.p_n(p, NodeKind.NkFn(0, 0, -1, fn_name, generics, params, fret, Vec[Int].new(), -1, Vec[Int].new()), span_start));
  }
  let _ = pc_expect_kind(p, TkRBrace, "'}'");
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkExtern(linkage, fns), span_start);
}

fn pc_parse_spawn_top_decl(p: &mut Parser) -> Int {
  let span = pc_tok_span(selfhost_parser_state.p_advance(p)); // consume 'spawn'
  var is_move = 0;
  if pc_kind_is(selfhost_parser_state.p_peek_kind(p), TkMove) {
    selfhost_parser_state.p_advance(p);
    is_move = 1;
  }
  let body = selfhost_parser_expr.pe_parse_block(p);
  if selfhost_parser_state.p_failed(p) { return -1; }
  return selfhost_parser_state.p_n(p, NodeKind.NkSpawnTop(is_move, body), span);
}
