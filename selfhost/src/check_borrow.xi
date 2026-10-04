// XIOM -- Selfhost borrow checker (Phase 3 sub-stage 5)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Port of `xiom_check::BorrowChecker` (crates/xiom-check/src/lib.rs
// 8989-9761) plus the place model (borrow/place.rs) and loan set
// (borrow/loans.rs). Runs in `--dump-check` after the type checker on the
// success path only, mirroring `compile()`'s non-strict pass: every borrow
// diagnostic is an `E001` warning appended after the checker diagnostics.
//
// Representation notes: places keep the exact `place_display` spelling
// (`p.x`, `v[_]`, `p.*`) and conflicts are resolved by a lockstep walk over
// the dot-split projections (field names cannot contain dots), so no nested
// Vec[enum] storage is needed. Scopes use the same flat append-only +
// live-count discipline as the checker state; no `Vec::pop`/`truncate`.

module selfhost_check_borrow

use xiom.string;
use xiom.io;
use selfhost_ast.Span;
use selfhost_ast.NodeKind;
use selfhost_check_state;
use selfhost_check_state.Diag;
use selfhost_parser_state;
use selfhost_parser_state.Parser;

// Owner states
const BO_OWNED: Int = 0;
const BO_MOVED: Int = 1;
const BO_READB: Int = 2;
const BO_WRITEB: Int = 3;

// Borrow kinds
const BB_READ: Int = 0;
const BB_WRITE: Int = 1;

// ExprResult
const ER_VALUE: Int = 0;
const ER_READREF: Int = 1;
const ER_WRITEREF: Int = 2;

// Place conflicts
const PC_DISJOINT: Int = 0;
const PC_OVERLAP: Int = 1;
const PC_AMBIGUOUS: Int = 2;

pub type BcOwner = {
  name: Str;
  state: Int;
  reads: Int;
  mutable: Int;
  ty: Str;
}

pub type BcBorrow = {
  name: Str;
  btype: Int;
}

pub type BcLoan = {
  place: Str;
  write: Int;
}

pub type Bc = {
  owners: Vec[BcOwner];
  nowners: Int;
  owner_marks: Vec[Int];
  bmarks: Vec[Int];
  nscopes: Int;
  borrows: Vec[BcBorrow];
  nborrows: Int;
  loans: Vec[BcLoan];
  nloans: Int;
  errors: Vec[Diag];
  params: Vec[Str];
}

pub fn bc_new() -> Bc {
  return Bc{
    owners: Vec[BcOwner].new(), nowners: 0,
    owner_marks: Vec[Int].new(), bmarks: Vec[Int].new(), nscopes: 0,
    borrows: Vec[BcBorrow].new(), nborrows: 0,
    loans: Vec[BcLoan].new(), nloans: 0,
    errors: Vec[Diag].new(), params: Vec[Str].new(),
  };
}

/// Run the borrow pass over a parsed program; returns E001 diagnostics in
/// emission order.
pub fn bc_run(p: &Parser, root: Int) -> Vec[Diag] {
  var bc = bc_new();
  bc_top(bc, p, root);
  return bc.errors;
}

fn bc_span(p: &Parser, idx: Int) -> Span {
  if idx < 0 { return Span{ line: 0, col: 0, byte_start: 0, byte_end: 0 }; }
  return p.nodes[idx].span;
}

fn bc_err(bc: &mut Bc, message: Str, sp: Span) {
  bc.errors.push(Diag{
    kind: "borrow_warning", code: "E001", message: message, line: sp.line, col: sp.col,
  });
}

// ============================================================================
// Scopes, owners, borrows
// ============================================================================

fn bc_push_scope(bc: &mut Bc) {
  if bc.owner_marks.len() > bc.nscopes {
    bc.owner_marks[bc.nscopes] = bc.nowners;
  } else {
    bc.owner_marks.push(bc.nowners);
  }
  if bc.bmarks.len() > bc.nscopes {
    bc.bmarks[bc.nscopes] = bc.nborrows;
  } else {
    bc.bmarks.push(bc.nborrows);
  }
  bc.nscopes = bc.nscopes + 1;
}

fn bc_pop_scope(bc: &mut Bc) {
  if bc.nscopes == 0 { return; }
  bc.nscopes = bc.nscopes - 1;
  let bm = bc.bmarks[bc.nscopes];
  bc_release_borrows_since(bc, bm);
  bc.nowners = bc.owner_marks[bc.nscopes];
  // Rust `pop_scope` -> `active_loans.release_all()`: every scope exit drops
  // ALL active place loans (temporary argument scopes included).
  bc.nloans = 0;
}

fn bc_release_borrows_since(bc: &mut Bc, mark: Int) {
  var i = mark;
  while i < bc.nborrows {
    let sb = bc.borrows[i];
    bc_release_borrow(bc, sb.name, sb.btype);
    i = i + 1;
  }
  bc.nborrows = mark;
}

fn bc_find(bc: &Bc, name: Str) -> Int {
  var i = bc.nowners - 1;
  while i >= 0 {
    if bc.owners[i].name == name { return i; }
    i = i - 1;
  }
  return -1;
}

fn bc_add_owner(bc: &mut Bc, name: Str, mutable: Int, ty: Str) {
  let o = BcOwner{ name: name, state: BO_OWNED, reads: 0, mutable: mutable, ty: ty };
  if bc.owners.len() > bc.nowners {
    bc.owners[bc.nowners] = o;
  } else {
    bc.owners.push(o);
  }
  bc.nowners = bc.nowners + 1;
}

fn bc_push_borrow(bc: &mut Bc, name: Str, btype: Int) {
  let b = BcBorrow{ name: name, btype: btype };
  if bc.borrows.len() > bc.nborrows {
    bc.borrows[bc.nborrows] = b;
  } else {
    bc.borrows.push(b);
  }
  bc.nborrows = bc.nborrows + 1;
}

fn bc_release_borrow(bc: &mut Bc, name: Str, btype: Int) {
  let idx = bc_find(bc, name);
  if idx < 0 { return; }
  if btype == BB_READ {
    if bc.owners[idx].reads > 0 { bc.owners[idx].reads = bc.owners[idx].reads - 1; }
    if bc.owners[idx].reads == 0 { bc.owners[idx].state = BO_OWNED; }
  } else {
    bc.owners[idx].state = BO_OWNED;
    bc.owners[idx].reads = 0;
  }
}

fn bc_is_copy_type(ty: Str) -> Bool {
  if ty == "Int" || ty == "Int8" || ty == "Int16" || ty == "Int32" || ty == "Int64" { return true; }
  if ty == "UInt" || ty == "UInt8" || ty == "UInt16" || ty == "UInt32" || ty == "UInt64" { return true; }
  if ty == "Bool" || ty == "Char" || ty == "Str" { return true; }
  if ty == "Float32" || ty == "Float64" { return true; }
  return false;
}

fn bc_param_type_name(p: &Parser, idx: Int) -> Str {
  if idx < 0 { return "Int"; }
  let node = p.nodes[idx];
  match node.kind {
    NkTyNamed(name, args) => { return selfhost_check_state.ck_ident_in(p, name); }
    NkTyRef(inner) => { return bc_param_type_name(p, inner); }
    NkTyMutRef(inner) => { return bc_param_type_name(p, inner); }
    _ => { return "Int"; }
  }
}

fn bc_infer_type_from_expr(p: &Parser, idx: Int) -> Str {
  if idx < 0 { return "Int"; }
  let node = p.nodes[idx];
  match node.kind {
    NkLitStr(data) => { return "Str"; }
    NkLitBool(v) => { return "Bool"; }
    NkLitFloat(lex) => { return "Float64"; }
    NkLitChar(cp) => { return "Char"; }
    NkExprStruct(name, fields, base) => { return "Struct"; }
    _ => { return "Int"; }
  }
}

// ============================================================================
// Ownership operations
// ============================================================================

fn bc_check_use(bc: &mut Bc, name: Str, sp: Span) -> Int {
  let idx = bc_find(bc, name);
  if idx < 0 { return ER_VALUE; }
  if bc.owners[idx].state == BO_MOVED {
    bc_err(bc, "use of moved value '" + name + "'", sp);
    return ER_VALUE;
  }
  if bc.owners[idx].state == BO_READB { return ER_READREF; }
  if bc.owners[idx].state == BO_WRITEB { return ER_WRITEREF; }
  return ER_VALUE;
}

fn bc_read_borrow(bc: &mut Bc, name: Str, sp: Span) {
  let idx = bc_find(bc, name);
  var ok = true;
  if idx >= 0 {
    if bc.owners[idx].state == BO_MOVED {
      bc_err(bc, "use of moved value '" + name + "'", sp);
      ok = false;
    } elif bc.owners[idx].state == BO_WRITEB {
      bc_err(bc, "cannot borrow '" + name + "' as immutable while mutably borrowed", sp);
      ok = false;
    }
  }
  if ok {
    let i2 = bc_find(bc, name);
    if i2 >= 0 {
      bc.owners[i2].state = BO_READB;
      bc.owners[i2].reads = bc.owners[i2].reads + 1;
    }
    bc_push_borrow(bc, name, BB_READ);
  }
}

fn bc_write_borrow(bc: &mut Bc, name: Str, sp: Span) {
  let idx = bc_find(bc, name);
  var ok = true;
  if idx >= 0 {
    if bc.owners[idx].state == BO_MOVED {
      bc_err(bc, "use of moved value '" + name + "'", sp);
      ok = false;
    } elif bc.owners[idx].state == BO_READB {
      bc_err(bc, "cannot borrow '" + name + "' as mutable while immutably borrowed", sp);
      ok = false;
    } elif bc.owners[idx].state == BO_WRITEB {
      bc_err(bc, "cannot borrow '" + name + "' as mutable more than once at a time", sp);
      ok = false;
    } elif bc.owners[idx].mutable == 0 {
      bc_err(bc, "cannot borrow immutable local variable '" + name + "' as mutable", sp);
      ok = false;
    }
  }
  if ok {
    let i2 = bc_find(bc, name);
    if i2 >= 0 {
      bc.owners[i2].state = BO_WRITEB;
      bc.owners[i2].reads = 0;
    }
    bc_push_borrow(bc, name, BB_WRITE);
  }
}

fn bc_move_var(bc: &mut Bc, name: Str, sp: Span) {
  let idx = bc_find(bc, name);
  if idx < 0 { return; }
  if bc_is_copy_type(bc.owners[idx].ty) { return; }
  var ok = true;
  if bc.owners[idx].state == BO_MOVED {
    bc_err(bc, "use of moved value '" + name + "'", sp);
    ok = false;
  } elif bc.owners[idx].state == BO_READB || bc.owners[idx].state == BO_WRITEB {
    bc_err(bc, "cannot move '" + name + "' while borrowed", sp);
    ok = false;
  }
  if ok {
    bc.owners[idx].state = BO_MOVED;
  }
}

fn bc_param_contains(bc: &Bc, name: Str) -> Bool {
  var i = bc.params.len() - 1;
  while i >= 0 {
    if bc.params[i] == name { return true; }
    i = i - 1;
  }
  return false;
}

// ============================================================================
// Place model + loan set
// ============================================================================

fn bc_place_local(place: Str) -> Str {
  var i = 0;
  while i < place.len() {
    if (string.byte_at(place, i) as Int) == 46 {
      return string.str_slice(place, 0, i);
    }
    i = i + 1;
  }
  return place;
}

/// Dot-split projection parts after the root ("p.x[_]" -> ["x", "[_]"]).
fn bc_place_projs(place: Str) -> Vec[Str] {
  var out = Vec[Str].new();
  var start = -1;
  var i = 0;
  while i < place.len() {
    if (string.byte_at(place, i) as Int) == 46 {
      if start >= 0 { out.push(string.str_slice(place, start, i)); }
      start = i + 1;
    }
    i = i + 1;
  }
  if start >= 0 { out.push(string.str_slice(place, start, place.len())); }
  return out;
}

fn bc_places_conflict(a: Str, b: Str) -> Int {
  if bc_place_local(a) != bc_place_local(b) { return PC_DISJOINT; }
  let pa = bc_place_projs(a);
  let pb = bc_place_projs(b);
  var n = pa.len();
  if pb.len() < n { n = pb.len(); }
  var i = 0;
  while i < n {
    let x = pa[i];
    let y = pb[i];
    if x == "[_]" || y == "[_]" { return PC_AMBIGUOUS; }
    if x == "*" || y == "*" { return PC_OVERLAP; }
    if x == y { i = i + 1; continue; }
    return PC_DISJOINT;
  }
  return PC_OVERLAP;
}

/// Grant a loan; emits the Rust conflict message on rejection.
fn bc_grant(bc: &mut Bc, place: Str, write: Int, sp: Span) {
  var i = 0;
  while i < bc.nloans {
    let existing = bc.loans[i];
    let c = bc_places_conflict(existing.place, place);
    if c != PC_DISJOINT {
      if existing.write == 1 && write == 1 {
        bc_err(bc, "cannot borrow `" + place + "` as mutable because it is already borrowed as mutable", sp);
        return;
      }
      if existing.write == 1 || write == 1 {
        var mine = "immutable";
        if write == 1 { mine = "mutable"; }
        var theirs = "immutable";
        if existing.write == 1 { theirs = "mutable"; }
        bc_err(bc, "cannot borrow `" + place + "` as " + mine + " because it is also borrowed as " + theirs, sp);
        return;
      }
    }
    i = i + 1;
  }
  let l = BcLoan{ place: place, write: write };
  if bc.loans.len() > bc.nloans {
    bc.loans[bc.nloans] = l;
  } else {
    bc.loans.push(l);
  }
  bc.nloans = bc.nloans + 1;
}

fn bc_expr_to_place(p: &Parser, idx: Int) -> Str {
  if idx < 0 { return ""; }
  let node = p.nodes[idx];
  match node.kind {
    NkExprIdent(name) => { return selfhost_check_state.ck_ident_in(p, name); }
    NkExprField(obj, field) => {
      let base = bc_expr_to_place(p, obj);
      if base.len() == 0 { return ""; }
      return base + "." + selfhost_check_state.ck_ident_in(p, field);
    }
    NkExprIndex(obj, index) => {
      let base = bc_expr_to_place(p, obj);
      if base.len() == 0 { return ""; }
      return base + "[_]";
    }
    _ => { return ""; }
  }
}

fn bc_borrow_place_read(bc: &mut Bc, p: &Parser, expr: Int, sp: Span) {
  let place = bc_expr_to_place(p, expr);
  if place.len() > 0 {
    bc_grant(bc, place, 0, sp);
  }
  match p.nodes[expr].kind {
    NkExprIdent(name) => {
      bc_read_borrow(bc, selfhost_check_state.ck_ident_in(p, name), sp);
    }
    _ => {
      if place.len() > 0 {
        bc_read_borrow(bc, bc_place_local(place), sp);
      }
    }
  }
}

fn bc_borrow_place_write(bc: &mut Bc, p: &Parser, expr: Int, sp: Span) {
  let place = bc_expr_to_place(p, expr);
  if place.len() > 0 {
    bc_grant(bc, place, 1, sp);
  }
  match p.nodes[expr].kind {
    NkExprIdent(name) => {
      bc_write_borrow(bc, selfhost_check_state.ck_ident_in(p, name), sp);
    }
    _ => {
      if place.len() > 0 {
        bc_write_borrow(bc, bc_place_local(place), sp);
      }
    }
  }
}

// ============================================================================
// Walk
// ============================================================================

fn bc_top(bc: &mut Bc, p: &Parser, idx: Int) {
  if idx < 0 { return; }
  let node = p.nodes[idx];
  match node.kind {
    NkProgram(items) => {
      var i = 0;
      while i < items.len() {
        bc_top(bc, p, items[i]);
        i = i + 1;
      }
    }
    NkModule(name, path, items, file_level, has_source, source) => {
      var i = 0;
      while i < items.len() {
        bc_top(bc, p, items[i]);
        i = i + 1;
      }
    }
    NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
      if body >= 0 { bc_fn(bc, p, idx); }
    }
    _ => {}
  }
}

fn bc_fn(bc: &mut Bc, p: &Parser, idx: Int) {
  bc.params = Vec[Str].new();
  match p.nodes[idx].kind {
    NkFn(is_pub, is_async, recv, name, generics, params, ret, contracts, body, attrs) => {
      bc_push_scope(bc);
      var i = 0;
      while i < params.len() {
        let pnode = p.nodes[params[i]];
        match pnode.kind {
          NkParam(pname, ty, mutself, refself) => {
            let nm = selfhost_check_state.ck_ident_in(p, pname);
            bc.params.push(nm);
            bc_add_owner(bc, nm, 1, bc_param_type_name(p, ty));
          }
          _ => {}
        }
        i = i + 1;
      }
      // 5c-E: const-generic parameters register as locals.
      i = 0;
      while i < generics.len() {
        let gnode = p.nodes[generics[i]];
        match gnode.kind {
          NkGeneric(gname, bounds, is_const, const_ty) => {
            if is_const == 1 {
              bc_add_owner(bc, selfhost_check_state.ck_ident_in(p, gname), 1, "Int");
            }
          }
          _ => {}
        }
        i = i + 1;
      }
      bc_block(bc, p, body);
      bc_pop_scope(bc);
    }
    _ => {}
  }
  bc.params = Vec[Str].new();
}

fn bc_block(bc: &mut Bc, p: &Parser, idx: Int) {
  if idx < 0 { return; }
  let node = p.nodes[idx];
  match node.kind {
    NkBlock(stmts) => {
      var i = 0;
      while i < stmts.len() {
        let inode = p.nodes[stmts[i]];
        match inode.kind {
          NkStmtW(stmt) => { bc_stmt(bc, p, stmt); }
          NkTailW(e) => { let _ = bc_expr(bc, p, e); }
          _ => {}
        }
        i = i + 1;
      }
    }
    _ => {}
  }
}

fn bc_stmt(bc: &mut Bc, p: &Parser, idx: Int) {
  let borrow_mark = bc.nborrows;
  let loan_mark = bc.nloans;
  bc_stmt_inner(bc, p, idx);
  let keeps = bc_stmt_binds_ref(p, idx);
  if !keeps {
    bc_release_borrows_since(bc, borrow_mark);
    bc.nloans = loan_mark;
  }
}

fn bc_stmt_binds_ref(p: &Parser, idx: Int) -> Bool {
  let node = p.nodes[idx];
  match node.kind {
    NkLet(name, ty, value) => { return bc_expr_binds_ref(p, value); }
    NkVar(name, ty, value) => { return bc_expr_binds_ref(p, value); }
    _ => {}
  }
  // Assign keeps a borrow only when the place is a bare ident.
  let value = bc_assign_rhs(p, idx);
  if value >= 0 {
    let place = bc_assign_lhs(p, idx);
    match p.nodes[place].kind {
      NkExprIdent(x) => { return bc_expr_binds_ref(p, value); }
      _ => { return false; }
    }
  }
  return false;
}

fn bc_expr_binds_ref(p: &Parser, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = p.nodes[idx];
  match node.kind {
    NkExprRef(inner) => { return true; }
    NkExprMutRef(inner) => { return true; }
    NkExprUnary(op, inner) => {
      if op == 2 || op == 3 { return true; }
      return false;
    }
    NkExprParen(inner) => { return bc_expr_binds_ref(p, inner); }
    NkExprTuple(items) => {
      var i = 0;
      while i < items.len() {
        if bc_expr_binds_ref(p, items[i]) { return true; }
        i = i + 1;
      }
      return false;
    }
    NkExprArray(items) => {
      var i = 0;
      while i < items.len() {
        if bc_expr_binds_ref(p, items[i]) { return true; }
        i = i + 1;
      }
      return false;
    }
    NkExprStruct(name, fields, base) => {
      var i = 0;
      while i < fields.len() {
        let fnode = p.nodes[fields[i]];
        match fnode.kind {
          NkFieldInit(fname, value) => {
            if bc_expr_binds_ref(p, value) { return true; }
          }
          _ => {}
        }
        i = i + 1;
      }
      return false;
    }
    NkExprSome(inner) => { return bc_expr_binds_ref(p, inner); }
    NkExprOk(inner) => { return bc_expr_binds_ref(p, inner); }
    NkExprErr(inner) => { return bc_expr_binds_ref(p, inner); }
    _ => { return false; }
  }
}

const BS_LET: Int = 1;
const BS_VAR: Int = 2;
const BS_ASSIGN: Int = 3;
const BS_RETURN: Int = 4;
const BS_EXPR: Int = 5;
const BS_IF: Int = 6;
const BS_MATCH: Int = 7;
const BS_WHILE: Int = 8;
const BS_FOR: Int = 9;
const BS_DESTRUCTURE: Int = 10;
const BS_SPAWN: Int = 11;
const BS_DEFER: Int = 12;
const BS_ASSERT: Int = 13;
const BS_OTHER: Int = 0;

fn bc_stmt_tag(p: &Parser, idx: Int) -> Int {
  if idx < 0 { return BS_OTHER; }
  let node = p.nodes[idx];
  match node.kind {
    NkLet(name, ty, value) => { return BS_LET; }
    NkVar(name, ty, value) => { return BS_VAR; }
    NkAssign(l, r) => { return BS_ASSIGN; }
    NkReturn(value) => { return BS_RETURN; }
    NkExprStmt(expr) => { return BS_EXPR; }
    NkStmtIf(cond, then_b, elifs, els) => { return BS_IF; }
    NkStmtMatch(scrut, arms) => { return BS_MATCH; }
    NkWhile(cond, body, inv, label) => { return BS_WHILE; }
    NkFor(name, iter, body, label) => { return BS_FOR; }
    NkDestructure(names, value) => { return BS_DESTRUCTURE; }
    NkStmtSpawn(move_, block) => { return BS_SPAWN; }
    NkDefer(block) => { return BS_DEFER; }
    NkAssert(cond, msg) => { return BS_ASSERT; }
    _ => { return BS_OTHER; }
  }
}

fn bc_stmt_inner(bc: &mut Bc, p: &Parser, idx: Int) {
  let tag = bc_stmt_tag(p, idx);
  if tag == BS_LET { bc_stmt_binding(bc, p, idx, 0); return; }
  if tag == BS_VAR { bc_stmt_binding(bc, p, idx, 1); return; }
  if tag == BS_ASSIGN { bc_stmt_assign(bc, p, idx); return; }
  if tag == BS_RETURN { bc_stmt_return(bc, p, idx); return; }
  if tag == BS_EXPR { bc_stmt_expr(bc, p, idx); return; }
  if tag == BS_IF { bc_stmt_if(bc, p, idx); return; }
  if tag == BS_MATCH { bc_stmt_match(bc, p, idx); return; }
  if tag == BS_WHILE { bc_stmt_while(bc, p, idx); return; }
  if tag == BS_FOR { bc_stmt_for(bc, p, idx); return; }
  if tag == BS_DESTRUCTURE { bc_stmt_destructure(bc, p, idx); return; }
  if tag == BS_SPAWN { bc_stmt_spawn(bc, p, idx); return; }
  if tag == BS_DEFER { bc_stmt_defer(bc, p, idx); return; }
  if tag == BS_ASSERT { bc_stmt_assert(bc, p, idx); return; }
  // Break/Continue/Asm/Debugger/other: nothing.
}

fn bc_stmt_binding(bc: &mut Bc, p: &Parser, idx: Int, is_var: Int) {
  match p.nodes[idx].kind {
    NkLet(name, ty, value) => { bc_binding(bc, p, name, ty, value, is_var); }
    NkVar(name, ty, value) => { bc_binding(bc, p, name, ty, value, is_var); }
    _ => {}
  }
}

fn bc_binding(bc: &mut Bc, p: &Parser, name: Int, ty: Int, value: Int, is_var: Int) {
  let nm = selfhost_check_state.ck_ident_in(p, name);
  let _ = bc_expr(bc, p, value);
  match p.nodes[value].kind {
    NkExprIdent(vname) => {
      let vn = selfhost_check_state.ck_ident_in(p, vname);
      if bc_param_contains(bc, vn) {
        bc_read_borrow(bc, vn, bc_span(p, value));
      } else {
        bc_move_var(bc, vn, bc_span(p, value));
      }
    }
    _ => {}
  }
  var xiom_type = bc_infer_type_from_expr(p, value);
  if ty >= 0 {
    let tn = p.nodes[ty];
    match tn.kind {
      NkTyNamed(tname, args) => {
        let raw = selfhost_check_state.ck_ident_in(p, tname);
        if raw != "_" { xiom_type = bc_param_type_name(p, ty); }
      }
      _ => { xiom_type = bc_param_type_name(p, ty); }
    }
  }
  bc_add_owner(bc, nm, is_var, xiom_type);
}

/// WORKAROUND (COMPILER_BUGS 2026-10-02 Phase 3): the inline `NkAssign`
/// destructure mis-reads payload fields in some functions; route field
/// access through one-arm accessors.
fn bc_assign_lhs(p: &Parser, idx: Int) -> Int {
  match p.nodes[idx].kind {
    NkAssign(l, r) => { return l; }
    _ => { return -1; }
  }
}

fn bc_assign_rhs(p: &Parser, idx: Int) -> Int {
  match p.nodes[idx].kind {
    NkAssign(l, r) => { return r; }
    _ => { return -1; }
  }
}

fn bc_stmt_assign(bc: &mut Bc, p: &Parser, idx: Int) {
  let place = bc_assign_lhs(p, idx);
  let value = bc_assign_rhs(p, idx);
  if place < 0 || value < 0 { return; }
  let _ = bc_expr(bc, p, value);
  match p.nodes[value].kind {
    NkExprIdent(vname) => {
      bc_move_var(bc, selfhost_check_state.ck_ident_in(p, vname), bc_span(p, value));
    }
    _ => {}
  }
  let _ = bc_expr(bc, p, place);
}

fn bc_stmt_return(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkReturn(value) => {
      if value >= 0 {
        let res = bc_expr(bc, p, value);
        if res == ER_READREF || res == ER_WRITEREF {
          bc_err(bc, "cannot return a borrow from a function", bc_span(p, idx));
        }
      }
    }
    _ => {}
  }
}

fn bc_stmt_expr(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkExprStmt(expr) => { let _ = bc_expr(bc, p, expr); }
    _ => {}
  }
}

fn bc_stmt_if(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkStmtIf(cond, then_b, elifs, els) => {
      let _ = bc_expr(bc, p, cond);
      bc_push_scope(bc);
      bc_block(bc, p, then_b);
      bc_pop_scope(bc);
      var i = 0;
      while i < elifs.len() {
        let enode = p.nodes[elifs[i]];
        match enode.kind {
          NkElif(econd, eblock) => {
            let _ = bc_expr(bc, p, econd);
            bc_push_scope(bc);
            bc_block(bc, p, eblock);
            bc_pop_scope(bc);
          }
          _ => {}
        }
        i = i + 1;
      }
      if els >= 0 {
        bc_push_scope(bc);
        bc_block(bc, p, els);
        bc_pop_scope(bc);
      }
    }
    _ => {}
  }
}

fn bc_stmt_match(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkStmtMatch(scrut, arms) => {
      let _ = bc_expr(bc, p, scrut);
      bc_arms(bc, p, arms);
    }
    _ => {}
  }
}

fn bc_arms(bc: &mut Bc, p: &Parser, arms: Vec[Int]) {
  var i = 0;
  while i < arms.len() {
    let anode = p.nodes[arms[i]];
    match anode.kind {
      NkMatchArm(pattern, guard, body, body_is_block) => {
        if body_is_block == 1 {
          bc_push_scope(bc);
          bc_block(bc, p, body);
          bc_pop_scope(bc);
        } else {
          let _ = bc_expr(bc, p, body);
        }
      }
      _ => {}
    }
    i = i + 1;
  }
}

fn bc_stmt_while(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkWhile(cond, body, inv, label) => {
      let _ = bc_expr(bc, p, cond);
      bc_push_scope(bc);
      bc_block(bc, p, body);
      bc_pop_scope(bc);
    }
    _ => {}
  }
}

fn bc_stmt_for(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkFor(name, iter, body, label) => {
      let _ = bc_expr(bc, p, iter);
      bc_add_owner(bc, selfhost_check_state.ck_ident_in(p, name), 1, "Int");
      bc_push_scope(bc);
      bc_block(bc, p, body);
      bc_pop_scope(bc);
    }
    _ => {}
  }
}

fn bc_stmt_destructure(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkDestructure(names, value) => {
      let _ = bc_expr(bc, p, value);
      match p.nodes[value].kind {
        NkExprIdent(vname) => {
          let vn = selfhost_check_state.ck_ident_in(p, vname);
          if bc_param_contains(bc, vn) {
            bc_read_borrow(bc, vn, bc_span(p, value));
          } else {
            bc_move_var(bc, vn, bc_span(p, value));
          }
        }
        _ => {}
      }
      var i = 0;
      while i < names.len() {
        bc_add_owner(bc, selfhost_check_state.ck_ident_in(p, names[i]), 1, "Int");
        i = i + 1;
      }
    }
    _ => {}
  }
}

fn bc_stmt_spawn(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkStmtSpawn(move_, block) => {
      bc_push_scope(bc);
      bc_block(bc, p, block);
      bc_pop_scope(bc);
    }
    _ => {}
  }
}

fn bc_stmt_defer(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkDefer(block) => { bc_block(bc, p, block); }
    _ => {}
  }
}

fn bc_stmt_assert(bc: &mut Bc, p: &Parser, idx: Int) {
  match p.nodes[idx].kind {
    NkAssert(cond, msg) => {
      let _ = bc_expr(bc, p, cond);
      if msg >= 0 { let _ = bc_expr(bc, p, msg); }
    }
    _ => {}
  }
}

fn bc_expr(bc: &mut Bc, p: &Parser, idx: Int) -> Int {
  if idx < 0 { return ER_VALUE; }
  let node = p.nodes[idx];
  match node.kind {
    NkExprIdent(name) => {
      return bc_check_use(bc, selfhost_check_state.ck_ident_in(p, name), bc_span(p, idx));
    }
    NkLitInt(v) => { return ER_VALUE; }
    NkLitBigInt(hi, lo) => { return ER_VALUE; }
    NkLitFloat(lex) => { return ER_VALUE; }
    NkLitStr(data) => { return ER_VALUE; }
    NkLitChar(cp) => { return ER_VALUE; }
    NkLitBool(v) => { return ER_VALUE; }
    NkExprParen(inner) => { return bc_expr(bc, p, inner); }
    NkExprTuple(items) => {
      var i = 0;
      while i < items.len() {
        let _ = bc_expr(bc, p, items[i]);
        i = i + 1;
      }
      return ER_VALUE;
    }
    NkExprUnary(op, inner) => {
      if op == 2 {
        let _ = bc_expr(bc, p, inner);
        bc_borrow_place_read(bc, p, inner, bc_span(p, idx));
        return ER_READREF;
      }
      if op == 3 {
        let _ = bc_expr(bc, p, inner);
        bc_borrow_place_write(bc, p, inner, bc_span(p, idx));
        return ER_WRITEREF;
      }
      let _ = bc_expr(bc, p, inner);
      return ER_VALUE;
    }
    NkExprBinary(l, op, r) => {
      let _ = bc_expr(bc, p, l);
      let _ = bc_expr(bc, p, r);
      return ER_VALUE;
    }
    NkExprTry(inner) => { return bc_expr(bc, p, inner); }
    NkExprImply(l, r) => {
      let _ = bc_expr(bc, p, l);
      let _ = bc_expr(bc, p, r);
      return ER_VALUE;
    }
    NkExprIs(inner, pattern) => {
      let _ = bc_expr(bc, p, inner);
      return ER_VALUE;
    }
    NkExprField(obj, field) => { return bc_expr(bc, p, obj); }
    NkExprCall(callee, args) => { return bc_call(bc, p, callee, args, bc_span(p, idx)); }
    NkExprGenericCall(callee, types, args) => { return bc_call(bc, p, callee, args, bc_span(p, idx)); }
    NkExprIndex(arr, ix) => {
      let _ = bc_expr(bc, p, arr);
      let _ = bc_expr(bc, p, ix);
      return ER_VALUE;
    }
    NkExprAtPre(inner) => { return bc_expr(bc, p, inner); }
    NkExprRef(inner) => {
      let _ = bc_expr(bc, p, inner);
      bc_borrow_place_read(bc, p, inner, bc_span(p, idx));
      return ER_READREF;
    }
    NkExprMutRef(inner) => {
      let _ = bc_expr(bc, p, inner);
      bc_borrow_place_write(bc, p, inner, bc_span(p, idx));
      return ER_WRITEREF;
    }
    NkExprSome(inner) => {
      let _ = bc_expr(bc, p, inner);
      return ER_VALUE;
    }
    NkExprNone => { return ER_VALUE; }
    NkExprOk(inner) => {
      let _ = bc_expr(bc, p, inner);
      return ER_VALUE;
    }
    NkExprErr(inner) => {
      let _ = bc_expr(bc, p, inner);
      return ER_VALUE;
    }
    NkExprStruct(name, fields, base) => {
      var i = 0;
      while i < fields.len() {
        let fnode = p.nodes[fields[i]];
        match fnode.kind {
          NkFieldInit(fname, value) => {
            let res = bc_expr(bc, p, value);
            if res == ER_READREF || res == ER_WRITEREF {
              bc_err(bc, "cannot store borrow in struct", bc_span(p, idx));
            }
          }
          _ => {}
        }
        i = i + 1;
      }
      return ER_VALUE;
    }
    NkExprArray(items) => {
      var i = 0;
      while i < items.len() {
        let _ = bc_expr(bc, p, items[i]);
        i = i + 1;
      }
      return ER_VALUE;
    }
    NkExprClosure(params, ret, body) => { return ER_VALUE; }
    NkExprPipeClosure(names, body) => { return ER_VALUE; }
    NkExprAwait(inner) => { return bc_expr(bc, p, inner); }
    NkExprComptime(inner) => { return bc_expr(bc, p, inner); }
    NkExprUnsafe(block) => {
      bc_block(bc, p, block);
      return ER_VALUE;
    }
    NkExprBlock(block) => {
      bc_block(bc, p, block);
      return ER_VALUE;
    }
    NkExprAs(inner, ty) => {
      let _ = bc_expr(bc, p, inner);
      return ER_VALUE;
    }
    NkExprIf(cond, then_b, elifs, els) => {
      // Rust's borrow pass checks only the conditions here (blocks are
      // checked by the statement walk); mirror it exactly.
      let _ = bc_expr(bc, p, cond);
      var i = 0;
      while i < elifs.len() {
        let enode = p.nodes[elifs[i]];
        match enode.kind {
          NkElif(econd, eblock) => {
            let _ = bc_expr(bc, p, econd);
          }
          _ => {}
        }
        i = i + 1;
      }
      return ER_VALUE;
    }
    NkExprMatch(scrut, arms) => {
      let _ = bc_expr(bc, p, scrut);
      bc_arms(bc, p, arms);
      return ER_VALUE;
    }
    NkExprConstBlock(inner) => { return bc_expr(bc, p, inner); }
    NkExprError => { return ER_VALUE; }
    _ => { return ER_VALUE; }
  }
}

fn bc_call(bc: &mut Bc, p: &Parser, callee: Int, args: Vec[Int], sp: Span) -> Int {
  // Special case: x.clone() read-borrows and immediately releases.
  match p.nodes[callee].kind {
    NkExprField(obj, method) => {
      let mname = selfhost_check_state.ck_ident_in(p, method);
      if mname == "clone" && args.len() == 0 {
        let _ = bc_expr(bc, p, obj);
        match p.nodes[obj].kind {
          NkExprIdent(name) => {
            let nm = selfhost_check_state.ck_ident_in(p, name);
            bc_read_borrow(bc, nm, sp);
            bc_release_borrow(bc, nm, BB_READ);
          }
          _ => {}
        }
        return ER_VALUE;
      }
    }
    _ => {}
  }
  bc_push_scope(bc);
  let _ = bc_expr(bc, p, callee);
  var i = 0;
  while i < args.len() {
    let arg = args[i];
    let res = bc_expr(bc, p, arg);
    match p.nodes[arg].kind {
      NkExprIdent(name) => {
        if res != ER_READREF && res != ER_WRITEREF {
          let nm = selfhost_check_state.ck_ident_in(p, name);
          if bc_param_contains(bc, nm) {
            bc_read_borrow(bc, nm, bc_span(p, arg));
          } else {
            bc_move_var(bc, nm, bc_span(p, arg));
          }
        }
      }
      _ => {}
    }
    i = i + 1;
  }
  bc_pop_scope(bc);
  return ER_VALUE;
}
