// XIOM -- Selfhost parser state and token helpers (Phase 2)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Parser state and shared helpers for the Phase 2 port of
// crates/xiom-parser/src/lib.rs. Internal helpers are FREE functions taking
// `&mut Parser` / `&Parser` (the Phase 1 receiver-mutation finding: nested
// `self` method calls do not propagate field mutation; COMPILER_BUGS
// 2026-10-02 (a)). The `p_` prefix mirrors the lexer's `lx_`.
//
// Error convention (port of Rust's `Result<T, ParseError>` + `?`):
//   * `p_err(p, msg)` latches `failed = 1`, stores `err_msg`/`err_span`, and
//     returns -1. Functions returning an index `return p_err(...)`.
//   * callers check `p_failed(p)` after any fallible call and unwind.
//   * `parse_program` catches a latched failure: records it via
//     `p_recoverable_error`, clears the latch, runs `p_recover_stmt`, and
//     continues (mirrors Rust's Phase 5c recovery).
//   * `expected`-set formatting is simplified to the single-token message
//     (`expected {label}, found {lexeme}`): diagnostic TEXT is not part of
//     the Phase 2 dump gate (Phase 3 owns diagnostic parity).
//
// `nodes` is the arena (selfhost_ast). Node pushes happen bottom-up exactly
// where the Rust parser constructs values, so the dump walker observes the
// same tree; the indices themselves never appear in the dump.

module selfhost_parser_state

use xiom.string;
use selfhost_ast;
use selfhost_ast.Node;
use selfhost_ast.NodeKind;
use selfhost_ast.Span;
use selfhost_lexer;
use selfhost_lexer.Token;
use selfhost_lexer.TokenKind;

pub type ParseError = {
  message: Str;
  span: Span;
}

pub type Parser = {
  toks: Vec[Token];
  pos: Int;
  /// When 1, a bare `Ident { ... }` is NOT parsed as a struct literal.
  restrict_struct: Int;
  /// Current recursion depth of the expression/type parsers (limit 128).
  depth: Int;
  /// Accumulated parse errors for recovery (Rust `errors: Vec<ParseError>`).
  errors: Vec[ParseError];
  /// `extern` blocks hoisted from function bodies (node indices).
  pending_externs: Vec[Int];
  /// LAX generic-closer mode (Rust default; env-var opt-in not ported yet).
  strict_brackets: Int;
  /// Arena: all AST nodes.
  nodes: Vec[Node];
  /// Latch: 1 after the first hard error (Rust `Err`).
  failed: Int;
  err_msg: Str;
  err_span: Span;
}

/// Rust MAX_EXPR_DEPTH.
pub const MAX_EXPR_DEPTH: Int = 128;
/// Rust MAX_PARSE_ERRORS.
pub const MAX_PARSE_ERRORS: Int = 100;

// ============================================================================
// Construction / reset
// ============================================================================

pub fn p_new(toks: Vec[Token]) -> Parser {
  return Parser{
    toks: toks,
    pos: 0,
    restrict_struct: 0,
    depth: 0,
    errors: Vec[ParseError].new(),
    pending_externs: Vec[Int].new(),
    strict_brackets: 0,
    nodes: Vec[Node].new(),
    failed: 0,
    err_msg: "",
    err_span: selfhost_ast.span_zero()
  };
}

/// Reset the per-item recursion depth (Rust `parse_top_decl`).
pub fn p_reset_depth(p: &mut Parser) {
  p.depth = 0;
}

// ============================================================================
// Token access
// ============================================================================

pub fn p_peek(p: &Parser) -> Token {
  if p.pos >= p.toks.len() { return p.toks[p.toks.len() - 1]; }
  return p.toks[p.pos];
}

pub fn p_peek_kind(p: &Parser) -> TokenKind {
  return p_peek(p).kind;
}

pub fn p_peek_span(p: &Parser) -> Span {
  let t = p_peek(p);
  return selfhost_ast.span_new(t.line, t.col, t.byte_start, t.byte_end);
}

/// Out-of-range reads return Eof (the stream always terminates with Eof, so
/// this only guards the tail; Rust returns None there).
pub fn p_peek_ahead_kind(p: &Parser, n: Int) -> TokenKind {
  let i = p.pos + n;
  if i >= p.toks.len() { return TkEof; }
  return p.toks[i].kind;
}

pub fn p_is_eof(p: &Parser) -> Bool {
  return tk_tag(p_peek_kind(p)) == TK_EOF;
}

pub fn p_advance(p: &mut Parser) -> Token {
  let t = p.toks[p.pos];
  p.pos = p.pos + 1;
  return t;
}

pub fn p_skip(p: &mut Parser, kind: TokenKind) -> Bool {
  if tk_tag(p_peek_kind(p)) == tk_tag(kind) {
    p_advance(p);
    return true;
  }
  return false;
}

pub fn p_peek_is(p: &Parser, kind: TokenKind) -> Bool {
  return tk_tag(p_peek_kind(p)) == tk_tag(kind);
}

// ============================================================================
// TokenKind tag codes
// ============================================================================
//
// `TokenKind` carries aggregate payloads (Vec[UInt8]); `==` on such an enum
// lowers to a whole-struct `icmp` and clang rejects the IR (COMPILER_BUGS
// 2026-10-02 (g)). Kind identity is therefore compared through stable Int
// tags; payloads are never part of an identity test in the parser.

pub const TK_LET: Int = 1;
pub const TK_SPAWN: Int = 15;
pub const TK_MODULE: Int = 21;
pub const TK_PUB: Int = 23;
pub const TK_TYPE: Int = 25;
pub const TK_ENUM: Int = 26;
pub const TK_INTERFACE: Int = 27;
pub const TK_IMPL: Int = 29;
pub const TK_USE: Int = 22;
pub const TK_CONST: Int = 3;
pub const TK_EXTERN: Int = 38;
pub const TK_FN: Int = 4;
pub const TK_LBRACE: Int = 53;
pub const TK_RBRACE: Int = 54;
pub const TK_LBRACKET: Int = 55;
pub const TK_RBRACKET: Int = 56;
pub const TK_SEMICOLON: Int = 48;
pub const TK_GT: Int = 76;
pub const TK_IDENT: Int = 40;
pub const TK_EOF: Int = 90;

pub fn tk_tag(k: TokenKind) -> Int {
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

// ============================================================================
// Errors
// ============================================================================

pub fn p_failed(p: &Parser) -> Bool {
  return p.failed != 0;
}

/// Latch a hard error and return -1 (Rust `self.error(msg)` + `?`).
pub fn p_err(p: &mut Parser, msg: Str) -> Int {
  p.failed = 1;
  p.err_msg = msg;
  p.err_span = p_peek_span(p);
  return -1;
}

/// Latch a hard error with an explicit span and return -1.
pub fn p_err_at(p: &mut Parser, msg: Str, span: Span) -> Int {
  p.failed = 1;
  p.err_msg = msg;
  p.err_span = span;
  return -1;
}

/// Record an error WITHOUT latching (Rust `recoverable_error`); parse_program
/// keeps going. Returns -1 once MAX_PARSE_ERRORS is reached (Rust Err).
pub fn p_recoverable_error(p: &mut Parser, msg: Str, span: Span) -> Int {
  p.errors.push(ParseError{ message: msg, span: span });
  if p.errors.len() >= MAX_PARSE_ERRORS {
    return p_err_at(p, "too many parse errors -- aborting", span);
  }
  return 0;
}

/// Panic-mode statement recovery (Rust recover_stmt).
pub fn p_recover_stmt(p: &mut Parser) {
  var depth = 0;
  while !p_is_eof(p) {
    let tag = tk_tag(p_peek_kind(p));
    if tag == TK_LBRACE {
      depth = depth + 1;
      p_advance(p);
    } elif tag == TK_RBRACE {
      if depth <= 0 {
        p_advance(p);
        return;
      }
      depth = depth - 1;
      p_advance(p);
    } elif tag == TK_SEMICOLON {
      if depth <= 0 {
        p_advance(p);
        return;
      }
      p_advance(p);
    } elif tag == TK_FN || tag == TK_TYPE || tag == TK_ENUM || tag == TK_INTERFACE || tag == TK_IMPL
      || tag == TK_MODULE || tag == TK_PUB || tag == TK_CONST || tag == TK_USE || tag == TK_EXTERN
    {
      if depth <= 0 { return; }
      p_advance(p);
    } else {
      p_advance(p);
    }
  }
}

/// Recursion budget (Rust enter_expr/exit_expr).
pub fn p_enter_expr(p: &mut Parser) -> Int {
  p.depth = p.depth + 1;
  if p.depth > MAX_EXPR_DEPTH {
    return p_err(p, "expression nesting too deep (max 128 levels) -- simplify the expression");
  }
  return 0;
}

pub fn p_exit_expr(p: &mut Parser) {
  if p.depth > 0 { p.depth = p.depth - 1; }
}

// ============================================================================
// Expectations
// ============================================================================

/// Consume `kind` or latch `expected {label}, found {lexeme}` (Rust
/// expect_kind). Returns the consumed token, or the offending peek on
/// failure (callers check `p_failed`).
pub fn p_expect_kind(p: &mut Parser, kind: TokenKind, label: Str) -> Token {
  if tk_tag(p_peek_kind(p)) == tk_tag(kind) {
    return p_advance(p);
  }
  let found = p_peek(p).lexeme;
  let _ = p_err(p, "expected " + label + ", found " + found);
  return p_peek(p);
}

/// Rust `skip(TokenKind::Semicolon)` shorthand.
pub fn p_skip_semi(p: &mut Parser) -> Bool {
  return p_skip(p, TkSemicolon);
}

// ============================================================================
// Generic closer (Rust close_generic_type; LAX default)
// ============================================================================

pub fn p_close_generic_type(p: &mut Parser, used_bracket: Bool) -> Int {
  if p.strict_brackets != 0 {
    if used_bracket {
      let _ = p_expect_kind(p, TkRBracket, "']'");
    } else {
      let _ = p_expect_kind(p, TkGt, "'>'");
    }
    return 0;
  }
  if used_bracket {
    if !p_skip(p, TkRBracket) {
      let _ = p_expect_kind(p, TkGt, "'>'");
    }
  } elif !p_skip(p, TkGt) {
    let _ = p_expect_kind(p, TkRBracket, "']'");
  }
  return 0;
}

// ============================================================================
// Arena
// ============================================================================

/// Push a node and return its arena index.
pub fn p_n(p: &mut Parser, kind: NodeKind, span: Span) -> Int {
  p.nodes.push(selfhost_ast.node_new(kind, span));
  return p.nodes.len() - 1;
}

pub fn p_n_ident(p: &mut Parser, name: Str, span: Span) -> Int {
  return p_n(p, NodeKind.NkIdent(name), span);
}

pub fn p_none() -> Int {
  return -1;
}

pub fn p_is_none(i: Int) -> Bool {
  return i < 0;
}

pub fn p_span_of(p: &Parser, i: Int) -> Span {
  return p.nodes[i].span;
}
