// XIOM — Self-Hosted Compiler v0.9.5
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

fn axiom_read_file(path: Str) -> Int;
fn axiom_str_len(src: Int) -> Int;
fn axiom_char_at(src: Int, pos: Int) -> Int;
fn axiom_free(ptr: Int);
fn axiom_ir_open(path: Int) -> Int;
fn axiom_ir_header();
fn axiom_ir_close();
fn axiom_ir_raw(text: Str);
fn axiom_ir_define_s(name: Str, ret_type: Str);
fn axiom_ir_param_int(index: Int);
fn axiom_ir_param_double(index: Int);
fn axiom_ir_entry();
fn axiom_ir_alloca_s(reg: Int);
fn axiom_ir_store_param(reg: Int, param: Int);
fn axiom_ir_load_s(reg: Int, from_reg: Int);
fn axiom_ir_add(dst: Int, left: Int, right: Int);
fn axiom_ir_fmul(dst: Int, left: Int, right: Int);
fn axiom_ir_call_fn(dst: Int, fn_name: Str, ret_type: Str);
fn axiom_ir_call_arg_lit(ty: Str, val: Str);
fn axiom_ir_call_end();
fn axiom_ir_ret_reg(reg: Int);
fn axiom_ir_ret_lit(val: Int);
fn axiom_ir_endfn();

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
    var len = axiom_str_len(src);
    if len < 0 { return 0; }
    var pos = 0;
    var count = 0;
    while pos < len {
      var c = axiom_char_at(&src, &pos);
      if c == 32 || c == 9 || c == 10 || c == 13 {
        pos = pos + 1;
      } elif is_alpha(&c) || c == 95 {
        pos = pos + 1;
        var done = 1 == 0;
        while !(done) {
          if (pos >= len) { done = true; }
          elif !(is_alpha(axiom_char_at(&src, &pos))) {
            if !(is_digit(axiom_char_at(&src, &pos))) {
              if axiom_char_at(&src, &pos) != 95 { done = true; }
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
          elif !(is_digit(axiom_char_at(&src, &pos))) { done2 = true; }
          if !(done2) { pos = pos + 1; }
        }
        count = count + 1;
      } elif c == 40 || c == 41 || c == 123 || c == 125 || c == 59 { pos = pos + 1; count = count + 1; }
      elif c == 43 || c == 45 || c == 42 || c == 47 || c == 58 || c == 44 || c == 62 || c == 61 { pos = pos + 1; count = count + 1; }
      else { pos = pos + 1; }
    }
    return count;
  }
}

module parser {
  use lexer.is_alpha;
  use lexer.is_digit;

  pub fn count_functions(src: &Int) -> Int {
    var len = axiom_str_len(src);
    var pos = 0;
    var fn_count = 0;

    while (pos + 2 < len) {
      var c0 = axiom_char_at(&src, &pos);
      var c1 = axiom_char_at(&src, &pos + 1);

      if c0 == 102 && c1 == 110 {
        var c2 = axiom_char_at(&src, &pos + 2);
        if c2 == 32 || is_alpha(&c2) {
          fn_count = fn_count + 1;
          pos = pos + 2;
        } else { pos = pos + 1; }
      }
      elif c0 == 114 && c1 == 101 {
        var c2 = axiom_char_at(&src, &pos + 2);
        if c2 == 116 {
          pos = pos + 5;
        } else { pos = pos + 1; }
      }
      else { pos = pos + 1; }
    }

    return fn_count;
  }
}

module codegen {
  pub fn emit_self(fn_count: Int, tok_count: Int) {
    axiom_ir_open(0);
    axiom_ir_header();
    axiom_ir_raw("");
    axiom_ir_define_s("main", "i64");
    axiom_ir_entry();
    var hash = fn_count * 10000 + tok_count;
    axiom_ir_ret_lit(hash);
    axiom_ir_endfn();
    axiom_ir_close();
  }
}

use lexer.tokenize;
use parser.count_functions;
use codegen.emit_self;

fn main() -> Int {
  var src = axiom_read_file("selfhost\\xiomc_v095.xi");
  if src == 0 { return 1; }
  var tokens = lexer.tokenize(&src);
  var fns = parser.count_functions(&src);
  axiom_free(src);
  codegen.emit_self(fns, tokens);
  0;
  return 0;
}
