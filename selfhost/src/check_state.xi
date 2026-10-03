// XIOM -- Selfhost checker: state + diagnostics + type compatibility
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Phase 3 stage 1: the shared state for the checker port. Symbol tables are
// flat `Vec`s with linear lookup (the gate compiles one file at a time; the
// corpus files carry tens of symbols, so HashMap parity is not required for
// correctness). Scopes use the same append-only + counter discipline the rest
// of the selfhost uses (no `Vec::pop`, no nested `Vec[Vec[..]]`, both of
// which are risky under the current compiler).
//
// Checked types are canonical NAME strings (`selfhost_check_types`); the
// diagnostics struct mirrors `crates/xiom-check::CheckError` (message + span)
// with the driver's stable code ("T001"/"WNNN") attached.

module selfhost_check_state

use xiom.string;
use selfhost_ast.Span;
use selfhost_ast.Node;
use selfhost_ast.NodeKind;
use selfhost_check_types;
use selfhost_parser_state.Parser;

pub type Diag = {
  kind: Str;
  code: Str;
  message: Str;
  line: Int;
  col: Int;
}

pub type Local = {
  name: Str;
  ty: Str;
}

pub type FnParam = {
  name: Str;
  ty: Str;
}

pub type FnSig = {
  key: Str;
  name: Str;
  params: Vec[FnParam];
  ret: Str;
  has_ret: Int;
  generics: Vec[Str];
  has_recv: Int;
}

pub type Field = {
  name: Str;
  ty: Str;
}

pub type TypeDecl = {
  name: Str;
  fields: Vec[Field];
  is_enum: Int;
}

pub type Variant = {
  key: Str;
  enum_name: Str;
  fields: Vec[Field];
}

pub type Checker = {
  p: Parser;
  src_dir: Str;
  locals: Vec[Local];
  nlocals: Int;
  scopes: Vec[Int];
  nscopes: Int;
  functions: Vec[FnSig];
  types: Vec[TypeDecl];
  variants: Vec[Variant];
  aliases: Vec[Local];
  globals: Vec[Local];
  interfaces: Vec[Str];
  use_aliases: Vec[Local];
  loaded_modules: Vec[Str];
  module_names: Vec[Str];
  stdlib_indexed: Int;
  stdlib_index: Vec[Local];
  warnings: Vec[Diag];
  errors: Vec[Diag];
  cur_ret: Str;
  cur_ret_set: Int;
  cur_recv: Str;
  has_uses: Int;
  unsafe_depth: Int;
}

pub fn ck_new(p: Parser, src_dir: Str) -> Checker {
  return Checker{
    p: p,
    src_dir: src_dir,
    locals: Vec[Local].new(),
    nlocals: 0,
    scopes: Vec[Int].new(),
    nscopes: 0,
    functions: Vec[FnSig].new(),
    types: Vec[TypeDecl].new(),
    variants: Vec[Variant].new(),
    aliases: Vec[Local].new(),
    globals: Vec[Local].new(),
    interfaces: Vec[Str].new(),
    use_aliases: Vec[Local].new(),
    loaded_modules: Vec[Str].new(),
    module_names: Vec[Str].new(),
    stdlib_indexed: 0,
    stdlib_index: Vec[Local].new(),
    warnings: Vec[Diag].new(),
    errors: Vec[Diag].new(),
    cur_ret: "",
    cur_ret_set: 0,
    cur_recv: "",
    has_uses: 0,
    unsafe_depth: 0,
  };
}

// ============================================================================
// Scopes and locals
// ============================================================================
//
// `locals`/`scopes` are append-only backing stores; `nlocals`/`nscopes` are
// the LIVE counts and the live region is always the first N entries. Pushing
// overwrites the stale slot when one exists (a plain push would leave dead
// entries between live ones after a scope pop, and lookups scanning
// `0..nlocals` would then see the wrong names).

pub fn ck_push_scope(c: &mut Checker) {
  if c.scopes.len() > c.nscopes {
    c.scopes[c.nscopes] = c.nlocals;
  } else {
    c.scopes.push(c.nlocals);
  }
  c.nscopes = c.nscopes + 1;
}

pub fn ck_pop_scope(c: &mut Checker) {
  if c.nscopes == 0 { return; }
  c.nscopes = c.nscopes - 1;
  c.nlocals = c.scopes[c.nscopes];
}

pub fn ck_add_local(c: &mut Checker, name: Str, ty: Str) {
  if c.locals.len() > c.nlocals {
    c.locals[c.nlocals] = Local{ name: name, ty: ty };
  } else {
    c.locals.push(Local{ name: name, ty: ty });
  }
  c.nlocals = c.nlocals + 1;
}

/// "" when absent (type names are never empty).
pub fn ck_lookup_local(c: &Checker, name: Str) -> Str {
  var i = c.nlocals - 1;
  while i >= 0 {
    let l = c.locals[i];
    if l.name == name { return l.ty; }
    i = i - 1;
  }
  return "";
}

pub fn ck_lookup_global(c: &Checker, name: Str) -> Str {
  var i = c.globals.len() - 1;
  while i >= 0 {
    let l = c.globals[i];
    if l.name == name { return l.ty; }
    i = i - 1;
  }
  return "";
}

// ============================================================================
// Diagnostics
// ============================================================================

pub fn ck_error_at(c: &mut Checker, message: Str, line: Int, col: Int) -> Str {
  c.errors.push(Diag{
    kind: "type_error", code: "T001", message: message, line: line, col: col,
  });
  return "<error>";
}

pub fn ck_warn_coded_at(c: &mut Checker, code: Str, message: Str, line: Int, col: Int) {
  c.warnings.push(Diag{
    kind: "warning", code: code, message: message, line: line, col: col,
  });
}

/// Rust `warn_at` (uncoded W000-class warning).
pub fn ck_warn_at(c: &mut Checker, message: Str, line: Int, col: Int) {
  c.warnings.push(Diag{
    kind: "warning", code: "W000", message: message, line: line, col: col,
  });
}

// ============================================================================
// Symbol tables
// ============================================================================

/// Last registered entry with this exact key wins (Rust `HashMap::insert`).
pub fn ck_find_fn(c: &Checker, key: Str) -> Int {
  var i = c.functions.len() - 1;
  while i >= 0 {
    if c.functions[i].key == key { return i; }
    i = i - 1;
  }
  return -1;
}

pub fn ck_has_fn(c: &Checker, key: Str) -> Bool {
  return ck_find_fn(c, key) >= 0;
}

pub fn ck_find_type(c: &Checker, name: Str) -> Int {
  var i = c.types.len() - 1;
  while i >= 0 {
    if c.types[i].name == name { return i; }
    i = i - 1;
  }
  return -1;
}

pub fn ck_find_variant(c: &Checker, key: Str) -> Int {
  var i = c.variants.len() - 1;
  while i >= 0 {
    if c.variants[i].key == key { return i; }
    i = i - 1;
  }
  return -1;
}

pub fn ck_find_alias(c: &Checker, name: Str) -> Int {
  var i = c.aliases.len() - 1;
  while i >= 0 {
    if c.aliases[i].name == name { return i; }
    i = i - 1;
  }
  return -1;
}

/// "" when the type or field is unknown.
pub fn ck_field_ty(c: &Checker, type_name: Str, field: Str) -> Str {
  let ti = ck_find_type(c, type_name);
  if ti < 0 { return ""; }
  let fields = c.types[ti].fields;
  var i = fields.len() - 1;
  while i >= 0 {
    if fields[i].name == field { return fields[i].ty; }
    i = i - 1;
  }
  return "";
}

pub fn ck_is_interface(c: &Checker, name: Str) -> Bool {
  var i = c.interfaces.len() - 1;
  while i >= 0 {
    if c.interfaces[i] == name { return true; }
    i = i - 1;
  }
  return false;
}

/// Rust `resolve_alias`: follow `type Foo = Bar;` chains (cycle guard 16).
pub fn ck_resolve_alias(c: &Checker, ty: Str) -> Str {
  var current = ty;
  var depth = 0;
  while depth <= 16 {
    let ai = ck_find_alias(c, current);
    if ai < 0 { break; }
    current = c.aliases[ai].ty;
    depth = depth + 1;
  }
  return current;
}

// ============================================================================
// Import aliases / loaded modules (Phase 3 catalog sub-stage)
// ============================================================================

/// First binding wins (Rust module-alias semantics).
pub fn ck_add_alias(c: &mut Checker, name: Str, module_key: Str) {
  var i = 0;
  while i < c.use_aliases.len() {
    if c.use_aliases[i].name == name { return; }
    i = i + 1;
  }
  c.use_aliases.push(Local{ name: name, ty: module_key });
}

pub fn ck_alias_key(c: &Checker, name: Str) -> Str {
  var i = c.use_aliases.len() - 1;
  while i >= 0 {
    if c.use_aliases[i].name == name { return c.use_aliases[i].ty; }
    i = i - 1;
  }
  return "";
}

pub fn ck_is_loaded(c: &Checker, key: Str) -> Bool {
  var i = c.loaded_modules.len() - 1;
  while i >= 0 {
    if c.loaded_modules[i] == key { return true; }
    i = i - 1;
  }
  return false;
}

pub fn ck_mark_loaded(c: &mut Checker, key: Str) {
  if ck_is_loaded(c, key) { return; }
  c.loaded_modules.push(key);
}

/// Bare-name resolution through loaded module exports (Rust registers a bare
/// fallback for every catalog/imported item): "fn" for functions, the type
/// name for types, the declared type for consts, "" when absent.
pub fn ck_loaded_bare(c: &Checker, name: Str) -> Str {
  var i = 0;
  while i < c.loaded_modules.len() {
    let key = c.loaded_modules[i] + "." + name;
    if ck_find_fn(c, key) >= 0 { return "fn"; }
    if ck_find_type(c, key) >= 0 { return name; }
    let g = ck_lookup_global(c, key);
    if g.len() > 0 { return g; }
    i = i + 1;
  }
  return "";
}

/// First loaded module exporting `name` as a function; -1 when none.
pub fn ck_loaded_fn(c: &Checker, name: Str) -> Int {
  var i = 0;
  while i < c.loaded_modules.len() {
    let fi = ck_find_fn(c, c.loaded_modules[i] + "." + name);
    if fi >= 0 { return fi; }
    i = i + 1;
  }
  return -1;
}

/// In-program module names (`module pipeline { ... }`) resolve as qualified
/// receivers like catalog modules.
pub fn ck_add_module_name(c: &mut Checker, name: Str) {
  if name.len() == 0 { return; }
  var i = 0;
  while i < c.module_names.len() {
    if c.module_names[i] == name { return; }
    i = i + 1;
  }
  c.module_names.push(name);
}

pub fn ck_is_module_name(c: &Checker, name: Str) -> Bool {
  var i = c.module_names.len() - 1;
  while i >= 0 {
    if c.module_names[i] == name { return true; }
    i = i - 1;
  }
  return false;
}

pub fn ck_index_lookup(c: &Checker, dotted: Str) -> Str {
  var i = c.stdlib_index.len() - 1;
  while i >= 0 {
    if c.stdlib_index[i].name == dotted { return c.stdlib_index[i].ty; }
    i = i - 1;
  }
  return "";
}

pub fn ck_index_add(c: &mut Checker, dotted: Str, path: Str) {
  if dotted.len() == 0 || path.len() == 0 { return; }
  var i = 0;
  while i < c.stdlib_index.len() {
    if c.stdlib_index[i].name == dotted { return; }
    i = i + 1;
  }
  c.stdlib_index.push(Local{ name: dotted, ty: path });
}

// ============================================================================
// AST helpers
// ============================================================================

pub fn ck_span_of(c: &Checker, idx: Int) -> Span {
  if idx < 0 { return Span{ line: 0, col: 0, byte_start: 0, byte_end: 0 }; }
  return c.p.nodes[idx].span;
}

pub fn ck_ident(c: &Checker, idx: Int) -> Str {
  return ck_ident_in(&c.p, idx);
}

pub fn ck_ident_in(p: &Parser, idx: Int) -> Str {
  if idx < 0 { return ""; }
  let node = p.nodes[idx];
  match node.kind {
    NkIdent(name) => { return name; }
    _ => { return ""; }
  }
}

/// Rust `Type::from_ast_type` for the arena Type nodes.
pub fn ck_type_from_ast(c: &Checker, idx: Int) -> Str {
  return ck_type_from_ast_in(&c.p, idx);
}

pub fn ck_type_from_ast_in(p: &Parser, idx: Int) -> Str {
  if idx < 0 { return "()"; }
  let node = p.nodes[idx];
  match node.kind {
    NkTyNamed(name, args) => { return selfhost_check_types.ct_from_str(ck_ident_in(p, name)); }
    NkTyRef(inner) => { return ck_type_from_ast_in(p, inner); }
    NkTyMutRef(inner) => { return ck_type_from_ast_in(p, inner); }
    NkTyOption(inner) => {
      return "Option[" + ck_type_from_ast_in(p, inner) + "]";
    }
    NkTyResult(ok, err) => {
      return "Result[" + ck_type_from_ast_in(p, ok) + ", " + ck_type_from_ast_in(p, err) + "]";
    }
    NkTyVec(inner) => {
      return "Vec[" + ck_type_from_ast_in(p, inner) + "]";
    }
    NkTySlice(inner) => {
      return "Slice[" + ck_type_from_ast_in(p, inner) + "]";
    }
    NkTyMap(k, v) => {
      return "Map[" + ck_type_from_ast_in(p, k) + ", " + ck_type_from_ast_in(p, v) + "]";
    }
    NkTySet(inner) => {
      return "Set[" + ck_type_from_ast_in(p, inner) + "]";
    }
    NkTyTuple(items) => {
      var parts = Vec[Str].new();
      var i = 0;
      while i < items.len() {
        parts.push(ck_type_from_ast_in(p, items[i]));
        i = i + 1;
      }
      return "Tuple__" + selfhost_check_types.ct_join(parts, "__");
    }
    NkTyPtr(inner) => {
      return "*" + ck_type_from_ast_in(p, inner);
    }
    NkTyArray(size_expr, inner) => {
      return "Array[" + ck_type_from_ast_in(p, inner) + "]";
    }
    NkTyFn(params, ret) => {
      var parts = Vec[Str].new();
      var i = 0;
      while i < params.len() {
        parts.push(ck_type_from_ast_in(p, params[i]));
        i = i + 1;
      }
      var r = "()";
      if ret >= 0 { r = ck_type_from_ast_in(p, ret); }
      return "fn(" + selfhost_check_types.ct_join(parts, ", ") + ") -> " + r;
    }
    NkTyImplTrait(bounds) => {
      var parts = Vec[Str].new();
      var i = 0;
      while i < bounds.len() {
        parts.push(ck_ident_in(p, bounds[i]));
        i = i + 1;
      }
      return "impl " + selfhost_check_types.ct_join(parts, " + ");
    }
    NkTyAnonStruct(fields) => {
      var parts = Vec[Str].new();
      var i = 0;
      while i < fields.len() {
        let fnode = p.nodes[fields[i]];
        match fnode.kind {
          NkField(name, ty) => {
            parts.push(ck_ident_in(p, name) + "_" + ck_type_from_ast_in(p, ty));
          }
          _ => {}
        }
        i = i + 1;
      }
      return "_Anon__" + selfhost_check_types.ct_join(parts, "__");
    }
    NkTyNever => { return "!"; }
    _ => { return "()"; }
  }
}

// ============================================================================
// Structural compatibility (port of Checker::types_compatible)
// ============================================================================

fn ck_arg_compatible(x: Str, y: Str) -> Bool {
  let xc = selfhost_check_types.ct_is_container(x);
  let yc = selfhost_check_types.ct_is_container(y);
  if xc || yc {
    if xc && yc {
      if selfhost_check_types.ct_base(x) != selfhost_check_types.ct_base(y) { return false; }
      let xa = selfhost_check_types.ct_args(x);
      let ya = selfhost_check_types.ct_args(y);
      if xa.len() != ya.len() { return false; }
      var i = 0;
      while i < xa.len() {
        if !ck_arg_compatible(xa[i], ya[i]) { return false; }
        i = i + 1;
      }
      return true;
    }
    return true;
  }
  if selfhost_check_types.ct_is_wildcard(x) || selfhost_check_types.ct_is_wildcard(y) { return true; }
  if selfhost_check_types.ct_is_generic_param(x) || selfhost_check_types.ct_is_generic_param(y) { return true; }
  if x == "" || y == "" { return true; }
  if selfhost_check_types.ct_is_unit(x) || selfhost_check_types.ct_is_unit(y) { return true; }
  if selfhost_check_types.ct_is_tuple_like(x) || selfhost_check_types.ct_is_tuple_like(y) { return true; }
  if selfhost_check_types.ct_is_pointer_like(x) && selfhost_check_types.ct_is_pointer_like(y) { return true; }
  let cx = selfhost_check_types.ct_from_str(selfhost_check_types.ct_scalar_name(x));
  let cy = selfhost_check_types.ct_from_str(selfhost_check_types.ct_scalar_name(y));
  if cx == cy { return true; }
  let cb = (selfhost_check_types.ct_is_numeric(cx) || cx == "Bool")
    && (selfhost_check_types.ct_is_numeric(cy) || cy == "Bool");
  return cb;
}

fn ck_arg_list_compatible(a: Vec[Str], b: Vec[Str]) -> Bool {
  if a.len() != b.len() { return false; }
  var i = 0;
  while i < a.len() {
    if !ck_arg_compatible(a[i], b[i]) { return false; }
    i = i + 1;
  }
  return true;
}

pub fn ck_types_compatible(c: &Checker, found_in: Str, expected_in: Str) -> Bool {
  let found = ck_resolve_alias(c, found_in);
  let expected = ck_resolve_alias(c, expected_in);
  if string.str_starts_with(found, "impl ") || string.str_starts_with(expected, "impl ") {
    return true;
  }
  // v0.56: Str is *UInt8 at the C FFI boundary.
  let s_ptr = (found == "Str" && (selfhost_check_types.ct_is_pointer_like(expected)))
    || (expected == "Str" && (selfhost_check_types.ct_is_pointer_like(found)));
  if s_ptr { return true; }
  if selfhost_check_types.ct_is_wildcard(found) || selfhost_check_types.ct_is_wildcard(expected) {
    return true;
  }
  // BUG 51 container erasure (Rust 8720-8733): when both sides are named
  // types with the SAME base, bare-vs-parameterized stays compatible (one
  // side erased its args); two parameterized forms must agree element-wise.
  if selfhost_check_types.ct_base(found) == selfhost_check_types.ct_base(expected)
    && (found != "" && expected != "")
  {
    let fa = selfhost_check_types.ct_args(found);
    let ea = selfhost_check_types.ct_args(expected);
    if fa.len() > 0 && ea.len() > 0 && !ck_arg_list_compatible(fa, ea) {
      return false;
    }
    return true;
  }
  if found == "<error>" || expected == "<error>" { return true; }
  if found == expected { return true; }
  if selfhost_check_types.ct_is_pointer_like(expected)
    && (selfhost_check_types.ct_is_numeric(found) || found == "Bool")
  {
    return true;
  }
  if selfhost_check_types.ct_is_generic_param(found) || selfhost_check_types.ct_is_generic_param(expected) {
    return true;
  }
  if selfhost_check_types.ct_is_pointer_like(found) && selfhost_check_types.ct_is_pointer_like(expected) {
    return true;
  }
  if selfhost_check_types.ct_is_tuple_like(found) || selfhost_check_types.ct_is_tuple_like(expected) {
    return true;
  }
  if found == "Int" && selfhost_check_types.ct_is_numeric(expected) { return true; }
  if expected == "Int" && selfhost_check_types.ct_is_numeric(found) { return true; }
  if found == "Self" || expected == "Self" { return true; }
  if ck_is_interface(c, found) || ck_is_interface(c, expected) { return true; }
  if ck_arithmetic_container_mix(found, expected) { return true; }
  if found == "fn" && selfhost_check_types.ct_is_fn_type(expected) { return true; }
  if expected == "fn" && selfhost_check_types.ct_is_fn_type(found) { return true; }
  if ck_numeric_promo(found, expected) { return true; }
  if ck_uint_promo(found, expected) { return true; }
  if expected == "()" { return true; }
  if selfhost_check_types.ct_is_fn_type(found) && selfhost_check_types.ct_is_fn_type(expected) {
    return ck_fn_compatible(c, found, expected);
  }
  return false;
}

/// `fn(P, Q) -> R` decomposition (params split at bracket/paren depth 0).
pub fn ck_fn_params(name: Str) -> Vec[Str] {
  var out = Vec[Str].new();
  let open = ct_position(name, 40);
  let arrow = ct_find_arrow(name);
  if open < 0 || arrow < 0 { return out; }
  let inner = string.str_slice(name, open + 1, arrow);
  var depth = 0;
  var start = 0;
  var i = 0;
  while i < inner.len() {
    let b = string.byte_at(inner, i) as Int;
    if b == 91 || b == 40 { depth = depth + 1; }
    if b == 93 || b == 41 { depth = depth - 1; }
    if b == 44 && depth == 0 {
      let arg = selfhost_check_types.ct_trim(string.str_slice(inner, start, i));
      if arg.len() > 0 { out.push(selfhost_check_types.ct_canonical(arg)); }
      start = i + 1;
    }
    i = i + 1;
  }
  let arg = selfhost_check_types.ct_trim(string.str_slice(inner, start, inner.len()));
  if arg.len() > 0 { out.push(selfhost_check_types.ct_canonical(arg)); }
  return out;
}

pub fn ck_fn_ret(name: Str) -> Str {
  let arrow = ct_find_arrow(name);
  if arrow < 0 { return "()"; }
  return selfhost_check_types.ct_canonical(string.str_slice(name, arrow + 2, name.len()));
}

fn ct_find_arrow(name: Str) -> Int {
  var i = 0;
  while i + 1 < name.len() {
    if (string.byte_at(name, i) as Int) == 45 && (string.byte_at(name, i + 1) as Int) == 62 {
      return i;
    }
    i = i + 1;
  }
  return -1;
}

fn ct_position(name: Str, ch: Int) -> Int {
  var i = 0;
  while i < name.len() {
    if (string.byte_at(name, i) as Int) == ch { return i; }
    i = i + 1;
  }
  return -1;
}

fn ck_fn_compatible(c: &Checker, found: Str, expected: Str) -> Bool {
  let fp = ck_fn_params(found);
  let ep = ck_fn_params(expected);
  if fp.len() != ep.len() { return false; }
  var i = 0;
  while i < fp.len() {
    if !ck_types_compatible(c, fp[i], ep[i]) { return false; }
    i = i + 1;
  }
  return ck_types_compatible(c, ck_fn_ret(found), ck_fn_ret(expected));
}

/// Unsigned-family promotions from the Rust matrix.
fn ck_uint_promo(found: Str, expected: Str) -> Bool {
  if found == "Int" && expected == "UInt32" { return true; }
  if found == "UInt32" && expected == "Int" { return true; }
  if found == "Int32" && expected == "UInt32" { return true; }
  if found == "UInt32" && expected == "Int32" { return true; }
  if found == "Int" && expected == "UInt" { return true; }
  if found == "UInt" && expected == "Int" { return true; }
  return ck_uint_family(found, expected);
}

fn ck_uint_family(found: Str, expected: Str) -> Bool {
  let f = ck_uint_idx(found);
  let e = ck_uint_idx(expected);
  if f < 0 || e < 0 { return false; }
  return true;
}

fn ck_uint_idx(name: Str) -> Int {
  if name == "UInt" { return 0; }
  if name == "UInt32" { return 1; }
  if name == "UInt16" { return 2; }
  if name == "UInt8" { return 3; }
  return -1;
}

/// Array/Slice/Vec share a runtime layout (different spellings stay legal).
fn ck_arithmetic_container_mix(a: Str, b: Str) -> Bool {
  if a == b { return false; }
  let ab = selfhost_check_types.ct_base(a);
  let bb = selfhost_check_types.ct_base(b);
  let a_ok = string.str_starts_with(ab, "Array")
    || string.str_starts_with(ab, "Slice") || string.str_starts_with(ab, "Vec");
  let b_ok = string.str_starts_with(bb, "Array")
    || string.str_starts_with(bb, "Slice") || string.str_starts_with(bb, "Vec");
  if a_ok && b_ok { return true; }
  return false;
}

fn ck_numeric_promo(found: Str, expected: Str) -> Bool {
  if found == "Int" && expected == "Float64" { return true; }
  if found == "Float64" && expected == "Int" { return true; }
  if found == "Float32" && expected == "Float64" { return true; }
  if found == "Float64" && expected == "Float32" { return true; }
  if found == "Int" && expected == "Char" { return true; }
  if found == "Char" && expected == "Int" { return true; }
  let fi = ck_int_pair(found);
  let ei = ck_int_pair(expected);
  if fi >= 0 && ei >= 0 { return true; }
  return false;
}

/// Signed integer width family index (-1 = not in the family). The Rust
/// matrix pairs every signed width with every other and Int with UInt32/UInt.
fn ck_int_pair(name: Str) -> Int {
  if name == "Int" { return 0; }
  if name == "Int32" { return 1; }
  if name == "Int16" { return 2; }
  if name == "Int8" { return 3; }
  return -1;
}
