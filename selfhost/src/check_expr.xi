// XIOM -- Selfhost checker: statements + expressions (Phase 3)
// Copyright (c) 2026 Elefterios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Port of `Checker::check_block` / `check_stmt` / `check_expr` for the
// stage-1 subset (docs/checklists/selfhost-phase3.md):
//   * let/var/assign/return compatibility through the canonical type names;
//   * if/while Bool conditions; for-in element binding;
//   * binary operator typing + the int/float mix rules + W008 (div/rem by a
//     zero literal);
//   * W003 unreachable-statement divergence analysis;
//   * bare calls / user-type methods / constructors with Rust's arity +
//     argument compatibility; permissive fallback ("_") for the catalog,
//     containers and unresolved dispatch, exactly where the Rust checker
//     would resolve through a registered signature the stage-1 port does
//     not carry.
//
// Every message string and span choice mirrors the Rust site that owns it
// (line references in the comments). Errors are emitted once per root cause
// (Checker::error poisoning): callers propagate "<error>" and downstream
// display checks skip it.

module selfhost_check_expr

use xiom.string;
use selfhost_ast.Span;
use selfhost_ast.NodeKind;
use selfhost_check_state;
use selfhost_check_state.Checker;
use selfhost_check_state.FnParam;
use selfhost_check_state.Field;
use selfhost_check_types;

// ============================================================================
// Literal / small AST predicates
// ============================================================================

pub fn ce_is_int_literal(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkLitInt(v) => { return true; }
    NkLitBigInt(hi, lo) => { return true; }
    NkExprParen(inner) => { return ce_is_int_literal(c, inner); }
    _ => { return false; }
  }
}

pub fn ce_is_zero_int_literal(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkLitInt(v) => {
      if v == 0 { return true; }
      return false;
    }
    NkLitBigInt(hi, lo) => {
      if hi == 0 && lo == 0 { return true; }
      return false;
    }
    NkExprParen(inner) => { return ce_is_zero_int_literal(c, inner); }
    NkExprUnary(op, inner) => {
      if op == selfhost_check_state_op_neg() {
        return ce_is_zero_int_literal(c, inner);
      }
      return false;
    }
    _ => { return false; }
  }
}

fn selfhost_check_state_op_neg() -> Int {
  return 0;
}

fn ce_is_placeholder_zero(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkLitInt(v) => {
      if v == 0 { return true; }
      return false;
    }
    _ => { return false; }
  }
}

fn ce_is_ref_expr(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprRef(inner) => { return true; }
    NkExprMutRef(inner) => { return true; }
    _ => { return false; }
  }
}

fn ce_is_wildcard_annot(c: &Checker, ty: Int) -> Bool {
  if ty < 0 { return false; }
  let t = c.p.nodes[ty];
  match t.kind {
    NkTyNamed(name, args) => {
      if ck_ident(c, name) == "_" { return true; }
      return false;
    }
    _ => { return false; }
  }
}

fn ce_is_lenient_cond(ty: Str) -> Bool {
  if selfhost_check_types.ct_is_tuple_like(ty) { return true; }
  if ty == "_" { return true; }
  if selfhost_check_types.ct_is_generic_param(ty) { return true; }
  return false;
}

/// Container receivers whose method sets live in the catalog: the stage-1
/// port defers them (returns "_") instead of false-flagging. Mirrors the
/// Rust `container_receiver` list.
pub fn ce_is_container_base_leaf(leaf: Str) -> Bool {
  if leaf == "Vec" { return true; }
  if leaf == "Slice" { return true; }
  if leaf == "Array" { return true; }
  if leaf == "Map" { return true; }
  if leaf == "Set" { return true; }
  if leaf == "HashMap" { return true; }
  if leaf == "BTreeMap" { return true; }
  if leaf == "Option" { return true; }
  if leaf == "Result" { return true; }
  if leaf == "Tuple" { return true; }
  if leaf == "Stack" { return true; }
  if leaf == "Deque" { return true; }
  if leaf == "Queue" { return true; }
  return false;
}

/// Last dotted component, bracket-stripped (`Vec[Int]` -> `Vec`).
fn ce_leaf_name(s: Str) -> Str {
  var i = s.len() - 1;
  while i >= 0 {
    if (string.byte_at(s, i) as Int) == 46 {
      var rest = string.str_slice(s, i + 1, s.len());
      return ce_strip_bracket(rest);
    }
    i = i - 1;
  }
  return ce_strip_bracket(s);
}

fn ce_strip_bracket(s: Str) -> Str {
  var i = 0;
  while i < s.len() {
    if (string.byte_at(s, i) as Int) == 91 {
      return string.str_slice(s, 0, i);
    }
    i = i + 1;
  }
  return s;
}

fn ce_strip_ref_marks(s: Str) -> Str {
  var t = selfhost_check_types.ct_trim(s);
  while t.len() > 0 {
    if (string.byte_at(t, 0) as Int) == 38 {
      t = selfhost_check_types.ct_trim(string.str_slice(t, 1, t.len()));
      if t.len() >= 4 && string.str_starts_with(t, "mut ") {
        t = selfhost_check_types.ct_trim(string.str_slice(t, 4, t.len()));
      }
    } else {
      break;
    }
  }
  return t;
}

// ============================================================================
// W003 divergence analysis (port of the Stage 6 helpers)
// ============================================================================

fn ce_span_index(c: &Checker, item: Int) -> Span {
  return ck_span_of(c, item);
}

pub fn ce_stmt_or_expr_span(c: &Checker, item: Int) -> Span {
  if item < 0 { return Span{ line: 0, col: 0, byte_start: 0, byte_end: 0 }; }
  let node = c.p.nodes[item];
  match node.kind {
    NkStmtW(s) => { return ck_span_of(c, s); }
    NkTailW(e) => { return ck_span_of(c, e); }
    _ => { return ck_span_of(c, item); }
  }
}

pub fn ce_block_always_returns(c: &Checker, block_idx: Int) -> Bool {
  if block_idx < 0 { return false; }
  let node = c.p.nodes[block_idx];
  match node.kind {
    NkBlock(stmts) => {
      var i = 0;
      while i < stmts.len() {
        let item = c.p.nodes[stmts[i]];
        match item.kind {
          NkStmtW(s) => {
            if ce_stmt_always_returns(c, s) { return true; }
          }
          NkTailW(e) => {
            if ce_expr_always_returns(c, e) { return true; }
          }
          _ => {}
        }
        i = i + 1;
      }
      return false;
    }
    _ => { return false; }
  }
}

pub fn ce_stmt_always_returns(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkReturn(value) => { return true; }
    NkExprStmt(expr) => { return ce_expr_always_returns(c, expr); }
    NkStmtIf(cond, then_b, elifs, els) => {
      if els < 0 { return false; }
      if !ce_block_always_returns(c, then_b) { return false; }
      var i = 0;
      while i < elifs.len() {
        let enode = c.p.nodes[elifs[i]];
        match enode.kind {
          NkElif(econd, eblock) => {
            if !ce_block_always_returns(c, eblock) { return false; }
          }
          _ => { return false; }
        }
        i = i + 1;
      }
      return ce_block_always_returns(c, els);
    }
    NkStmtMatch(scrut, arms) => {
      if arms.len() == 0 { return false; }
      return ce_all_arms_return(c, arms);
    }
    _ => { return false; }
  }
}

pub fn ce_expr_always_returns(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprUnsafe(block) => { return ce_block_always_returns(c, block); }
    NkExprBlock(block) => { return ce_block_always_returns(c, block); }
    NkExprParen(inner) => { return ce_expr_always_returns(c, inner); }
    NkExprIf(cond, then_b, elifs, els) => {
      if els < 0 { return false; }
      if !ce_block_always_returns(c, then_b) { return false; }
      var i = 0;
      while i < elifs.len() {
        let enode = c.p.nodes[elifs[i]];
        match enode.kind {
          NkElif(econd, eblock) => {
            if !ce_block_always_returns(c, eblock) { return false; }
          }
          _ => { return false; }
        }
        i = i + 1;
      }
      return ce_block_always_returns(c, els);
    }
    NkExprMatch(scrut, arms) => {
      if arms.len() == 0 { return false; }
      return ce_all_arms_return(c, arms);
    }
    _ => { return false; }
  }
}

fn ce_all_arms_return(c: &Checker, arms: Vec[Int]) -> Bool {
  var i = 0;
  while i < arms.len() {
    let anode = c.p.nodes[arms[i]];
    match anode.kind {
      NkMatchArm(pattern, guard, body, body_is_block) => {
        if body_is_block == 1 {
          if !ce_block_always_returns(c, body) { return false; }
        } else {
          if !ce_expr_always_returns(c, body) { return false; }
        }
      }
      _ => { return false; }
    }
    i = i + 1;
  }
  return true;
}

fn ce_stmt_always_diverges(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkReturn(value) => { return true; }
    NkBreak(label) => { return true; }
    NkContinue(label) => { return true; }
    NkExprStmt(expr) => { return ce_expr_always_diverges(c, expr); }
    NkStmtIf(cond, then_b, elifs, els) => {
      if els < 0 { return false; }
      if !ce_block_always_diverges(c, then_b) { return false; }
      var i = 0;
      while i < elifs.len() {
        let enode = c.p.nodes[elifs[i]];
        match enode.kind {
          NkElif(econd, eblock) => {
            if !ce_block_always_diverges(c, eblock) { return false; }
          }
          _ => { return false; }
        }
        i = i + 1;
      }
      return ce_block_always_diverges(c, els);
    }
    NkStmtMatch(scrut, arms) => {
      return ce_match_arms_all_diverge(c, arms);
    }
    NkWhile(cond, body, inv, label) => {
      if !ce_is_true_literal(c, cond) { return false; }
      if ce_block_has_break(c, body) { return false; }
      return true;
    }
    _ => { return false; }
  }
}

fn ce_expr_always_diverges(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprUnsafe(block) => { return ce_block_always_diverges(c, block); }
    NkExprBlock(block) => { return ce_block_always_diverges(c, block); }
    NkExprParen(inner) => { return ce_expr_always_diverges(c, inner); }
    NkExprIf(cond, then_b, elifs, els) => {
      if els < 0 { return false; }
      if !ce_block_always_diverges(c, then_b) { return false; }
      var i = 0;
      while i < elifs.len() {
        let enode = c.p.nodes[elifs[i]];
        match enode.kind {
          NkElif(econd, eblock) => {
            if !ce_block_always_diverges(c, eblock) { return false; }
          }
          _ => { return false; }
        }
        i = i + 1;
      }
      return ce_block_always_diverges(c, els);
    }
    NkExprMatch(scrut, arms) => {
      return ce_match_arms_all_diverge(c, arms);
    }
    _ => { return false; }
  }
}

fn ce_block_always_diverges(c: &Checker, block_idx: Int) -> Bool {
  if block_idx < 0 { return false; }
  let node = c.p.nodes[block_idx];
  match node.kind {
    NkBlock(stmts) => {
      var i = 0;
      while i < stmts.len() {
        if ce_item_always_diverges(c, stmts[i]) { return true; }
        i = i + 1;
      }
      return false;
    }
    _ => { return false; }
  }
}

/// W003 match diverger: at least one catch-all arm AND every arm diverges
/// (Rust `match_arms_all_diverge`, lib.rs 5141).
fn ce_match_arms_all_diverge(c: &Checker, arms: Vec[Int]) -> Bool {
  if arms.len() == 0 { return false; }
  var has_catch_all = false;
  var i = 0;
  while i < arms.len() {
    let anode = c.p.nodes[arms[i]];
    match anode.kind {
      NkMatchArm(pattern, guard, body, body_is_block) => {
        if ce_pattern_is_catch_all(c, pattern) { has_catch_all = true; }
      }
      _ => {}
    }
    i = i + 1;
  }
  if !has_catch_all { return false; }
  i = 0;
  while i < arms.len() {
    let anode = c.p.nodes[arms[i]];
    match anode.kind {
      NkMatchArm(pattern, guard, body, body_is_block) => {
        if body_is_block == 1 {
          if !ce_block_always_diverges(c, body) { return false; }
        } else {
          if !ce_expr_always_diverges(c, body) { return false; }
        }
      }
      _ => { return false; }
    }
    i = i + 1;
  }
  return true;
}

fn ce_pattern_is_catch_all(c: &Checker, pat: Int) -> Bool {
  if pat < 0 { return false; }
  let node = c.p.nodes[pat];
  match node.kind {
    NkPatWildcard => { return true; }
    NkPatIdent(name) => {
      let nm = ck_ident(c, name);
      var has_dot = false;
      var i = 0;
      while i < nm.len() {
        if (string.byte_at(nm, i) as Int) == 46 { has_dot = true; }
        i = i + 1;
      }
      if !has_dot { return true; }
      return false;
    }
    NkPatOr(items) => {
      var i = 0;
      while i < items.len() {
        if ce_pattern_is_catch_all(c, items[i]) { return true; }
        i = i + 1;
      }
      return false;
    }
    _ => { return false; }
  }
}

fn ce_is_true_literal(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkLitBool(v) => {
      if v == 1 { return true; }
      return false;
    }
    _ => { return false; }
  }
}

fn ce_block_has_break(c: &Checker, block_idx: Int) -> Bool {
  if block_idx < 0 { return false; }
  let node = c.p.nodes[block_idx];
  match node.kind {
    NkBlock(stmts) => {
      var i = 0;
      while i < stmts.len() {
        let inode = c.p.nodes[stmts[i]];
        match inode.kind {
          NkStmtW(s) => {
            if ce_stmt_has_break(c, s) { return true; }
          }
          NkTailW(e) => {
            if ce_expr_has_break(c, e) { return true; }
          }
          _ => {}
        }
        i = i + 1;
      }
      return false;
    }
    _ => { return false; }
  }
}

fn ce_stmt_has_break(c: &Checker, idx: Int) -> Bool {
  let node = c.p.nodes[idx];
  match node.kind {
    NkBreak(label) => { return true; }
    NkStmtIf(cond, then_b, elifs, els) => {
      if ce_block_has_break(c, then_b) { return true; }
      var i = 0;
      while i < elifs.len() {
        let enode = c.p.nodes[elifs[i]];
        match enode.kind {
          NkElif(econd, eblock) => {
            if ce_block_has_break(c, eblock) { return true; }
          }
          _ => {}
        }
        i = i + 1;
      }
      if els >= 0 { return ce_block_has_break(c, els); }
      return false;
    }
    NkStmtMatch(scrut, arms) => {
      var i = 0;
      while i < arms.len() {
        let anode = c.p.nodes[arms[i]];
        match anode.kind {
          NkMatchArm(pattern, guard, body, body_is_block) => {
            if body_is_block == 1 {
              if ce_block_has_break(c, body) { return true; }
            } else {
              if ce_expr_has_break(c, body) { return true; }
            }
          }
          _ => {}
        }
        i = i + 1;
      }
      return false;
    }
    NkWhile(cond, body, inv, label) => {
      if ce_block_has_break(c, body) { return true; }
      return false;
    }
    NkFor(name, iter, body, label) => {
      if ce_block_has_break(c, body) { return true; }
      return false;
    }
    _ => { return false; }
  }
}

fn ce_expr_has_break(c: &Checker, idx: Int) -> Bool {
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprBlock(block) => { return ce_block_has_break(c, block); }
    NkExprUnsafe(block) => { return ce_block_has_break(c, block); }
    NkExprParen(inner) => { return ce_expr_has_break(c, inner); }
    NkExprIf(cond, then_b, elifs, els) => {
      if ce_block_has_break(c, then_b) { return true; }
      var i = 0;
      while i < elifs.len() {
        let enode = c.p.nodes[elifs[i]];
        match enode.kind {
          NkElif(econd, eblock) => {
            if ce_block_has_break(c, eblock) { return true; }
          }
          _ => {}
        }
        i = i + 1;
      }
      if els >= 0 { return ce_block_has_break(c, els); }
      return false;
    }
    NkExprMatch(scrut, arms) => {
      var i = 0;
      while i < arms.len() {
        let anode = c.p.nodes[arms[i]];
        match anode.kind {
          NkMatchArm(pattern, guard, body, body_is_block) => {
            if body_is_block == 1 {
              if ce_block_has_break(c, body) { return true; }
            } else {
              if ce_expr_has_break(c, body) { return true; }
            }
          }
          _ => {}
        }
        i = i + 1;
      }
      return false;
    }
    _ => { return false; }
  }
}

pub fn ce_item_always_diverges(c: &Checker, item: Int) -> Bool {
  if item < 0 { return false; }
  let node = c.p.nodes[item];
  match node.kind {
    NkStmtW(s) => { return ce_stmt_always_diverges(c, s); }
    NkTailW(e) => { return ce_expr_always_diverges(c, e); }
    _ => { return false; }
  }
}

// ============================================================================
// Pattern bindings (permissive: payload names bind "_")
// ============================================================================

fn ce_bind_pattern(c: &mut Checker, pat: Int, scrut_ty: Str) {
  if pat < 0 { return; }
  let node = c.p.nodes[pat];
  match node.kind {
    NkPatIdent(name) => {
      let nm = ck_ident(c, name);
      if nm.len() > 0 { ck_add_local(c, nm, "_"); }
    }
    NkPatSome(inner) => { ce_bind_pattern(c, inner, "_"); }
    NkPatOk(inner) => { ce_bind_pattern(c, inner, "_"); }
    NkPatErr(inner) => { ce_bind_pattern(c, inner, "_"); }
    NkPatTuple(items) => {
      var i = 0;
      while i < items.len() {
        ce_bind_pattern(c, items[i], "_");
        i = i + 1;
      }
    }
    NkPatOr(items) => {
      var i = 0;
      while i < items.len() {
        ce_bind_pattern(c, items[i], "_");
        i = i + 1;
      }
    }
    NkPatVariant(name, fields) => {
      var i = 0;
      while i < fields.len() {
        ce_bind_pattern(c, fields[i], "_");
        i = i + 1;
      }
    }
    NkPatStruct(name, fields) => {
      var i = 0;
      while i < fields.len() {
        ce_bind_pattern(c, fields[i], "_");
        i = i + 1;
      }
    }
    NkPatField(name, pattern) => { ce_bind_pattern(c, pattern, "_"); }
    _ => {}
  }
}

// ============================================================================
// Blocks and statements
// ============================================================================

/// `expected`/`has_expected`: the function's declared return type (Rust
/// `check_block(block, Some/None)`). Returns the tail expression's type
/// ("" = none).
pub fn ck_check_block(c: &mut Checker, block_idx: Int, expected: Str, has_expected: Int) -> Str {
  if block_idx < 0 { return ""; }
  let bnode = c.p.nodes[block_idx];
  match bnode.kind {
    NkBlock(stmts) => {
      ck_push_scope(c);
      var last_ty = "";
      var has_return = false;
      var tail_diverges = false;
      var w003_diverged = false;
      var w003_warned = false;
      var i = 0;
      while i < stmts.len() {
        let item = stmts[i];
        if w003_diverged && !w003_warned {
          let sp = ce_stmt_or_expr_span(c, item);
          ck_warn_coded_at(c, "W003", "unreachable statement (the previous statement always exits)", sp.line, sp.col);
          w003_warned = true;
        }
        let inode = c.p.nodes[item];
        match inode.kind {
          NkStmtW(stmt) => {
            ck_check_stmt(c, stmt);
            if ce_stmt_always_returns(c, stmt) {
              has_return = true;
              last_ty = "";
            }
            tail_diverges = false;
          }
          NkTailW(expr) => {
            last_ty = ce_check_expr(c, expr);
            tail_diverges = ce_expr_always_returns(c, expr);
          }
          _ => {}
        }
        if ce_item_always_diverges(c, item) {
          w003_diverged = true;
        }
        i = i + 1;
      }
      if has_expected == 1 && !has_return && !tail_diverges {
        if last_ty.len() > 0 {
          if last_ty != "<error>" && expected != "<error>" {
            if !ck_types_compatible(c, last_ty, expected) {
              let sp = ck_span_of(c, block_idx);
              let _ = ck_error_at(c, "return type mismatch: expected " + expected + ", found " + last_ty, sp.line, sp.col);
            }
          }
        }
      }
      ck_pop_scope(c);
      return last_ty;
    }
    _ => { return ""; }
  }
}

fn ce_stmt_span(c: &Checker, idx: Int) -> Span {
  return ck_span_of(c, idx);
}

// ============================================================================
// Statement dispatch
// ============================================================================
//
// WORKAROUND (COMPILER_BUGS 2026-10-02 Phase 3 finding): a large
// `match stmt.kind { ... }` dispatch silently lost the rest of the enclosing
// function when one arm was an `NkAssign` destructure (repro: the Phase 3
// checker's `ck_check_stmt` printed NOTHING for `x = 2;` while exit stayed
// 0). Identical in shape to the Phase 2 finding (h) `NkExprGenericCall`
// destructure mis-mapping in a large dispatch; the established workaround is
// a small Int-TAG selector (`ce_stmt_tag`) plus per-kind helpers.

pub const ST_LET: Int = 1;
pub const ST_VAR: Int = 2;
pub const ST_ASSIGN: Int = 3;
pub const ST_RETURN: Int = 4;
pub const ST_EXPR: Int = 5;
pub const ST_IF: Int = 6;
pub const ST_MATCH: Int = 7;
pub const ST_WHILE: Int = 8;
pub const ST_FOR: Int = 9;
pub const ST_DESTRUCTURE: Int = 10;
pub const ST_BREAK: Int = 11;
pub const ST_CONTINUE: Int = 12;
pub const ST_ASSERT: Int = 13;
pub const ST_DEFER: Int = 14;
pub const ST_SPAWN: Int = 15;
pub const ST_ASM: Int = 16;
pub const ST_DEBUGGER: Int = 17;
pub const ST_OTHER: Int = 0;

fn ce_stmt_tag(c: &Checker, idx: Int) -> Int {
  if idx < 0 { return ST_OTHER; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkLet(name, ty, value) => { return ST_LET; }
    NkVar(name, ty, value) => { return ST_VAR; }
    NkAssign(l, r) => { return ST_ASSIGN; }
    NkReturn(value) => { return ST_RETURN; }
    NkExprStmt(expr) => { return ST_EXPR; }
    NkStmtIf(cond, then_b, elifs, els) => { return ST_IF; }
    NkStmtMatch(scrut, arms) => { return ST_MATCH; }
    NkWhile(cond, body, inv, label) => { return ST_WHILE; }
    NkFor(name, iter, body, label) => { return ST_FOR; }
    NkDestructure(names, value) => { return ST_DESTRUCTURE; }
    NkBreak(label) => { return ST_BREAK; }
    NkContinue(label) => { return ST_CONTINUE; }
    NkAssert(cond, msg) => { return ST_ASSERT; }
    NkDefer(block) => { return ST_DEFER; }
    NkStmtSpawn(move_, block) => { return ST_SPAWN; }
    NkAsm(template, outputs, inputs, clobbers) => { return ST_ASM; }
    NkDebugger => { return ST_DEBUGGER; }
    _ => { return ST_OTHER; }
  }
}

pub fn ck_check_stmt(c: &mut Checker, idx: Int) {
  let tag = ce_stmt_tag(c, idx);
  if tag == ST_LET { ce_stmt_let(c, idx); return; }
  if tag == ST_VAR { ce_stmt_var(c, idx); return; }
  if tag == ST_ASSIGN { ce_stmt_assign(c, idx); return; }
  if tag == ST_RETURN { ce_stmt_return(c, idx); return; }
  if tag == ST_EXPR { ce_stmt_expr(c, idx); return; }
  if tag == ST_IF { ce_stmt_if(c, idx); return; }
  if tag == ST_MATCH { ce_stmt_match(c, idx); return; }
  if tag == ST_WHILE { ce_stmt_while(c, idx); return; }
  if tag == ST_FOR { ce_stmt_for(c, idx); return; }
  if tag == ST_DESTRUCTURE { ce_stmt_destructure(c, idx); return; }
  if tag == ST_ASSERT { ce_stmt_assert(c, idx); return; }
  if tag == ST_DEFER { ce_stmt_block_child(c, idx); return; }
  if tag == ST_SPAWN { ce_stmt_block_child(c, idx); return; }
  // Break/Continue/Asm/Debugger/Other: nothing to check.
}

fn ce_stmt_let(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkLet(name, ty, value) => { ce_check_binding(c, name, ty, value, idx, 0); }
    _ => {}
  }
}

fn ce_stmt_var(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkVar(name, ty, value) => { ce_check_binding(c, name, ty, value, idx, 1); }
    _ => {}
  }
}

fn ce_stmt_expr(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprStmt(expr) => { let _ = ce_check_expr(c, expr); }
    _ => {}
  }
}

fn ce_stmt_if(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkStmtIf(cond, then_b, elifs, els) => { ce_check_if_parts(c, cond, then_b, elifs, els); }
    _ => {}
  }
}

fn ce_stmt_match(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkStmtMatch(scrut, arms) => {
      let matched_ty = ce_check_expr(c, scrut);
      ce_check_arms(c, arms, matched_ty);
    }
    _ => {}
  }
}

fn ce_stmt_while(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkWhile(cond, body, inv, label) => {
      let cond_ty = ce_check_expr(c, cond);
      if cond_ty != "Bool" && cond_ty != "<error>" {
        let sp = ck_span_of(c, cond);
        let _ = ck_error_at(c, "while condition must be Bool, found " + cond_ty, sp.line, sp.col);
      }
      let _ = ck_check_block(c, body, "", 0);
    }
    _ => {}
  }
}

fn ce_stmt_for(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkFor(name, iter, body, label) => {
      let iter_ty = ce_check_expr(c, iter);
      let var_ty = ce_for_element_type(c, iter, iter_ty);
      ck_push_scope(c);
      ck_add_local(c, ck_ident(c, name), var_ty);
      let _ = ck_check_block(c, body, "", 0);
      ck_pop_scope(c);
    }
    _ => {}
  }
}

fn ce_stmt_destructure(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkDestructure(names, value) => {
      let _ = ce_check_expr(c, value);
      var i = 0;
      while i < names.len() {
        ck_add_local(c, ck_ident(c, names[i]), "_");
        i = i + 1;
      }
    }
    _ => {}
  }
}

fn ce_stmt_assert(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkAssert(cond, msg) => {
      let _ = ce_check_expr(c, cond);
      if msg >= 0 { let _ = ce_check_expr(c, msg); }
    }
    _ => {}
  }
}

fn ce_stmt_block_child(c: &mut Checker, idx: Int) {
  let tag = ce_stmt_tag(c, idx);
  let node = c.p.nodes[idx];
  if tag == ST_SPAWN {
    match node.kind {
      NkStmtSpawn(move_, block) => { let _ = ck_check_block(c, block, "", 0); }
      _ => {}
    }
    return;
  }
  match node.kind {
    NkDefer(block) => { let _ = ck_check_block(c, block, "", 0); }
    _ => {}
  }
}

fn ce_assign_lhs(c: &Checker, idx: Int) -> Int {
  match c.p.nodes[idx].kind {
    NkAssign(l, r) => { return l; }
    _ => { return -1; }
  }
}

fn ce_assign_rhs(c: &Checker, idx: Int) -> Int {
  match c.p.nodes[idx].kind {
    NkAssign(l, r) => { return r; }
    _ => { return -1; }
  }
}

fn ce_stmt_assign(c: &mut Checker, idx: Int) {
  // WORKAROUND (COMPILER_BUGS Phase 3 finding): the inline `NkAssign(place,
  // value)` destructure in this function returns a garbage second field
  // (pointer bits), while the identical match in the one-arm helpers above
  // returns the right indices. Field access goes through the helpers.
  let place = ce_assign_lhs(c, idx);
  let value = ce_assign_rhs(c, idx);
  let place_ty = ce_check_expr(c, place);
  let val_ty = ce_check_expr(c, value);
  let bool_int = (place_ty == "Bool" && val_ty == "Int") || (place_ty == "Int" && val_ty == "Bool");
  if !bool_int && !ck_types_compatible(c, place_ty, val_ty) && place_ty != "<error>" && val_ty != "<error>" {
    let sp = ce_stmt_span(c, idx);
    let _ = ck_error_at(c, "assignment type mismatch: " + place_ty + " = " + val_ty, sp.line, sp.col);
  }
}

fn ce_stmt_return(c: &mut Checker, idx: Int) {
  let node = c.p.nodes[idx];
  match node.kind {
    NkReturn(value) => {
      var ret_ty = "()";
      if value >= 0 { ret_ty = ce_check_expr(c, value); }
      if c.cur_ret_set == 1 {
        if !ck_types_compatible(c, ret_ty, c.cur_ret) && ret_ty != "<error>" {
          let sp = ce_stmt_span(c, idx);
          let _ = ck_error_at(c, "return type mismatch: expected " + c.cur_ret + ", found " + ret_ty, sp.line, sp.col);
        }
      }
    }
    _ => {}
  }
}

fn ce_check_if_parts(c: &mut Checker, cond: Int, then_b: Int, elifs: Vec[Int], els: Int) {
  let cond_ty = ce_check_expr(c, cond);
  if cond_ty != "Bool" && cond_ty != "<error>" && !ce_is_lenient_cond(cond_ty) {
    let sp = ck_span_of(c, cond);
    let _ = ck_error_at(c, "if condition must be Bool, found " + cond_ty, sp.line, sp.col);
  }
  let _ = ck_check_block(c, then_b, "", 0);
  var i = 0;
  while i < elifs.len() {
    let enode = c.p.nodes[elifs[i]];
    match enode.kind {
      NkElif(econd, eblock) => {
        let econd_ty = ce_check_expr(c, econd);
        if econd_ty != "Bool" && econd_ty != "<error>" && !ce_is_lenient_cond(econd_ty) {
          let sp = ck_span_of(c, econd);
          let _ = ck_error_at(c, "elif condition must be Bool, found " + econd_ty, sp.line, sp.col);
        }
        let _ = ck_check_block(c, eblock, "", 0);
      }
      _ => {}
    }
    i = i + 1;
  }
  if els >= 0 {
    let _ = ck_check_block(c, els, "", 0);
  }
}

fn ce_check_binding(c: &mut Checker, name: Int, ty: Int, value: Int, stmt_idx: Int, is_var: Int) {
  let nm = ck_ident(c, name);
  let val_ty = ce_check_expr(c, value);
  let what = if is_var == 1 { "var" } else { "let" };
  if ty >= 0 {
    if ce_is_wildcard_annot(c, ty) {
      ck_add_local(c, nm, val_ty);
      return;
    }
    let annot_ty = ck_type_from_ast(c, ty);
    if ce_is_placeholder_zero(c, value) {
      ck_add_local(c, nm, annot_ty);
      return;
    }
    let is_ref_coercion = ce_is_ref_expr(c, value)
      && (annot_ty.len() > 1)
      && ((string.byte_at(annot_ty, 0) as Int) == 42)
      && string.str_slice(annot_ty, 1, annot_ty.len()) == val_ty;
    let bind_mix_err = (selfhost_check_types.ct_is_int_family(val_ty)
        && selfhost_check_types.ct_is_float_family(annot_ty)
        && !ce_is_int_literal(c, value))
      || (selfhost_check_types.ct_is_float_family(val_ty)
        && selfhost_check_types.ct_is_int_family(annot_ty));
    let sp = ce_stmt_span(c, stmt_idx);
    if bind_mix_err {
      let _ = ck_error_at(c, "type mismatch in " + what + ": cannot bind " + val_ty + " to " + annot_ty + " -- convert explicitly with `as`", sp.line, sp.col);
    }
    if !ck_types_compatible(c, val_ty, annot_ty) && val_ty != "<error>" && val_ty != "_" && !is_ref_coercion {
      let _ = ck_error_at(c, "type mismatch in " + what + ": annotated " + annot_ty + ", found " + val_ty, sp.line, sp.col);
    }
    ck_add_local(c, nm, annot_ty);
    return;
  }
  ck_add_local(c, nm, ce_inferred_binding_type(c, value, val_ty));
}

/// R70: array literals register `Vec[elem]`; other ctors keep their value
/// type (stage 1; Rust also generalizes generic ctors).
fn ce_inferred_binding_type(c: &Checker, value: Int, val_ty: Str) -> Str {
  if value < 0 { return val_ty; }
  let node = c.p.nodes[value];
  match node.kind {
    NkExprArray(items) => {
      if items.len() > 0 {
        let elem = ce_check_expr(c, items[0]);
        return "Vec[" + elem + "]";
      }
      return val_ty;
    }
    NkExprParen(inner) => { return ce_inferred_binding_type(c, inner, val_ty); }
    _ => { return val_ty; }
  }
}

fn ce_check_arms(c: &mut Checker, arms: Vec[Int], matched_ty: Str) {
  var i = 0;
  while i < arms.len() {
    let anode = c.p.nodes[arms[i]];
    match anode.kind {
      NkMatchArm(pattern, guard, body, body_is_block) => {
        ck_push_scope(c);
        ce_bind_pattern(c, pattern, matched_ty);
        if guard >= 0 { let _ = ce_check_expr(c, guard); }
        if body_is_block == 1 {
          let _ = ck_check_block(c, body, "", 0);
        } else {
          let _ = ce_check_expr(c, body);
        }
        ck_pop_scope(c);
      }
      _ => {}
    }
    i = i + 1;
  }
}

/// `CheckedType::for_loop_element_type` (Vec/Slice/Set/Array -> elem, else Int).
fn ce_for_element_type(c: &Checker, iter: Int, iter_ty: Str) -> Str {
  let name0 = selfhost_check_types.ct_trim(iter_ty);
  var name = ce_strip_leading_refs(name0);
  let bases = Vec[Str].new();
  bases.push("Vec");
  bases.push("Slice");
  bases.push("Set");
  var b = 0;
  while b < bases.len() {
    if string.str_starts_with(name, bases[b] + "[") {
      let args = selfhost_check_types.ct_args(name);
      if args.len() == 1 { return args[0]; }
    }
    b = b + 1;
  }
  if (string.byte_at(name, 0) as Int) == 91 {
    let close = ce_find_byte(name, 93);
    if close > 0 {
      var elem = selfhost_check_types.ct_trim(string.str_slice(name, close + 1, name.len()));
      if elem.len() > 0 { return selfhost_check_types.ct_from_str(elem); }
    }
  }
  if name == "Range" || string.str_starts_with(name, "Range[") { return "Int"; }
  return "Int";
}

fn ce_strip_leading_refs(name: Str) -> Str {
  var t = selfhost_check_types.ct_trim(name);
  while t.len() > 0 {
    if (string.byte_at(t, 0) as Int) == 38 {
      t = selfhost_check_types.ct_trim(string.str_slice(t, 1, t.len()));
      if t.len() >= 4 && string.str_starts_with(t, "mut ") {
        t = selfhost_check_types.ct_trim(string.str_slice(t, 4, t.len()));
      }
    } else {
      break;
    }
  }
  return t;
}

fn ce_find_byte(s: Str, ch: Int) -> Int {
  var i = 0;
  while i < s.len() {
    if (string.byte_at(s, i) as Int) == ch { return i; }
    i = i + 1;
  }
  return -1;
}

// ============================================================================
// Expressions
// ============================================================================

pub fn ce_check_expr(c: &mut Checker, idx: Int) -> Str {
  if idx < 0 { return "()"; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprIdent(name_idx) => { return ce_check_ident(c, idx, name_idx); }
    NkLitInt(v) => { return "Int"; }
    NkLitBigInt(hi, lo) => { return "Int"; }
    NkLitFloat(lex) => { return "Float64"; }
    NkLitStr(data) => { return "Str"; }
    NkLitChar(cp) => { return "Char"; }
    NkLitBool(v) => { return "Bool"; }
    NkExprParen(inner) => { return ce_check_expr(c, inner); }
    NkExprAtPre(inner) => { return ce_check_expr(c, inner); }
    NkExprUnary(op, inner) => { return ce_check_unary(c, idx, op, inner); }
    NkExprBinary(l, op, r) => { return ce_check_binary(c, idx, l, op, r); }
    NkExprTry(inner) => { return ce_check_try(c, idx, inner); }
    NkExprImply(l, r) => {
      let _ = ce_check_expr(c, l);
      let _ = ce_check_expr(c, r);
      return "Bool";
    }
    NkExprIs(inner, pattern) => {
      let _ = ce_check_expr(c, inner);
      ck_push_scope(c);
      ce_bind_pattern(c, pattern, "_");
      ck_pop_scope(c);
      return "Bool";
    }
    NkExprField(inner, name) => { return ce_check_field(c, idx, inner, name); }
    NkExprCall(callee, args) => { return ce_check_call(c, idx, callee, args); }
    NkExprGenericCall(callee, types, args) => { return ce_check_generic_call(c, idx, callee, types, args); }
    NkExprIndex(base, index) => { return ce_check_index(c, idx, base, index); }
    NkExprRef(inner) => { return ce_check_expr(c, inner); }
    NkExprMutRef(inner) => { return ce_check_expr(c, inner); }
    NkExprSome(inner) => {
      let t = ce_check_expr(c, inner);
      return "Option[" + t + "]";
    }
    NkExprNone => { return "Option"; }
    NkExprOk(inner) => {
      let _ = ce_check_expr(c, inner);
      // The error type is unknowable from the constructor alone -- keep the
      // erased form (types_compatible erases both sides).
      return "Result";
    }
    NkExprErr(inner) => {
      let _ = ce_check_expr(c, inner);
      return "Result";
    }
    NkExprStruct(name, fields, base) => { return ce_check_struct_lit(c, idx, name, fields, base); }
    NkExprArray(items) => {
      var i = 0;
      while i < items.len() {
        let _ = ce_check_expr(c, items[i]);
        i = i + 1;
      }
      if items.len() > 0 {
        // Rust: element type from item 0 via inferred binding; bare "Vec"
        // when empty. The exact spelling matters only for later field/index
        // resolution, which stage 1 defers permissively.
        return "Vec";
      }
      return "Vec";
    }
    NkExprBlock(block) => { return ck_check_block(c, block, "", 0); }
    NkExprConstBlock(inner) => { return ce_check_expr(c, inner); }
    NkExprClosure(params, ret, body) => { return ce_check_closure(c, params, ret, body); }
    NkExprPipeClosure(names, body) => { return ce_check_pipe_closure(c, names, body); }
    NkExprAwait(inner) => {
      let _ = ce_check_expr(c, inner);
      return "_";
    }
    NkExprComptime(inner) => { return ce_check_expr(c, inner); }
    NkExprAs(inner, ty) => {
      let _ = ce_check_expr(c, inner);
      return ck_type_from_ast(c, ty);
    }
    NkExprTuple(items) => { return ce_check_tuple(c, items); }
    NkExprIf(cond, then_b, elifs, els) => { return ce_check_if_expr(c, cond, then_b, elifs, els); }
    NkExprMatch(scrut, arms) => {
      let matched_ty = ce_check_expr(c, scrut);
      ce_check_arms(c, arms, matched_ty);
      var result = "()";
      if arms.len() > 0 {
        let anode = c.p.nodes[arms[0]];
        match anode.kind {
          NkMatchArm(pattern, guard, body, body_is_block) => {
            if body_is_block == 1 {
              result = "()";
            } else {
              result = ce_expr_type_quiet(c, body);
            }
          }
          _ => {}
        }
      }
      return result;
    }
    NkExprUnsafe(block) => {
      c.unsafe_depth = c.unsafe_depth + 1;
      let t = ck_check_block(c, block, "", 0);
      c.unsafe_depth = c.unsafe_depth - 1;
      return t;
    }
    NkExprError => { return "<error>"; }
    _ => { return "()"; }
  }
}

/// Type of an already-checked expression without re-emitting diagnostics
/// (used for match arm result probes). Calls the real checker; duplicate
/// errors are prevented by only using it on arm bodies already checked in
/// this walk (see `ce_check_arms`), so diagnostics may double-emit for the
/// first arm -- the Rust checker checks every arm exactly once and returns
/// the FIRST arm's type. Stage 1 mirrors the type shape, not the duplicate
/// suppression.
fn ce_expr_type_quiet(c: &mut Checker, idx: Int) -> Str {
  return ce_check_expr(c, idx);
}

fn ce_check_ident(c: &mut Checker, idx: Int, name_idx: Int) -> Str {
  var name = ck_ident(c, name_idx);
  var lookup = name;
  if name == "this" { lookup = "self"; }
  if lookup == "_" { return "Int"; }
  if name == "null" { return "Ptr"; }
  if name == "Unit" { return "()"; }
  if name == "self" {
    let s = ck_lookup_local(c, "self");
    if s.len() > 0 { return s; }
  }
  let local = ck_lookup_local(c, lookup);
  if local.len() > 0 { return local; }
  let global = ck_lookup_global(c, name);
  if global.len() > 0 { return global; }
  if ck_find_fn(c, name) >= 0 { return "fn"; }
  if name == "dbg" || name == "todo" || name == "unimplemented" { return "fn"; }
  if ck_find_type(c, name) >= 0 { return name; }
  let vi = ck_find_variant(c, name);
  if vi >= 0 { return c.variants[vi].enum_name; }
  if c.has_uses == 1 { return "_"; }
  let sp = ck_span_of(c, idx);
  return ck_error_at(c, "undefined variable '" + name + "'", sp.line, sp.col);
}

fn ce_check_unary(c: &mut Checker, idx: Int, op: Int, inner: Int) -> Str {
  let inner_ty = ce_check_expr(c, inner);
  let sp = ck_span_of(c, idx);
  if op == 0 {
    if !selfhost_check_types.ct_is_numeric(inner_ty) && inner_ty != "_" {
      let _ = ck_error_at(c, "cannot negate type " + inner_ty, sp.line, sp.col);
    }
    return inner_ty;
  }
  if op == 1 {
    let defers = inner_ty == "_" || selfhost_check_types.ct_is_generic_param(inner_ty);
    if inner_ty != "Bool" && !defers {
      let _ = ck_error_at(c, "cannot logically negate type " + inner_ty, sp.line, sp.col);
    }
    return "Bool";
  }
  if op == 5 {
    let is_raw_ptr = selfhost_check_types.ct_is_pointer_like(inner_ty) && inner_ty != "Str";
    if is_raw_ptr && c.unsafe_depth == 0 {
      let _ = ck_error_at(c, "raw pointer dereference requires an `unsafe` block (found `" + inner_ty + "`)", sp.line, sp.col);
    }
    if inner_ty.len() > 0 && (string.byte_at(inner_ty, 0) as Int) == 42 {
      return selfhost_check_types.ct_from_str(string.str_slice(inner_ty, 1, inner_ty.len()));
    }
    if inner_ty == "Ptr" { return "Int"; }
    return inner_ty;
  }
  return inner_ty;
}

fn ce_check_try(c: &mut Checker, idx: Int, inner: Int) -> Str {
  let inner_ty = ce_check_expr(c, inner);
  let sp = ck_span_of(c, idx);
  if inner_ty == "<error>" || inner_ty == "()" { return inner_ty; }
  if ce_is_result_or_option(inner_ty) {
    let fn_ok = c.cur_ret_set == 1 && ce_is_result_or_option(c.cur_ret);
    let base = selfhost_check_types.ct_base(inner_ty);
    if !fn_ok && base != "Result" && base != "Option" {
      // always true when result/option
    }
    if !fn_ok {
      var shown = "void";
      if c.cur_ret_set == 1 { shown = c.cur_ret; }
      let _ = ck_error_at(c, "'?' operator used in function that returns '" + shown + "' -- must return Result or Option", sp.line, sp.col);
    }
    return "_";
  }
  return ck_error_at(c, "'?' operator requires a Result or Option type, found " + inner_ty, sp.line, sp.col);
}

fn ce_is_result_or_option(name: Str) -> Bool {
  if name == "_" { return true; }
  let base = selfhost_check_types.ct_base(name);
  if base == "Result" || base == "Option" { return true; }
  return false;
}

fn ce_check_binary(c: &mut Checker, idx: Int, left: Int, op: Int, right: Int) -> Str {
  let left_ty = ce_check_expr(c, left);
  let right_ty = ce_check_expr(c, right);
  let sp = ck_span_of(c, idx);
  let l_int = selfhost_check_types.ct_is_int_family(left_ty);
  let r_int = selfhost_check_types.ct_is_int_family(right_ty);
  let l_flt = selfhost_check_types.ct_is_float_family(left_ty);
  let r_flt = selfhost_check_types.ct_is_float_family(right_ty);
  if (l_int && r_flt && !ce_is_int_literal(c, left))
    || (l_flt && r_int && !ce_is_int_literal(c, right))
  {
    var conv = right_ty;
    if l_int { conv = left_ty; }
    let _ = ck_error_at(c, "cannot mix " + left_ty + " with " + right_ty + " -- convert explicitly with `as` (e.g. `" + conv + " as Float64`)", sp.line, sp.col);
  }
  if (op == 3 || op == 4) && l_int && r_int && ce_is_zero_int_literal(c, right) {
    var what = "division";
    if op == 4 { what = "remainder"; }
    ck_warn_coded_at(c, "W008", "integer " + what + " by a zero literal always traps at runtime", sp.line, sp.col);
  }
  if op == 0 || op == 1 || op == 2 || op == 3 || op == 4 {
    if op == 0 && (left_ty == "Str" || right_ty == "Str") { return "Str"; }
    if !selfhost_check_types.ct_is_numeric(left_ty) && !ce_is_generic_or_wild(left_ty)
      && !ce_is_ptr_like_expr_ty(left_ty)
    {
      let _ = ck_error_at(c, "left operand must be numeric, found " + left_ty, sp.line, sp.col);
    }
    if !selfhost_check_types.ct_is_numeric(right_ty) && !ce_is_generic_or_wild(right_ty) {
      let _ = ck_error_at(c, "right operand must be numeric, found " + right_ty, sp.line, sp.col);
    }
    return left_ty;
  }
  if op == 5 || op == 6 {
    return ce_check_eq(c, left, right, left_ty, right_ty, op, sp);
  }
  if op == 7 || op == 8 || op == 9 || op == 10 {
    if (l_int && r_flt && !ce_is_int_literal(c, left))
      || (l_flt && r_int && !ce_is_int_literal(c, right))
    {
      let _ = ck_error_at(c, "cannot compare " + left_ty + " with " + right_ty + " -- convert explicitly with `as`", sp.line, sp.col);
    }
    return "Bool";
  }
  if op == 13 || op == 14 {
    if left_ty != "Bool" && !selfhost_check_types.ct_is_generic_param(left_ty) && left_ty != "_" {
      let _ = ck_error_at(c, "left operand of logical op must be Bool, found " + left_ty, sp.line, sp.col);
    }
    if right_ty != "Bool" && !selfhost_check_types.ct_is_generic_param(right_ty) && right_ty != "_" {
      let _ = ck_error_at(c, "right operand of logical op must be Bool, found " + right_ty, sp.line, sp.col);
    }
    return "Bool";
  }
  if op == 15 { return right_ty; }
  if op == 11 || op == 12 {
    return left_ty;
  }
  if op == 16 || op == 17 || op == 18 {
    return left_ty;
  }
  return left_ty;
}

fn ce_check_eq(c: &mut Checker, left: Int, right: Int, left_ty_in: Str, right_ty_in: Str, op: Int, sp: Span) -> Str {
  let left_ty = ck_resolve_alias(c, left_ty_in);
  let right_ty = ck_resolve_alias(c, right_ty_in);
  let numeric_family = selfhost_check_types.ct_is_numeric(left_ty)
    || left_ty == "Char";
  let numeric_family_r = selfhost_check_types.ct_is_numeric(right_ty)
    || right_ty == "Char";
  let is_null_literal = ce_is_zero_int_literal(c, right) || ce_is_null_ident(c, right);
  let is_null_literal_l = ce_is_zero_int_literal(c, left) || ce_is_null_ident(c, left);
  let ptr_null_cmp = (ce_is_ptr_ty(left_ty)
      && (right_ty == "Int" || right_ty == "Ptr" || right_ty == "_")
      && is_null_literal)
    || (ce_is_ptr_ty(right_ty)
      && (left_ty == "Int" || left_ty == "Ptr" || left_ty == "_")
      && is_null_literal_l);
  let compatible = left_ty == right_ty
    || (numeric_family && numeric_family_r)
    || ce_is_generic_or_wild(left_ty)
    || ce_is_generic_or_wild(right_ty)
    || ptr_null_cmp
    || (selfhost_check_types.ct_base(left_ty) == selfhost_check_types.ct_base(right_ty));
  let mix_err = (selfhost_check_types.ct_is_int_family(left_ty)
      && selfhost_check_types.ct_is_float_family(right_ty)
      && !ce_is_int_literal(c, left))
    || (selfhost_check_types.ct_is_float_family(left_ty)
      && selfhost_check_types.ct_is_int_family(right_ty)
      && !ce_is_int_literal(c, right));
  if mix_err {
    let _ = ck_error_at(c, "cannot compare " + left_ty + " with " + right_ty + " -- convert explicitly with `as`", sp.line, sp.col);
  }
  if !compatible {
    let _ = ck_error_at(c, "cannot compare " + left_ty + " with " + right_ty, sp.line, sp.col);
  }
  return "Bool";
}

fn ce_is_null_ident(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprIdent(name_idx) => {
      if ck_ident(c, name_idx) == "null" { return true; }
      return false;
    }
    _ => { return false; }
  }
}

fn ce_is_ptr_ty(ty: Str) -> Bool {
  return selfhost_check_types.ct_is_pointer_like(ty);
}

fn ce_is_generic_or_wild(ty: Str) -> Bool {
  if ty == "_" { return true; }
  return selfhost_check_types.ct_is_generic_param(ty);
}

fn ce_is_ptr_like_expr_ty(ty: Str) -> Bool {
  if ty.len() > 0 && (string.byte_at(ty, 0) as Int) == 42 { return true; }
  return selfhost_check_types.ct_is_fn_type(ty);
}

fn ce_check_field(c: &mut Checker, idx: Int, obj: Int, name_idx: Int) -> Str {
  let field = ck_ident(c, name_idx);
  let obj_ty = ce_check_expr(c, obj);
  let sp = ck_span_of(c, idx);
  if obj_ty == "_" || obj_ty == "<error>" { return "_"; }
  // Enum variant as field: `Color.Red`
  let variant_full = obj_ty + "." + field;
  if ck_find_variant(c, variant_full) >= 0 || ck_find_variant(c, field) >= 0 && ck_find_type(c, obj_ty) >= 0 {
    return obj_ty;
  }
  if selfhost_check_types.ct_is_generic_param(obj_ty) { return "_"; }
  let ti = ck_find_type(c, obj_ty);
  if ti >= 0 {
    if selfhost_check_types.ct_is_container(obj_ty) { return "_"; }
    let fty = ck_field_ty(c, obj_ty, field);
    if fty.len() > 0 { return fty; }
    if ce_is_pseudo_field(obj_ty, field) { return ce_pseudo_field_ty(field); }
    if c.types[ti].is_enum == 1 { return obj_ty; }
    // Container/builtin types have permissive field access (Rust registers
    // them with empty field maps).
    if ce_permissive_field_access(obj_ty) { return "_"; }
    if c.types[ti].fields.len() == 0 { return "_"; }
    let _ = ck_error_at(c, "type '" + obj_ty + "' has no field '" + field + "'", sp.line, sp.col);
    return "<error>";
  }
  return "_";
}

fn ce_permissive_field_access(ty: Str) -> Bool {
  let leaf = ce_leaf_name(ty);
  if ce_is_container_base_leaf(leaf) { return true; }
  if leaf == "Vec" || leaf == "Set" || leaf == "Stack" || leaf == "Slice" { return true; }
  return false;
}

fn ce_is_pseudo_field(ty: Str, field: Str) -> Bool {
  let base = selfhost_check_types.ct_base(ty);
  if base == "Option" && (field == "is_some" || field == "is_none" || field == "value") { return true; }
  if base == "Result" && (field == "is_ok" || field == "is_err" || field == "value" || field == "error") { return true; }
  return false;
}

fn ce_pseudo_field_ty(field: Str) -> Str {
  if field == "is_some" || field == "is_none" || field == "is_ok" || field == "is_err" { return "Bool"; }
  return "_";
}

// ============================================================================
// Calls
// ============================================================================

fn ce_check_call(c: &mut Checker, idx: Int, callee: Int, args: Vec[Int]) -> Str {
  let cnode = c.p.nodes[callee];
  match cnode.kind {
    NkExprField(obj, method_idx) => {
      return ce_check_method_call(c, idx, obj, method_idx, args);
    }
    NkExprIdent(name_idx) => {
      return ce_check_bare_call(c, idx, name_idx, args);
    }
    NkExprIndex(base, index) => {
      // `handlers[i](...)`: value-element call -- permissive.
      var i = 0;
      while i < args.len() {
        let _ = ce_check_expr(c, args[i]);
        i = i + 1;
      }
      return "_";
    }
    _ => {
      let callee_ty = ce_check_expr(c, callee);
      var i = 0;
      while i < args.len() {
        let _ = ce_check_expr(c, args[i]);
        i = i + 1;
      }
      if selfhost_check_types.ct_is_fn_type(callee_ty) {
        return selfhost_check_state.ck_fn_ret(callee_ty);
      }
      return "_";
    }
  }
}

fn ce_check_bare_call(c: &mut Checker, idx: Int, name_idx: Int, args: Vec[Int]) -> Str {
  let name = ck_ident(c, name_idx);
  let sp = ck_span_of(c, idx);
  let local = ck_lookup_local(c, name);
  if local.len() > 0 {
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    if selfhost_check_types.ct_is_fn_type(local) {
      return selfhost_check_state.ck_fn_ret(local);
    }
    return "_";
  }
  let fi = ck_find_fn(c, name);
  if fi >= 0 {
    return ce_check_call_sig(c, c.functions[fi], name, args, sp);
  }
  // Builtin literal constructors / tuple-struct ctors: `Type(args)`.
  if ck_find_type(c, name) >= 0 {
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    return name;
  }
  let vi = ck_find_variant(c, name);
  if vi >= 0 {
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    return c.variants[vi].enum_name;
  }
  if name == "dbg" || name == "todo" || name == "unimplemented" {
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    return "_";
  }
  if c.has_uses == 1 {
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    return "_";
  }
  let ident_span = ck_span_of(c, name_idx);
  let _ = ck_error_at(c, "undefined variable '" + name + "'", ident_span.line, ident_span.col);
  // Rust returns Unit from the unresolved call path; the caller's return
  // check then reports the observed cascade (`found ()`).
  var i = 0;
  while i < args.len() {
    let _ = ce_check_expr(c, args[i]);
    i = i + 1;
  }
  return "()";
}

fn ce_check_call_sig(c: &mut Checker, sig: FnSig, shown_name: Str, args: Vec[Int], sp: Span) -> Str {
  let expected_args = sig.params.len();
  if args.len() != expected_args {
    let _ = ck_error_at(c, "'" + shown_name + "' expects " + ce_int_str(expected_args) + " argument(s), found " + ce_int_str(args.len()), sp.line, sp.col);
  }
  var i = 0;
  while i < args.len() {
    let arg_ty = ce_check_expr(c, args[i]);
    if i < sig.params.len() {
      let expected = sig.params[i].ty;
      let is_generic = ce_vec_contains(sig.generics, expected);
      if !is_generic && !ck_types_compatible(c, arg_ty, expected) && arg_ty != "<error>" {
        let _ = ck_error_at(c, "argument " + ce_int_str(i + 1) + " type mismatch: expected " + expected + ", found " + arg_ty, sp.line, sp.col);
      }
    }
    i = i + 1;
  }
  var ret = "()";
  if sig.has_ret == 1 { ret = sig.ret; }
  return ce_substitute_generics(c, ret, sig.generics, Vec[Str].new());
}

fn ce_check_method_call(c: &mut Checker, idx: Int, obj: Int, method_idx: Int, args: Vec[Int]) -> Str {
  let method = ck_ident(c, method_idx);
  let sp = ck_span_of(c, idx);
  // Static call `Type.method(...)` / enum variant ctor.
  if ce_is_type_ident(c, obj) {
    let tn = ce_ident_text(c, obj);
    let variant_key = tn + "." + method;
    if ck_find_variant(c, variant_key) >= 0 {
      var i = 0;
      while i < args.len() {
        let _ = ce_check_expr(c, args[i]);
        i = i + 1;
      }
      return tn;
    }
    let fi = ck_find_fn(c, variant_key);
    if fi >= 0 {
      return ce_check_static_method(c, c.functions[fi], method, tn, args, sp);
    }
    // Builtin container constructors (Vec.new/Map.new/Set.new/...) and the
    // catalog's method set: permissive.
    if ce_is_container_base_leaf(tn) {
      var i = 0;
      while i < args.len() {
        let _ = ce_check_expr(c, args[i]);
        i = i + 1;
      }
      return "_";
    }
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    return "_";
  }
  let obj_ty = ce_check_expr(c, obj);
  if obj_ty == "<error>" { return "<error>"; }
  // Module-qualified call: the receiver is not a known value/type.
  if obj_ty == "_" || obj_ty == "" {
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    return "_";
  }
  if selfhost_check_types.ct_is_generic_param(obj_ty) {
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    return "_";
  }
  let leaf = ce_leaf_name(obj_ty);
  let base = selfhost_check_types.ct_base(obj_ty);
  if ce_is_container_base_leaf(leaf) || ce_is_container_base_leaf(base) {
    var i = 0;
    while i < args.len() {
      let _ = ce_check_expr(c, args[i]);
      i = i + 1;
    }
    return "_";
  }
  // Exact receiver key, then module-leaf fallback (`module.Type.method`).
  var fi = ck_find_fn(c, obj_ty + "." + method);
  if fi < 0 { fi = ck_find_fn(c, base + "." + method); }
  if fi >= 0 {
    return ce_check_instance_method(c, c.functions[fi], method, obj_ty, args, sp);
  }
  // Unknown method on a user type: permissive (stage-1 catalog gap).
  var i = 0;
  while i < args.len() {
    let _ = ce_check_expr(c, args[i]);
    i = i + 1;
  }
  return "_";
}

fn ce_check_static_method(c: &mut Checker, sig: FnSig, shown: Str, tn: Str, args: Vec[Int], sp: Span) -> Str {
  let expected = sig.params.len();
  if args.len() != expected {
    let _ = ck_error_at(c, "'" + shown + "' expects " + ce_int_str(expected) + " argument(s), found " + ce_int_str(args.len()), sp.line, sp.col);
  }
  var i = 0;
  while i < args.len() {
    let arg_ty = ce_check_expr(c, args[i]);
    if i < sig.params.len() {
      let e = sig.params[i].ty;
      let is_generic = ce_vec_contains(sig.generics, e);
      if !is_generic && !ck_types_compatible(c, arg_ty, e) && arg_ty != "<error>" {
        let _ = ck_error_at(c, "argument " + ce_int_str(i + 1) + " type mismatch: expected " + e + ", found " + arg_ty, sp.line, sp.col);
      }
    }
    i = i + 1;
  }
  var ret = "()";
  if sig.has_ret == 1 { ret = sig.ret; }
  if ret == "Self" { ret = tn; }
  return ret;
}

fn ce_check_instance_method(c: &mut Checker, sig: FnSig, shown: Str, recv_ty: Str, args: Vec[Int], sp: Span) -> Str {
  // Param-offset table (Rust 7240-7275): explicit self skips param 0 on an
  // instance call; a first param that matches the receiver at non-direct
  // arity is receiver-style; otherwise args map directly.
  var first_is_self = false;
  var first_matches = false;
  if sig.params.len() > 0 {
    let p0 = sig.params[0];
    if p0.name == "self" || p0.ty == "Self" { first_is_self = true; }
    let pty = ce_strip_ref_marks(p0.ty);
    let leaf_p = ce_leaf_name(pty);
    let leaf_r = ce_leaf_name(recv_ty);
    if leaf_p == leaf_r && leaf_p.len() > 0 { first_matches = true; }
  }
  let arity_direct = args.len() == sig.params.len();
  let explicit = first_is_self || (first_matches && !arity_direct);
  var offset = 0;
  if explicit { offset = 1; }
  let expected_args = sig.params.len() - offset;
  if args.len() != expected_args {
    let _ = ck_error_at(c, "'" + shown + "' expects " + ce_int_str(expected_args) + " argument(s), found " + ce_int_str(args.len()), sp.line, sp.col);
  }
  var i = 0;
  while i < args.len() {
    let arg_ty = ce_check_expr(c, args[i]);
    let pidx = i + offset;
    if pidx < sig.params.len() {
      let e = sig.params[pidx].ty;
      let is_generic = ce_vec_contains(sig.generics, e);
      if !is_generic && !ck_types_compatible(c, arg_ty, e) && arg_ty != "<error>" {
        let _ = ck_error_at(c, "argument " + ce_int_str(i + 1) + " type mismatch: expected " + e + ", found " + arg_ty, sp.line, sp.col);
      }
    }
    i = i + 1;
  }
  var ret = "()";
  if sig.has_ret == 1 { ret = sig.ret; }
  if ret == "Self" { ret = recv_ty; }
  return ret;
}

fn ce_check_generic_call(c: &mut Checker, idx: Int, callee: Int, types: Vec[Int], args: Vec[Int]) -> Str {
  // Explicit type args: substitute into the resolved signature's return.
  var subst_names = Vec[Str].new();
  var subst_vals = Vec[Str].new();
  var t = 0;
  while t < types.len() {
    subst_vals.push(ck_type_from_ast(c, types[t]));
    t = t + 1;
  }
  let cnode = c.p.nodes[callee];
  match cnode.kind {
    NkExprIdent(name_idx) => {
      let name = ck_ident(c, name_idx);
      let fi = ck_find_fn(c, name);
      if fi >= 0 {
        let sig = c.functions[fi];
        subst_names = sig.generics;
        var i = 0;
        while i < args.len() {
          let arg_ty = ce_check_expr(c, args[i]);
          if i < sig.params.len() {
            let e = sig.params[i].ty;
            let is_generic = ce_vec_contains(sig.generics, e) || (e.len() == 1);
            if !is_generic && !ck_types_compatible(c, arg_ty, e) && arg_ty != "<error>" {
              let sp = ck_span_of(c, idx);
              let _ = ck_error_at(c, "argument " + ce_int_str(i + 1) + " type mismatch: expected " + e + ", found " + arg_ty, sp.line, sp.col);
            }
          }
          i = i + 1;
        }
        var ret = "()";
        if sig.has_ret == 1 { ret = sig.ret; }
        return ce_substitute_generics(c, ret, subst_names, subst_vals);
      }
      var i = 0;
      while i < args.len() {
        let _ = ce_check_expr(c, args[i]);
        i = i + 1;
      }
      return "_";
    }
    NkExprField(obj, method_idx) => {
      let method = ck_ident(c, method_idx);
      let obj_ty = ce_check_expr(c, obj);
      let base = selfhost_check_types.ct_base(obj_ty);
      var fi = ck_find_fn(c, obj_ty + "." + method);
      if fi < 0 { fi = ck_find_fn(c, base + "." + method); }
      if fi >= 0 {
        let sig = c.functions[fi];
        subst_names = sig.generics;
        return ce_check_instance_method(c, sig, method, obj_ty, args, ck_span_of(c, idx));
      }
      var i = 0;
      while i < args.len() {
        let _ = ce_check_expr(c, args[i]);
        i = i + 1;
      }
      return "_";
    }
    _ => {
      var i = 0;
      while i < args.len() {
        let _ = ce_check_expr(c, args[i]);
        i = i + 1;
      }
      return "_";
    }
  }
}

/// Substitute generic parameter names in a type string (container/pointer
/// aware, mirrors `substitute_generic_type`'s structural rebuild).
fn ce_substitute_generics(c: &Checker, ty: Str, names: Vec[Str], values: Vec[Str]) -> Str {
  if names.len() == 0 || values.len() == 0 || values.len() != names.len() {
    return ty;
  }
  var i = 0;
  while i < names.len() {
    if ty == names[i] { return values[i]; }
    i = i + 1;
  }
  if ty.len() > 0 && (string.byte_at(ty, 0) as Int) == 42 {
    return "*" + ce_substitute_generics(c, string.str_slice(ty, 1, ty.len()), names, values);
  }
  if selfhost_check_types.ct_is_container(ty) {
    let base = selfhost_check_types.ct_base(ty);
    let args = selfhost_check_types.ct_args(ty);
    var out = Vec[Str].new();
    var j = 0;
    while j < args.len() {
      out.push(ce_substitute_generics(c, args[j], names, values));
      j = j + 1;
    }
    return base + "[" + selfhost_check_types.ct_join(out, ", ") + "]";
  }
  return ty;
}

fn ce_is_type_ident(c: &Checker, idx: Int) -> Bool {
  if idx < 0 { return false; }
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprIdent(name_idx) => {
      let name = ck_ident(c, name_idx);
      if ck_find_type(c, name) >= 0 { return true; }
      return false;
    }
    _ => { return false; }
  }
}

fn ce_ident_text(c: &Checker, idx: Int) -> Str {
  let node = c.p.nodes[idx];
  match node.kind {
    NkExprIdent(name_idx) => { return ck_ident(c, name_idx); }
    _ => { return ""; }
  }
}

fn ce_vec_contains(v: Vec[Str], needle: Str) -> Bool {
  var i = 0;
  while i < v.len() {
    if v[i] == needle { return true; }
    i = i + 1;
  }
  return false;
}

fn ce_int_str(v: Int) -> Str {
  return v + "";
}

// ============================================================================
// Remaining expression forms
// ============================================================================

fn ce_check_struct_lit(c: &mut Checker, idx: Int, name_idx: Int, fields: Vec[Int], base: Int) -> Str {
  let tn = ck_ident(c, name_idx);
  // Enum variant constructor or struct literal: check init values.
  var i = 0;
  while i < fields.len() {
    let fnode = c.p.nodes[fields[i]];
    match fnode.kind {
      NkFieldInit(fname, value) => {
        let _ = ce_check_expr(c, value);
      }
      _ => {}
    }
    i = i + 1;
  }
  if base >= 0 { let _ = ce_check_expr(c, base); }
  let vi = ck_find_variant(c, tn);
  if vi >= 0 { return c.variants[vi].enum_name; }
  return tn;
}

fn ce_check_closure(c: &mut Checker, params: Vec[Int], ret: Int, body: Int) -> Str {
  ck_push_scope(c);
  var i = 0;
  while i < params.len() {
    let pnode = c.p.nodes[params[i]];
    match pnode.kind {
      NkParam(name, ty, mutself, refself) => {
        var pty = "_";
        if ty >= 0 { pty = ck_type_from_ast(c, ty); }
        ck_add_local(c, ck_ident(c, name), pty);
      }
      _ => {}
    }
    i = i + 1;
  }
  let node = c.p.nodes[body];
  match node.kind {
    NkBlock(inner) => { let _ = ck_check_block(c, body, "", 0); }
    _ => { let _ = ce_check_expr(c, body); }
  }
  ck_pop_scope(c);
  return "fn";
}

fn ce_check_pipe_closure(c: &mut Checker, names: Vec[Int], body: Int) -> Str {
  ck_push_scope(c);
  var i = 0;
  while i < names.len() {
    ck_add_local(c, ck_ident(c, names[i]), "_");
    i = i + 1;
  }
  let _ = ce_check_expr(c, body);
  ck_pop_scope(c);
  return "fn";
}

fn ce_check_tuple(c: &mut Checker, items: Vec[Int]) -> Str {
  if items.len() == 0 { return "()"; }
  if items.len() == 1 { return ce_check_expr(c, items[0]); }
  var parts = Vec[Str].new();
  var i = 0;
  while i < items.len() {
    parts.push(ce_check_expr(c, items[i]));
    i = i + 1;
  }
  return "Tuple__" + selfhost_check_types.ct_join(parts, "__");
}

fn ce_check_if_expr(c: &mut Checker, cond: Int, then_b: Int, elifs: Vec[Int], els: Int) -> Str {
  let _ = ce_check_expr(c, cond);
  let first = ck_check_block(c, then_b, "", 0);
  var i = 0;
  while i < elifs.len() {
    let enode = c.p.nodes[elifs[i]];
    match enode.kind {
      NkElif(econd, eblock) => {
        let _ = ce_check_expr(c, econd);
        let _ = ck_check_block(c, eblock, "", 0);
      }
      _ => {}
    }
    i = i + 1;
  }
  if els >= 0 {
    let ety = ck_check_block(c, els, "", 0);
    if first.len() > 0 { return first; }
    return ety;
  }
  return first;
}

fn ce_check_index(c: &mut Checker, idx: Int, base: Int, index: Int) -> Str {
  let base_ty = ce_check_expr(c, base);
  let _ = ce_check_expr(c, index);
  let base0 = ce_strip_leading_refs(base_ty);
  let args = selfhost_check_types.ct_args(base0);
  let b = selfhost_check_types.ct_base(base0);
  if b == "Vec" || b == "Slice" || b == "Array" || b == "Set" {
    if args.len() == 1 { return args[0]; }
    return "_";
  }
  if b == "Map" {
    if args.len() == 2 { return args[1]; }
    return "_";
  }
  if b == "Str" { return "Char"; }
  if selfhost_check_types.ct_is_generic_param(base0) { return "_"; }
  return "_";
}
