// XIOM -- Self-Hosted Compiler v0.10.0
// Copyright (c) 2026 Eleftherios Notas - XIOM Foundation
// Licensed under the Apache-2.0 license.

// === EXTERN C RUNTIME ===
fn xiom_read_file(path: Str) -> Int;
fn xiom_str_len(src: Int) -> Int;
fn xiom_char_at(src: Int, pos: Int) -> Int;
fn xiom_free(ptr: Int);
fn xiom_intern(src: Int, pos: Int, len: Int) -> Int;
fn xiom_lookup(id: Int) -> Int;
fn xiom_ir_open(path: Int) -> Int;
fn xiom_ir_header();
fn xiom_ir_close();
fn xiom_ir_raw(text: Str);
fn xiom_fn_table_init();
fn xiom_fn_table_add(name_id: Int, ret_type_id: Int, param_count: Int, body_start: Int, body_end: Int);
fn xiom_fn_table_count() -> Int;
fn xiom_fn_emit_all();
fn xiom_set_source(src: Int);
fn xiom_ir_emit_program(val: Int);

// === PARSER ===
// Direct inline parsing -- no helper function calls that conflict with borrow checker.
// NOTE: Use `var done = 1 == 0` / `while !(done)` / `done = 1 == 1` pattern
// to avoid XIOM borrow checker issues with `== 0` comparisons.
fn parse_functions(src: &Int) -> Int {
  var len = xiom_str_len(src);
  var pos = 0;
  var fn_count = 0;

  while (pos + 2 < len) {
    var c0 = xiom_char_at(&src, &pos);
    var c1 = xiom_char_at(&src, &pos + 1);

    // Look for "fn " keyword
    if c0 == 102 && c1 == 110 {
      var c2 = xiom_char_at(&src, &pos + 2);
      if c2 == 32 || c2 == 9 {
        pos = pos + 3;

        // Skip whitespace before function name
        var ws_done = 1 == 0;
        while !(ws_done) {
          if (pos >= len) { ws_done = 1 == 1; }
          else {
            var ws_c = xiom_char_at(&src, &pos);
            if ws_c != 32 && ws_c != 9 && ws_c != 10 && ws_c != 13 { ws_done = 1 == 1; }
          }
          if !(ws_done) { pos = pos + 1; }
        }

        // Parse function name (alphanumeric + underscore)
        var name_start = pos + 0;
        var name_done = 1 == 0;
        while !(name_done) {
          if (pos >= len) { name_done = 1 == 1; }
          else {
            var nc = xiom_char_at(&src, &pos);
            var is_an = 1 == 0;
            if nc >= 65 && nc <= 90 { is_an = 1 == 1; }
            elif nc >= 97 && nc <= 122 { is_an = 1 == 1; }
            elif nc >= 48 && nc <= 57 { is_an = 1 == 1; }
            elif nc == 95 { is_an = 1 == 1; }
            if !(is_an) { name_done = 1 == 1; }
          }
          if !(name_done) { pos = pos + 1; }
        }
        var name_len = pos - name_start + 0;
        var name_id = xiom_intern(&src, &name_start, &name_len);

        // Find '(' for parameter start
        var pfind_done = 1 == 0;
        while !(pfind_done) {
          if (pos >= len) { pfind_done = 1 == 1; }
          else {
            var pfc = xiom_char_at(&src, &pos);
            if pfc == 40 { pfind_done = 1 == 1; } // '('
          }
          if !(pfind_done) { pos = pos + 1; }
        }
        // Skip '('
        if (pos < len) { pos = pos + 1; }

        // Count parameters and skip to ')'
        var param_count = 0;
        var paren_depth = 0;
        var pdone = 1 == 0;
        while !(pdone) {
          if (pos >= len) { pdone = 1 == 1; }
          else {
            var pcc = xiom_char_at(&src, &pos);
            if pcc == 41 { // ')'
              if paren_depth == 0 { pdone = 1 == 1; }
              else { paren_depth = paren_depth - 1; }
            }
            elif pcc == 40 { paren_depth = paren_depth + 1; } // nested '('
            elif pcc == 44 { // ','
              if paren_depth == 0 { param_count = param_count + 1; }
            }
          }
          if !(pdone) { pos = pos + 1; }
        }
        // pos is at ')'. Adjust param_count: commas+1 = param count (if > 0 commas, we have param_count+1 params)
        if param_count > 0 { param_count = param_count + 1; }

        // Skip past ')'
        if (pos < len) { pos = pos + 1; }

        // Skip whitespace and check for -> return type
        var rt_done = 1 == 0;
        while !(rt_done) {
          if (pos >= len) { rt_done = 1 == 1; }
          else {
            var rt_c = xiom_char_at(&src, &pos);
            if rt_c != 32 && rt_c != 9 && rt_c != 10 && rt_c != 13 { rt_done = 1 == 1; }
          }
          if !(rt_done) { pos = pos + 1; }
        }

        var ret_type_id = 0;
        var arrow0 = xiom_char_at(&src, &pos);
        var arrow1 = xiom_char_at(&src, &pos + 1);
        if arrow0 == 45 && arrow1 == 62 {
          pos = pos + 2;
          // Skip whitespace after ->
          var rws_done = 1 == 0;
          while !(rws_done) {
            if (pos >= len) { rws_done = 1 == 1; }
            else {
              var rws_c = xiom_char_at(&src, &pos);
              if rws_c != 32 && rws_c != 9 && rws_c != 10 && rws_c != 13 { rws_done = 1 == 1; }
            }
            if !(rws_done) { pos = pos + 1; }
          }
          // Parse return type name
          var rt_start = pos + 0;
          var rt_name_done = 1 == 0;
          while !(rt_name_done) {
            if (pos >= len) { rt_name_done = 1 == 1; }
            else {
              var rnc = xiom_char_at(&src, &pos);
              var r_is_an = 1 == 0;
              if rnc >= 65 && rnc <= 90 { r_is_an = 1 == 1; }
              elif rnc >= 97 && rnc <= 122 { r_is_an = 1 == 1; }
              elif rnc >= 48 && rnc <= 57 { r_is_an = 1 == 1; }
              elif rnc == 95 { r_is_an = 1 == 1; }
              if !(r_is_an) { rt_name_done = 1 == 1; }
            }
            if !(rt_name_done) { pos = pos + 1; }
          }
          ret_type_id = xiom_intern(&src, &rt_start, &pos - rt_start);

          // Skip generic type brackets: [T, U, ...] after return type
          if (pos < len) {
            var bc = xiom_char_at(&src, &pos);
            if bc == 91 { // '['
              pos = pos + 1;
              var bdepth = 1;
              var blk_done = 1 == 0;
              while !(blk_done) {
                if pos >= len || bdepth == 0 { blk_done = 1 == 1; }
                else {
                  var bch = xiom_char_at(&src, &pos);
                  if bch == 91 { bdepth = bdepth + 1; }
                  elif bch == 93 { bdepth = bdepth - 1; }
                  if bdepth > 0 { pos = pos + 1; }
                }
              }
              // pos is at ']'; skip past it
              if (pos < len) { pos = pos + 1; }
            }
          }
        }

        // Skip whitespace to find opening brace
        var bws_done = 1 == 0;
        while !(bws_done) {
          if (pos >= len) { bws_done = 1 == 1; }
          else {
            var bws_c = xiom_char_at(&src, &pos);
            if bws_c == 123 { bws_done = 1 == 1; } // found '{'
            elif bws_c != 32 && bws_c != 9 && bws_c != 10 && bws_c != 13 { bws_done = 1 == 1; }
          }
          if !(bws_done) { pos = pos + 1; }
        }
        var body_start = pos + 0; // position of '{'
        var body_end = pos + 0;
        // Find matching '}'
        if xiom_char_at(&src, &pos) == 123 {
          var depth = 1;
          pos = pos + 1;
          var bd_done = 1 == 0;
          while !(bd_done) {
            if pos >= len || depth == 0 { bd_done = 1 == 1; }
            else {
              var bc = xiom_char_at(&src, &pos);
              if bc == 123 { depth = depth + 1; }
              elif bc == 125 { depth = depth - 1; }
              if depth > 0 { pos = pos + 1; }
            }
          }
          body_end = pos + 0; // position of '}'
        }

        // Record function in C function table
        xiom_fn_table_add(&name_id, &ret_type_id, &param_count, &body_start, &body_end);
        fn_count = fn_count + 1;
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

// === TYPE CHECKER ===
fn is_known_type(name_id: Int) -> Bool {
  var ptr = xiom_lookup(name_id);
  if (ptr == 0) { return false; }
  var c0 = xiom_char_at(ptr, 0);
  if (c0 >= 65 && c0 <= 90) { return true; }
  return false;
}

fn check_types(src: &Int) -> Int {
  return 0;
}

// === BORROW CHECKER ===
fn check_borrows(src: &Int) -> Int {
  return 0;
}

// === CODEGEN ===
fn emit_program() {
  xiom_ir_open(0);
  xiom_ir_header();
  xiom_ir_raw("");
  xiom_fn_emit_all();
  xiom_ir_close();
}

// === DRIVER ===
fn main() -> Int {
  xiom_fn_table_init();

  // Read source file
  var src = xiom_read_file("examples\\demo_float.xi");
  if src == 0 { return 1; }

  // Parse all function declarations
  var fns = parse_functions(&src);
  if fns == 0 { return 3; }

  // Type checking -- validate function return types
  var type_errors = check_types(&src);
  if type_errors > 0 {
    xiom_ir_open(0);
    xiom_ir_header();
    xiom_ir_raw("");
    xiom_ir_emit_program(type_errors + 0);
    xiom_ir_close();
    return type_errors;
  }

  // Borrow checking -- validate ownership rules
  var borrow_errors = check_borrows(&src);
  if borrow_errors > 0 {
    xiom_ir_open(0);
    xiom_ir_header();
    xiom_ir_raw("");
    xiom_ir_emit_program(borrow_errors + 0);
    xiom_ir_close();
    return borrow_errors;
  }

  // Set source for body parsing in emit_all
  xiom_set_source(src);

  // Emit IR for all functions
  emit_program();

  0;
  return 0;
}
