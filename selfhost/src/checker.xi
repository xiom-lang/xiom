// XIOM -- Selfhost checker facade (Phase 3: parity port of crates/xiom-check)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Canonical `--dump-check` gate entry point. Owns the format definition
// (mirrors `crates/xiom/src/main.rs::dump_check` byte for byte):
//
//   {kind} {code} {line}:{col} {escaped-message}    one line per diagnostic
//   CHECK-OK                                        no diagnostics
//   PARSE-ERROR                                     input/lex/parse failed
//
// Diagnostics are emitted in the Rust order: warnings (checker push order)
// first, then hard errors (checker push order) -- exactly what
// `CompileResult.diagnostics` carries. `message` escapes `\`, LF, CR and TAB
// as `\\`, `\n`, `\r`, `\t`; every other byte is emitted verbatim.
//
// Stage boundaries (docs/checklists/selfhost-phase3.md): the signature/body
// checker subset plus the non-strict borrow pass are ported; catalog module
// bodies and the remaining permissive fallbacks are documented there.

module selfhost_checker

use xiom.io;
use xiom.string;
use selfhost_check_borrow;
use selfhost_check_core;
use selfhost_check_expr;
use selfhost_check_modules;
use selfhost_check_state;
use selfhost_check_state.Diag;
use selfhost_lexer;
use selfhost_parser_core;
use selfhost_parser_state;

/// Phase 0 compatibility shim used by the default compile path: only HARD
/// errors stop codegen (the Rust driver exits 0 on warnings, e.g. W003/W008
/// in the corpus). The checker gate uses `dump_check` directly.
pub fn check_count(src: Str, src_path: Str) -> Int {
  var lx = selfhost_lexer.Lexer.new(src);
  let toks = selfhost_lexer.lx_tokenize(&mut lx);
  var p = selfhost_parser_state.p_new(toks);
  let root = selfhost_parser_core.pc_parse_program(&mut p);
  if root < 0 || selfhost_parser_state.p_failed(&p) { return 1; }
  var c = selfhost_check_state.ck_new(p, selfhost_check_modules.cm_src_dir(src_path));
  selfhost_check_core.cc_collect(&mut c, root);
  selfhost_check_core.cc_check_program(&mut c, root);
  return c.errors.len();
}

fn ck_push_escaped(out: &mut Vec[UInt8], s: Str) {
  var i = 0;
  while i < s.len() {
    let b = string.byte_at(s, i) as Int;
    if b == 92 {
      out.push(92);
      out.push(92);
    } elif b == 10 {
      out.push(92);
      out.push(110);
    } elif b == 13 {
      out.push(92);
      out.push(114);
    } elif b == 9 {
      out.push(92);
      out.push(116);
    } else {
      out.push(b as UInt8);
    }
    i = i + 1;
  }
}

fn ck_push_dec(out: &mut Vec[UInt8], v: Int) {
  if v < 0 {
    out.push(45);
    ck_push_dec(out, 0 - v);
    return;
  }
  if v == 0 {
    out.push(48);
    return;
  }
  var digits = Vec[Int].new();
  var n = v;
  while n > 0 {
    digits.push(n % 10);
    n = n / 10;
  }
  var i = digits.len() - 1;
  while i >= 0 {
    out.push((48 + digits[i]) as UInt8);
    i = i - 1;
  }
}

fn ck_push_diag(out: &mut Vec[UInt8], kind: Str, code: Str, line: Int, col: Int, message: Str) {
  var i = 0;
  while i < kind.len() {
    out.push(string.byte_at(kind, i) as UInt8);
    i = i + 1;
  }
  out.push(32);
  i = 0;
  while i < code.len() {
    out.push(string.byte_at(code, i) as UInt8);
    i = i + 1;
  }
  out.push(32);
  ck_push_dec(out, line);
  out.push(58);
  ck_push_dec(out, col);
  out.push(32);
  ck_push_escaped(out, message);
  out.push(10);
}

/// Canonical checker dump (Phase 3 parity gate entry point). `src_path` is
/// the path AS GIVEN on the command line: it anchors local `use` module
/// resolution (`use vecmod;` -> sibling `vecmod.xi`).
pub fn dump_check(src: Str, src_path: Str) -> Int {
  var lx = selfhost_lexer.Lexer.new(src);
  let toks = selfhost_lexer.lx_tokenize(&mut lx);
  var p = selfhost_parser_state.p_new(toks);
  let root = selfhost_parser_core.pc_parse_program(&mut p);
  if root < 0 || selfhost_parser_state.p_failed(&p) {
    io.println("PARSE-ERROR");
    return 0;
  }
  var c = selfhost_check_state.ck_new(p, selfhost_check_modules.cm_src_dir(src_path));
  selfhost_check_core.cc_collect(&mut c, root);
  selfhost_check_core.cc_check_program(&mut c, root);
  // Phase 3 sub-stage 5: on the type-check success path run the borrow pass
  // (non-strict E001 warnings), mirroring `compile()` and the canonical
  // Rust `--dump-check`.
  var bw = Vec[Diag].new();
  if c.errors.len() == 0 {
    bw = selfhost_check_borrow.bc_run(&c.p, root);
  }
  var out = Vec[UInt8].new();
  if c.warnings.len() == 0 && c.errors.len() == 0 && bw.len() == 0 {
    io.println("CHECK-OK");
    return 0;
  }
  var i = 0;
  while i < c.warnings.len() {
    let d = c.warnings[i];
    ck_push_diag(&mut out, d.kind, d.code, d.line, d.col, d.message);
    i = i + 1;
  }
  i = 0;
  while i < c.errors.len() {
    let d = c.errors[i];
    ck_push_diag(&mut out, d.kind, d.code, d.line, d.col, d.message);
    i = i + 1;
  }
  i = 0;
  while i < bw.len() {
    let d = bw[i];
    ck_push_diag(&mut out, d.kind, d.code, d.line, d.col, d.message);
    i = i + 1;
  }
  io.print(Str::from_utf8(out));
  return 0;
}
