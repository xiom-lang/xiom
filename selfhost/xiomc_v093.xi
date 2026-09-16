// XIOM -- Self-Hosted Compiler v0.9.3
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// Extern C runtime declarations
fn xiom_read_file(path: Str) -> Int;
fn xiom_str_len(src: Int) -> Int;
fn xiom_char_at(src: Int, pos: Int) -> Int;
fn xiom_free(ptr: Int);
fn xiom_ir_open(path: Int) -> Int;
fn xiom_ir_header();
fn xiom_ir_emit_program(val: Int);
fn xiom_ir_close();
fn xiom_ir_raw(text: Str);

module lexer {
  pub fn is_alpha(c: Int) -> Bool {
    if c >= 65 && c <= 90 { return true; }
    if c >= 97 && c <= 122 { return true; }
    return false;
  }
  pub fn is_digit(c: Int) -> Bool {
    if c >= 48 && c <= 57 { return true; }
    return false;
  }

  pub fn tokenize(src: &Int) -> Int {
    var len = xiom_str_len(src);
    if len < 0 { return 0; }
    var pos = 0;
    var count = 0;
    var kw_count = 0;

    while pos < len {
      var c = xiom_char_at(&src, &pos);
      if c == 32 || c == 9 || c == 10 || c == 13 {
        pos = pos + 1;
      } elif is_alpha(&c) || c == 95 {
        var next_pos = pos + 1;
        var c1 = xiom_char_at(&src, &next_pos);
        if c == 102 && c1 == 110 { kw_count = kw_count + 1; }
        if c == 114 && c1 == 101 { kw_count = kw_count + 1; }
        if c == 108 && c1 == 101 { kw_count = kw_count + 1; }
        if c == 118 && c1 == 97 { kw_count = kw_count + 1; }
        if c == 105 && c1 == 102 { kw_count = kw_count + 1; }
        if c == 109 && c1 == 111 { kw_count = kw_count + 1; }
        if c == 116 && c1 == 121 { kw_count = kw_count + 1; }
        if c == 101 && c1 == 110 { kw_count = kw_count + 1; }
        pos = pos + 2;
        var done = 1 == 0;
        while !(done) {
          if (pos >= len) { done = true; }
          elif !(is_alpha(xiom_char_at(&src, &pos))) {
            if !(is_digit(xiom_char_at(&src, &pos))) {
              if xiom_char_at(&src, &pos) != 95 { done = true; }
            }
          }
          if !(done) { pos = pos + 1; }
        }
        count = count + 1;
      } elif is_digit(&c) {
        pos = pos + 1;
        var done2 = 1 == 0;
        while !(done2) {
          if (pos >= len) { done2 = true; }
          elif !(is_digit(xiom_char_at(&src, &pos))) { done2 = true; }
          if !(done2) { pos = pos + 1; }
        }
        count = count + 1;
      } elif c == 40 || c == 41 || c == 123 || c == 125 || c == 59 {
        pos = pos + 1; count = count + 1;
      } elif c == 43 || c == 45 || c == 42 || c == 47 || c == 58 || c == 44 || c == 62 || c == 61 {
        pos = pos + 1; count = count + 1;
      } else {
        pos = pos + 1;
      }
    }

    return count + kw_count * 1000;
  }
}

use lexer.tokenize;

fn main() -> Int {
  var src = xiom_read_file("selfhost\\xiomc_v093.xi");
  if src == 0 { return 1; }

  var tok_res = lexer.tokenize(&src);

  xiom_free(src);

  xiom_ir_open(0);
  xiom_ir_header();

  var token_count = tok_res - (tok_res / 1000) * 1000;
  xiom_ir_raw("declare i32 @printf(i8*, ...)");
  xiom_ir_raw("declare i32 @puts(i8*)");
  xiom_ir_raw("");
  xiom_ir_emit_program(token_count);

  xiom_ir_close();

  0;
  return 0;
}
