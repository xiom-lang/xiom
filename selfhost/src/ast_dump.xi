// XIOM -- Selfhost canonical AST dump (Phase 2 parity gate)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Byte-stable mirror of `crates/xiom/src/main.rs::dump_ast` (AstDump). The
// arena (selfhost_ast) resolves indices back into the same tree, so the
// emitted lines match the Rust walker exactly:
//
//   {indent}{Kind}[ key=value]... [span=l:c:bs:be]
//
// * indent = two spaces per depth level.
// * text payloads lowercase hex; ints zero-padded hex where the Rust format
//   pads (Int: 16; BigInt: 32 = hi then lo; Char: unpadded).
// * Float literals dump the token LEXEME bytes (NkLitFloat payload), which
//   equals the Rust dump's source slice at the node byte range for every
//   float token the lexer produces; float VALUE parity stays deferred.
// * spans are `line:col:byte_start:byte_end` (POST-BOM).
//
// The CRLF the XIOM CRT printf emits on Windows pipes is normalized by the
// harness (`lines_of`), exactly like `--dump-tokens`.

module selfhost_ast_dump

use xiom.io;

use selfhost_ast;
use selfhost_ast.Node;
use selfhost_ast.NodeKind;
use selfhost_ast.Span;

// ============================================================================
// Byte-buffer emission helpers
// ============================================================================

fn ad_nibble(n: Int) -> UInt8 {
  if n < 10 { return (48 + n) as UInt8; }
  return (87 + n) as UInt8;
}

fn ad_byte(buf: &mut Vec[UInt8], b: Int) {
  buf.push(b as UInt8);
}

fn ad_str(buf: &mut Vec[UInt8], s: Str) {
  var i = 0;
  let n = s.len();
  while i < n {
    buf.push(string_byte(s, i));
    i = i + 1;
  }
}

/// xiom.string.byte_at wrapper kept local so the dump module has a single
/// import surface.
fn string_byte(s: Str, i: Int) -> UInt8 {
  return s.byte_at(i);
}

fn ad_dec(buf: &mut Vec[UInt8], v: Int) {
  if v <= 0 {
    buf.push(48 as UInt8);
    return;
  }
  var digits: [24]UInt8;
  var pos = 24;
  var n = v;
  while n > 0 {
    pos = pos - 1;
    digits[pos] = ad_nibble(n % 10);
    n = n / 10;
  }
  while pos < 24 {
    buf.push(digits[pos]);
    pos = pos + 1;
  }
}

/// Lowercase hex of a Str's bytes.
fn ad_hex_str(buf: &mut Vec[UInt8], s: Str) {
  var i = 0;
  let n = s.len();
  while i < n {
    let b = string_byte(s, i) as Int;
    buf.push(ad_nibble(b >> 4));
    buf.push(ad_nibble(b & 15));
    i = i + 1;
  }
}

fn ad_hex_vec(buf: &mut Vec[UInt8], bytes: Vec[UInt8]) {
  var i = 0;
  while i < bytes.len() {
    let b = bytes[i] as Int;
    buf.push(ad_nibble(b >> 4));
    buf.push(ad_nibble(b & 15));
    i = i + 1;
  }
}

/// Fixed 16-digit lowercase hex (Rust `{:016x}`).
fn ad_u64_hex(buf: &mut Vec[UInt8], v: UInt) {
  var k = 15;
  while k >= 0 {
    let n = ((v >> (4 * k)) & (15 as UInt)) as Int;
    buf.push(ad_nibble(n));
    k = k - 1;
  }
}

/// Unpadded lowercase hex (Rust `{:x}`).
fn ad_hex_int(buf: &mut Vec[UInt8], v: Int) {
  if v <= 0 {
    buf.push(48 as UInt8);
    return;
  }
  var tmp: [16]UInt8;
  var pos = 16;
  var x = v;
  while x > 0 {
    pos = pos - 1;
    tmp[pos] = ad_nibble(x & 15);
    x = x >> 4;
  }
  while pos < 16 {
    buf.push(tmp[pos]);
    pos = pos + 1;
  }
}

fn ad_indent(buf: &mut Vec[UInt8], depth: Int) {
  var i = 0;
  while i < depth {
    buf.push(32 as UInt8);
    buf.push(32 as UInt8);
    i = i + 1;
  }
}

fn ad_nl(buf: &mut Vec[UInt8]) {
  buf.push(10 as UInt8);
}

fn ad_span(buf: &mut Vec[UInt8], s: Span) {
  ad_str(buf, "span=");
  ad_dec(buf, s.line);
  ad_byte(buf, 58);
  ad_dec(buf, s.col);
  ad_byte(buf, 58);
  ad_dec(buf, s.byte_start);
  ad_byte(buf, 58);
  ad_dec(buf, s.byte_end);
}

fn ad_kv_span(buf: &mut Vec[UInt8], s: Span) {
  ad_byte(buf, 32);
  ad_span(buf, s);
}

// ============================================================================
// Small helpers over the arena
// ============================================================================

fn ad_ident_name(n: &Vec[Node], idx: Int) -> Str {
  match n[idx].kind {
    NkIdent(name) => { return name; }
    _ => { return ""; }
  }
}

fn ad_ident_line(buf: &mut Vec[UInt8], n: &Vec[Node], idx: Int, depth: Int, label: Str) {
  ad_indent(buf, depth);
  ad_str(buf, label);
  ad_str(buf, " name=");
  ad_hex_str(buf, ad_ident_name(n, idx));
  ad_kv_span(buf, n[idx].span);
  ad_nl(buf);
}

/// Hex path segments joined with '.'; empty path emits '-'.
fn ad_path(buf: &mut Vec[UInt8], n: &Vec[Node], path: Vec[Int]) {
  if path.len() == 0 {
    ad_str(buf, "-");
    return;
  }
  var i = 0;
  while i < path.len() {
    if i > 0 { ad_byte(buf, 46); }
    ad_hex_str(buf, ad_ident_name(n, path[i]));
    i = i + 1;
  }
}

fn ad_opt_label(buf: &mut Vec[UInt8], n: &Vec[Node], idx: Int) {
  if idx < 0 {
    ad_str(buf, "-");
    return;
  }
  ad_hex_str(buf, ad_ident_name(n, idx));
}

fn ad_derive_name(code: Int) -> Str {
  if code == 0 { return "Eq"; }
  if code == 1 { return "Clone"; }
  if code == 2 { return "Display"; }
  if code == 3 { return "Hash"; }
  if code == 4 { return "Ord"; }
  if code == 5 { return "Debug"; }
  return "?";
}

fn ad_unop_name(code: Int) -> Str {
  if code == 0 { return "Neg"; }
  if code == 1 { return "Not"; }
  if code == 2 { return "Ref"; }
  if code == 3 { return "MutRef"; }
  if code == 4 { return "BitNot"; }
  if code == 5 { return "Deref"; }
  return "?";
}

fn ad_binop_name(code: Int) -> Str {
  if code == 0 { return "Add"; }
  if code == 1 { return "Sub"; }
  if code == 2 { return "Mul"; }
  if code == 3 { return "Div"; }
  if code == 4 { return "Rem"; }
  if code == 5 { return "Eq"; }
  if code == 6 { return "Neq"; }
  if code == 7 { return "Lt"; }
  if code == 8 { return "Gt"; }
  if code == 9 { return "Le"; }
  if code == 10 { return "Ge"; }
  if code == 11 { return "Shl"; }
  if code == 12 { return "Shr"; }
  if code == 13 { return "And"; }
  if code == 14 { return "Or"; }
  if code == 15 { return "Assign"; }
  if code == 16 { return "BitXor"; }
  if code == 17 { return "BitAnd"; }
  if code == 18 { return "BitOr"; }
  return "?";
}

fn ad_derive_list(buf: &mut Vec[UInt8], n: &Vec[Node], codes: Vec[Int], depth: Int) {
  var i = 0;
  while i < codes.len() {
    ad_indent(buf, depth);
    ad_str(buf, "Derive ");
    ad_str(buf, ad_derive_name(codes[i]));
    ad_nl(buf);
    i = i + 1;
  }
}

// ============================================================================
// Forward declarations are not needed: XIOM resolves module-local functions
// regardless of definition order. The walker is one big dispatch.
// ============================================================================

fn ad_types_list(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_generics(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_fields(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_exprs(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_params(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_contracts(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_attrs(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_members(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_arms(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_elif_list(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

fn ad_kids(buf: &mut Vec[UInt8], n: &Vec[Node], items: Vec[Int], depth: Int) {
  var i = 0;
  while i < items.len() {
    ad_node(buf, n, items[i], depth);
    i = i + 1;
  }
}

// ============================================================================
// The walker
// ============================================================================

fn ad_node(buf: &mut Vec[UInt8], n: &Vec[Node], idx: Int, depth: Int) {
  let node = n[idx];
  let sp = node.span;
  match node.kind {
    // --- program / top ---
    NkProgram(items) => {
      ad_indent(buf, depth);
      ad_str(buf, "Program items=");
      ad_dec(buf, items.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_types_list(buf, n, items, depth + 1);
    }
    NkModule(name, path, items, file_level, has_source, source) => {
      ad_indent(buf, depth);
      ad_str(buf, "Module name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " path=");
      ad_path(buf, n, path);
      ad_str(buf, " filelevel=");
      ad_dec(buf, file_level);
      ad_str(buf, " source=");
      if has_source != 0 { ad_hex_str(buf, source); } else { ad_str(buf, "-"); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_types_list(buf, n, items, depth + 1);
    }
    NkUse(path, glob, alias) => {
      ad_indent(buf, depth);
      ad_str(buf, "Use path=");
      ad_path(buf, n, path);
      ad_str(buf, " glob=");
      ad_dec(buf, glob);
      ad_str(buf, " alias=");
      ad_opt_label(buf, n, alias);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkTypeDecl(is_pub, name, generics, fields, derived, invariants, derives, alias) => {
      ad_indent(buf, depth);
      ad_str(buf, "Type pub=");
      ad_dec(buf, is_pub);
      ad_str(buf, " name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " generic=");
      ad_dec(buf, generics.len());
      ad_str(buf, " fields=");
      ad_dec(buf, fields.len());
      ad_str(buf, " derived=");
      ad_dec(buf, derived.len());
      ad_str(buf, " invariants=");
      ad_dec(buf, invariants.len());
      ad_str(buf, " derives=");
      ad_dec(buf, derives.len());
      ad_str(buf, " alias=");
      if alias >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_generics(buf, n, generics, depth + 1);
      ad_fields(buf, n, fields, depth + 1);
      ad_kids(buf, n, derived, depth + 1);
      ad_exprs(buf, n, invariants, depth + 1);
      ad_derive_list(buf, n, derives, depth + 1);
      if alias >= 0 { ad_node(buf, n, alias, depth + 1); }
    }
    NkDerivedField(name, ty, expr) => {
      ad_indent(buf, depth);
      ad_str(buf, "Derived name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, ty, depth + 1);
      ad_node(buf, n, expr, depth + 1);
    }
    NkEnumDecl(is_pub, name, generics, variants, derives) => {
      ad_indent(buf, depth);
      ad_str(buf, "Enum pub=");
      ad_dec(buf, is_pub);
      ad_str(buf, " name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " generic=");
      ad_dec(buf, generics.len());
      ad_str(buf, " variants=");
      ad_dec(buf, variants.len());
      ad_str(buf, " derives=");
      ad_dec(buf, derives.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_generics(buf, n, generics, depth + 1);
      ad_kids(buf, n, variants, depth + 1);
      ad_derive_list(buf, n, derives, depth + 1);
    }
    NkVariant(name, fields) => {
      ad_indent(buf, depth);
      ad_str(buf, "Variant name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " fields=");
      ad_dec(buf, fields.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_fields(buf, n, fields, depth + 1);
    }
    NkInterface(is_pub, name, generics, parent, members) => {
      ad_indent(buf, depth);
      ad_str(buf, "Interface pub=");
      ad_dec(buf, is_pub);
      ad_str(buf, " name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " generic=");
      ad_dec(buf, generics.len());
      ad_str(buf, " parent=");
      ad_opt_label(buf, n, parent);
      ad_str(buf, " members=");
      ad_dec(buf, members.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_generics(buf, n, generics, depth + 1);
      ad_members(buf, n, members, depth + 1);
    }
    NkImpl(trait_name, trait_args, type_name, members) => {
      ad_indent(buf, depth);
      ad_str(buf, "Impl trait=");
      ad_hex_str(buf, ad_ident_name(n, trait_name));
      ad_str(buf, " traitargs=");
      ad_dec(buf, trait_args.len());
      ad_str(buf, " type=");
      ad_hex_str(buf, ad_ident_name(n, type_name));
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_types_list(buf, n, trait_args, depth + 1);
      ad_members(buf, n, members, depth + 1);
    }
    NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
      ad_indent(buf, depth);
      ad_str(buf, "Fn pub=");
      ad_dec(buf, is_pub);
      ad_str(buf, " async=");
      ad_dec(buf, is_async);
      ad_str(buf, " recv=");
      ad_opt_label(buf, n, recv);
      ad_str(buf, " name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " generics=");
      ad_dec(buf, generics.len());
      ad_str(buf, " params=");
      ad_dec(buf, params.len());
      ad_str(buf, " ret=");
      if ret >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_str(buf, " contracts=");
      ad_dec(buf, contracts.len());
      ad_str(buf, " body=");
      if body >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_str(buf, " attrs=");
      ad_dec(buf, attrs.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_attrs(buf, n, attrs, depth + 1);
      ad_generics(buf, n, generics, depth + 1);
      ad_params(buf, n, params, depth + 1);
      if ret >= 0 { ad_node(buf, n, ret, depth + 1); }
      ad_contracts(buf, n, contracts, depth + 1);
      if body >= 0 { ad_node(buf, n, body, depth + 1); }
    }
    NkConst(is_pub, is_mut, name, ty, value) => {
      ad_indent(buf, depth);
      ad_str(buf, "Const pub=");
      ad_dec(buf, is_pub);
      ad_str(buf, " mut=");
      ad_dec(buf, is_mut);
      ad_str(buf, " name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, ty, depth + 1);
      ad_node(buf, n, value, depth + 1);
    }
    NkExtern(linkage, fns) => {
      ad_indent(buf, depth);
      ad_str(buf, "Extern linkage=");
      ad_hex_str(buf, linkage);
      ad_str(buf, " fns=");
      ad_dec(buf, fns.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_kids(buf, n, fns, depth + 1);
    }
    NkSpawnTop(move_, block) => {
      ad_indent(buf, depth);
      ad_str(buf, "Spawn move=");
      ad_dec(buf, move_);
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, block, depth + 1);
    }
    NkField(name, ty) => {
      ad_indent(buf, depth);
      ad_str(buf, "Field name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, ty, depth + 1);
    }
    NkIdent(name) => {
      ad_indent(buf, depth);
      ad_str(buf, "Ident name=");
      ad_hex_str(buf, name);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkParam(name, ty, mutself, refself) => {
      ad_indent(buf, depth);
      ad_str(buf, "Param name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " mutself=");
      ad_dec(buf, mutself);
      ad_str(buf, " refself=");
      ad_dec(buf, refself);
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, ty, depth + 1);
    }
    NkGeneric(name, bounds, is_const, const_ty) => {
      ad_indent(buf, depth);
      ad_str(buf, "Generic name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " bounds=");
      ad_dec(buf, bounds.len());
      ad_str(buf, " const=");
      ad_dec(buf, is_const);
      ad_str(buf, " constty=");
      if const_ty >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_nl(buf);
      var i = 0;
      while i < bounds.len() {
        ad_ident_line(buf, n, bounds[i], depth + 1, "Bound");
        i = i + 1;
      }
      if const_ty >= 0 { ad_node(buf, n, const_ty, depth + 1); }
    }
    NkAttr(name, args) => {
      ad_indent(buf, depth);
      ad_str(buf, "Attr name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " args=");
      ad_dec(buf, args.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      var i = 0;
      while i < args.len() {
        ad_node(buf, n, args[i], depth + 1);
        i = i + 1;
      }
    }
    NkAttrArg(key, value) => {
      ad_indent(buf, depth);
      ad_str(buf, "Arg key=");
      ad_hex_str(buf, key);
      ad_str(buf, " value=");
      ad_hex_str(buf, value);
      ad_nl(buf);
    }
    NkRequires(expr) => {
      ad_indent(buf, depth);
      ad_str(buf, "Requires");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, expr, depth + 1);
    }
    NkEnsures(expr) => {
      ad_indent(buf, depth);
      ad_str(buf, "Ensures");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, expr, depth + 1);
    }
    _ => {
      ad_node_node(buf, n, idx, depth);
    }
  }
}

/// Second dispatch level (kept separate so no single match arm list is
/// enormous; continues the same exhaustive walker).
fn ad_node_node(buf: &mut Vec[UInt8], n: &Vec[Node], idx: Int, depth: Int) {
  let node = n[idx];
  let sp = node.span;
  match node.kind {
    // --- types ---
    NkTyNamed(name, args) => {
      ad_indent(buf, depth);
      ad_str(buf, "Named name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " span=");
      ad_dec(buf, n[name].span.line);
      ad_byte(buf, 58);
      ad_dec(buf, n[name].span.col);
      ad_byte(buf, 58);
      ad_dec(buf, n[name].span.byte_start);
      ad_byte(buf, 58);
      ad_dec(buf, n[name].span.byte_end);
      ad_str(buf, " args=");
      ad_dec(buf, args.len());
      ad_nl(buf);
      ad_types_list(buf, n, args, depth + 1);
    }
    NkTyRef(inner) => {
      ad_line1(buf, depth, "Ref");
      ad_node(buf, n, inner, depth + 1);
    }
    NkTyMutRef(inner) => {
      ad_line1(buf, depth, "MutRef");
      ad_node(buf, n, inner, depth + 1);
    }
    NkTyOption(inner) => {
      ad_line1(buf, depth, "Opt");
      ad_node(buf, n, inner, depth + 1);
    }
    NkTyResult(ok, err) => {
      ad_line1(buf, depth, "Res");
      ad_node(buf, n, ok, depth + 1);
      ad_node(buf, n, err, depth + 1);
    }
    NkTyVec(inner) => {
      ad_line1(buf, depth, "Vec");
      ad_node(buf, n, inner, depth + 1);
    }
    NkTySlice(inner) => {
      ad_line1(buf, depth, "SliceTy");
      ad_node(buf, n, inner, depth + 1);
    }
    NkTyMap(key, value) => {
      ad_line1(buf, depth, "MapTy");
      ad_node(buf, n, key, depth + 1);
      ad_node(buf, n, value, depth + 1);
    }
    NkTySet(inner) => {
      ad_line1(buf, depth, "SetTy");
      ad_node(buf, n, inner, depth + 1);
    }
    NkTyTuple(items) => {
      ad_indent(buf, depth);
      ad_str(buf, "TupleTy n=");
      ad_dec(buf, items.len());
      ad_nl(buf);
      ad_types_list(buf, n, items, depth + 1);
    }
    NkTyPtr(inner) => {
      ad_line1(buf, depth, "Ptr");
      ad_node(buf, n, inner, depth + 1);
    }
    NkTyArray(size_expr, inner) => {
      ad_line1(buf, depth, "ArrayTy");
      ad_node(buf, n, size_expr, depth + 1);
      ad_node(buf, n, inner, depth + 1);
    }
    NkTyFn(params, ret) => {
      ad_indent(buf, depth);
      ad_str(buf, "FnTy params=");
      ad_dec(buf, params.len());
      ad_nl(buf);
      ad_types_list(buf, n, params, depth + 1);
      ad_node(buf, n, ret, depth + 1);
    }
    NkTyImplTrait(bounds) => {
      ad_indent(buf, depth);
      ad_str(buf, "ImplTrait n=");
      ad_dec(buf, bounds.len());
      ad_nl(buf);
      var i = 0;
      while i < bounds.len() {
        ad_ident_line(buf, n, bounds[i], depth + 1, "Bound");
        i = i + 1;
      }
    }
    NkTyAnonStruct(fields) => {
      ad_indent(buf, depth);
      ad_str(buf, "AnonStruct fields=");
      ad_dec(buf, fields.len());
      ad_nl(buf);
      ad_fields(buf, n, fields, depth + 1);
    }
    NkTyNever => {
      ad_line1(buf, depth, "Never");
    }

    // --- literals ---
    NkLitInt(value) => {
      ad_indent(buf, depth);
      ad_str(buf, "LitInt value=");
      ad_u64_hex(buf, value);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkLitBigInt(hi, lo) => {
      ad_indent(buf, depth);
      ad_str(buf, "LitBigInt value=");
      ad_u64_hex(buf, hi);
      ad_u64_hex(buf, lo);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkLitFloat(lex) => {
      ad_indent(buf, depth);
      ad_str(buf, "LitFloat lex=");
      ad_hex_str(buf, lex);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkLitStr(data) => {
      ad_indent(buf, depth);
      ad_str(buf, "LitStr data=");
      ad_hex_vec(buf, data);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkLitChar(cp) => {
      ad_indent(buf, depth);
      ad_str(buf, "LitChar cp=");
      ad_hex_int(buf, cp);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkLitBool(value) => {
      ad_indent(buf, depth);
      ad_str(buf, "LitBool value=");
      ad_dec(buf, value);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }

    _ => {
      ad_node_exprs(buf, n, idx, depth);
    }
  }
}

/// Third dispatch level: expressions, statements, blocks, arms, patterns.
fn ad_node_exprs(buf: &mut Vec[UInt8], n: &Vec[Node], idx: Int, depth: Int) {
  let node = n[idx];
  let sp = node.span;
  match node.kind {
    NkExprIdent(name) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprIdent name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkExprParen(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "Paren");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprUnary(op, inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "Unary op=");
      ad_str(buf, ad_unop_name(op));
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprBinary(l, op, r) => {
      ad_indent(buf, depth);
      ad_str(buf, "Binary op=");
      ad_str(buf, ad_binop_name(op));
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, l, depth + 1);
      ad_node(buf, n, r, depth + 1);
    }
    NkExprTry(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "Try");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprImply(l, r) => {
      ad_indent(buf, depth);
      ad_str(buf, "Imply");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, l, depth + 1);
      ad_node(buf, n, r, depth + 1);
    }
    NkExprIs(inner, pattern) => {
      ad_indent(buf, depth);
      ad_str(buf, "Is");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
      ad_node(buf, n, pattern, depth + 1);
    }
    NkExprField(inner, name) => {
      ad_indent(buf, depth);
      ad_str(buf, "Field name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprCall(callee, args) => {
      ad_indent(buf, depth);
      ad_str(buf, "Call args=");
      ad_dec(buf, args.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, callee, depth + 1);
      ad_exprs(buf, n, args, depth + 1);
    }
    NkExprGenericCall(callee, types, args) => {
      ad_indent(buf, depth);
      ad_str(buf, "GenericCall types=");
      ad_dec(buf, types.len());
      ad_str(buf, " args=");
      ad_dec(buf, args.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, callee, depth + 1);
      ad_types_list(buf, n, types, depth + 1);
      ad_exprs(buf, n, args, depth + 1);
    }
    NkExprIndex(base, index) => {
      ad_indent(buf, depth);
      ad_str(buf, "Index");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, base, depth + 1);
      ad_node(buf, n, index, depth + 1);
    }
    NkExprAtPre(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "AtPre");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprRef(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprRef");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprMutRef(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprMutRef");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprSome(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprSome");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprNone => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprNone");
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkExprOk(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprOk");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprErr(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprErr");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprStruct(name, fields, base) => {
      ad_indent(buf, depth);
      ad_str(buf, "StructLit name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " fields=");
      ad_dec(buf, fields.len());
      ad_str(buf, " base=");
      if base >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      var i = 0;
      while i < fields.len() {
        ad_node(buf, n, fields[i], depth + 1);
        i = i + 1;
      }
      if base >= 0 {
        ad_indent(buf, depth + 1);
        ad_str(buf, "Base");
        ad_nl(buf);
        ad_node(buf, n, base, depth + 2);
      }
    }
    NkFieldInit(name, value) => {
      ad_indent(buf, depth);
      ad_str(buf, "Init name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, n[name].span);
      ad_nl(buf);
      ad_node(buf, n, value, depth + 1);
    }
    NkExprArray(items) => {
      ad_indent(buf, depth);
      ad_str(buf, "ArrayLit n=");
      ad_dec(buf, items.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_exprs(buf, n, items, depth + 1);
    }
    NkExprBlock(block) => {
      ad_indent(buf, depth);
      ad_str(buf, "BlockExpr");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, block, depth + 1);
    }
    NkExprConstBlock(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "ConstBlock");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprClosure(params, ret, body) => {
      ad_indent(buf, depth);
      ad_str(buf, "Closure params=");
      ad_dec(buf, params.len());
      ad_str(buf, " ret=");
      if ret >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_params(buf, n, params, depth + 1);
      if ret >= 0 { ad_node(buf, n, ret, depth + 1); }
      ad_node(buf, n, body, depth + 1);
    }
    NkExprPipeClosure(names, body) => {
      ad_indent(buf, depth);
      ad_str(buf, "PipeClosure names=");
      ad_dec(buf, names.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      var i = 0;
      while i < names.len() {
        ad_ident_line(buf, n, names[i], depth + 1, "Name");
        i = i + 1;
      }
      ad_node(buf, n, body, depth + 1);
    }
    NkExprAwait(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "Await");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprComptime(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "Comptime");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkExprAs(inner, ty) => {
      ad_indent(buf, depth);
      ad_str(buf, "As");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
      ad_node(buf, n, ty, depth + 1);
    }
    NkExprTuple(items) => {
      ad_indent(buf, depth);
      ad_str(buf, "TupleLit n=");
      ad_dec(buf, items.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_exprs(buf, n, items, depth + 1);
    }
    NkExprIf(cond, then, elifs, els) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprIf elifs=");
      ad_dec(buf, elifs.len());
      ad_str(buf, " else=");
      if els >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, cond, depth + 1);
      ad_node(buf, n, then, depth + 1);
      ad_elif_list(buf, n, elifs, depth + 1);
      if els >= 0 {
        ad_indent(buf, depth + 1);
        ad_str(buf, "Else");
        ad_nl(buf);
        ad_node(buf, n, els, depth + 2);
      }
    }
    NkElif(cond, block) => {
      ad_indent(buf, depth);
      ad_str(buf, "Elif");
      ad_nl(buf);
      ad_node(buf, n, cond, depth + 1);
      ad_node(buf, n, block, depth + 1);
    }
    NkExprMatch(scrut, arms) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprMatch arms=");
      ad_dec(buf, arms.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, scrut, depth + 1);
      ad_arms(buf, n, arms, depth + 1);
    }
    NkExprUnsafe(block) => {
      ad_indent(buf, depth);
      ad_str(buf, "Unsafe");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, block, depth + 1);
    }
    NkExprError => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprError");
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }

    // --- statements ---
    NkLet(name, ty, value) => {
      ad_indent(buf, depth);
      ad_str(buf, "Let name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " ty=");
      if ty >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      if ty >= 0 { ad_node(buf, n, ty, depth + 1); }
      ad_node(buf, n, value, depth + 1);
    }
    NkVar(name, ty, value) => {
      ad_indent(buf, depth);
      ad_str(buf, "Var name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " ty=");
      if ty >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      if ty >= 0 { ad_node(buf, n, ty, depth + 1); }
      ad_node(buf, n, value, depth + 1);
    }
    NkAssign(l, r) => {
      ad_indent(buf, depth);
      ad_str(buf, "Assign");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, l, depth + 1);
      ad_node(buf, n, r, depth + 1);
    }
    NkReturn(value) => {
      ad_indent(buf, depth);
      ad_str(buf, "Return has=");
      if value >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      if value >= 0 { ad_node(buf, n, value, depth + 1); }
    }
    NkExprStmt(expr) => {
      ad_indent(buf, depth);
      ad_str(buf, "ExprStmt");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, expr, depth + 1);
    }
    NkStmtIf(cond, then, elifs, els) => {
      ad_indent(buf, depth);
      ad_str(buf, "StmtIf elifs=");
      ad_dec(buf, elifs.len());
      ad_str(buf, " else=");
      if els >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, cond, depth + 1);
      ad_node(buf, n, then, depth + 1);
      ad_elif_list(buf, n, elifs, depth + 1);
      if els >= 0 {
        ad_indent(buf, depth + 1);
        ad_str(buf, "Else");
        ad_nl(buf);
        ad_node(buf, n, els, depth + 2);
      }
    }
    NkStmtMatch(scrut, arms) => {
      ad_indent(buf, depth);
      ad_str(buf, "StmtMatch arms=");
      ad_dec(buf, arms.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, scrut, depth + 1);
      ad_arms(buf, n, arms, depth + 1);
    }
    NkWhile(cond, body, inv, label) => {
      ad_indent(buf, depth);
      ad_str(buf, "While inv=");
      if inv >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_str(buf, " label=");
      ad_opt_label(buf, n, label);
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, cond, depth + 1);
      ad_node(buf, n, body, depth + 1);
      if inv >= 0 { ad_node(buf, n, inv, depth + 1); }
    }
    NkFor(name, iter, body, label) => {
      ad_indent(buf, depth);
      ad_str(buf, "For name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " label=");
      ad_opt_label(buf, n, label);
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, iter, depth + 1);
      ad_node(buf, n, body, depth + 1);
    }
    NkStmtSpawn(move_, block) => {
      ad_indent(buf, depth);
      ad_str(buf, "StmtSpawn move=");
      ad_dec(buf, move_);
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, block, depth + 1);
    }
    NkDestructure(names, value) => {
      ad_indent(buf, depth);
      ad_str(buf, "Destructure names=");
      ad_dec(buf, names.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      var i = 0;
      while i < names.len() {
        ad_ident_line(buf, n, names[i], depth + 1, "Name");
        i = i + 1;
      }
      ad_node(buf, n, value, depth + 1);
    }
    NkBreak(label) => {
      ad_indent(buf, depth);
      ad_str(buf, "Break label=");
      ad_opt_label(buf, n, label);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkContinue(label) => {
      ad_indent(buf, depth);
      ad_str(buf, "Continue label=");
      ad_opt_label(buf, n, label);
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkAsm(template, outputs, inputs, clobbers) => {
      ad_indent(buf, depth);
      ad_str(buf, "Asm template=");
      ad_hex_str(buf, template);
      ad_str(buf, " outputs=");
      ad_dec(buf, outputs.len());
      ad_str(buf, " inputs=");
      ad_dec(buf, inputs.len());
      ad_str(buf, " clobbers=");
      ad_dec(buf, clobbers.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      var i = 0;
      while i < outputs.len() {
        ad_node(buf, n, outputs[i], depth + 1);
        i = i + 1;
      }
      i = 0;
      while i < inputs.len() {
        ad_node(buf, n, inputs[i], depth + 1);
        i = i + 1;
      }
      i = 0;
      while i < clobbers.len() {
        ad_indent(buf, depth + 1);
        ad_str(buf, "Clobber name=");
        ad_hex_str(buf, clobbers[i]);
        ad_nl(buf);
        i = i + 1;
      }
    }
    NkAsmOut(constraint, name) => {
      ad_indent(buf, depth);
      ad_str(buf, "Out constraint=");
      ad_hex_str(buf, constraint);
      ad_str(buf, " name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, n[name].span);
      ad_nl(buf);
    }
    NkAsmIn(constraint, expr) => {
      ad_indent(buf, depth);
      ad_str(buf, "In constraint=");
      ad_hex_str(buf, constraint);
      ad_nl(buf);
      ad_node(buf, n, expr, depth + 1);
    }
    NkDefer(block) => {
      ad_indent(buf, depth);
      ad_str(buf, "Defer");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, block, depth + 1);
    }
    NkAssert(cond, msg) => {
      ad_indent(buf, depth);
      ad_str(buf, "Assert msg=");
      if msg >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, cond, depth + 1);
      if msg >= 0 { ad_node(buf, n, msg, depth + 1); }
    }
    NkDebugger => {
      ad_indent(buf, depth);
      ad_str(buf, "Debugger");
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }

    // --- blocks / wrappers / arms ---
    NkBlock(stmts) => {
      ad_indent(buf, depth);
      ad_str(buf, "Block stmts=");
      ad_dec(buf, stmts.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_kids(buf, n, stmts, depth + 1);
    }
    NkStmtW(stmt) => {
      ad_indent(buf, depth);
      ad_str(buf, "Stmt");
      ad_nl(buf);
      ad_node(buf, n, stmt, depth + 1);
    }
    NkTailW(expr) => {
      ad_indent(buf, depth);
      ad_str(buf, "Tail");
      ad_nl(buf);
      ad_node(buf, n, expr, depth + 1);
    }
    NkMatchArm(pattern, guard, body, body_is_block) => {
      ad_indent(buf, depth);
      ad_str(buf, "Arm guard=");
      if guard >= 0 { ad_dec(buf, 1); } else { ad_dec(buf, 0); }
      ad_str(buf, " body=");
      if body_is_block != 0 { ad_str(buf, "block"); } else { ad_str(buf, "expr"); }
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, pattern, depth + 1);
      if guard >= 0 { ad_node(buf, n, guard, depth + 1); }
      ad_node(buf, n, body, depth + 1);
    }

    // --- patterns ---
    NkPatWildcard => {
      ad_indent(buf, depth);
      ad_str(buf, "Wildcard");
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkPatIdent(name) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatIdent name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkPatVariant(name, fields) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatVariant name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " fields=");
      ad_dec(buf, fields.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      var i = 0;
      while i < fields.len() {
        ad_ident_line(buf, n, fields[i], depth + 1, "Name");
        i = i + 1;
      }
    }
    NkPatStruct(name, fields) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatStruct name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_str(buf, " fields=");
      ad_dec(buf, fields.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      var i = 0;
      while i < fields.len() {
        ad_node(buf, n, fields[i], depth + 1);
        i = i + 1;
      }
    }
    NkPatField(name, pattern) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatField name=");
      ad_hex_str(buf, ad_ident_name(n, name));
      ad_kv_span(buf, n[name].span);
      ad_nl(buf);
      ad_node(buf, n, pattern, depth + 2);
    }
    NkPatTuple(items) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatTuple n=");
      ad_dec(buf, items.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_kids(buf, n, items, depth + 1);
    }
    NkPatSome(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatSome");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkPatNone => {
      ad_indent(buf, depth);
      ad_str(buf, "PatNone");
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
    NkPatOk(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatOk");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkPatErr(inner) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatErr");
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_node(buf, n, inner, depth + 1);
    }
    NkPatOr(items) => {
      ad_indent(buf, depth);
      ad_str(buf, "PatOr n=");
      ad_dec(buf, items.len());
      ad_kv_span(buf, sp);
      ad_nl(buf);
      ad_kids(buf, n, items, depth + 1);
    }

    // First-level kinds that reach this dispatch only via nested helpers
    // (Defensive: emit the same headers as the first level would).
    _ => {
      ad_indent(buf, depth);
      ad_str(buf, "Unknown");
      ad_kv_span(buf, sp);
      ad_nl(buf);
    }
  }
}

fn ad_line1(buf: &mut Vec[UInt8], depth: Int, text: Str) {
  ad_indent(buf, depth);
  ad_str(buf, text);
  ad_nl(buf);
}

// ============================================================================
// Entry point
// ============================================================================

pub fn dump_nodes(nodes: &Vec[Node], root: Int) -> Int {
  var out = Vec[UInt8].new();
  ad_node(&mut out, nodes, root, 0);
  io.print(Str::from_utf8(out));
  return 0;
}
