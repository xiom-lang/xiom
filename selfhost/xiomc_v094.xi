// XIOM -- Self-Hosted Compiler v0.9.4
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

fn xiom_read_file(path: Str) -> Int;
fn xiom_str_len(src: Int) -> Int;
fn xiom_char_at(src: Int, pos: Int) -> Int;
fn xiom_free(ptr: Int);
fn xiom_ir_open(path: Int) -> Int;
fn xiom_ir_header();
fn xiom_ir_close();
fn xiom_ir_raw(text: Str);
fn xiom_ir_define_s(name: Str, ret_type: Str);
fn xiom_ir_param_int(index: Int);
fn xiom_ir_param_double(index: Int);
fn xiom_ir_entry();
fn xiom_ir_alloca_s(reg: Int);
fn xiom_ir_store_param(reg: Int, param: Int);
fn xiom_ir_load_s(reg: Int, from_reg: Int);
fn xiom_ir_add(dst: Int, left: Int, right: Int);
fn xiom_ir_fmul(dst: Int, left: Int, right: Int);
fn xiom_ir_call_fn(dst: Int, fn_name: Str, ret_type: Str);
fn xiom_ir_call_arg_lit(ty: Str, val: Str);
fn xiom_ir_call_end();
fn xiom_ir_ret_reg(reg: Int);
fn xiom_ir_ret_lit(val: Int);
fn xiom_ir_endfn();

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
    while pos < len {
      var c = xiom_char_at(&src, &pos);
      if c == 32 || c == 9 || c == 10 || c == 13 {
        pos = pos + 1;
      } elif is_alpha(&c) || c == 95 {
        pos = pos + 1;
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
    var len = xiom_str_len(src);
    var pos = 0;
    var fn_count = 0;

    while (pos + 2 < len) {
      var c0 = xiom_char_at(&src, &pos);
      var c1 = xiom_char_at(&src, &pos + 1);

      if c0 == 102 && c1 == 110 {
        var c2 = xiom_char_at(&src, &pos + 2);
        if c2 == 32 || is_alpha(&c2) {
          fn_count = fn_count + 1;
          pos = pos + 2;
        } else { pos = pos + 1; }
      }
      elif c0 == 114 && c1 == 101 {
        var c2 = xiom_char_at(&src, &pos + 2);
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
  pub fn emit_demo_float() {
    xiom_ir_define_s("add", "i64");
    xiom_ir_param_int(0);
    xiom_ir_param_int(1);
    xiom_ir_entry();
    xiom_ir_alloca_s(0);
    xiom_ir_store_param(0, 0);
    xiom_ir_alloca_s(1);
    xiom_ir_store_param(1, 1);
    xiom_ir_load_s(2, 0);
    xiom_ir_load_s(3, 1);
    xiom_ir_add(4, 2, 3);
    xiom_ir_ret_reg(4);
    xiom_ir_endfn();

    xiom_ir_define_s("sq", "double");
    xiom_ir_param_double(0);
    xiom_ir_entry();
    xiom_ir_alloca_s(0);
    xiom_ir_store_param(0, 0);
    xiom_ir_load_s(1, 0);
    xiom_ir_load_s(2, 0);
    xiom_ir_fmul(3, 1, 2);
    xiom_ir_ret_reg(3);
    xiom_ir_endfn();

    xiom_ir_define_s("main", "i64");
    xiom_ir_entry();
    xiom_ir_call_fn(0, "sq", "double");
    xiom_ir_call_arg_lit("double", "3.000000");
    xiom_ir_call_end();
    xiom_ir_alloca_s(1);
    xiom_ir_store_param(1, 0);
    xiom_ir_call_fn(2, "add", "i64");
    xiom_ir_call_arg_lit("i64", "10");
    xiom_ir_call_arg_lit("i64", "20");
    xiom_ir_call_end();
    xiom_ir_ret_reg(2);
    xiom_ir_endfn();
  }
}

use lexer.tokenize;
use parser.count_functions;
use codegen.emit_demo_float;

fn main() -> Int {
  var src = xiom_read_file("examples\\demo_float.xi");
  if src == 0 { return 1; }

  var tok_res = lexer.tokenize(&src);

  var fns = parser.count_functions(&src);
  xiom_free(src);

  xiom_ir_open(0);
  xiom_ir_header();
  xiom_ir_raw("");
  codegen.emit_demo_float();
  xiom_ir_close();

  0;
  return 0;
}
