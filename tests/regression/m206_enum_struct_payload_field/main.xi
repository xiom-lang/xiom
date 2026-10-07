// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m206 (packages graphql relay): a STRUCT enum payload matched from a
// Vec-index scrutinee (`op.set.selections[i]` -> `Sel.Field(fs)`) must bind
// `fs` pointer-backed to the boxed payload. Pre-fix the binding kept the raw
// i64 handle, every payload field read as the constant `0`
// (`fs.name == "hello"` was false), and the graphql conformance suite failed
// 9/10 ("validate valid operation").
// Declaration order matters: the enum is declared BEFORE its payload structs
// (like graphql.xi's GraphQLSelection), so the layout is the BOXED
// `{ i64 tag, i64 payload-ptr }` form. When the payload types are declared
// first the compiler inlines them (`{ tag, FieldSel, SpreadSel, ... }`) and
// the raw-i64 fallback happened to work.
enum Sel {
  Field(selection: FieldSel),
  Spread(selection: SpreadSel),
  Inline(selection: InlineSel),
} derive[Clone]

type FieldSel = {
  name: Str;
  alias: Str;
  args: Vec[Str];
  directives: Vec[Str];
  sub: SelSet;
} derive[Clone]

type SpreadSel = {
  name: Str;
} derive[Clone]

type InlineSel = {
  type_condition: Str;
} derive[Clone]

type SelSet = {
  selections: Vec[Sel];
} derive[Clone]

type TypeField = {
  name: Str;
} derive[Clone]

type GType = {
  name: Str;
  fields: Vec[TypeField];
} derive[Clone]

type Op = {
  name: Str;
  set: SelSet;
} derive[Clone]

fn find_field(ty: &GType, name: Str) -> Bool {
  var i = 0;
  while i < ty.fields.len() {
    if ty.fields[i].name == name { return true; }
    i = i + 1;
  };
  return false;
}

fn find_type(name: Str) -> Option[GType] {
  if name == "Query" {
    var t = GType{ name: "Query", fields: Vec[TypeField].new() };
    t.fields.push(TypeField{ name: "hello" });
    return Some(t);
  };
  return None;
}

fn validate(op: &Op) -> Bool {
  var root_type = find_type("Query");
  match root_type {
    None => { return false; },
    Some(_) => {},
  };
  var root: GType = GType{ name: "", fields: Vec[TypeField].new() };
  match root_type {
    Some(rt) => { root = rt; },
    None => { return false; },
  };
  var i = 0;
  while i < op.set.selections.len() {
    match op.set.selections[i] {
      Sel.Field(fs) => {
        if !find_field(&root, fs.name) { return false; }
      },
      Sel.Spread(_) => {},
      Sel.Inline(_) => {},
    };
    i = i + 1;
  };
  return true;
}

fn main() -> Int {
  var v: Vec[Sel] = Vec[Sel].new();
  v.push(Sel.Field(FieldSel{
    name: "hello",
    alias: "",
    args: Vec[Str].new(),
    directives: Vec[Str].new(),
    sub: SelSet{ selections: Vec[Sel].new() },
  }));
  var op = Op{ name: "q", set: SelSet{ selections: v } };
  if validate(&op) { return 0; }
  return 1;
}
