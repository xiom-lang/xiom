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
  return p_peek_kind(p) == TkEof;
}

pub fn p_advance(p: &mut Parser) -> Token {
  let t = p.toks[p.pos];
  p.pos = p.pos + 1;
  return t;
}

pub fn p_skip(p: &mut Parser, kind: TokenKind) -> Bool {
  if p_peek_kind(p) == kind {
    p_advance(p);
    return true;
  }
  return false;
}

pub fn p_peek_is(p: &Parser, kind: TokenKind) -> Bool {
  return p_peek_kind(p) == kind;
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
    let k = p_peek_kind(p);
    if k == TkLBrace {
      depth = depth + 1;
      p_advance(p);
    } elif k == TkRBrace {
      if depth <= 0 {
        p_advance(p);
        return;
      }
      depth = depth - 1;
      p_advance(p);
    } elif k == TkSemicolon {
      if depth <= 0 {
        p_advance(p);
        return;
      }
      p_advance(p);
    } elif k == TkFn || k == TkType || k == TkEnum || k == TkInterface || k == TkImpl
      || k == TkModule || k == TkPub || k == TkConst || k == TkUse || k == TkExtern
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
  if p_peek_kind(p) == kind {
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
