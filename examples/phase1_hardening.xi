// XIOM — phase1_hardening
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

// Covers: module system, structs with derive, enums with derive,
// generic functions, ownership/borrowing, control flow, error handling, contracts.

// ============================================================
// MODULE: tokenizer -- lexer types and helpers
// ============================================================
module tokenizer {
  pub type Token = {
    kind: Int;
    line: Int;
    col: Int;
    text: Str;
  } derive[Eq, Clone]

  pub enum TokenKind {
    Eof,
    Ident(name: Str),
    IntLit(value: Int),
    FloatLit(value: Float64),
    StrLit(text: Str),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Plus,
    Minus,
    Star,
    Slash,
  }

  pub fn kind_name(kind: Int) -> Int {
    match kind {
      0 => 1, 1 => 2, 2 => 3, 3 => 4, 4 => 5, _ => 0,
    }
  }

  pub fn make_token(kind: Int, line: Int, col: Int, text: Str) -> Token {
    return Token{ kind: kind, line: line, col: col, text: text };
  }

  pub fn is_operator(kind: Int) -> Bool {
    if kind == 9 { return true; }
    if kind == 10 { return true; }
    if kind == 11 { return true; }
    if kind == 12 { return true; }
    return false;
  }

  pub fn get_line(tok: &Token) -> Int { return tok.line; }
  pub fn clone_tok(tok: &Token) -> Token { return Token{ kind: tok.kind, line: tok.line, col: tok.col, text: tok.text }; }
}

module ast_types {
  pub type Span = { line: Int; col: Int; } derive[Eq, Clone, Hash]
  pub enum Expr { IntVal(value: Int), FloatVal(value: Float64), StrVal(value: Str), Ident(name: Str), BinaryOp } derive[Eq, Clone]
  pub type AstNode = { tag: Int; start_line: Int; end_line: Int; } derive[Eq, Clone, Hash, Ord]
}

module parser_core {
  use tokenizer.Token;
  use ast_types.Span;

  pub fn advance(pos: &mut Int) -> Int { let cur = pos; pos = pos + 1; return cur; }
  pub fn make_span(line: Int, col: Int) -> Span { return Span{ line: line, col: col }; }
  pub fn peek_token(tok: &Token) -> Int { return tok.kind; }
}

module checker {
  pub fn max(a: Int, b: Int) -> Int { if a > b { return a; } return b; }
  pub fn min(a: Int, b: Int) -> Int { if a < b { return a; } return b; }
  pub fn identity[T](x: T) -> T { return x; }
  pub fn compare_ints[T](a: Int, b: Int) -> Int { if a == b { return 0; } if a < b { return -1; } return 1; }
  pub fn is_digit(c: Int) -> Bool requires: c >= 0 ensures: result || true { if c >= 48 { if c <= 57 { return true; } } return false; }
}

module error_mod {
  pub type ParseError = { message: Str; line: Int; col: Int; } derive[Eq, Clone, Display]
  pub fn make_error(msg: Str, line: Int, col: Int) -> ParseError { return ParseError{ message: msg, line: line, col: col }; }
}

module pipeline {
  use tokenizer.Token; use tokenizer.make_token; use tokenizer.kind_name;
  use tokenizer.is_operator; use tokenizer.get_line; use tokenizer.clone_tok;
  use ast_types.Span; use ast_types.Expr; use ast_types.AstNode;
  use parser_core.advance; use parser_core.make_span; use parser_core.peek_token;
  use checker.max; use checker.min; use checker.identity; use checker.compare_ints;
  use error_mod.make_error;

  pub fn run_tokenizer() -> Int { let tok = make_token(1, 1, 1, "hello"); let name = kind_name(tok.kind); let op = is_operator(tok.kind); return name; }
  pub fn run_parser() -> Int { var pos = 0; let val = advance(&mut pos); return val + pos; }
  pub fn run_checker() -> Int { let a = max(10, 20); let b = min(30, 5); let c = identity(42); let d = compare_ints(42, 42); return a + b + c + d; }
  pub fn run_error() -> Int { let err = make_error("syntax error", 1, 10); return err.line; }
  pub fn run_borrowing(tok: &Token) -> Int { let line = get_line(tok); let kind = peek_token(tok); let owned = clone_tok(tok); return line + owned.kind; }
  pub fn run_control_flow() -> Int { var i = 0; var sum = 0; while i < 5 { sum = sum + i; i = i + 1; } return sum; }
  pub fn run_if_chain(x: Int) -> Int { if x > 10 { return 1; } elif x > 5 { return 2; } elif x > 0 { return 3; } else { return 0; } }
  pub fn run_option_demo() -> Int { let x = Some(42); if (x.is_some) { return x.value; } return 0; }
  pub fn run_span(line: Int, col: Int) -> Span { return make_span(line, col); }
  pub fn make_node(tag: Int, start_line: Int, end_line: Int) -> AstNode { return AstNode{ tag: tag, start_line: start_line, end_line: end_line }; }
}

use pipeline.run_tokenizer;
use error_mod.ParseError as PErr;
use tokenizer.*;

type BoundedInt = { value: Int; lo: Int; hi: Int; invariant: value >= lo; invariant: value <= hi; }
fn make_bounded(val: Int, lo: Int, hi: Int) -> BoundedInt { return BoundedInt{ value: val, lo: lo, hi: hi }; }
fn safe_divide(a: Float64, b: Float64) -> Float64 requires: b != 0.0 ensures: result * b == a { return a / b; }
fn wrap[T](x: T) -> T { return x; }
fn safe_divide_result(a: Float64, b: Float64) -> Result[Float64, Str] { if b == 0.0 { return Err("division by zero"); } return Ok(a / b); }

fn main() -> Int {
  var out = 0;
  let t_val = pipeline.run_tokenizer(); out = out + t_val;
  let p_val = pipeline.run_parser(); out = out + p_val;
  let c_val = pipeline.run_checker(); out = out + c_val;
  let e_val = pipeline.run_error(); out = out + e_val;
  let tok = make_token(2, 3, 5, "test");
  let borrow_val = pipeline.run_borrowing(&tok); out = out + borrow_val;
  let flow_val = pipeline.run_control_flow(); out = out + flow_val;
  let chain_val = pipeline.run_if_chain(7); out = out + chain_val;
  let opt_val = pipeline.run_option_demo(); out = out + opt_val;
  let sp = pipeline.run_span(1, 2);
  let node = pipeline.make_node(0, 1, 5);
  let generic_val = wrap(100); out = out + generic_val;
  let bi = make_bounded(5, 0, 10);
  let d = safe_divide(10.0, 2.0);
  let digit = checker.is_digit(53);
  if (digit) { out = out + 1; }
  let r = safe_divide_result(10.0, 2.0);
  match r { Ok(v) => { out = out + 1; } Err(e) => { out = out - 1; } }
  let owned_tok = clone_tok(&tok); out = out + owned_tok.kind;
  let nested = pipeline.run_if_chain(3); out = out + nested;
  return out;
}
