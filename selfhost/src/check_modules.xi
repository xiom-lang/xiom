// XIOM -- Selfhost checker: catalog/imports resolution (Phase 3 sub-stage 1)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Loads the modules a source file imports so qualified calls resolve like the
// Rust checker's module map:
//   * `use xiom.a.b;`  -> <stdlib-root>/xiom/a/b.xi
//   * `use local.mod;` -> <source-dir>/local/mod.xi
//   * `use m.fn;` / `use m.*;` / `use m.x as y;` -> load the PREFIX module
//     `m` (item/glob imports do not bind a module alias; bare resolution
//     reaches the prefix's exports through the alias fallback in
//     `check_expr.xi`).
//
// Exports are collected from the file-level module chain only (the parser
// nests `module a.b` as Module(a){Module(b){...}}): pub fns (receiver
// methods keyed `K.Type.method`), pub types/enums and pub consts. Rust has
// zero `pub use` re-exports in this stdlib, so transitive import closure is
// not needed for export collection; module BODIES are not checked here
// (deferred: the corpus has no catalog-body diagnostics).
//
// A file that cannot be found or parsed stays UNLOADED and unaliased, so the
// permissive `_` fallback keeps such imports from producing false positives
// (Rust accepts a missing module too -- probe use_missing_mod.xi).

module selfhost_check_modules

use xiom.io;
use xiom.string;
use selfhost_ast.NodeKind;
use selfhost_check_state;
use selfhost_check_state.Checker;
use selfhost_check_state.Field;
use selfhost_check_state.FnParam;
use selfhost_check_state.FnSig;
use selfhost_check_state.Local;
use selfhost_check_state.TypeDecl;
use selfhost_check_types;
use selfhost_lexer;
use selfhost_parser_core;
use selfhost_parser_state;
use selfhost_parser_state.Parser;

// ============================================================================
// Path helpers
// ============================================================================

/// Directory part of a path (both separators), "" when no separator.
pub fn cm_src_dir(path: Str) -> Str {
  var i = path.len() - 1;
  while i >= 0 {
    let b = string.byte_at(path, i) as Int;
    if b == 47 || b == 92 { return string.str_slice(path, 0, i); }
    i = i - 1;
  }
  return "";
}

fn cm_join(a: Str, b: Str) -> Str {
  if a.len() == 0 { return b; }
  return a + "/" + b;
}

fn cm_leaf(dotted: Str) -> Str {
  var i = dotted.len() - 1;
  while i >= 0 {
    if (string.byte_at(dotted, i) as Int) == 46 {
      return string.str_slice(dotted, i + 1, dotted.len());
    }
    i = i - 1;
  }
  return dotted;
}

fn cm_prefix(dotted: Str) -> Str {
  var i = dotted.len() - 1;
  while i >= 0 {
    if (string.byte_at(dotted, i) as Int) == 46 {
      return string.str_slice(dotted, 0, i);
    }
    i = i - 1;
  }
  return "";
}

fn cm_dotted_to_rel(dotted: Str) -> Str {
  let parts = string.str_split(dotted, ".");
  var out = "";
  var i = 0;
  while i < parts.len() {
    if i > 0 { out = out + "/"; }
    out = out + parts[i];
    i = i + 1;
  }
  return out + ".xi";
}

/// Stdlib shape 1: `xiom.a.b` -> <root>/xiom/a/b.xi (nested leaf modules like
/// `xiom.io.fs`). Stdlib shape 2: the umbrella `<leaf>/<leaf>.xi` (e.g.
/// `xiom.io` -> <root>/xiom/io/io.xi). Local shape: the source-dir-relative
/// dotted path (`use m37_catmod;` -> <src-dir>/m37_catmod.xi). Files whose
/// location does not match their declared module (e.g.
/// `stdlib/xiom/os/path.xi` declaring `module xiom.path`) resolve through the
/// lazy declared-header index, mirroring the Rust catalog index.
pub fn cm_resolve_module_file(c: &mut Checker, dotted: Str) -> Str {
  if dotted.len() == 0 { return ""; }
  if string.str_starts_with(dotted, "xiom.") {
    let rel = cm_dotted_to_rel(dotted);
    let stem = string.str_slice(rel, 0, rel.len() - 3);
    let leaf = cm_leaf(dotted);
    var roots = Vec[Str].new();
    roots.push("stdlib");
    roots.push("../stdlib");
    roots.push("../../stdlib");
    var r = 0;
    while r < roots.len() {
      let p1 = roots[r] + "/" + rel;
      if io.file_exists(p1) { return p1; }
      let p2 = roots[r] + "/" + stem + "/" + leaf + ".xi";
      if io.file_exists(p2) { return p2; }
      let tail = cm_static_module_path(dotted);
      if tail.len() > 0 {
        let p3 = roots[r] + "/" + tail;
        if io.file_exists(p3) { return p3; }
      }
      r = r + 1;
    }
    return "";
  }
  let rel2 = cm_dotted_to_rel(dotted);
  let p = cm_join(c.src_dir, rel2);
  if io.file_exists(p) { return p; }
  let p2 = cm_join(c.src_dir, dotted + ".xi");
  if io.file_exists(p2) { return p2; }
  return "";
}

/// Static relocation table for stdlib modules whose declared dotted name
/// does not match any path shape (19 of 517 at v0.62.3). Computed from the
/// stdlib tree by declared-header (repro:
/// `tmp/sprintc/phase3_checker/module_overrides.ps1`); the dynamic header
/// index is blocked on `io.list_dir` returning corrupted strings
/// (COMPILER_BUGS 2026-10-03). Paths are stdlib-root relative.
pub fn cm_static_module_path(dotted: Str) -> Str {
  if dotted == "xiom.bigfloat" { return "xiom/num/bigfloat_agg.xi"; }
  if dotted == "xiom.bigint" { return "xiom/num/bigint.xi"; }
  if dotted == "xiom.chacha" { return "xiom/crypto/chacha.xi"; }
  if dotted == "xiom.char" { return "xiom/string/char.xi"; }
  if dotted == "xiom.cmp" { return "xiom/core/cmp.xi"; }
  if dotted == "xiom.complex" { return "xiom/math/complex.xi"; }
  if dotted == "xiom.contracts" { return "xiom/core/contracts.xi"; }
  if dotted == "xiom.crypto.md5" { return "xiom/crypto/legacy/md5.xi"; }
  if dotted == "xiom.crypto.sha" { return "xiom/crypto/legacy/sha.xi"; }
  if dotted == "xiom.des" { return "xiom/crypto/legacy/des.xi"; }
  if dotted == "xiom.ecc" { return "xiom/crypto/ecc.xi"; }
  if dotted == "xiom.env" { return "xiom/os/env.xi"; }
  if dotted == "xiom.fmt" { return "xiom/format/fmt.xi"; }
  if dotted == "xiom.path" { return "xiom/os/path.xi"; }
  if dotted == "xiom.platform" { return "xiom/core/platform.xi"; }
  if dotted == "xiom.poly1305" { return "xiom/crypto/poly1305.xi"; }
  if dotted == "xiom.process" { return "xiom/os/process.xi"; }
  if dotted == "xiom.rsa" { return "xiom/crypto/rsa.xi"; }
  if dotted == "xiom.utf8" { return "xiom/string/utf8.xi"; }
  return "";
}

// ============================================================================
// Module-path chain resolution (shared with check_expr)
// ============================================================================

/// Key for an Ident/Field chain that starts at the `xiom` namespace root, a
/// `use` alias or an in-program module name; "" when the chain is not a
/// module path. The final segment is returned verbatim even when it is not a
/// known module (callers resolve type/fn/const members).
pub fn cm_module_chain_key(c: &Checker, idx: Int) -> Str {
  if idx < 0 { return ""; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprIdent(name_idx) => {
      let nm = selfhost_check_state.ck_ident(c, name_idx);
      return cm_root_key(c, nm);
    }
    NkExprField(inner, name_idx) => {
      let base = cm_module_chain_key(c, inner);
      if base.len() == 0 { return ""; }
      let seg = selfhost_check_state.ck_ident(c, name_idx);
      return base + "." + seg;
    }
    _ => { return ""; }
  }
}

fn cm_root_key(c: &Checker, name: Str) -> Str {
  if name == "xiom" { return "xiom"; }
  let ak = selfhost_check_state.ck_alias_key(c, name);
  if ak.len() > 0 { return ak; }
  if selfhost_check_state.ck_is_module_name(c, name) { return name; }
  return "";
}

/// Syntactic dotted text of a chain (for Rust-shaped `{path} expects ...`
/// messages).
pub fn cm_chain_text(c: &Checker, idx: Int) -> Str {
  if idx < 0 { return ""; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprIdent(name_idx) => {
      return selfhost_check_state.ck_ident(c, name_idx);
    }
    NkExprField(inner, name_idx) => {
      let b = cm_chain_text(c, inner);
      let s = selfhost_check_state.ck_ident(c, name_idx);
      if b.len() == 0 { return s; }
      return b + "." + s;
    }
    _ => { return ""; }
  }
}

// ============================================================================
// Signature helpers shared with the program registration pass
// ============================================================================

pub fn cm_read_params(p: &Parser, params: Vec[Int]) -> Vec[FnParam] {
  var out = Vec[FnParam].new();
  var i = 0;
  while i < params.len() {
    let pnode = p.nodes[params[i]];
    match pnode.kind {
      NkParam(name, ty, mutself, refself) => {
        out.push(FnParam{ name: selfhost_check_state.ck_ident_in(p, name), ty: selfhost_check_state.ck_type_from_ast_in(p, ty) });
      }
      _ => {}
    }
    i = i + 1;
  }
  return out;
}

pub fn cm_read_generics(p: &Parser, generics: Vec[Int]) -> Vec[Str] {
  var out = Vec[Str].new();
  var i = 0;
  while i < generics.len() {
    let gnode = p.nodes[generics[i]];
    match gnode.kind {
      NkGeneric(name, bounds, is_const, const_ty) => {
        out.push(selfhost_check_state.ck_ident_in(p, name));
      }
      _ => {}
    }
    i = i + 1;
  }
  return out;
}

/// First registration wins (Rust `entry().or_insert` shape).
pub fn cm_push_fn(c: &mut Checker, key: Str, name: Str, params: Vec[FnParam],
                  ret: Str, has_ret: Int, generics: Vec[Str], has_recv: Int) {
  let _ = cm_push_fn_idx(c, key, name, params, ret, has_ret, generics, has_recv);
}

/// Same as `cm_push_fn` but returns the registered index (-1 when a sig with
/// this key already exists).
pub fn cm_push_fn_idx(c: &mut Checker, key: Str, name: Str, params: Vec[FnParam],
                      ret: Str, has_ret: Int, generics: Vec[Str], has_recv: Int) -> Int {
  var i = 0;
  while i < c.functions.len() {
    if c.functions[i].key == key { return -1; }
    i = i + 1;
  }
  c.functions.push(FnSig{
    key: key, name: name, params: params, ret: ret, has_ret: has_ret,
    generics: generics, has_recv: has_recv,
  });
  return c.functions.len() - 1;
}

pub fn cm_push_type(c: &mut Checker, name: Str, fields: Vec[Field], is_enum: Int) {
  var i = 0;
  while i < c.types.len() {
    if c.types[i].name == name { return; }
    i = i + 1;
  }
  c.types.push(TypeDecl{ name: name, fields: fields, is_enum: is_enum });
}

pub fn cm_push_global(c: &mut Checker, name: Str, ty: Str) {
  var i = 0;
  while i < c.globals.len() {
    if c.globals[i].name == name { return; }
    i = i + 1;
  }
  c.globals.push(Local{ name: name, ty: ty });
}

// ============================================================================
// Builtin method table (port of Checker::register_builtins)
// ============================================================================

fn cm_p1(n: Str, t: Str) -> FnParam {
  return FnParam{ name: n, ty: t };
}

fn cm_push_sig(c: &mut Checker, key: Str, params: Vec[FnParam], ret: Str, generics: Vec[Str]) {
  let idx = cm_push_fn_idx(c, key, cm_leaf(key), params, ret, 1, generics, 0);
  if idx >= 0 {
    let recv = cm_recv_of_key(key);
    if recv.len() > 0 {
      selfhost_check_state.ck_add_method(c, recv, cm_leaf(key), idx);
    }
  }
}

/// Receiver part of a `Type.method` builtin key, "" for free functions.
fn cm_recv_of_key(key: Str) -> Str {
  var i = key.len() - 1;
  while i >= 0 {
    if (string.byte_at(key, i) as Int) == 46 {
      return string.str_slice(key, 0, i);
    }
    i = i - 1;
  }
  return "";
}

/// The Rust checker's builtin `functions` entries. Instance dispatch reaches
/// these through `Vec.push` style keys; bare intrinsics (sizeof/panic/...)
/// use their plain name. Rust also mirrors a subset into `methods`; the port
/// resolves instance calls by key scan, so only `functions` is needed.
pub fn cm_register_builtin_fns(c: &mut Checker) {
  // Vec constructors/accessors.
  cm_push_sig(c, "Vec.new", Vec[FnParam].new(), "Vec", Vec[Str].new());
  var cap = Vec[FnParam].new();
  cap.push(cm_p1("capacity", "Int"));
  cm_push_sig(c, "Vec.with_capacity", cap, "Vec", Vec[Str].new());
  var vec_t = Vec[Str].new();
  vec_t.push("T");
  var push_p = Vec[FnParam].new();
  push_p.push(cm_p1("self", "Vec"));
  push_p.push(cm_p1("val", "T"));
  cm_push_sig(c, "Vec.push", push_p, "()", vec_t);
  var self_v = Vec[FnParam].new();
  self_v.push(cm_p1("self", "Vec"));
  cm_push_sig(c, "Vec.len", self_v, "Int", Vec[Str].new());
  cm_push_sig(c, "Vec.as_ptr", self_v, "_", Vec[Str].new());
  cm_push_sig(c, "Vec.as_mut_ptr", self_v, "_", Vec[Str].new());
  cm_push_sig(c, "Vec.pop", self_v, "Option", Vec[Str].new());
  cm_push_sig(c, "Vec.sort", self_v, "void", Vec[Str].new());
  var ins_p = Vec[FnParam].new();
  ins_p.push(cm_p1("self", "Vec"));
  ins_p.push(cm_p1("idx", "Int"));
  ins_p.push(cm_p1("val", "T"));
  cm_push_sig(c, "Vec.insert", ins_p, "()", vec_t);
  var rem_p = Vec[FnParam].new();
  rem_p.push(cm_p1("self", "Vec"));
  rem_p.push(cm_p1("idx", "Int"));
  cm_push_sig(c, "Vec.remove", rem_p, "Option", vec_t);
  cm_push_sig(c, "Vec.clear", self_v, "()", vec_t);
  cm_push_sig(c, "Vec.is_empty", self_v, "Bool", vec_t);
  var self_s = Vec[FnParam].new();
  self_s.push(cm_p1("self", "Slice"));
  cm_push_sig(c, "Slice.as_ptr", self_s, "_", Vec[Str].new());
  cm_push_sig(c, "Slice.as_mut_ptr", self_s, "_", Vec[Str].new());
  // Map/Set constructors (Map itself is not a builtin TYPE in Rust).
  cm_push_sig(c, "Map.new", Vec[FnParam].new(), "Map", Vec[Str].new());
  cm_push_sig(c, "Set.new", Vec[FnParam].new(), "Set", Vec[Str].new());
  // Free intrinsics.
  var gen_t = Vec[Str].new();
  gen_t.push("T");
  cm_push_sig(c, "sizeof", Vec[FnParam].new(), "Int", gen_t);
  cm_push_sig(c, "align_of", Vec[FnParam].new(), "Int", gen_t);
  cm_push_sig(c, "type_id", Vec[FnParam].new(), "Int", gen_t);
  var fo_p = Vec[FnParam].new();
  fo_p.push(cm_p1("field_name", "Str"));
  cm_push_sig(c, "field_offset", fo_p, "Int", gen_t);
  cm_push_sig(c, "is_signed", Vec[FnParam].new(), "Bool", gen_t);
  var tf_p = Vec[FnParam].new();
  tf_p.push(cm_p1("n", "Int"));
  cm_push_sig(c, "to_float", tf_p, "Float64", Vec[Str].new());
  var ti_p = Vec[FnParam].new();
  ti_p.push(cm_p1("f", "Float64"));
  cm_push_sig(c, "to_int", ti_p, "Int", Vec[Str].new());
  var tic_p = Vec[FnParam].new();
  tic_p.push(cm_p1("c", "Char"));
  cm_push_sig(c, "to_int_from_char", tic_p, "Int", Vec[Str].new());
  var tc_p = Vec[FnParam].new();
  tc_p.push(cm_p1("n", "Int"));
  cm_push_sig(c, "to_char", tc_p, "Char", Vec[Str].new());
  cm_push_sig(c, "unreachable", Vec[FnParam].new(), "!", Vec[Str].new());
  var panic_p = Vec[FnParam].new();
  panic_p.push(cm_p1("msg", "Str"));
  cm_push_sig(c, "panic", panic_p, "!", Vec[Str].new());
}

// ============================================================================
// Loading
// ============================================================================

pub fn cm_load_use(c: &mut Checker, dotted: Str) {
  if dotted.len() == 0 { return; }
  let full = cm_resolve_module_file(c, dotted);
  if full.len() > 0 {
    // R49-1: only a file whose DECLARED module matches the requested path
    // claims the alias (`dmod.xi` without `module dmod` stays qualified-
    // invisible like Rust, but its exports still join bare resolution).
    if cm_load_file(c, dotted, full) == 2 {
      selfhost_check_state.ck_add_alias(c, cm_leaf(dotted), dotted);
    }
    return;
  }
  // Item/glob/aliased import: load the prefix module so its exports are
  // reachable by the bare fallback; no module alias is bound.
  let prefix = cm_prefix(dotted);
  if prefix.len() == 0 { return; }
  let pf = cm_resolve_module_file(c, prefix);
  if pf.len() > 0 {
    let _ = cm_load_file(c, prefix, pf);
  }
}

/// 0 = not loaded, 1 = loaded (no module-header match), 2 = loaded and the
/// file's declared module path equals `key`.
pub fn cm_load_file(c: &mut Checker, key: Str, file: Str) -> Int {
  if selfhost_check_state.ck_is_loaded(c, key) { return 2; }
  if !io.file_exists(file) { return 0; }
  var src = "";
  match io.read_file(file) {
    Ok(text) => { src = text; }
    Err(e) => { return 0; }
  }
  var lx = selfhost_lexer.Lexer.new(src);
  let toks = selfhost_lexer.lx_tokenize(&mut lx);
  var p = selfhost_parser_state.p_new(toks);
  let root = selfhost_parser_core.pc_parse_program(&mut p);
  if root < 0 || selfhost_parser_state.p_failed(&p) { return 0; }
  selfhost_check_state.ck_mark_loaded(c, key);
  cm_collect_decls(c, key, &p, root);
  let declared = cm_declared_module(&p, root, "");
  if declared == key { return 2; }
  return 1;
}

/// Dotted name declared by the file-level module chain, "" when absent.
fn cm_declared_module(p: &Parser, idx: Int, acc: Str) -> Str {
  if idx < 0 { return acc; }
  let node = p.nodes[idx];
  match node.kind {
    NkProgram(items) => {
      if items.len() == 1 { return cm_declared_module(p, items[0], acc); }
      return acc;
    }
    NkModule(name, path, items, file_level, has_source, source) => {
      let nm = selfhost_check_state.ck_ident_in(p, name);
      var next = nm;
      if acc.len() > 0 { next = acc + "." + nm; }
      if items.len() == 1 && cm_node_is_module(p, items[0]) {
        return cm_declared_module(p, items[0], next);
      }
      return next;
    }
    _ => { return acc; }
  }
}

// ============================================================================
// Export collection
// ============================================================================

fn cm_node_is_module(p: &Parser, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = p.nodes[idx];
  match node.kind {
    NkModule(name, path, items, file_level, has_source, source) => { return true; }
    _ => { return false; }
  }
}

/// Descend the single-child module chain (`module a.b` nests), then collect
/// the pub declarations of the innermost module.
fn cm_collect_decls(c: &mut Checker, key: Str, p: &Parser, idx: Int) {
  if idx < 0 { return; }
  let node = p.nodes[idx];
  match node.kind {
    NkProgram(items) => {
      if items.len() == 1 && cm_node_is_module(p, items[0]) {
        cm_collect_decls(c, key, p, items[0]);
      } else {
        cm_collect_items(c, key, p, items);
      }
    }
    NkModule(name, path, items, file_level, has_source, source) => {
      if items.len() == 1 && cm_node_is_module(p, items[0]) {
        cm_collect_decls(c, key, p, items[0]);
      } else {
        cm_collect_items(c, key, p, items);
      }
    }
    _ => {}
  }
}

const CM_FN: Int = 1;
const CM_TYPE: Int = 2;
const CM_ENUM: Int = 3;
const CM_CONST: Int = 4;
const CM_EXTERN: Int = 5;
const CM_IMPL: Int = 6;
const CM_IFACE: Int = 7;
const CM_OTHER: Int = 0;

fn cm_decl_tag(p: &Parser, idx: Int) -> Int {
  if idx < 0 { return CM_OTHER; }
  let node = p.nodes[idx];
  match node.kind {
    NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => { return CM_FN; }
    NkTypeDecl(is_pub, name, generics, fields, derived, invariants, derives, alias) => { return CM_TYPE; }
    NkEnumDecl(is_pub, name, generics, variants, derives) => { return CM_ENUM; }
    NkConst(is_pub, is_mut, name, ty, value) => { return CM_CONST; }
    NkExtern(linkage, fns) => { return CM_EXTERN; }
    NkImpl(trait_name, trait_args, type_name, members) => { return CM_IMPL; }
    NkInterface(is_pub, name, generics, parent, members) => { return CM_IFACE; }
    _ => { return CM_OTHER; }
  }
}

/// Tag-based dispatch keeps each destructure in its own one-purpose helper
/// (COMPILER_BUGS 2026-10-02 Phase 3: multi-field destructures can mis-read
/// payload fields depending on the surrounding function).
fn cm_collect_items(c: &mut Checker, key: Str, p: &Parser, items: Vec[Int]) {
  var i = 0;
  while i < items.len() {
    let idx = items[i];
    let tag = cm_decl_tag(p, idx);
    if tag == CM_FN { cm_try_fn(c, key, p, idx); }
    if tag == CM_TYPE { cm_try_type(c, key, p, idx); }
    if tag == CM_ENUM { cm_try_enum(c, key, p, idx); }
    if tag == CM_CONST { cm_try_const(c, key, p, idx); }
    if tag == CM_EXTERN { cm_try_extern(c, key, p, idx); }
    if tag == CM_IMPL { cm_try_impl(c, key, p, idx); }
    if tag == CM_IFACE { cm_try_iface(c, key, p, idx); }
    i = i + 1;
  }
}

/// Extern fns (pub or not) register like Rust's catalog: the module's own
/// `extern "C"` block feeds bare/qualified resolution (m37_bug50 relies on
/// `malloc` from `xiom.core`).
fn cm_try_extern(c: &mut Checker, key: Str, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkExtern(linkage, fns) => {
      var i = 0;
      while i < fns.len() {
        let fnode = p.nodes[fns[i]];
        match fnode.kind {
          NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
            let nm = selfhost_check_state.ck_ident_in(p, name);
            let ps = cm_read_params(p, params);
            let gs = cm_read_generics(p, generics);
            var rt = "()";
            var has_ret = 0;
            if ret >= 0 {
              rt = selfhost_check_state.ck_type_from_ast_in(p, ret);
              has_ret = 1;
            }
            cm_push_fn(c, key + "." + nm, nm, ps, rt, has_ret, gs, 0);
          }
          _ => {}
        }
        i = i + 1;
      }
    }
    _ => {}
  }
}

fn cm_try_fn(c: &mut Checker, key: Str, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
      // Receiver-style fns register even when module-private (Rust's method
      // table is populated regardless of visibility: collections.xi's
      // `fn Map.insert` is reachable as a method from other modules).
      if is_pub != 1 && recv < 0 { return; }
      let nm = selfhost_check_state.ck_ident_in(p, name);
      let ps = cm_read_params(p, params);
      let gs = cm_read_generics(p, generics);
      var rt = "()";
      var has_ret = 0;
      if ret >= 0 {
        rt = selfhost_check_state.ck_type_from_ast_in(p, ret);
        has_ret = 1;
      }
      var recv_name = "";
      if recv >= 0 { recv_name = selfhost_check_state.ck_ident_in(p, recv); }
      if recv_name.len() > 0 {
        let fidx = cm_push_fn_idx(c, key + "." + recv_name + "." + cm_leaf(nm), cm_leaf(nm), ps, rt, has_ret, gs, 1);
        if fidx >= 0 { selfhost_check_state.ck_add_method(c, recv_name, cm_leaf(nm), fidx); }
      } else {
        cm_push_fn(c, key + "." + nm, nm, ps, rt, has_ret, gs, 0);
      }
    }
    _ => {}
  }
}

/// `impl Type { fn ... }` members become methods of the impl type (Rust
/// `register_impl_decl` -> `methods[impl_ty]`), even module-private.
fn cm_try_impl(c: &mut Checker, key: Str, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkImpl(trait_name, trait_args, type_name, members) => {
      let tn = selfhost_check_state.ck_ident_in(p, type_name);
      if tn.len() == 0 { return; }
      var i = 0;
      while i < members.len() {
        let mnode = p.nodes[members[i]];
        match mnode.kind {
          NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
            let nm = cm_leaf(selfhost_check_state.ck_ident_in(p, name));
            let ps = cm_read_params(p, params);
            let gs = cm_read_generics(p, generics);
            var rt = "()";
            var has_ret = 0;
            if ret >= 0 {
              rt = selfhost_check_state.ck_type_from_ast_in(p, ret);
              has_ret = 1;
            }
            let fidx = cm_push_fn_idx(c, key + "." + tn + "." + nm, nm, ps, rt, has_ret, gs, 1);
            if fidx >= 0 { selfhost_check_state.ck_add_method(c, tn, nm, fidx); }
          }
          _ => {}
        }
        i = i + 1;
      }
    }
    _ => {}
  }
}

/// `interface Name { fn m(params) -> ret; }` members (Rust `interfaces`).
fn cm_try_iface(c: &mut Checker, key: Str, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkInterface(is_pub, name, generics, parent, members) => {
      let nm = selfhost_check_state.ck_ident_in(p, name);
      cm_add_iface_name(c, key + "." + nm);
      cm_add_iface_name(c, nm);
      var i = 0;
      while i < members.len() {
        let mnode = p.nodes[members[i]];
        match mnode.kind {
          NkFn(is_pub2, is_async, recv, mname, generics2, params, ret, contracts, body, attrs) => {
            let mn = cm_leaf(selfhost_check_state.ck_ident_in(p, mname));
            let ps = cm_read_params(p, params);
            var types = Vec[Str].new();
            var j = 0;
            while j < ps.len() {
              types.push(ps[j].ty);
              j = j + 1;
            }
            var rt = "()";
            if ret >= 0 { rt = selfhost_check_state.ck_type_from_ast_in(p, ret); }
            selfhost_check_state.ck_add_iface_member(c, key + "." + nm, mn, types, rt);
            selfhost_check_state.ck_add_iface_member(c, nm, mn, types, rt);
          }
          _ => {}
        }
        i = i + 1;
      }
    }
    _ => {}
  }
}

fn cm_add_iface_name(c: &mut Checker, name: Str) {
  cm_push_interface_impl(c, name);
}

fn cm_push_interface_impl(c: &mut Checker, name: Str) {
  var i = 0;
  while i < c.interfaces.len() {
    if c.interfaces[i] == name { return; }
    i = i + 1;
  }
  c.interfaces.push(name);
}

fn cm_try_type(c: &mut Checker, key: Str, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkTypeDecl(is_pub, name, generics, fields, derived, invariants, derives, alias) => {
      if is_pub != 1 { return; }
      let nm = selfhost_check_state.ck_ident_in(p, name);
      cm_push_type(c, key + "." + nm, cm_read_fields(p, fields), 0);
    }
    _ => {}
  }
}

fn cm_try_enum(c: &mut Checker, key: Str, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkEnumDecl(is_pub, name, generics, variants, derives) => {
      if is_pub != 1 { return; }
      let nm = selfhost_check_state.ck_ident_in(p, name);
      cm_push_type(c, key + "." + nm, Vec[Field].new(), 1);
    }
    _ => {}
  }
}

fn cm_try_const(c: &mut Checker, key: Str, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkConst(is_pub, is_mut, name, ty, value) => {
      if is_pub != 1 { return; }
      let nm = selfhost_check_state.ck_ident_in(p, name);
      var declared = "_";
      if ty >= 0 { declared = selfhost_check_state.ck_type_from_ast_in(p, ty); }
      cm_push_global(c, key + "." + nm, declared);
    }
    _ => {}
  }
}

fn cm_read_fields(p: &Parser, fields: Vec[Int]) -> Vec[Field] {
  var out = Vec[Field].new();
  var i = 0;
  while i < fields.len() {
    let fnode = p.nodes[fields[i]];
    match fnode.kind {
      NkField(fname, fty) => {
        let nm = selfhost_check_state.ck_ident_in(p, fname);
        var dup = false;
        var j = 0;
        while j < out.len() {
          if out[j].name == nm { dup = true; }
          j = j + 1;
        }
        if !dup {
          out.push(Field{ name: nm, ty: selfhost_check_state.ck_type_from_ast_in(p, fty) });
        }
      }
      _ => {}
    }
    i = i + 1;
  }
  return out;
}
