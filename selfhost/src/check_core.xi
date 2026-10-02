// XIOM -- Selfhost checker: signature collection + program walk (Phase 3)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Port of `Checker::collect_signatures` / `Checker::check_top_decl` /
// `Checker::check_fn_decl` for the stage-1 subset (see
// docs/checklists/selfhost-phase3.md):
//   * functions (free, module-nested, `fn T.m`, impl members) with the
//     Rust key scheme: qualified key + BARE fallback (first registration
//     wins), methods keyed `Type.method` and never claiming the bare leaf;
//   * structs + aliases + global consts; enums + variants (three keys:
//     qualified, bare, `Enum.Variant`);
//   * extern blocks as callable signatures;
//   * interfaces by name (permissive dispatch);
//   * `use` declarations latch permissive mode (the catalog is a later
//     stage; a file that imports modules must not report undefined names).
//
// Phase 3 stage boundaries: this file owns the program/fn structure and the
// contracts; `check_expr.xi` owns statements/expressions.

module selfhost_check_core

use xiom.string;
use selfhost_ast.NodeKind;
use selfhost_check_expr;
use selfhost_check_state;
use selfhost_check_state.Checker;
use selfhost_check_state.FnParam;
use selfhost_check_state.FnSig;
use selfhost_check_state.Local;
use selfhost_check_state.TypeDecl;
use selfhost_check_state.Field;
use selfhost_check_state.Variant;
use selfhost_check_types;

// ============================================================================
// Small helpers
// ============================================================================

fn cc_leaf(name: Str) -> Str {
  var i = name.len() - 1;
  while i >= 0 {
    if (string.byte_at(name, i) as Int) == 46 {
      return string.str_slice(name, i + 1, name.len());
    }
    i = i - 1;
  }
  return name;
}

fn cc_dotted(prefix: Str, name: Str) -> Str {
  if prefix.len() == 0 { return name; }
  return prefix + "." + name;
}

fn cc_push_fn(c: &mut Checker, key: Str, name: Str, params: Vec[FnParam],
              ret: Str, has_ret: Int, generics: Vec[Str], has_recv: Int) {
  var i = 0;
  while i < c.functions.len() {
    if c.functions[i].key == key { return; }
    i = i + 1;
  }
  c.functions.push(FnSig{
    key: key, name: name, params: params, ret: ret, has_ret: has_ret,
    generics: generics, has_recv: has_recv,
  });
}

/// Register `key` plus the bare fallback (Rust `entry().or_insert`).
fn cc_push_fn_pair(c: &mut Checker, key: Str, bare: Str, name: Str,
                   params: Vec[FnParam], ret: Str, has_ret: Int,
                   generics: Vec[Str], has_recv: Int) {
  cc_push_fn(c, key, name, params, ret, has_ret, generics, has_recv);
  if key != bare {
    cc_push_fn(c, bare, name, params, ret, has_ret, generics, has_recv);
  }
}

fn cc_push_local_vec(v: &mut Vec[Local], name: Str, ty: Str) {
  var i = 0;
  while i < v.len() {
    if v[i].name == name { return; }
    i = i + 1;
  }
  v.push(Local{ name: name, ty: ty });
}

fn cc_read_params(c: &Checker, params: Vec[Int]) -> Vec[FnParam] {
  var out = Vec[FnParam].new();
  var i = 0;
  while i < params.len() {
    let pnode = c.p.nodes[params[i]];
    match pnode.kind {
      NkParam(name, ty, mutself, refself) => {
        out.push(FnParam{ name: ck_ident(c, name), ty: ck_type_from_ast(c, ty) });
      }
      _ => {}
    }
    i = i + 1;
  }
  return out;
}

fn cc_read_generics(c: &Checker, generics: Vec[Int]) -> Vec[Str] {
  var out = Vec[Str].new();
  var i = 0;
  while i < generics.len() {
    let gnode = c.p.nodes[generics[i]];
    match gnode.kind {
      NkGeneric(name, bounds, is_const, const_ty) => {
        out.push(ck_ident(c, name));
      }
      _ => {}
    }
    i = i + 1;
  }
  return out;
}

/// Elided/anonymous annotations resolve structurally (Rust
/// `infer_global_init_type`, literals only for stage 1).
fn cc_infer_init(c: &Checker, value: Int) -> Str {
  if value < 0 { return "()"; }
  let node = c.p.nodes[value];
  match node.kind {
    NkLitInt(v) => { return "Int"; }
    NkLitBigInt(hi, lo) => { return "Int"; }
    NkLitFloat(lex) => { return "Float64"; }
    NkLitStr(data) => { return "Str"; }
    NkLitChar(cp) => { return "Char"; }
    NkLitBool(v) => { return "Bool"; }
    NkExprArray(items) => {
      if items.len() > 0 { return "Vec[" + cc_infer_init(c, items[0]) + "]"; }
      return "Vec";
    }
    _ => { return "_"; }
  }
}

// ============================================================================
// Registration pass
// ============================================================================

pub fn cc_collect(c: &mut Checker, root: Int) {
  cc_register_builtin_types(c);
  cc_scan(c, root, "");
}

/// Rust `register_builtins` type table: every primitive plus the compound
/// builtins (Map is NOT a builtin -- it lives in collections). These make
/// bare `Vec`/`Int` identifiers resolve instead of reporting undefined.
fn cc_register_builtin_types(c: &mut Checker) {
  let prims = Vec[Str].new();
  prims.push("Bool");
  prims.push("Int");
  prims.push("Int8");
  prims.push("Int16");
  prims.push("Int32");
  prims.push("Int64");
  prims.push("Int128");
  prims.push("UInt");
  prims.push("UInt8");
  prims.push("UInt16");
  prims.push("UInt32");
  prims.push("UInt64");
  prims.push("UInt128");
  prims.push("Float32");
  prims.push("Float64");
  prims.push("Float128");
  prims.push("Char");
  prims.push("Str");
  var i = 0;
  while i < prims.len() {
    cc_push_builtin_type(c, prims[i]);
    i = i + 1;
  }
  cc_push_builtin_type(c, "Vec");
  cc_push_builtin_type(c, "Set");
  cc_push_builtin_type(c, "Stack");
  cc_push_builtin_type(c, "Slice");
  cc_push_builtin_type(c, "Option");
  cc_push_builtin_type(c, "Result");
}

fn cc_push_builtin_type(c: &mut Checker, name: Str) {
  var i = 0;
  while i < c.types.len() {
    if c.types[i].name == name { return; }
    i = i + 1;
  }
  c.types.push(TypeDecl{ name: name, fields: Vec[Field].new(), is_enum: 0 });
}

fn cc_scan(c: &mut Checker, idx: Int, modpath: Str) {
  if idx < 0 { return; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkProgram(items) => {
      var i = 0;
      while i < items.len() {
        cc_scan(c, items[i], modpath);
        i = i + 1;
      }
    }
    NkModule(name, path, items, file_level, has_source, source) => {
      let nm = ck_ident(c, name);
      let newp = cc_dotted(modpath, nm);
      var i = 0;
      while i < items.len() {
        cc_scan(c, items[i], newp);
        i = i + 1;
      }
    }
    NkUse(path, glob, alias) => {
      c.has_uses = 1;
    }
    NkTypeDecl(is_pub, name, generics, fields, derived, invariants, derives, alias) => {
      cc_register_type(c, name, fields, alias, modpath);
    }
    NkEnumDecl(is_pub, name, generics, variants, derives) => {
      cc_register_enum(c, name, variants, modpath);
    }
    NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
      cc_register_fn_node(c, recv, name, generics, params, ret, modpath, "");
    }
    NkImpl(trait_name, trait_args, type_name, members) => {
      cc_register_impl(c, type_name, members, modpath);
    }
    NkConst(is_pub, is_mut, name, ty, value) => {
      let nm = ck_ident(c, name);
      var declared = "_";
      if ty >= 0 { declared = ck_type_from_ast(c, ty); }
      var resolved = declared;
      let is_elided = declared == "Int" && ty >= 0 && cc_is_wildcard_annot(c, ty);
      if declared == "_" || is_elided {
        resolved = cc_infer_init(c, value);
      }
      cc_push_local_vec(&mut c.globals, nm, resolved);
    }
    NkExtern(linkage, fns) => {
      var i = 0;
      while i < fns.len() {
        let fnode = c.p.nodes[fns[i]];
        match fnode.kind {
          NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
            cc_register_fn_node(c, recv, name, generics, params, ret, modpath, "");
          }
          _ => {}
        }
        i = i + 1;
      }
    }
    NkInterface(is_pub, name, generics, parent, members) => {
      let nm = ck_ident(c, name);
      let key = cc_dotted(modpath, nm);
      cc_push_interface(c, key);
      if key != nm { cc_push_interface(c, nm); }
    }
    _ => {}
  }
}

fn cc_is_wildcard_annot(c: &Checker, ty: Int) -> Bool {
  let t = c.p.nodes[ty];
  match t.kind {
    NkTyNamed(name, args) => {
      if ck_ident(c, name) == "_" { return true; }
      return false;
    }
    _ => { return false; }
  }
}

fn cc_push_interface(c: &mut Checker, name: Str) {
  var i = 0;
  while i < c.interfaces.len() {
    if c.interfaces[i] == name { return; }
    i = i + 1;
  }
  c.interfaces.push(name);
}

fn cc_register_type(c: &mut Checker, name: Int, fields: Vec[Int], alias: Int, modpath: Str) {
  let nm = ck_ident(c, name);
  let key = cc_dotted(modpath, nm);
  var fs = Vec[Field].new();
  var i = 0;
  while i < fields.len() {
    let fnode = c.p.nodes[fields[i]];
    match fnode.kind {
      NkField(fname, fty) => {
        // First field of a name wins (BTreeMap semantic keyed by name).
        let fkey = ck_ident(c, fname);
        var dup = false;
        var j = 0;
        while j < fs.len() {
          if fs[j].name == fkey { dup = true; }
          j = j + 1;
        }
        if !dup {
          fs.push(Field{ name: fkey, ty: ck_type_from_ast(c, fty) });
        }
      }
      _ => {}
    }
    i = i + 1;
  }
  var exists = false;
  var k = 0;
  while k < c.types.len() {
    if c.types[k].name == key { exists = true; }
    k = k + 1;
  }
  if !exists {
    c.types.push(TypeDecl{ name: key, fields: fs, is_enum: 0 });
    if key != nm {
      c.types.push(TypeDecl{ name: nm, fields: fs, is_enum: 0 });
    }
  }
  if alias >= 0 {
    let resolved = ck_type_from_ast(c, alias);
    cc_push_local_vec(&mut c.aliases, key, resolved);
    if key != nm { cc_push_local_vec(&mut c.aliases, nm, resolved); }
  }
}

fn cc_register_enum(c: &mut Checker, name: Int, variants: Vec[Int], modpath: Str) {
  let nm = ck_ident(c, name);
  let key = cc_dotted(modpath, nm);
  var exists = false;
  var k = 0;
  while k < c.types.len() {
    if c.types[k].name == key { exists = true; }
    k = k + 1;
  }
  if !exists {
    c.types.push(TypeDecl{ name: key, fields: Vec[Field].new(), is_enum: 1 });
    if key != nm {
      c.types.push(TypeDecl{ name: nm, fields: Vec[Field].new(), is_enum: 1 });
    }
  }
  var i = 0;
  while i < variants.len() {
    let vnode = c.p.nodes[variants[i]];
    match vnode.kind {
      NkVariant(vname, vfields) => {
        let vn = ck_ident(c, vname);
        var fs = Vec[Field].new();
        var j = 0;
        while j < vfields.len() {
          let fnode = c.p.nodes[vfields[j]];
          match fnode.kind {
            NkField(fname, fty) => {
              fs.push(Field{ name: ck_ident(c, fname), ty: ck_type_from_ast(c, fty) });
            }
            _ => {}
          }
          j = j + 1;
        }
        // Rust registers three spellings; first registration wins per key.
        cc_push_variant(c, cc_dotted(modpath, vn), nm, fs);
        if modpath.len() > 0 {
          cc_push_variant(c, vn, nm, fs);
        }
        cc_push_variant(c, nm + "." + vn, nm, fs);
        if vfields.len() > 0 {
          cc_push_variant_fields(c, cc_dotted(modpath, vn), nm, fs);
          if modpath.len() > 0 {
            cc_push_variant_fields(c, vn, nm, fs);
          }
          cc_push_variant_fields(c, nm + "." + vn, nm, fs);
        }
      }
      _ => {}
    }
    i = i + 1;
  }
}

fn cc_push_variant(c: &mut Checker, key: Str, enum_name: Str, fields: Vec[Field]) {
  var i = 0;
  while i < c.variants.len() {
    if c.variants[i].key == key { return; }
    i = i + 1;
  }
  c.variants.push(Variant{ key: key, enum_name: enum_name, fields: fields });
}

fn cc_push_variant_fields(c: &mut Checker, key: Str, enum_name: Str, fields: Vec[Field]) {
  cc_push_variant(c, key, enum_name, fields);
}

fn cc_register_impl(c: &mut Checker, type_name: Int, members: Vec[Int], modpath: Str) {
  let tn = ck_ident(c, type_name);
  var i = 0;
  while i < members.len() {
    let mnode = c.p.nodes[members[i]];
    match mnode.kind {
      NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
        cc_register_fn_node(c, recv, name, generics, params, ret, modpath, cc_dotted("", tn));
      }
      _ => {}
    }
    i = i + 1;
  }
}

fn cc_register_fn_node(c: &mut Checker, recv: Int, name: Int, generics: Vec[Int],
                       params: Vec[Int], ret: Int, modpath: Str, impl_type: Str) {
  let nm = ck_ident(c, name);
  let ps = cc_read_params(c, params);
  let gs = cc_read_generics(c, generics);
  var rt = "()";
  var has_ret = 0;
  if ret >= 0 {
    rt = ck_type_from_ast(c, ret);
    has_ret = 1;
  }
  var recv_name = impl_type;
  var has_recv = 0;
  if recv >= 0 {
    recv_name = ck_ident(c, recv);
    has_recv = 1;
  }
  if recv_name.len() > 0 {
    let leaf = cc_leaf(nm);
    let bare_key = recv_name + "." + leaf;
    let key = cc_dotted(modpath, bare_key);
    cc_push_fn_pair(c, key, bare_key, leaf, ps, rt, has_ret, gs, 1);
  } else {
    let bare_key = cc_leaf(nm);
    let key = cc_dotted(modpath, nm);
    cc_push_fn_pair(c, key, bare_key, nm, ps, rt, has_ret, gs, has_recv);
  }
}

// ============================================================================
// Body pass
// ============================================================================

pub fn cc_check_program(c: &mut Checker, root: Int) {
  cc_check_item(c, root, "");
}

fn cc_check_item(c: &mut Checker, idx: Int, impl_type: Str) {
  if idx < 0 { return; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkProgram(items) => {
      var i = 0;
      while i < items.len() {
        cc_check_item(c, items[i], "");
        i = i + 1;
      }
    }
    NkModule(name, path, items, file_level, has_source, source) => {
      var i = 0;
      while i < items.len() {
        cc_check_item(c, items[i], "");
        i = i + 1;
      }
    }
    NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
      cc_check_fn_node(c, recv, name, params, generics, ret, contracts, body, impl_type);
    }
    NkImpl(trait_name, trait_args, type_name, members) => {
      let tn = ck_ident(c, type_name);
      var i = 0;
      while i < members.len() {
        let mnode = c.p.nodes[members[i]];
        match mnode.kind {
          NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
            cc_check_fn_node(c, recv, name, params, generics, ret, contracts, body, tn);
          }
          _ => {}
        }
        i = i + 1;
      }
    }
    _ => {}
  }
}

fn cc_check_fn_node(c: &mut Checker, recv: Int, name: Int, params: Vec[Int],
                    generics: Vec[Int], ret: Int, contracts: Vec[Int], body: Int,
                    impl_type: Str) {
  if body < 0 { return; }
  ck_push_scope(c);
  var i = 0;
  while i < params.len() {
    let pnode = c.p.nodes[params[i]];
    match pnode.kind {
      NkParam(name_idx, ty, mutself, refself) => {
        ck_add_local(c, ck_ident(c, name_idx), ck_type_from_ast(c, ty));
      }
      _ => {}
    }
    i = i + 1;
  }
  // Const-generic params are locals; type params resolve to their own name.
  i = 0;
  while i < generics.len() {
    let gnode = c.p.nodes[generics[i]];
    match gnode.kind {
      NkGeneric(gname, bounds, is_const, const_ty) => {
        var gty = "type";
        if is_const == 1 && const_ty >= 0 {
          gty = ck_type_from_ast(c, const_ty);
        }
        ck_add_local(c, ck_ident(c, gname), gty);
      }
      _ => {}
    }
    i = i + 1;
  }
  // Receiver context: `self` binds to the impl type; receiver fields are
  // injected as bare locals (Rust G-20), except names shadowed by params.
  var recv_ty = "";
  if recv >= 0 { recv_ty = ck_ident(c, recv); }
  if impl_type.len() > 0 { recv_ty = impl_type; }
  if recv_ty.len() > 0 {
    ck_add_local(c, "self", recv_ty);
    c.cur_recv = recv_ty;
    let ti = ck_find_type(c, recv_ty);
    if ti >= 0 {
      let fields = c.types[ti].fields;
      var j = 0;
      while j < fields.len() {
        var shadowed = false;
        var k = 0;
        while k < params.len() {
          let pnode = c.p.nodes[params[k]];
          match pnode.kind {
            NkParam(name_idx, ty, mutself, refself) => {
              if ck_ident(c, name_idx) == fields[j].name { shadowed = true; }
            }
            _ => {}
          }
          k = k + 1;
        }
        if !shadowed {
          ck_add_local(c, fields[j].name, fields[j].ty);
        }
        j = j + 1;
      }
    }
  }
  var expected = "()";
  var has_expected = 0;
  if ret >= 0 {
    expected = ck_type_from_ast(c, ret);
    has_expected = 1;
  }
  c.cur_ret = expected;
  c.cur_ret_set = has_expected;
  // Contracts: strict mode (default) checks each clause as a Bool predicate;
  // `ensures` also binds `result` to the declared return type.
  i = 0;
  while i < contracts.len() {
    let cnode = c.p.nodes[contracts[i]];
    match cnode.kind {
      NkRequires(expr) => {
        cc_check_clause(c, contracts[i], expr);
      }
      NkEnsures(expr) => {
        cc_check_clause(c, contracts[i], expr);
      }
      _ => {}
    }
    i = i + 1;
  }
  ck_check_block(c, body, expected, has_expected);
  c.cur_ret = "";
  c.cur_ret_set = 0;
  c.cur_recv = "";
  ck_pop_scope(c);
}

fn cc_check_clause(c: &mut Checker, clause_idx: Int, expr: Int) {
  ck_push_scope(c);
  if c.cur_ret_set == 1 { ck_add_local(c, "result", c.cur_ret); }
  let ty = selfhost_check_expr.ce_check_expr(c, expr);
  if ty != "Bool" && ty != "<error>" && ty != "_" {
    let sp = ck_span_of(c, clause_idx);
    let _ = ck_error_at(c, "contract clause must be Bool, found " + ty + " (strict mode)", sp.line, sp.col);
  }
  ck_pop_scope(c);
}
