// XIOM -- Selfhost checker: canonical type names (Phase 3, stage 1)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Port of `crates/xiom-ast/src/structural.rs` (re-exported by
// `crates/xiom-check/src/structural.rs`): XIOM encodes compound types as
// strings (`"Vec[Option[Int]]"`, `"Result[Int, Str]"`, `"*T"`,
// `"Tuple__A__B"`). This module parses/renders the canonical spelling
// (`"Result[Int,Str]"` -> `"Result[Int, Str]"`) exactly like the Rust side,
// so checker type equality is string equality and every message that embeds
// `CheckedType::name()` matches byte for byte.
//
// The selfhost represents every checked type as its canonical NAME string
// (see `crates/xiom-check/src/types.rs::CheckedType::name`); scalars are
// just their names ("Int", "Bool", "()", "!"), `fn(P, Q) -> R` keeps that
// spelling, and user/compound types intern to their canonical text.

module selfhost_check_types

use xiom.string;

/// ASCII whitespace (type names only ever carry formatting whitespace).
fn ct_is_space(b: Int) -> Bool {
  if b == 32 { return true; }
  if b == 9 { return true; }
  if b == 10 { return true; }
  if b == 13 { return true; }
  return false;
}

/// Rust `str::trim` for the byte alphabet type names use.
pub fn ct_trim(s: Str) -> Str {
  var a = 0;
  var b = s.len();
  while a < b && ct_is_space(string.byte_at(s, a) as Int) {
    a = a + 1;
  }
  while b > a && ct_is_space(string.byte_at(s, b - 1) as Int) {
    b = b - 1;
  }
  return string.str_slice(s, a, b);
}

fn ct_is_upper_ascii(b: Int) -> Bool {
  if b >= 65 && b <= 90 { return true; }
  return false;
}

/// Index of the first '[' in `text`, or -1.
fn ct_find_open(text: Str) -> Int {
  var i = 0;
  while i < text.len() {
    if (string.byte_at(text, i) as Int) == 91 { return i; }
    i = i + 1;
  }
  return -1;
}

/// Index of the ']' matching the '[' at `open`, or -1 when unclosed.
fn ct_bracket_close(text: Str, open: Int) -> Int {
  var depth = 0;
  var i = open;
  while i < text.len() {
    let b = string.byte_at(text, i) as Int;
    if b == 91 { depth = depth + 1; }
    if b == 93 {
      depth = depth - 1;
      if depth == 0 { return i; }
    }
    i = i + 1;
  }
  return -1;
}

/// Join with a separator (avoiding trailing separators).
pub fn ct_join(items: Vec[Str], sep: Str) -> Str {
  var out = "";
  var i = 0;
  while i < items.len() {
    if i > 0 { out = out + sep; }
    out = out + items[i];
    i = i + 1;
  }
  return out;
}

/// Split a top-level comma list at bracket depth 0 (Rust
/// `parse_type_arg_list`); every entry is canonicalized.
fn ct_parse_arg_list(inner: Str) -> Vec[Str] {
  var out = Vec[Str].new();
  var depth = 0;
  var start = 0;
  var i = 0;
  while i < inner.len() {
    let b = string.byte_at(inner, i) as Int;
    if b == 91 { depth = depth + 1; }
    if b == 93 { depth = depth - 1; }
    if b == 44 && depth == 0 {
      let arg = ct_trim(string.str_slice(inner, start, i));
      if arg.len() > 0 { out.push(ct_canonical(arg)); }
      start = i + 1;
    }
    i = i + 1;
  }
  let arg = ct_trim(string.str_slice(inner, start, inner.len()));
  if arg.len() > 0 { out.push(ct_canonical(arg)); }
  return out;
}

/// Rust `canonical_type_name`: whitespace-normalized, arguments separated by
/// exactly ", ". Malformed bracket text stays verbatim (Opaque).
pub fn ct_canonical(name: Str) -> Str {
  let text = ct_trim(name);
  if text.len() == 0 { return ""; }
  if text == "_" { return "_"; }
  if text == "()" { return "()"; }
  let first = string.byte_at(text, 0) as Int;
  if first == 42 {
    let rest = ct_trim(string.str_slice(text, 1, text.len()));
    if rest.len() == 0 { return "*"; }
    return "*" + ct_canonical(rest);
  }
  let open = ct_find_open(text);
  if open >= 0 {
    let base = ct_trim(string.str_slice(text, 0, open));
    let close = ct_bracket_close(text, open);
    if close < 0 { return text; }
    let rest = ct_trim(string.str_slice(text, close + 1, text.len()));
    if rest.len() > 0 { return text; }
    let inner = string.str_slice(text, open + 1, close);
    let args = ct_parse_arg_list(inner);
    return base + "[" + ct_join(args, ", ") + "]";
  }
  if text.len() == 1 && ct_is_upper_ascii(first) { return text; }
  return text;
}

/// Rust `CheckedType::from_str`: canonical, except the `_` placeholder which
/// classifies as Int ("wildcard placeholder").
pub fn ct_from_str(s: Str) -> Str {
  let c = ct_canonical(s);
  if c == "_" { return "Int"; }
  return c;
}

/// Base name without arguments ("Vec[Int]" -> "Vec").
pub fn ct_base(name: Str) -> Str {
  let open = ct_find_open(name);
  if open >= 0 { return string.str_slice(name, 0, open); }
  return name;
}

/// Top-level arguments (canonical renderings); empty for non-containers.
pub fn ct_args(name: Str) -> Vec[Str] {
  let open = ct_find_open(name);
  if open < 0 { return Vec[Str].new(); }
  let close = ct_bracket_close(name, open);
  if close < 0 { return Vec[Str].new(); }
  let rest = ct_trim(string.str_slice(name, close + 1, name.len()));
  if rest.len() > 0 { return Vec[Str].new(); }
  return ct_parse_arg_list(string.str_slice(name, open + 1, close));
}

pub fn ct_is_container(name: Str) -> Bool {
  return ct_args(name).len() > 0;
}

pub fn ct_is_wildcard(name: Str) -> Bool {
  if name == "_" { return true; }
  return false;
}

pub fn ct_is_unit(name: Str) -> Bool {
  if name == "()" { return true; }
  return false;
}

/// Single uppercase generic parameter, including one pointer level (`T`, `*T`).
pub fn ct_is_generic_param(name: Str) -> Bool {
  var t = name;
  let b0 = string.byte_at(t, 0) as Int;
  if t.len() > 0 && b0 == 42 {
    t = string.str_slice(t, 1, t.len());
  }
  if t.len() == 1 && ct_is_upper_ascii(string.byte_at(t, 0) as Int) { return true; }
  return false;
}

/// Pointer-like: `*T` (parsed as a pointer) or the legacy bare `Ptr`.
pub fn ct_is_pointer_like(name: Str) -> Bool {
  if name.len() > 0 && (string.byte_at(name, 0) as Int) == 42 { return true; }
  let base = ct_base(name);
  if base == "Ptr" { return true; }
  if base.len() > 0 && (string.byte_at(base, 0) as Int) == 42 { return true; }
  return false;
}

pub fn ct_is_tuple_like(name: Str) -> Bool {
  return string.str_starts_with(ct_base(name), "Tuple");
}

pub fn ct_is_fn_type(name: Str) -> Bool {
  return string.str_starts_with(name, "fn(");
}

/// `CheckedType::is_numeric` (includes UInt128).
pub fn ct_is_numeric(name: Str) -> Bool {
  if name == "Int" { return true; }
  if name == "Int8" { return true; }
  if name == "Int16" { return true; }
  if name == "Int32" { return true; }
  if name == "Int64" { return true; }
  if name == "Int128" { return true; }
  if name == "UInt" { return true; }
  if name == "UInt8" { return true; }
  if name == "UInt16" { return true; }
  if name == "UInt32" { return true; }
  if name == "UInt64" { return true; }
  if name == "UInt128" { return true; }
  if name == "Float32" { return true; }
  if name == "Float64" { return true; }
  if name == "Float128" { return true; }
  return false;
}

/// Rust `is_float_family` (includes the legacy `Float`/`fp128` spellings).
pub fn ct_is_float_family(name: Str) -> Bool {
  if name == "Float32" { return true; }
  if name == "Float64" { return true; }
  if name == "Float128" { return true; }
  if name == "Float" { return true; }
  if name == "fp128" { return true; }
  return false;
}

pub fn ct_is_int_family(name: Str) -> Bool {
  if ct_is_float_family(name) { return false; }
  return ct_is_numeric(name);
}

/// `TypeShape::scalar_name()` for `type_arg_compatible`: the base for named
/// shapes, the canonical rendering for pointer/opaque shapes.
pub fn ct_scalar_name(name: Str) -> Str {
  if name.len() > 0 && (string.byte_at(name, 0) as Int) == 42 {
    let rest = ct_trim(string.str_slice(name, 1, name.len()));
    if rest.len() == 0 { return "*"; }
    return "*" + ct_canonical(rest);
  }
  if ct_is_container(name) { return ct_base(name); }
  return name;
}
