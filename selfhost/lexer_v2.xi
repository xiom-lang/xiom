// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M20-B2a: Minimal Lexer -- read real files, tokenize, NO &mut
use stdlib.xiom.io;
use stdlib.xiom.string;

pub type Lexer = {
  source: Str;
  pos: Int;
  line: Int;
  col: Int;
}

pub fn lexer_new(source: Str) -> Lexer {
  return Lexer{ source: source, pos: 0, line: 1, col: 1 };
}

fn peek(l: &Lexer) -> Int {
  if l.pos >= string.str_len(l.source) { return 0; }
  return string.byte_at(l.source, l.pos) as Int;
}

fn peek_ahead(l: &Lexer, off: Int) -> Int {
  let idx = l.pos + off;
  if idx >= string.str_len(l.source) { return 0; }
  return string.byte_at(l.source, idx) as Int;
}

fn advance(l: Lexer) -> Lexer {
  let c = peek(&l);
  var new_line = l.line;
  var new_col = l.col;
  if c == 10 { new_line = new_line + 1; new_col = 1; }
  else { new_col = new_col + 1; }
  return Lexer{ source: l.source, pos: l.pos + 1, line: new_line, col: new_col };
}

fn skip_ws(l: Lexer) -> Lexer {
  var cur = l;
  var done = false;
  while !done {
    let c = peek(&cur);
    if c == 32 || c == 9 || c == 10 || c == 13 { cur = advance(cur); continue; }
    if c == 47 && peek_ahead(&cur, 1) == 47 {
      while peek(&cur) != 10 && peek(&cur) != 0 { cur = advance(cur); }
      if peek(&cur) == 10 { cur = advance(cur); }
      continue;
    }
    done = true;
  }
  return cur;
}

fn is_alpha(c: Int) -> Bool {
  if c >= 65 && c <= 90 { return true; }
  if c >= 97 && c <= 122 { return true; }
  return false;
}
fn is_digit(c: Int) -> Bool {
  if c >= 48 && c <= 57 { return true; }
  return false;
}
fn is_alnum(c: Int) -> Bool {
  if is_alpha(c) || is_digit(c) || c == 95 { return true; }
  return false;
}

pub type Token = {
  kind: Int;
  text: Str;
  line: Int;
  col: Int;
}

fn make_tok(kind: Int, text: Str, line: Int, col: Int) -> Token {
  return Token{ kind: kind, text: text, line: line, col: col };
}

fn read_word(l: Lexer) -> (Token, Lexer) {
  let start_col = l.col;
  let start = l.pos;
  var cur = l;
  while is_alnum(peek(&cur)) { cur = advance(cur); }
  let text = string.str_slice(cur.source, start, cur.pos);
  var kind = 100;
  if text == "fn" { kind = 1; }
  elif text == "return" { kind = 2; }
  elif text == "if" { kind = 3; }
  elif text == "else" { kind = 4; }
  elif text == "let" { kind = 5; }
  elif text == "var" { kind = 6; }
  elif text == "while" { kind = 7; }
  elif text == "for" { kind = 13; }
  elif text == "match" { kind = 14; }
  elif text == "Int" { kind = 8; }
  elif text == "Bool" { kind = 9; }
  elif text == "Str" { kind = 11; }
  elif text == "pub" { kind = 16; }
  elif text == "use" { kind = 17; }
  elif text == "type" { kind = 10; }
  elif text == "enum" { kind = 15; }
  elif text == "module" { kind = 18; }
  elif text == "unsafe" { kind = 19; }
  elif text == "elif" { kind = 20; }
  elif text == "true" { kind = 21; }
  elif text == "false" { kind = 22; }
  return (make_tok(kind, text, l.line, start_col), cur);
}

fn read_num(l: Lexer) -> (Token, Lexer) {
  let start_col = l.col;
  let start = l.pos;
  var cur = l;
  var is_float = false;
  while is_digit(peek(&cur)) { cur = advance(cur); }
  if peek(&cur) == 46 {
    is_float = true;
    cur = advance(cur);
    while is_digit(peek(&cur)) { cur = advance(cur); }
  }
  let text = string.str_slice(cur.source, start, cur.pos);
  if is_float { return (make_tok(201, text, l.line, start_col), cur); }
  return (make_tok(200, text, l.line, start_col), cur);
}

fn read_str(l: Lexer) -> (Token, Lexer) {
  let start_col = l.col;
  var cur = advance(l); // skip "
  let start = cur.pos;
  while peek(&cur) != 34 && peek(&cur) != 0 { cur = advance(cur); }
  let text = string.str_slice(cur.source, start, cur.pos);
  if peek(&cur) == 34 { cur = advance(cur); }
  return (make_tok(202, text, l.line, start_col), cur);
}

pub fn next_token(l: Lexer) -> (Token, Lexer) {
  var cur = skip_ws(l);
  let c = peek(&cur);
  if c == 0 { return (make_tok(0, "", cur.line, cur.col), cur); }
  // Words
  if is_alpha(c) || c == 95 { return read_word(cur); }
  // Numbers
  if is_digit(c) { return read_num(cur); }
  // Strings
  if c == 34 { return read_str(cur); }
  // Single-char tokens
  let start_col = cur.col;
  cur = advance(cur);
  if c == 40 { return (make_tok(50, "(", l.line, start_col), cur); }
  if c == 41 { return (make_tok(51, ")", l.line, start_col), cur); }
  if c == 123 { return (make_tok(52, "{", l.line, start_col), cur); }
  if c == 125 { return (make_tok(53, "}", l.line, start_col), cur); }
  if c == 59 { return (make_tok(54, ";", l.line, start_col), cur); }
  if c == 58 { return (make_tok(55, ":", l.line, start_col), cur); }
  if c == 43 { return (make_tok(43, "+", l.line, start_col), cur); }
  if c == 42 { return (make_tok(45, "*", l.line, start_col), cur); }
  if c == 47 { return (make_tok(46, "/", l.line, start_col), cur); }
  if c == 46 { return (make_tok(61, ".", l.line, start_col), cur); }
  if c == 44 { return (make_tok(62, ",", l.line, start_col), cur); }
  if c == 61 {
    if peek(&cur) == 61 { cur = advance(cur); return (make_tok(41, "==", l.line, start_col), cur); }
    return (make_tok(40, "=", l.line, start_col), cur);
  }
  if c == 45 {
    if peek(&cur) == 62 { cur = advance(cur); return (make_tok(56, "->", l.line, start_col), cur); }
    return (make_tok(44, "-", l.line, start_col), cur);
  }
  if c == 62 {
    if peek(&cur) == 61 { cur = advance(cur); return (make_tok(59, ">=", l.line, start_col), cur); }
    return (make_tok(57, ">", l.line, start_col), cur);
  }
  if c == 60 {
    if peek(&cur) == 61 { cur = advance(cur); return (make_tok(60, "<=", l.line, start_col), cur); }
    return (make_tok(58, "<", l.line, start_col), cur);
  }
  return (make_tok(0, "?", l.line, start_col), cur);
}

// ============================================================================
fn main() -> Int {
  io.println("Lexer v2 -- value-passing style");
  let file_result = io.read_file("selfhost/ast.xi");
  match file_result {
    Ok(source) => {
      var lex = lexer_new(source);
      var count = 0;
      var done = false;
      while !done && count < 30 {
        let result = next_token(lex);
        lex = result.1;
        let tok = result.0;
        if tok.kind == 0 { done = true; }
        else {
          io.println("  [" + tok.text + "] k=" + "");
          count = count + 1;
        }
      }
      io.println("Tokenized " + count.to_string() + " tokens from selfhost/ast.xi");
      if count > 0 { return 0; }
      return 1;
    }
    Err(e) => { io.println("ERROR: " + e.message); return 1; }
  }
}
