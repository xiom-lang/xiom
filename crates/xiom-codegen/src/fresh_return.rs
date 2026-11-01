// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! m375 / engine Vec-leak ownership prepass.
//!
//! `vec_drop` proves a *local* Vec is uniquely owned. This module lifts that
//! proof across a call: a function is *fresh-return* when every return path
//! yields a uniquely owned Vec -- a fresh ctor/array, a fresh-return callee, or
//! a fresh local whose ownership transfers out. A caller binding such a call
//! may then drop it (`font_shape`, `ui2_draw_text`, `io.read_at`, ...).
//!
//! The analysis is deliberately conservative. A false negative leaves the
//! historical leak; a false positive would free a borrowed buffer. A leaf name
//! is marked only when EVERY Vec-returning function with that leaf is fresh,
//! and a local is accepted only when it is declared exactly once, every use is
//! non-escaping, and the return is its only escape.

use crate::vec_drop::{is_fresh_vec_call, is_fresh_vec_init, local_is_owned_fresh};
use std::collections::{HashMap, HashSet};
use xiom_ast::*;

/// Whole-program fixed point over Vec-returning functions.
pub(crate) fn collect_fresh_vec_returns(program: &Program) -> HashSet<String> {
    let mut groups: HashMap<String, Vec<&FnDecl>> = HashMap::new();
    collect_fn_groups(&program.items, &mut groups);
    for fns in groups.values_mut() {
        fns.retain(|fd| fn_returns_vec(fd));
    }
    groups.retain(|_, fns| !fns.is_empty());

    let mut fresh: HashSet<String> = HashSet::new();
    for _ in 0..8 {
        let mut changed = false;
        for (leaf, fns) in &groups {
            if fresh.contains(leaf) {
                continue;
            }
            if fns.iter().all(|fd| fn_returns_fresh(fd, &fresh)) {
                fresh.insert(leaf.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    fresh
}

/// All ownership sets, computed once per program.
pub(crate) fn collect_fresh_ownership(
    program: &Program,
) -> (HashSet<String>, HashSet<String>, HashSet<String>) {
    let vec = collect_fresh_vec_returns(program);
    let payload = collect_fresh_payload_returns(program, &vec);
    let aggregate = collect_fresh_aggregate_returns(program);
    (vec, payload, aggregate)
}

/// Aggregate builders (`#[owns_return]` on a named-struct return). The caller
/// may drop every Vec field of the returned frame. All same-leaf aggregate
/// functions must carry the contract.
fn collect_fresh_aggregate_returns(program: &Program) -> HashSet<String> {
    let mut groups: HashMap<String, Vec<&FnDecl>> = HashMap::new();
    collect_fn_groups(&program.items, &mut groups);
    for fns in groups.values_mut() {
        fns.retain(|fd| fn_returns_named_struct(fd));
    }
    groups.retain(|_, fns| !fns.is_empty());
    groups
        .into_iter()
        .filter_map(|(leaf, fns)| {
            if fns.iter().all(|fd| fn_has_owns_return(fd)) {
                Some(leaf)
            } else {
                None
            }
        })
        .collect()
}

fn fn_returns_named_struct(fd: &FnDecl) -> bool {
    match fd.return_type.as_ref() {
        Some(Type::Named(id, _)) => is_named_struct_type(id),
        _ => false,
    }
}

fn is_named_struct_type(id: &Ident) -> bool {
    !matches!(
        id.name.as_str(),
        "Int"
            | "Int8"
            | "Int16"
            | "Int32"
            | "Int64"
            | "Int128"
            | "UInt"
            | "UInt8"
            | "UInt16"
            | "UInt32"
            | "UInt64"
            | "UInt128"
            | "Float"
            | "Float32"
            | "Float64"
            | "Bool"
            | "Char"
            | "Str"
            | "Unit"
            | "Vec"
    )
}

/// Result/Option<Vec> returns (io.read_at shape): the wrapper is a value, but
/// its Ok/Some payload is a fresh Vec that the caller's match may own/drop.
fn collect_fresh_payload_returns(program: &Program, fresh_vec: &HashSet<String>) -> HashSet<String> {
    let mut groups: HashMap<String, Vec<&FnDecl>> = HashMap::new();
    collect_fn_groups(&program.items, &mut groups);
    if std::env::var("XIOM_DEBUG_OWNERSHIP").is_ok() {
        if let Some(fns) = groups.get("read_at") {
            for fd in fns {
                eprintln!(
                    "read_at candidate ret={:?} attrs={:?} payload_vec={}",
                    fd.return_type.as_ref().map(type_name_from_ast),
                    fd.attributes
                        .iter()
                        .map(|a| a.name.name.clone())
                        .collect::<Vec<_>>(),
                    fn_returns_payload_vec(fd)
                );
            }
        } else {
            eprintln!("read_at candidate group missing");
        }
    }
    for fns in groups.values_mut() {
        fns.retain(|fd| fn_returns_payload_vec(fd));
    }
    groups.retain(|_, fns| !fns.is_empty());

    let mut fresh: HashSet<String> = HashSet::new();
    for _ in 0..8 {
        let mut changed = false;
        for (leaf, fns) in &groups {
            if fresh.contains(leaf) {
                continue;
            }
            if fns.iter().all(|fd| fn_returns_fresh_payload(fd, fresh_vec)) {
                fresh.insert(leaf.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    fresh
}

fn fn_returns_payload_vec(fd: &FnDecl) -> bool {
    match fd.return_type.as_ref() {
        Some(Type::Result(ok, _)) | Some(Type::Option(ok)) => is_vec_type(ok.as_ref()),
        _ => false,
    }
}

fn fn_returns_fresh_payload(fd: &FnDecl, fresh_vec: &HashSet<String>) -> bool {
    // Explicit ownership contract; the return-type filter guarantees
    // Result/Option<Vec>.
    if fn_has_owns_return(fd) {
        return true;
    }
    let Some(body) = &fd.body else { return false; };
    if body_has_unanalyzable_return(body) {
        return false;
    }
    let mut exprs: Vec<&Expr> = Vec::new();
    collect_return_exprs(&body.stmts, &mut exprs);
    if let Some(last) = body.stmts.last() {
        match last {
            StmtOrExpr::Expr(expr) => exprs.push(expr),
            StmtOrExpr::Stmt(Stmt::Expr(expr, _)) => exprs.push(expr),
            _ => {}
        }
    }
    !exprs.is_empty() && exprs.iter().all(|expr| payload_return_fresh(expr, body, fresh_vec))
}

fn payload_return_fresh(expr: &Expr, body: &Block, fresh_vec: &HashSet<String>) -> bool {
    match expr {
        Expr::Paren(inner, _) => payload_return_fresh(inner, body, fresh_vec),
        Expr::Ok(inner, _) | Expr::Some(inner, _) => expr_returns_fresh(inner, body, fresh_vec),
        // Err/None carry no Vec payload; the wrapper is still safe to own.
        Expr::Err(..) | Expr::None(..) => true,
        Expr::If(..) | Expr::Match(..) | Expr::BlockExpr(..) => {
            payload_control_fresh(expr, body, fresh_vec)
        }
        _ => false,
    }
}

fn payload_control_fresh(expr: &Expr, body: &Block, fresh_vec: &HashSet<String>) -> bool {
    match expr {
        Expr::If(_, then_block, elifs, else_block, _) => {
            let Some(else_block) = else_block else {
                return false;
            };
            payload_block_fresh(then_block, body, fresh_vec)
                && elifs
                    .iter()
                    .all(|(_, block)| payload_block_fresh(block, body, fresh_vec))
                && payload_block_fresh(else_block, body, fresh_vec)
        }
        Expr::Match(_, arms, _) => {
            !arms.is_empty()
                && arms.iter().all(|arm| match &arm.body {
                    MatchBody::Block(block) => payload_block_fresh(block, body, fresh_vec),
                    MatchBody::Expr(expr) => payload_return_fresh(expr, body, fresh_vec),
                })
        }
        Expr::BlockExpr(block, _) => payload_block_fresh(block, body, fresh_vec),
        _ => false,
    }
}

fn payload_block_fresh(block: &Block, body: &Block, fresh_vec: &HashSet<String>) -> bool {
    match block.stmts.last() {
        Some(StmtOrExpr::Expr(expr)) => payload_return_fresh(expr, body, fresh_vec),
        Some(StmtOrExpr::Stmt(Stmt::Expr(expr, _))) => payload_return_fresh(expr, body, fresh_vec),
        _ => false,
    }
}

fn collect_fn_groups<'a>(items: &'a [TopDecl], groups: &mut HashMap<String, Vec<&'a FnDecl>>) {
    for item in items {
        match item {
            TopDecl::Fn(fd) => add_fn_group(fd, groups),
            TopDecl::Module(module) => collect_fn_groups(&module.items, groups),
            TopDecl::Impl(im) => {
                for member in &im.members {
                    if let ImplItem::Fn(fd) = member {
                        add_fn_group(fd, groups);
                    }
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn fn_has_owns_return(fd: &FnDecl) -> bool {
    fd.attributes
        .iter()
        .any(|a| matches!(a.name.name.as_str(), "owns_return" | "fresh_return"))
}

/// `#[consumes_str]` contract (REL-20261031-2337-compiler): on a successful
/// call, every `Str` parameter is MOVED into an owned `Vec[Str]` field of the
/// `&mut` aggregate argument. Map/Set `put`/`set`/`add` use this. Returns
/// leaf -> the parameter indices that are `Str`; a leaf is only accepted when
/// every same-leaf function carries the attribute and agrees on those
/// positions (conservative).
pub(crate) fn collect_str_consuming_callees(program: &Program) -> HashMap<String, Vec<usize>> {
    let mut groups: HashMap<String, Vec<&FnDecl>> = HashMap::new();
    collect_fn_groups(&program.items, &mut groups);
    let mut out = HashMap::new();
    for (leaf, fns) in groups {
        if fns.is_empty() || !fns.iter().all(|fd| fn_has_consumes_str(fd)) {
            continue;
        }
        let indices: Vec<usize> = fns[0]
            .params
            .iter()
            .enumerate()
            .filter(|(_, p)| is_str_param_type(&p.ty))
            .map(|(i, _)| i)
            .collect();
        if indices.is_empty() {
            continue;
        }
        if fns.iter().all(|fd| {
            indices
                .iter()
                .all(|&i| fd.params.get(i).map_or(false, |p| is_str_param_type(&p.ty)))
        }) {
            out.insert(leaf, indices);
        }
    }
    out
}

pub(crate) fn fn_has_consumes_str(fd: &FnDecl) -> bool {
    fd.attributes
        .iter()
        .any(|a| a.name.name.as_str() == "consumes_str")
}

/// Leaf names of functions with the `#[borrows_vec]` contract: the callee may
/// MUTATE a `Vec` parameter through a `&mut` reference (or read a by-value
/// header) during the call, but it does NOT retain the header/data anywhere.
/// The caller keeps ownership and drops the buffer at its own scope exit.
/// Used for the engine's per-frame Vulkan UI batch builders/submits, whose
/// `&mut`/by-value Vec args otherwise disarm the caller drop (REL-20261101-
/// 0332-engine: ~2.9 MB/frame). All same-leaf functions must carry it.
pub(crate) fn collect_borrows_vec_callees(program: &Program) -> HashSet<String> {
    let mut groups: HashMap<String, Vec<&FnDecl>> = HashMap::new();
    collect_fn_groups(&program.items, &mut groups);
    groups
        .into_iter()
        .filter_map(|(leaf, fns)| {
            if !fns.is_empty()
                && fns.iter().all(|fd| {
                    fd.attributes
                        .iter()
                        .any(|a| a.name.name.as_str() == "borrows_vec")
                })
            {
                Some(leaf)
            } else {
                None
            }
        })
        .collect()
}

fn is_str_param_type(ty: &Type) -> bool {
    match ty {
        Type::Named(id, _) => id.name == "Str" || id.name.ends_with(".Str"),
        _ => false,
    }
}

/// The `#[owns_return]` contract in its *sound* form: the attribute is only
/// honored when the function returns an owning shape (a Vec, a `Result`/
/// `Option<Vec>` payload, or a named aggregate). An attribute on a raw-pointer
/// or `Str` return grants no trust, so a stray pointer export cannot make a
/// local look non-escaping and then free the buffer it points into.
pub(crate) fn fn_owns_return_transfer(fd: &FnDecl) -> bool {
    if !fn_has_owns_return(fd) {
        return false;
    }
    match fd.return_type.as_ref() {
        Some(Type::Vec(_)) => true,
        Some(Type::Named(id, _)) => is_named_struct_type(id),
        Some(Type::Result(ok, _)) | Some(Type::Option(ok)) => is_vec_type(ok.as_ref()),
        _ => false,
    }
}

fn add_fn_group<'a>(fd: &'a FnDecl, groups: &mut HashMap<String, Vec<&'a FnDecl>>) {
    if fd.body.is_none() {
        return;
    }
    let leaf = fd.name.name.rsplit('.').next().unwrap_or(&fd.name.name);
    groups.entry(leaf.to_string()).or_default().push(fd);
}

fn is_vec_type(ty: &Type) -> bool {
    match ty {
        Type::Vec(_) => true,
        Type::Named(id, args) => id.name == "Vec" && !args.is_empty(),
        _ => false,
    }
}

fn fn_returns_vec(fd: &FnDecl) -> bool {
    fd.return_type.as_ref().map_or(false, is_vec_type)
}

fn fn_returns_fresh(fd: &FnDecl, fresh: &HashSet<String>) -> bool {
    // Explicit ownership contract for a body the conservative scan cannot see
    // through (FFI/unsafe fill). The return-type filter guarantees Vec.
    if fn_has_owns_return(fd) {
        return true;
    }
    let Some(body) = &fd.body else { return false; };
    // A return inside an unsafe block/closure belongs to another function;
    // do not claim the outer function is fresh in that case.
    if body_has_unanalyzable_return(body) {
        return false;
    }
    let mut exprs: Vec<&Expr> = Vec::new();
    collect_return_exprs(&body.stmts, &mut exprs);
    if let Some(last) = body.stmts.last() {
        match last {
            StmtOrExpr::Expr(expr) => exprs.push(expr),
            StmtOrExpr::Stmt(Stmt::Expr(expr, _)) => exprs.push(expr),
            _ => {}
        }
    }
    !exprs.is_empty() && exprs.iter().all(|expr| expr_returns_fresh(expr, body, fresh))
}

fn expr_returns_fresh(expr: &Expr, body: &Block, fresh: &HashSet<String>) -> bool {
    match expr {
        Expr::Paren(inner, _) => expr_returns_fresh(inner, body, fresh),
        e if is_fresh_vec_init(e) => true,
        e if is_fresh_vec_call(e, fresh) => true,
        Expr::Ok(inner, _) | Expr::Some(inner, _) => expr_returns_fresh(inner, body, fresh),
        Expr::Ident(id) => local_is_owned_fresh(body, &id.name, fresh),
        Expr::If(..) | Expr::Match(..) | Expr::BlockExpr(..) => {
            expr_control_returns_fresh(expr, body, fresh)
        }
        _ => false,
    }
}

fn expr_control_returns_fresh(expr: &Expr, body: &Block, fresh: &HashSet<String>) -> bool {
    match expr {
        Expr::If(_, then_block, elifs, else_block, _) => {
            let Some(else_block) = else_block else {
                return false;
            };
            block_value_fresh(then_block, body, fresh)
                && elifs
                    .iter()
                    .all(|(_, block)| block_value_fresh(block, body, fresh))
                && block_value_fresh(else_block, body, fresh)
        }
        Expr::Match(_, arms, _) => {
            !arms.is_empty()
                && arms.iter().all(|arm| match &arm.body {
                    MatchBody::Block(block) => block_value_fresh(block, body, fresh),
                    MatchBody::Expr(expr) => expr_returns_fresh(expr, body, fresh),
                })
        }
        Expr::BlockExpr(block, _) => block_value_fresh(block, body, fresh),
        _ => false,
    }
}

fn block_value_fresh(block: &Block, body: &Block, fresh: &HashSet<String>) -> bool {
    match block.stmts.last() {
        Some(StmtOrExpr::Expr(expr)) => expr_returns_fresh(expr, body, fresh),
        Some(StmtOrExpr::Stmt(Stmt::Expr(expr, _))) => expr_returns_fresh(expr, body, fresh),
        _ => false,
    }
}

fn collect_return_exprs<'a>(stmts: &'a [StmtOrExpr], out: &mut Vec<&'a Expr>) {
    for item in stmts {
        match item {
            StmtOrExpr::Stmt(stmt) => collect_stmt_returns(stmt, out),
            StmtOrExpr::Expr(expr) => collect_expr_returns(expr, out),
        }
    }
}

fn collect_block_returns<'a>(block: &'a Block, out: &mut Vec<&'a Expr>) {
    collect_return_exprs(&block.stmts, out);
}

fn collect_stmt_returns<'a>(stmt: &'a Stmt, out: &mut Vec<&'a Expr>) {
    match stmt {
        Stmt::Return(Some(expr), _) => out.push(expr),
        Stmt::Return(None, _)
        | Stmt::Break(..)
        | Stmt::Continue(..)
        | Stmt::Debugger(_)
        | Stmt::Defer(_, _)
        | Stmt::Spawn(_, _, _)
        | Stmt::Asm(_) => {}
        Stmt::Expr(expr, _) => collect_expr_returns(expr, out),
        Stmt::Let(_, _, expr, _) | Stmt::Var(_, _, expr, _) => collect_expr_returns(expr, out),
        Stmt::Assign(place, expr, _) => {
            collect_expr_returns(place, out);
            collect_expr_returns(expr, out);
        }
        Stmt::If(_, then_block, elifs, else_block, _) => {
            collect_block_returns(then_block, out);
            for (_, block) in elifs {
                collect_block_returns(block, out);
            }
            if let Some(block) = else_block {
                collect_block_returns(block, out);
            }
        }
        Stmt::Match(_, arms, _) => {
            for arm in arms {
                match &arm.body {
                    MatchBody::Block(block) => collect_block_returns(block, out),
                    MatchBody::Expr(expr) => collect_expr_returns(expr, out),
                }
            }
        }
        Stmt::While(_, block, _, _, _) | Stmt::For(_, _, block, _, _) => {
            collect_block_returns(block, out)
        }
        Stmt::Destructure(_, expr, _) => collect_expr_returns(expr, out),
        Stmt::Assert(expr, _, _) => collect_expr_returns(expr, out),
    }
}

fn collect_expr_returns<'a>(expr: &'a Expr, out: &mut Vec<&'a Expr>) {
    match expr {
        Expr::If(_, then_block, elifs, else_block, _) => {
            collect_block_returns(then_block, out);
            for (_, block) in elifs {
                collect_block_returns(block, out);
            }
            if let Some(block) = else_block {
                collect_block_returns(block, out);
            }
        }
        Expr::Match(_, arms, _) => {
            for arm in arms {
                match &arm.body {
                    MatchBody::Block(block) => collect_block_returns(block, out),
                    MatchBody::Expr(expr) => collect_expr_returns(expr, out),
                }
            }
        }
        Expr::BlockExpr(block, _) => collect_block_returns(block, out),
        Expr::Paren(inner, _) | Expr::Unary(_, inner, _) | Expr::As(inner, _, _) => {
            collect_expr_returns(inner, out)
        }
        Expr::Binary(left, _, right, _) | Expr::Imply(left, right, _) => {
            collect_expr_returns(left, out);
            collect_expr_returns(right, out);
        }
        Expr::Call(func, args, _) => {
            collect_expr_returns(func, out);
            for arg in args {
                collect_expr_returns(arg, out);
            }
        }
        Expr::GenericCall(func, _, args, _) => {
            collect_expr_returns(func, out);
            for arg in args {
                collect_expr_returns(arg, out);
            }
        }
        Expr::Field(base, _, _) => collect_expr_returns(base, out),
        Expr::Index(base, idx, _) => {
            collect_expr_returns(base, out);
            collect_expr_returns(idx, out);
        }
        Expr::Struct(_, fields, spread, _) => {
            for (_, value) in fields {
                collect_expr_returns(value, out);
            }
            if let Some(spread) = spread {
                collect_expr_returns(spread, out);
            }
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
            for elem in elems {
                collect_expr_returns(elem, out);
            }
        }
        _ => {}
    }
}

fn body_has_unanalyzable_return(body: &Block) -> bool {
    body.stmts.iter().any(|item| match item {
        StmtOrExpr::Stmt(stmt) => stmt_has_nested_return(stmt),
        StmtOrExpr::Expr(expr) => expr_has_nested_return(expr),
    })
}

fn stmt_has_nested_return(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(expr, _) | Stmt::Let(_, _, expr, _) | Stmt::Var(_, _, expr, _) => {
            expr_has_nested_return(expr)
        }
        Stmt::Return(_, _) | Stmt::Break(..) | Stmt::Continue(..) | Stmt::Debugger(_) => false,
        Stmt::If(cond, then_block, elifs, else_block, _) => {
            expr_has_nested_return(cond)
                || block_has_nested_return(then_block)
                || elifs.iter().any(|(cond, block)| {
                    expr_has_nested_return(cond) || block_has_nested_return(block)
                })
                || else_block
                    .as_ref()
                    .map_or(false, |block| block_has_nested_return(block))
        }
        Stmt::Match(scrutinee, arms, _) => {
            expr_has_nested_return(scrutinee)
                || arms.iter().any(|arm| match &arm.body {
                    MatchBody::Block(block) => block_has_nested_return(block),
                    MatchBody::Expr(expr) => expr_has_nested_return(expr),
                })
        }
        Stmt::While(_, block, _, _, _) | Stmt::For(_, _, block, _, _) => {
            block_has_nested_return(block)
        }
        Stmt::Spawn(block, _, _) => block_has_nested_return(block),
        Stmt::Defer(block, _) => block_has_nested_return(block),
        Stmt::Assign(_, expr, _) => expr_has_nested_return(expr),
        Stmt::Destructure(_, expr, _) => expr_has_nested_return(expr),
        Stmt::Assert(expr, _, _) => expr_has_nested_return(expr),
        Stmt::Asm(_) => false,
    }
}

fn block_has_nested_return(block: &Block) -> bool {
    block.stmts.iter().any(|item| match item {
        StmtOrExpr::Stmt(stmt) => stmt_has_nested_return(stmt),
        StmtOrExpr::Expr(expr) => expr_has_nested_return(expr),
    })
}

fn expr_has_nested_return(expr: &Expr) -> bool {
    match expr {
        Expr::Unsafe(block, _) | Expr::Closure(_, _, block, _) => block_has_nested_return(block),
        Expr::PipeClosure(_, body, _) => expr_has_nested_return(body),
        Expr::If(_, then_block, elifs, else_block, _) => {
            block_has_nested_return(then_block)
                || elifs.iter().any(|(_, block)| block_has_nested_return(block))
                || else_block
                    .as_ref()
                    .map_or(false, |block| block_has_nested_return(block))
        }
        Expr::Match(_, arms, _) => arms.iter().any(|arm| match &arm.body {
            MatchBody::Block(block) => block_has_nested_return(block),
            MatchBody::Expr(expr) => expr_has_nested_return(expr),
        }),
        Expr::BlockExpr(block, _) => block_has_nested_return(block),
        Expr::Paren(inner, _)
        | Expr::Unary(_, inner, _)
        | Expr::As(inner, _, _)
        | Expr::Try(inner, _) => expr_has_nested_return(inner),
        Expr::Binary(left, _, right, _) | Expr::Imply(left, right, _) => {
            expr_has_nested_return(left) || expr_has_nested_return(right)
        }
        Expr::Call(func, args, _) => {
            expr_has_nested_return(func) || args.iter().any(expr_has_nested_return)
        }
        Expr::GenericCall(func, _, args, _) => {
            expr_has_nested_return(func) || args.iter().any(expr_has_nested_return)
        }
        Expr::Field(base, _, _) => expr_has_nested_return(base),
        Expr::Index(base, idx, _) => expr_has_nested_return(base) || expr_has_nested_return(idx),
        Expr::Struct(_, fields, spread, _) => {
            fields.iter().any(|(_, value)| expr_has_nested_return(value))
                || spread
                    .as_ref()
                    .map_or(false, |spread| expr_has_nested_return(spread))
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => elems.iter().any(expr_has_nested_return),
        _ => false,
    }
}

/// Leaf names of functions that can retain a by-value aggregate argument:
/// any `&mut` parameter (store through the reference), or a return type that
/// can carry an aggregate (Option/Result/Vec/Map/Set/Tuple/pointer, or a named
/// struct/enum). Helpers like `_ok_outcome(done)` are ordinary calls but
/// return the value, so the caller must transfer ownership (REL-2110 u4).
pub(crate) fn collect_retaining_callees(program: &Program) -> HashSet<String> {
    let mut groups: HashMap<String, Vec<&FnDecl>> = HashMap::new();
    collect_fn_groups(&program.items, &mut groups);
    groups
        .into_iter()
        .filter(|(_, fns)| {
            fns.iter().any(|fd| {
                fd.params
                    .iter()
                    .any(|p| matches!(p.ty, Type::MutRef(_)))
                    || ret_can_carry(&fd.return_type)
            })
        })
        .map(|(leaf, _)| leaf)
        .collect()
}

/// Leaf names of functions whose by-value `Vec[T]` parameters are only READ
/// (header/length queries), never reallocated or stored anywhere.
///
/// A by-value Vec parameter is a shallow header: the caller's data buffer is
/// shared. A `push`/`insert`/`remove`/`reserve` on the parameter reallocs and
/// frees that shared buffer, and any bare use (return, call arg, assignment,
/// element projection) can let the callee retain a pointer into it. Only when
/// every such param is used solely through `len`/`is_empty`/`capacity`/`count`
/// may the caller keep ownership and free the buffer at its own scope exit
/// (m375 p_panel_move / p_vecrebind). Anything else stays leak-only, the
/// historical conservative behavior.
pub(crate) fn collect_byval_vec_readers(program: &Program) -> HashSet<String> {
    let mut groups: HashMap<String, Vec<&FnDecl>> = HashMap::new();
    collect_fn_groups(&program.items, &mut groups);
    groups
        .into_iter()
        .filter_map(|(leaf, fns)| {
            if fns.iter().all(|fd| fn_byval_vec_params_read_only(fd)) {
                Some(leaf)
            } else {
                None
            }
        })
        .collect()
}

/// Methods that read the Vec header only and return a scalar. Element-returning
/// methods (`get`/`first`/`last`) are deliberately absent: their result can
/// alias an element the caller's buffer owns.
const READ_ONLY_VEC_METHODS: &[&str] = &["len", "is_empty", "capacity", "count"];

fn fn_byval_vec_params_read_only(fd: &FnDecl) -> bool {
    let Some(body) = &fd.body else { return false };
    let params: HashSet<String> = fd
        .params
        .iter()
        .filter(|p| is_vec_type(&p.ty))
        .map(|p| p.name.name.clone())
        .collect();
    // No by-value Vec parameter: the leaf cannot participate in the
    // caller-keeps-ownership rule for this analysis.
    if params.is_empty() {
        return false;
    }
    !block_uses_params_unsafely(body, &params)
}

/// True when any name in `params` is used somewhere other than as the receiver
/// of a header-only method. Conservative on purpose: a false positive keeps the
/// caller's drop disarmed (leak), never a wrong free.
fn block_uses_params_unsafely(block: &Block, params: &HashSet<String>) -> bool {
    block.stmts.iter().any(|item| match item {
        StmtOrExpr::Stmt(stmt) => stmt_uses_params_unsafely(stmt, params),
        StmtOrExpr::Expr(expr) => expr_uses_params_unsafely(expr, params),
    })
}

fn stmt_uses_params_unsafely(stmt: &Stmt, params: &HashSet<String>) -> bool {
    match stmt {
        Stmt::Let(_, _, expr, _) | Stmt::Var(_, _, expr, _) => {
            // `var b = param` deep-copies the Vec header at the binding
            // (vec_abi::emit_vec_value_deep_copy), so the callee owns a
            // private buffer and the caller's param buffer is untouched.
            if let Expr::Ident(id) = expr {
                if params.contains(&id.name) {
                    return false;
                }
            }
            expr_uses_params_unsafely(expr, params)
        }
        Stmt::Expr(expr, _) | Stmt::Return(Some(expr), _) => {
            expr_uses_params_unsafely(expr, params)
        }
        Stmt::Assign(place, value, _) => {
            expr_uses_params_unsafely(place, params) || expr_uses_params_unsafely(value, params)
        }
        Stmt::If(cond, then_block, elifs, else_block, _) => {
            expr_uses_params_unsafely(cond, params)
                || block_uses_params_unsafely(then_block, params)
                || elifs.iter().any(|(cond, block)| {
                    expr_uses_params_unsafely(cond, params)
                        || block_uses_params_unsafely(block, params)
                })
                || else_block
                    .as_ref()
                    .map_or(false, |block| block_uses_params_unsafely(block, params))
        }
        Stmt::Match(scrutinee, arms, _) => {
            expr_uses_params_unsafely(scrutinee, params)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .map_or(false, |guard| expr_uses_params_unsafely(guard, params))
                        || match &arm.body {
                            MatchBody::Block(block) => block_uses_params_unsafely(block, params),
                            MatchBody::Expr(expr) => expr_uses_params_unsafely(expr, params),
                        }
                })
        }
        Stmt::While(cond, block, invariant, _, _) => {
            expr_uses_params_unsafely(cond, params)
                || invariant
                    .as_ref()
                    .map_or(false, |inv| expr_uses_params_unsafely(inv, params))
                || block_uses_params_unsafely(block, params)
        }
        Stmt::For(_, iter, block, _, _) => {
            expr_uses_params_unsafely(iter, params) || block_uses_params_unsafely(block, params)
        }
        Stmt::Spawn(block, _, _) | Stmt::Defer(block, _) => {
            block_uses_params_unsafely(block, params)
        }
        Stmt::Destructure(_, expr, _) => expr_uses_params_unsafely(expr, params),
        Stmt::Assert(expr, msg, _) => {
            expr_uses_params_unsafely(expr, params)
                || msg
                    .as_ref()
                    .map_or(false, |msg| expr_uses_params_unsafely(msg, params))
        }
        Stmt::Asm(asm_block) => asm_block
            .inputs
            .iter()
            .any(|(_, expr)| expr_uses_params_unsafely(expr, params)),
        Stmt::Return(None, _) | Stmt::Break(..) | Stmt::Continue(..) | Stmt::Debugger(_) => false,
    }
}

fn expr_uses_params_unsafely(expr: &Expr, params: &HashSet<String>) -> bool {
    match expr {
        Expr::Ident(id) => params.contains(&id.name),
        Expr::Paren(inner, _) => expr_uses_params_unsafely(inner, params),
        Expr::Field(base, method, _) => {
            if READ_ONLY_VEC_METHODS.contains(&method.name.as_str()) {
                // Header-only receiver: `v.len()`, `v.is_empty()`. The method
                // itself cannot realloc or retain the buffer.
                false
            } else {
                expr_uses_params_unsafely(base, params)
            }
        }
        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
            expr_uses_params_unsafely(func, params)
                || args.iter().any(|arg| expr_uses_params_unsafely(arg, params))
        }
        Expr::Index(base, index, _) => {
            expr_uses_params_unsafely(base, params)
                || expr_uses_params_unsafely(index, params)
        }
        Expr::Unary(_, inner, _)
        | Expr::Ref(inner, _)
        | Expr::MutRef(inner, _)
        | Expr::Try(inner, _)
        | Expr::AtPre(inner, _)
        | Expr::Await(inner, _)
        | Expr::Comptime(inner, _)
        | Expr::As(inner, _, _)
        | Expr::Some(inner, _)
        | Expr::Ok(inner, _)
        | Expr::Err(inner, _)
        | Expr::Is(inner, _, _)
        | Expr::ConstBlock(inner, _) => expr_uses_params_unsafely(inner, params),
        Expr::Binary(left, _, right, _) | Expr::Imply(left, right, _) => {
            expr_uses_params_unsafely(left, params) || expr_uses_params_unsafely(right, params)
        }
        Expr::Struct(_, fields, spread, _) => {
            fields
                .iter()
                .any(|(_, value)| expr_uses_params_unsafely(value, params))
                || spread
                    .as_ref()
                    .map_or(false, |spread| expr_uses_params_unsafely(spread, params))
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => elems
            .iter()
            .any(|elem| expr_uses_params_unsafely(elem, params)),
        Expr::BlockExpr(block, _) | Expr::Unsafe(block, _) => {
            block_uses_params_unsafely(block, params)
        }
        Expr::Closure(_, _, block, _) => block_uses_params_unsafely(block, params),
        Expr::PipeClosure(_, body, _) => expr_uses_params_unsafely(body, params),
        Expr::If(cond, then_block, elifs, else_block, _) => {
            expr_uses_params_unsafely(cond, params)
                || block_uses_params_unsafely(then_block, params)
                || elifs.iter().any(|(cond, block)| {
                    expr_uses_params_unsafely(cond, params)
                        || block_uses_params_unsafely(block, params)
                })
                || else_block
                    .as_ref()
                    .map_or(false, |block| block_uses_params_unsafely(block, params))
        }
        Expr::Match(scrutinee, arms, _) => {
            expr_uses_params_unsafely(scrutinee, params)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .map_or(false, |guard| expr_uses_params_unsafely(guard, params))
                        || match &arm.body {
                            MatchBody::Block(block) => block_uses_params_unsafely(block, params),
                            MatchBody::Expr(expr) => expr_uses_params_unsafely(expr, params),
                        }
                })
        }
        Expr::None(_) | Expr::Int(..) | Expr::BigInt(..) | Expr::Float(..) | Expr::Str(..)
        | Expr::Char(..) | Expr::Bool(..) | Expr::Error(..) => false,
    }
}

/// A return type that can carry a by-value aggregate argument. Scalars, unit
/// and Str cannot carry a struct/Vec (Str is a separate pointer payload and is
/// deliberately excluded so read-only `-> Str` helpers keep the caller drop).
fn ret_can_carry(ret: &Option<Type>) -> bool {
    match ret {
        None => false,
        Some(Type::Named(id, _)) => !matches!(
            id.name.as_str(),
            "Int"
                | "UInt"
                | "Float"
                | "Float32"
                | "Float64"
                | "Bool"
                | "Str"
                | "Unit"
                | "Char"
                | "Int8"
                | "Int16"
                | "Int32"
                | "Int64"
                | "UInt8"
                | "UInt16"
                | "UInt32"
                | "UInt64"
        ),
        Some(_) => true,
    }
}