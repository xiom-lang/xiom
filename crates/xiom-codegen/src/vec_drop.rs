// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! m368 (REL-20261011-2032-engine): reclaim OWNED Vec locals at scope exit.
//!
//! The builtin `%struct.Vec` header {data, len, cap, elem_size} is allocated
//! by codegen (malloc/realloc) but historically never freed: `mem.drop` is a
//! no-op and there is no drop glue, so an alloc+drop loop grew RSS without
//! bound (the engine measured 15.9 GB / 10 s at v0.64.4). This module is the
//! compiler-side drop path for the safe subset.
//!
//! It is deliberately CONSERVATIVE. A local name is reclaimed only when
//!   1. every declaration of the name is initialized from a FRESH, uniquely
//!      owned Vec value (`Vec[T].new()` / `Vec[T].with_capacity(..)` / an
//!      array literal), and
//!   2. every use of the name is non-escaping: a known non-view Vec method
//!      (`v.push(..)`, `v.len()`, ...) or a `.len`-style field read.
//! Anything else (return, assignment, argument, index, field store, closure/
//! spawn/unsafe capture, address-of) keeps the historical leak instead of
//! risking a use-after-free: the container ABI carries no refcount, so a
//! header copy aliases the buffer.
//!
//! At codegen each candidate gets an i1 drop flag (rustc's ElaborateDrops
//! mechanism: `compiler/rustc_mir_transform/src/elaborate_drops.rs` inserts
//! drop flags for every place that may be dropped to decide whether the
//! destructor runs). The flag is initialized false in the entry block, set
//! true by the binding, and tested by `compile_vec_drops` on every return
//! path, which makes repeated cleanup sites idempotent and keeps paths that
//! return before the binding from freeing an uninitialized pointer.

use super::IrEmitter;
use crate::context::VecDrop;
use xiom_ast::*;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct VecDeclInfo {
    decls: usize,
    all_fresh: bool,
}

#[derive(Default)]
struct ScanState {
    decls: HashMap<String, VecDeclInfo>,
    /// v0.65.2 Str pass: fresh Str-producing bindings, filtered by the same
    /// escape set as Vecs.
    str_decls: HashMap<String, VecDeclInfo>,
    escaping: HashSet<String>,
    /// Leaf names of functions proven to return a fresh owned Vec. A binding
    /// from one of these is a drop candidate (m375 call-return leak).
    fresh: HashSet<String>,
    /// Leaf names whose Result/Option<Vec> payload is freshly owned. A match
    /// arm binding of that payload is a Vec drop candidate (io.read_at).
    fresh_payload: HashSet<String>,
    /// Locals bound directly from a fresh-payload call (`let r = io.read_at`).
    /// Matching on such a local transfers the payload to the arm binding.
    fresh_payload_locals: HashSet<String>,
    /// Leaf names of `#[owns_return]` aggregate functions (frames).
    fresh_aggregate: HashSet<String>,
    /// Locals bound from a fresh aggregate return; their Vec fields drop.
    aggregate: HashSet<String>,
    /// Fresh-return analysis only: the local being returned transfers
    /// ownership to the caller, so this one return use is not an escape.
    skip_return_local: Option<String>,
    /// `#[owns_return]` function body: the author contracts that unsafe/FFI
    /// does not retain its owned Vec buffers. Scan unsafe-block bodies
    /// normally instead of marking every mention as an escape, so transient
    /// buffers (font_shape's buf/slots) are dropped at function exit.
    trust_unsafe: bool,
    /// Any unsafe block (trusted or not). A `&` borrow inside an unsafe block
    /// stays conservative (foreign code may retain); an ordinary safe call
    /// cannot retain a borrow past the borrow-checked call.
    in_unsafe: bool,
    /// Locals whose owned buffer was MOVED into a struct literal. The
    /// aggregate owns the field, so the source local must not drop it.
    moved: HashSet<String>,
    /// Aggregate locals returned from this function: ownership transfers to
    /// the caller, so the callee must not drop their Vec fields (m283/m318).
    returned: HashSet<String>,
    /// Currently scanning the body of a TRUSTED `unsafe` block (only ever set
    /// while `trust_unsafe` is true). Pointer-exporting reads (`as_ptr`,
    /// `as_mut_ptr`) are passed to FFI under the contract's no-retain promise,
    /// so the receiver keeps ownership and is dropped at scope exit.
    in_trusted_unsafe: bool,
}

pub(crate) struct OwnedLocals {
    pub vec: HashSet<String>,
    pub str: HashSet<String>,
    pub aggregate: HashSet<String>,
}

/// Returns the local names that are owned and private for the function body.
pub(crate) fn collect_owned_vec_locals(
    block: &Block,
    fresh: &HashSet<String>,
    fresh_payload: &HashSet<String>,
    fresh_aggregate: &HashSet<String>,
    trust_unsafe: bool,
) -> OwnedLocals {
    let mut state = ScanState::default();
    state.fresh = fresh.clone();
    state.fresh_payload = fresh_payload.clone();
    state.fresh_aggregate = fresh_aggregate.clone();
    state.trust_unsafe = trust_unsafe;
    scan_block(block, &mut state);
    let vec = state
        .decls
        .into_iter()
        .filter_map(|(name, info)| {
            let owned = info.decls > 0
                && info.all_fresh
                && !state.escaping.contains(&name)
                && !state.moved.contains(&name);
            if owned { Some(name) } else { None }
        })
        .collect();
    let str = state
        .str_decls
        .into_iter()
        .filter_map(|(name, info)| {
            let owned = info.decls > 0
                && info.all_fresh
                && !state.escaping.contains(&name)
                && !state.moved.contains(&name);
            if owned { Some(name) } else { None }
        })
        .collect();
    OwnedLocals {
        vec,
        str,
        aggregate: state
            .aggregate
            .into_iter()
            .filter(|n| !state.returned.contains(n) && !state.moved.contains(n))
            .collect(),
    }
}

/// v0.65.2: fresh Str-producing initializer shapes. Conservative: a concat
/// with a literal/conversion operand, or an explicit `to_str`/`to_string`
/// call. Literals, `from_cstring`/`from_c_str` (foreign/transfer ambiguity),
/// and borrows are excluded.
pub(crate) fn is_fresh_str_init(expr: &Expr) -> bool {
    fn str_operand(expr: &Expr) -> bool {
        match expr {
            Expr::Str(..) => true,
            Expr::Paren(inner, _) => str_operand(inner),
            Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => matches!(
                func.as_ref(),
                Expr::Field(_, method, _)
                    if matches!(
                        method.name.as_str(),
                        "to_str"
                            | "to_string"
                            | "int_to_string"
                            | "float_to_string"
                            | "bool_to_string"
                            | "str_concat"
                    )
            ),
            _ => false,
        }
    }
    // Direct bindings: `Str.to_str` / `Str.to_string` are IDENTITY (they return
    // the receiver), so they must not be dropped. Only the conversions that
    // always allocate are accepted here.
    fn fresh_str_call(func: &Expr) -> bool {
        match func {
            Expr::Field(_, method, _) => matches!(
                method.name.as_str(),
                "int_to_string" | "float_to_string" | "bool_to_string" | "str_concat"
            ),
            Expr::Ident(id) => matches!(
                id.name.as_str(),
                "int_to_string" | "float_to_string" | "bool_to_string" | "str_concat"
            ),
            _ => false,
        }
    }
    match expr {
        Expr::Paren(inner, _) => is_fresh_str_init(inner),
        Expr::Binary(left, BinOp::Add, right, _) => str_operand(left) || str_operand(right),
        Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => fresh_str_call(func),
        _ => false,
    }
}

/// Fresh, uniquely owned Vec initializer shapes. Mirrors vec_abi.rs's
/// `vec_binding_rhs_aliases` "fresh values" arm (Vec.new / array literal) and
/// excludes call returns: a callee may return a header aliasing a field,
/// element, param or global the caller does not own.
pub(crate) fn is_fresh_vec_init(expr: &Expr) -> bool {
    match expr {
        Expr::Paren(inner, _) => is_fresh_vec_init(inner),
        Expr::Array(..) => true,
        Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => match func.as_ref() {
            Expr::Field(obj, method, _) => {
                let name = method.name.as_str();
                if matches!(name, "new" | "with_capacity" | "with_len") {
                    matches!(
                        obj.as_ref(),
                        Expr::Index(base, _, _)
                            if matches!(base.as_ref(), Expr::Ident(b) if b.name == "Vec")
                    )
                } else {
                    // Deep-copy / eager transform: the result owns a fresh
                    // buffer even when the receiver is another Vec.
                    matches!(name, "clone" | "map" | "filter" | "to_owned")
                }
            }
            _ => false,
        },
        _ => false,
    }
}

/// Vec methods that only read the header, mutate elements in place, or build a
/// FRESH result (deep copy / scalar). View-returning or pointer-exporting
/// methods (`slice`, `as_ptr`, `as_mut_ptr`, iterators, `into_*`) are
/// intentionally absent: their result aliases the buffer being freed.
fn is_safe_vec_method(name: &str) -> bool {
    matches!(
        name,
        "len"
            | "is_empty"
            | "capacity"
            | "contains"
            | "index_of"
            | "count"
            | "push"
            | "pop"
            | "insert"
            | "remove"
            | "clear"
            | "sort"
            | "sort_by"
            | "reverse"
            | "resize"
            | "reserve"
            | "shrink_to_fit"
            | "truncate"
            | "extend"
            | "retain"
            | "dedup"
            | "fill"
            | "swap"
            | "swap_remove"
            | "get"
            | "first"
            | "last"
            | "peek"
            | "binary_search"
            | "clone"
            | "map"
            | "filter"
            | "fold"
            | "reduce"
            | "sum"
            | "min"
            | "max"
            | "any"
            | "all"
            | "for_each"
            | "join"
            | "to_string"
            | "eq"
            | "hash"
            | "compare"
    )
}

/// Pointer-export methods whose result aliases the Vec buffer. They are only
/// non-escaping under the `#[owns_return]` no-retain contract inside an
/// `unsafe` block; elsewhere they keep the historical conservative escape.
fn is_trusted_ptr_export(name: &str) -> bool {
    matches!(name, "as_ptr" | "as_mut_ptr")
}

#[derive(Clone, Copy, PartialEq)]
enum UseCtx {
    /// A bare mention escapes the function (default).
    Escape,
    /// The expression is the receiver of a known non-escaping Vec method or
    /// of a `.len`-style field read; a bare identifier there is a safe use.
    SafeBase,
}

fn scan_block(block: &Block, state: &mut ScanState) {
    for item in &block.stmts {
        match item {
            StmtOrExpr::Stmt(stmt) => scan_stmt(stmt, state),
            StmtOrExpr::Expr(expr) => scan_expr(expr, UseCtx::Escape, state),
        }
    }
}

fn scan_stmt(stmt: &Stmt, state: &mut ScanState) {
    match stmt {
        Stmt::Let(name, _, value, _) | Stmt::Var(name, _, value, _) => {
            // REL-20261027-1717-engine: a local struct literal that owns a
            // fresh Vec field is an aggregate drop site. The source Vec local
            // is MOVED into the field (do not drop it separately); a fresh
            // Vec field expression is owned by the literal.
            if let Expr::Struct(_, fields, _, _) = value {
                let mut owns_vec_field = false;
                for (_, fv) in fields {
                    if let Expr::Ident(id) = fv {
                        let owned_vec = state
                            .decls
                            .get(&id.name)
                            .map_or(false, |i| i.decls > 0 && i.all_fresh);
                        if owned_vec
                            && !state.escaping.contains(&id.name)
                            && !state.moved.contains(&id.name)
                        {
                            state.moved.insert(id.name.clone());
                            owns_vec_field = true;
                        }
                    }
                    if is_fresh_vec_init(fv) {
                        owns_vec_field = true;
                    }
                }
                if owns_vec_field {
                    state.aggregate.insert(name.name.clone());
                }
            }
            let fresh = is_fresh_vec_init(value) || is_fresh_vec_call(value, &state.fresh);
            if is_fresh_payload_call(value, &state.fresh_payload) {
                state.fresh_payload_locals.insert(name.name.clone());
            }
            if is_fresh_aggregate_call(value, &state.fresh_aggregate) {
                state.aggregate.insert(name.name.clone());
            }
            let info = state.decls.entry(name.name.clone()).or_insert_with(|| VecDeclInfo {
                decls: 0,
                all_fresh: true,
            });
            info.decls += 1;
            info.all_fresh &= fresh;
            let fresh_str = is_fresh_str_init(value);
            let sinfo = state
                .str_decls
                .entry(name.name.clone())
                .or_insert_with(|| VecDeclInfo {
                    decls: 0,
                    all_fresh: true,
                });
            sinfo.decls += 1;
            sinfo.all_fresh &= fresh_str;
            scan_expr(value, UseCtx::Escape, state);
        }
        Stmt::Assign(place, value, _) => {
            // Rebinding an owned local would replace the header the drop flag
            // tracks. Under the `#[owns_return]` contract a fresh-to-fresh
            // rebind keeps the local owned: the flag stays armed and the final
            // buffer is dropped at scope exit (the replaced buffer is the only
            // residual leak). Any other rebind may install an alias, so the
            // historical conservative escape remains.
            if let Expr::Ident(id) = place {
                let fresh_rebind = state.trust_unsafe
                    && state
                        .decls
                        .get(&id.name)
                        .map_or(false, |info| info.decls == 1 && info.all_fresh)
                    && (is_fresh_vec_init(value)
                        || is_fresh_vec_call(value, &state.fresh));
                if !fresh_rebind {
                    state.escaping.insert(id.name.clone());
                }
            } else {
                scan_expr(place, UseCtx::Escape, state);
            }
            scan_expr(value, UseCtx::Escape, state);
        }
        Stmt::Return(Some(expr), _) => {
            // Returning an aggregate transfers ownership to the caller; its
            // Vec fields must not be freed by this function's exit drop.
            mark_returned_roots(expr, state);
            // Fresh-return analysis: returning the local transfers its owned
            // buffer to the caller; do not count that use as an escape.
            if state
                .skip_return_local
                .as_deref()
                .map_or(false, |name| is_returned_local(expr, name))
            {
                return;
            }
            scan_expr(expr, UseCtx::Escape, state)
        }
        Stmt::Return(None, _) | Stmt::Break(..) | Stmt::Continue(..) | Stmt::Debugger(_) => {}
        Stmt::Expr(expr, _) => scan_expr(expr, UseCtx::Escape, state),
        Stmt::If(cond, then_block, elifs, else_block, _) => {
            scan_expr(cond, UseCtx::Escape, state);
            scan_block(then_block, state);
            for (elif_cond, elif_block) in elifs {
                scan_expr(elif_cond, UseCtx::Escape, state);
                scan_block(elif_block, state);
            }
            if let Some(else_block) = else_block {
                scan_block(else_block, state);
            }
        }
        Stmt::Match(scrutinee, arms, _) => {
            // A match on a fresh-payload Result/Option<Vec> call (or a local
            // bound from one) binds an owned Vec. Register the arm binding for
            // a scope-exit drop.
            if is_fresh_payload_call(scrutinee, &state.fresh_payload)
                || is_fresh_payload_local(scrutinee, &state.fresh_payload_locals)
            {
                for arm in arms {
                    if let Some(name) = payload_binding_name(&arm.pattern) {
                        let info = state.decls.entry(name).or_insert_with(|| VecDeclInfo {
                            decls: 0,
                            all_fresh: true,
                        });
                        info.decls += 1;
                        info.all_fresh = true;
                    }
                }
            }
            scan_expr(scrutinee, UseCtx::Escape, state);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    scan_expr(guard, UseCtx::Escape, state);
                }
                match &arm.body {
                    MatchBody::Block(block) => scan_block(block, state),
                    MatchBody::Expr(expr) => scan_expr(expr, UseCtx::Escape, state),
                }
            }
        }
        Stmt::While(cond, body, invariant, _, _) => {
            scan_expr(cond, UseCtx::Escape, state);
            if let Some(inv) = invariant {
                scan_expr(inv, UseCtx::Escape, state);
            }
            scan_block(body, state);
        }
        Stmt::For(_, iter, body, _, _) => {
            scan_expr(iter, UseCtx::Escape, state);
            scan_block(body, state);
        }
        Stmt::Spawn(body, _, _) => mark_block_mentions(body, state),
        Stmt::Destructure(_, value, _) => scan_expr(value, UseCtx::Escape, state),
        Stmt::Asm(asm_block) => {
            for (_, expr) in &asm_block.inputs {
                scan_expr(expr, UseCtx::Escape, state);
            }
        }
        Stmt::Defer(body, _) => {
            // A defer body is re-compiled at every return path; its own Vec
            // declarations cannot get per-path drop state. Only the outer
            // names it mentions matter (a defer runs before the drop).
            mark_block_mentions(body, state);
        }
        Stmt::Assert(cond, msg, _) => {
            scan_expr(cond, UseCtx::Escape, state);
            if let Some(msg) = msg {
                scan_expr(msg, UseCtx::Escape, state);
            }
        }
    }
}

fn scan_expr(expr: &Expr, ctx: UseCtx, state: &mut ScanState) {
    match expr {
        Expr::Ident(id) => {
            if ctx != UseCtx::SafeBase {
                state.escaping.insert(id.name.clone());
            }
        }
        Expr::Paren(inner, _) => scan_expr(inner, ctx, state),
        Expr::Field(base, field, _) => {
            let safe_read = matches!(field.name.as_str(), "len" | "capacity" | "is_empty");
            if safe_read && matches!(base.as_ref(), Expr::Ident(_)) {
                scan_expr(base, UseCtx::SafeBase, state);
            } else {
                scan_expr(base, UseCtx::Escape, state);
            }
        }
        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
            // REL-20261027-1956-xvector: a by-value aggregate pushed into a
            // container is MOVED into the container; the caller must not free
            // its Vec fields at scope exit (the container owns the copy).
            let container_store = matches!(
                func.as_ref(),
                Expr::Field(_, method, _)
                    if matches!(
                        method.name.as_str(),
                        "push" | "insert" | "set" | "add" | "append" | "put"
                    )
            );
            match func.as_ref() {
                Expr::Field(recv, method, _) if is_safe_vec_method(method.name.as_str()) => {
                    scan_expr(recv, UseCtx::SafeBase, state);
                }
                // `#[owns_return]` contract inside `unsafe`: an FFI call is
                // promised not to retain the buffer, so exporting its pointer
                // does not transfer ownership. The Vec stays owned by this
                // function and is dropped at scope exit (font_shape's buf).
                Expr::Field(recv, method, _)
                    if state.trust_unsafe
                        && state.in_trusted_unsafe
                        && is_trusted_ptr_export(method.name.as_str()) =>
                {
                    scan_expr(recv, UseCtx::SafeBase, state);
                }
                other => scan_expr(other, UseCtx::Escape, state),
            }
            for arg in args {
                if container_store {
                    if let Expr::Ident(id) = arg {
                        if state.aggregate.contains(&id.name) {
                            state.moved.insert(id.name.clone());
                            continue;
                        }
                    }
                }
                // REL-20261027-1717-engine: a non-retaining `&`/`&mut`
                // borrow to an ordinary call does not transfer the Vec/Str
                // buffer. The borrow checker keeps the reference inside the
                // call, so the caller still owns and drops the local. Inside
                // an unsafe/FFI block keep the conservative escape (foreign
                // code may retain), except under the trusted owns_return
                // contract.
                let borrow_safe = !state.in_unsafe
                    || (state.trust_unsafe && state.in_trusted_unsafe);
                if borrow_safe {
                    if let Expr::Ref(inner, _) | Expr::MutRef(inner, _) = arg {
                        scan_borrow_arg(inner, state);
                        continue;
                    }
                }
                scan_expr(arg, UseCtx::Escape, state);
            }
        }
        Expr::Index(base, index, _) => {
            // Reading/writing an ELEMENT does not transfer the Vec header or
            // its buffer; the buffer stays owned by the local. A reference to
            // the element (`&v[i]`) is handled by the Ref/MutRef arm below and
            // DOES escape. This is the engine vecprobe `v[0] = i` shape.
            if matches!(base.as_ref(), Expr::Ident(id) if state.decls.contains_key(&id.name)) {
                scan_expr(index, UseCtx::Escape, state);
            } else {
                // Field/nested containers may alias the outer buffer.
                scan_expr(base, UseCtx::Escape, state);
                scan_expr(index, UseCtx::Escape, state);
            }
        }
        Expr::Ref(inner, _) | Expr::MutRef(inner, _) => scan_ref_target(inner, state),
        Expr::Unary(_, inner, _)
        | Expr::Try(inner, _)
        | Expr::AtPre(inner, _)
        | Expr::Await(inner, _)
        | Expr::Comptime(inner, _)
        | Expr::As(inner, _, _)
        | Expr::Some(inner, _)
        | Expr::Ok(inner, _)
        | Expr::Err(inner, _) => scan_expr(inner, UseCtx::Escape, state),
        Expr::Binary(left, _, right, _) | Expr::Imply(left, right, _) => {
            scan_expr(left, UseCtx::Escape, state);
            scan_expr(right, UseCtx::Escape, state);
        }
        Expr::Is(inner, _, _) => scan_expr(inner, UseCtx::Escape, state),
        Expr::Struct(_, fields, spread, _) => {
            for (_, value) in fields {
                scan_expr(value, UseCtx::Escape, state);
            }
            if let Some(spread) = spread {
                scan_expr(spread, UseCtx::Escape, state);
            }
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
            for elem in elems {
                scan_expr(elem, UseCtx::Escape, state);
            }
        }
        Expr::BlockExpr(block, _) => scan_block(block, state),
        Expr::Closure(_, _, body, _) => mark_block_mentions(body, state),
        Expr::PipeClosure(_, body, _) => scan_expr(body, UseCtx::Escape, state),
        Expr::If(cond, then_block, elifs, else_block, _) => {
            scan_expr(cond, UseCtx::Escape, state);
            scan_block(then_block, state);
            for (elif_cond, elif_block) in elifs {
                scan_expr(elif_cond, UseCtx::Escape, state);
                scan_block(elif_block, state);
            }
            if let Some(else_block) = else_block {
                scan_block(else_block, state);
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            scan_expr(scrutinee, UseCtx::Escape, state);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    scan_expr(guard, UseCtx::Escape, state);
                }
                match &arm.body {
                    MatchBody::Block(block) => scan_block(block, state),
                    MatchBody::Expr(expr) => scan_expr(expr, UseCtx::Escape, state),
                }
            }
        }
        Expr::Unsafe(body, _) => {
            let was_unsafe = state.in_unsafe;
            state.in_unsafe = true;
            if state.trust_unsafe {
                // #[owns_return] contract: the unsafe/FFI does not retain the
                // function's owned Vec buffers; transient buffers stay owned
                // and are dropped at this function's scope exit. Only inside
                // this block may a pointer export count as a non-escaping use.
                let was_trusted = state.in_trusted_unsafe;
                state.in_trusted_unsafe = true;
                scan_block(body, state);
                state.in_trusted_unsafe = was_trusted;
            } else {
                // Captured locals are passed BY ADDRESS to a standalone block
                // fn; any outer Vec mentioned there escapes this fn's lifetime.
                mark_block_mentions(body, state);
            }
            state.in_unsafe = was_unsafe;
        }
        Expr::ConstBlock(inner, _) => scan_expr(inner, UseCtx::Escape, state),
        Expr::None(_) | Expr::Int(..) | Expr::BigInt(..) | Expr::Float(..) | Expr::Str(..)
        | Expr::Char(..) | Expr::Bool(..) | Expr::Error(..) => {}
    }
}

fn call_leaf(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Ident(id) => id.name.rsplit('.').next(),
        Expr::Field(_, method, _) => Some(method.name.as_str()),
        Expr::Paren(inner, _) => call_leaf(inner),
        _ => None,
    }
}

/// True when `expr` is a call to a leaf the fresh-return prepass proved owns
/// its returned Vec. Used by the binding scan and the drop-candidate check.
pub(crate) fn is_fresh_vec_call(expr: &Expr, fresh: &HashSet<String>) -> bool {
    match expr {
        Expr::Paren(inner, _) => is_fresh_vec_call(inner, fresh),
        Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
            call_leaf(func).map_or(false, |leaf| fresh.contains(leaf))
        }
        _ => false,
    }
}

/// Same shape, for functions whose Result/Option payload is a fresh Vec.
pub(crate) fn is_fresh_payload_call(expr: &Expr, fresh: &HashSet<String>) -> bool {
    match expr {
        Expr::Paren(inner, _) => is_fresh_payload_call(inner, fresh),
        Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
            call_leaf(func).map_or(false, |leaf| fresh.contains(leaf))
        }
        _ => false,
    }
}

fn is_fresh_payload_local(expr: &Expr, locals: &HashSet<String>) -> bool {
    match expr {
        Expr::Paren(inner, _) => is_fresh_payload_local(inner, locals),
        Expr::Ident(id) => locals.contains(&id.name),
        _ => false,
    }
}

/// Call to an `#[owns_return]` aggregate function (struct with owned Vec
/// fields), e.g. an editor frame builder.
pub(crate) fn is_fresh_aggregate_call(expr: &Expr, fresh: &HashSet<String>) -> bool {
    match expr {
        Expr::Paren(inner, _) => is_fresh_aggregate_call(inner, fresh),
        Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
            call_leaf(func).map_or(false, |leaf| fresh.contains(leaf))
        }
        _ => false,
    }
}

fn payload_binding_name(pattern: &Pattern) -> Option<String> {
    match pattern {
        Pattern::Ok(inner, _) | Pattern::Some(inner, _) => match inner.as_ref() {
            Pattern::Ident(id) => Some(id.name.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn is_returned_local(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Paren(inner, _) | Expr::Ok(inner, _) | Expr::Some(inner, _) => {
            is_returned_local(inner, name)
        }
        Expr::Ident(id) => id.name == name,
        _ => false,
    }
}

/// Fresh-return analysis helper: is `name`, returned from this function body,
/// a uniquely owned Vec local (fresh declaration + no escaping use other than
/// the return itself)? Ownership transfers to the caller, so the callee's drop
/// pass does not free it.
pub(crate) fn local_is_owned_fresh(
    body: &Block,
    name: &str,
    fresh: &HashSet<String>,
) -> bool {
    let mut state = ScanState::default();
    state.fresh = fresh.clone();
    state.skip_return_local = Some(name.to_string());
    if let Some((last, rest)) = body.stmts.split_last() {
        for item in rest {
            match item {
                StmtOrExpr::Stmt(stmt) => scan_stmt(stmt, &mut state),
                StmtOrExpr::Expr(expr) => scan_expr(expr, UseCtx::Escape, &mut state),
            }
        }
        match last {
            StmtOrExpr::Expr(expr) if is_returned_local(expr, name) => {}
            StmtOrExpr::Expr(expr) => scan_expr(expr, UseCtx::Escape, &mut state),
            StmtOrExpr::Stmt(stmt) => scan_stmt(stmt, &mut state),
        }
    }
    state
        .decls
        .get(name)
        .map_or(false, |info| info.decls == 1 && info.all_fresh)
        && !state.escaping.contains(name)
}

/// A reference to an element/field aliases the owning Vec buffer: mark the
/// base local escaping even though a plain index read/write is safe.
fn scan_ref_target(expr: &Expr, state: &mut ScanState) {
    match expr {
        Expr::Paren(inner, _) => scan_ref_target(inner, state),
        Expr::Index(base, index, _) => {
            scan_ref_target(base, state);
            scan_expr(index, UseCtx::Escape, state);
        }
        Expr::Field(base, _, _) => scan_ref_target(base, state),
        _ => scan_expr(expr, UseCtx::Escape, state),
    }
}

/// A borrow passed to a non-retaining ordinary call: keep the root local
/// owned; still scan index expressions for their own escapes. Roots reached
/// through Paren/Field/Index are recognized; anything else falls back to the
/// conservative escape.
fn scan_borrow_arg(expr: &Expr, state: &mut ScanState) {
    match expr {
        Expr::Paren(inner, _) => scan_borrow_arg(inner, state),
        Expr::Index(base, index, _) => {
            scan_borrow_arg(base, state);
            scan_expr(index, UseCtx::Escape, state);
        }
        Expr::Field(base, _, _) => scan_borrow_arg(base, state),
        Expr::Ident(_) => {}
        _ => scan_expr(expr, UseCtx::Escape, state),
    }
}

/// Mark the owned roots of a returned expression as transferred. The caller
/// owns the returned aggregate; the callee's scope-exit drop must skip its
/// Vec fields (m283 `return b`, m318 `return w`/`grid`).
fn mark_returned_roots(expr: &Expr, state: &mut ScanState) {
    match expr {
        Expr::Paren(inner, _) => mark_returned_roots(inner, state),
        Expr::Ident(id) => {
            if state.decls.contains_key(&id.name)
                || state.str_decls.contains_key(&id.name)
                || state.aggregate.contains(&id.name)
            {
                state.returned.insert(id.name.clone());
            }
        }
        Expr::Field(base, _, _) | Expr::Index(base, _, _) => mark_returned_roots(base, state),
        Expr::Struct(_, fields, spread, _) => {
            for (_, value) in fields {
                mark_returned_roots(value, state);
            }
            if let Some(spread) = spread {
                mark_returned_roots(spread, state);
            }
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
            for elem in elems {
                mark_returned_roots(elem, state);
            }
        }
        Expr::Unary(_, inner, _)
        | Expr::Try(inner, _)
        | Expr::AtPre(inner, _)
        | Expr::Await(inner, _)
        | Expr::Comptime(inner, _)
        | Expr::As(inner, _, _)
        | Expr::Some(inner, _)
        | Expr::Ok(inner, _)
        | Expr::Err(inner, _) => mark_returned_roots(inner, state),
        _ => {}
    }
}

/// Mark every identifier mentioned anywhere under `block` as escaping. Used
/// for bodies emitted into a DIFFERENT function (closures, spawn, confined
/// block fns): their captures are by-value/by-address and their own locals do
/// not share this function's drop state.
fn mark_block_mentions(block: &Block, state: &mut ScanState) {
    for item in &block.stmts {
        match item {
            StmtOrExpr::Stmt(stmt) => mark_stmt_mentions(stmt, state),
            StmtOrExpr::Expr(expr) => mark_expr_mentions(expr, state),
        }
    }
}

fn mark_stmt_mentions(stmt: &Stmt, state: &mut ScanState) {
    match stmt {
        Stmt::Let(_, _, value, _) | Stmt::Var(_, _, value, _) => mark_expr_mentions(value, state),
        Stmt::Assign(place, value, _) => {
            mark_expr_mentions(place, state);
            mark_expr_mentions(value, state);
        }
        Stmt::Return(Some(expr), _) | Stmt::Expr(expr, _) => mark_expr_mentions(expr, state),
        Stmt::Return(None, _) | Stmt::Break(..) | Stmt::Continue(..) | Stmt::Debugger(_) => {}
        Stmt::If(cond, then_block, elifs, else_block, _) => {
            mark_expr_mentions(cond, state);
            mark_block_mentions(then_block, state);
            for (elif_cond, elif_block) in elifs {
                mark_expr_mentions(elif_cond, state);
                mark_block_mentions(elif_block, state);
            }
            if let Some(else_block) = else_block {
                mark_block_mentions(else_block, state);
            }
        }
        Stmt::Match(scrutinee, arms, _) => {
            mark_expr_mentions(scrutinee, state);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    mark_expr_mentions(guard, state);
                }
                match &arm.body {
                    MatchBody::Block(block) => mark_block_mentions(block, state),
                    MatchBody::Expr(expr) => mark_expr_mentions(expr, state),
                }
            }
        }
        Stmt::While(cond, body, invariant, _, _) => {
            mark_expr_mentions(cond, state);
            if let Some(inv) = invariant {
                mark_expr_mentions(inv, state);
            }
            mark_block_mentions(body, state);
        }
        Stmt::For(_, iter, body, _, _) => {
            mark_expr_mentions(iter, state);
            mark_block_mentions(body, state);
        }
        Stmt::Spawn(body, _, _) => mark_block_mentions(body, state),
        Stmt::Destructure(_, value, _) => mark_expr_mentions(value, state),
        Stmt::Asm(asm_block) => {
            for (_, expr) in &asm_block.inputs {
                mark_expr_mentions(expr, state);
            }
        }
        Stmt::Defer(body, _) => mark_block_mentions(body, state),
        Stmt::Assert(cond, msg, _) => {
            mark_expr_mentions(cond, state);
            if let Some(msg) = msg {
                mark_expr_mentions(msg, state);
            }
        }
    }
}

fn mark_expr_mentions(expr: &Expr, state: &mut ScanState) {
    match expr {
        Expr::Ident(id) => {
            state.escaping.insert(id.name.clone());
        }
        Expr::Paren(inner, _)
        | Expr::Unary(_, inner, _)
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
        | Expr::ConstBlock(inner, _) => mark_expr_mentions(inner, state),
        Expr::Binary(left, _, right, _) | Expr::Imply(left, right, _) => {
            mark_expr_mentions(left, state);
            mark_expr_mentions(right, state);
        }
        Expr::Field(base, _, _) => mark_expr_mentions(base, state),
        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
            mark_expr_mentions(func, state);
            for arg in args {
                mark_expr_mentions(arg, state);
            }
        }
        Expr::Index(base, index, _) => {
            mark_expr_mentions(base, state);
            mark_expr_mentions(index, state);
        }
        Expr::Struct(_, fields, spread, _) => {
            for (_, value) in fields {
                mark_expr_mentions(value, state);
            }
            if let Some(spread) = spread {
                mark_expr_mentions(spread, state);
            }
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
            for elem in elems {
                mark_expr_mentions(elem, state);
            }
        }
        Expr::BlockExpr(block, _) | Expr::Closure(_, _, block, _) | Expr::Unsafe(block, _) => {
            mark_block_mentions(block, state);
        }
        Expr::PipeClosure(_, body, _) => mark_expr_mentions(body, state),
        Expr::If(cond, then_block, elifs, else_block, _) => {
            mark_expr_mentions(cond, state);
            mark_block_mentions(then_block, state);
            for (elif_cond, elif_block) in elifs {
                mark_expr_mentions(elif_cond, state);
                mark_block_mentions(elif_block, state);
            }
            if let Some(else_block) = else_block {
                mark_block_mentions(else_block, state);
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            mark_expr_mentions(scrutinee, state);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    mark_expr_mentions(guard, state);
                }
                match &arm.body {
                    MatchBody::Block(block) => mark_block_mentions(block, state),
                    MatchBody::Expr(expr) => mark_expr_mentions(expr, state),
                }
            }
        }
        Expr::None(_)
        | Expr::Int(..)
        | Expr::BigInt(..)
        | Expr::Float(..)
        | Expr::Str(..)
        | Expr::Char(..)
        | Expr::Bool(..)
        | Expr::Error(..) => {}
    }
}

/// Saved per-function Vec-drop state while a detached function body (closure,
/// spawn, confined block) is emitted into its own output buffer.
pub(crate) struct VecDropScope {
    drops: Vec<VecDrop>,
    safe: HashSet<String>,
    safe_str: HashSet<String>,
    safe_aggregate: HashSet<String>,
    entry_prelude: Vec<String>,
}

impl IrEmitter {
    /// Build the owned-and-private local set for the function about to be
    /// compiled. Called at function-body start (regular and mono paths).
    pub(crate) fn prepass_owned_vec_locals(&mut self, body: &Block, trust_unsafe: bool) {
        let fresh = self.fresh_vec_returns.clone();
        let fresh_payload = self.fresh_payload_returns.clone();
        let fresh_aggregate = self.fresh_aggregate_returns.clone();
        let owned = collect_owned_vec_locals(
            body,
            &fresh,
            &fresh_payload,
            &fresh_aggregate,
            trust_unsafe,
        );
        self.fctx.vec_drop_safe = owned.vec;
        self.fctx.str_drop_safe = owned.str;
        self.fctx.aggregate_drop_safe = owned.aggregate;
    }

    /// Whole-program ownership prepass (m375 / engine Vec leak). Runs before
    /// any function body is compiled so callers can see fresh-return callees.
    pub(crate) fn prepass_fresh_vec_returns(&mut self, program: &Program) {
        let (vec, payload, aggregate) = crate::fresh_return::collect_fresh_ownership(program);
        if std::env::var("XIOM_DEBUG_OWNERSHIP").is_ok() {
            eprintln!(
                "ownership: vec={} payload={} aggregate={} read_at_vec={} read_at_payload={} make_frame_aggregate={}",
                vec.len(),
                payload.len(),
                aggregate.len(),
                vec.contains("read_at"),
                payload.contains("read_at"),
                aggregate.contains("make_frame")
            );
        }
        self.fresh_vec_returns = vec;
        self.fresh_payload_returns = payload;
        self.fresh_aggregate_returns = aggregate;
    }

    /// Reset per-function drop state. Called at every function-body start.
    pub(crate) fn clear_vec_drops(&mut self) {
        self.fctx.vec_drops.clear();
        self.fctx.vec_drop_safe.clear();
        self.fctx.str_drop_safe.clear();
        self.fctx.aggregate_drop_safe.clear();
        self.local.entry_prelude.clear();
    }

    /// Detach the enclosing function's drop state while emitting a function
    /// body into a separate output buffer (a return inside that body must not
    /// free the enclosing frame's Vecs).
    pub(crate) fn suspend_vec_drops(&mut self) -> VecDropScope {
        VecDropScope {
            drops: std::mem::take(&mut self.fctx.vec_drops),
            safe: std::mem::take(&mut self.fctx.vec_drop_safe),
            safe_str: std::mem::take(&mut self.fctx.str_drop_safe),
            safe_aggregate: std::mem::take(&mut self.fctx.aggregate_drop_safe),
            entry_prelude: std::mem::take(&mut self.local.entry_prelude),
        }
    }

    pub(crate) fn resume_vec_drops(&mut self, saved: VecDropScope) {
        self.fctx.vec_drops = saved.drops;
        self.fctx.vec_drop_safe = saved.safe;
        self.fctx.str_drop_safe = saved.safe_str;
        self.fctx.aggregate_drop_safe = saved.safe_aggregate;
        self.local.entry_prelude = saved.entry_prelude;
    }

    /// True when a `%struct.Vec` binding is an owned/private reclamation
    /// candidate. Callers use this BEFORE emitting the local's alloca so the
    /// slot can be hoisted into the entry block (a cleanup emitted after an
    /// enclosing `if` must be dominated by the alloca it loads).
    pub(crate) fn vec_drop_candidate(&self, name: &str, llvm_ty: &str, init: &Expr) -> bool {
        llvm_ty == "%struct.Vec"
            && self.fctx.vec_drop_safe.contains(name)
            && (is_fresh_vec_init(init)
                || is_fresh_vec_call(init, &self.fresh_vec_returns))
    }

    /// Register a freshly bound Vec local for reclamation. Emits the flag
    /// initialization (entry block, via finish_hoisted_allocas) and arms the
    /// flag right after the binding stores its header.
    pub(crate) fn register_vec_drop(&mut self, name: &str, alloca: &str, llvm_ty: &str, init: &Expr) {
        if !self.vec_drop_candidate(name, llvm_ty, init) {
            return;
        }
        let flag = self.fresh_tmp();
        self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
        self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
        self.fctx.vec_drops.push(VecDrop::Vec {
            alloca: alloca.to_string(),
            flag: flag.clone(),
        });
        self.emitln(&format!("  store i1 true, i1* {flag}"));
    }

    /// Register a fresh, non-escaping Str local for reclamation. The pointer
    /// is captured HERE into a hoisted i8* slot so the scope-exit drop never
    /// loads an alloca from a non-dominating if-arm (engine sfr_unique).
    pub(crate) fn register_str_drop(&mut self, name: &str, alloca: &str, llvm_ty: &str, init: &Expr) {
        if llvm_ty != "i8*"
            || !self.fctx.str_drop_safe.contains(name)
            || !is_fresh_str_init(init)
        {
            return;
        }
        let data_ptr = self.fresh_tmp();
        self.emitln(&format!("  {data_ptr} = load i8*, i8** {alloca}"));
        let data_slot = self.fresh_tmp();
        self.local.hoisted_allocas.push((data_slot.clone(), "i8*".to_string()));
        self.local
            .entry_prelude
            .push(format!("  store i8* null, i8** {data_slot}"));
        self.emitln(&format!("  store i8* {data_ptr}, i8** {data_slot}"));
        let flag = self.fresh_tmp();
        self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
        self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
        self.fctx.vec_drops.push(VecDrop::Ptr {
            data_alloca: data_slot,
            flag: flag.clone(),
        });
        self.emitln(&format!("  store i1 true, i1* {flag}"));
    }

    /// Register the Vec fields of an `#[owns_return]` aggregate binding. The
    /// struct slot is local to the binding scope; the loop/arm/function drop
    /// range covers it (the struct alloca dominates that exit).
    pub(crate) fn register_aggregate_drops(&mut self, name: &str, alloca: &str, llvm_ty: &str) {
        if !self.fctx.aggregate_drop_safe.contains(name) || !llvm_ty.starts_with("%struct.") {
            return;
        }
        let type_name = llvm_ty.trim_start_matches("%struct.").to_string();
        let meta = self
            .types
            .type_meta
            .get(&type_name)
            .or_else(|| {
                self.types
                    .type_meta
                    .entries()
                    .into_iter()
                    .find(|(k, _)| k.ends_with(&format!(".{type_name}")))
                    .map(|(_, meta)| meta)
            });
        let Some(meta) = meta else { return; };
        for (idx, _) in meta.fields.iter().enumerate() {
            if self.field_llvm_type(&type_name, idx) != "%struct.Vec" {
                continue;
            }
            // Capture the field's data pointer HERE (the struct slot is live
            // and dominates this block). The drop loads the hoisted i8* slot,
            // so a struct literal in a nested arm never forces a
            // non-dominating alloca use at scope exit.
            let field_ptr = self.fresh_tmp();
            self.emitln(&format!(
                "  {field_ptr} = getelementptr {llvm_ty}, {llvm_ty}* {alloca}, i32 0, i32 {idx}"
            ));
            let data_gep = self.fresh_tmp();
            self.emitln(&format!(
                "  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {field_ptr}, i32 0, i32 0"
            ));
            let data_ptr = self.fresh_tmp();
            self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
            let data_slot = self.fresh_tmp();
            self.local.hoisted_allocas.push((data_slot.clone(), "i8*".to_string()));
            self.local
                .entry_prelude
                .push(format!("  store i8* null, i8** {data_slot}"));
            self.emitln(&format!("  store i8* {data_ptr}, i8** {data_slot}"));
            let flag = self.fresh_tmp();
            self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
            self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
            self.fctx.vec_drops.push(VecDrop::Ptr {
                data_alloca: data_slot,
                flag: flag.clone(),
            });
            self.emitln(&format!("  store i1 true, i1* {flag}"));
        }
    }

    /// Register a match-bound Vec payload the prepass proved uniquely owned
    /// (fresh-payload Result/Option, e.g. `match io.read_at(..) { Ok(v) => .. }`).
    /// The payload slot already lives in the arm; the scope drop frees it there.
    pub(crate) fn register_owned_vec_binding(&mut self, name: &str, alloca: &str, llvm_ty: &str) {
        if llvm_ty != "%struct.Vec" || !self.fctx.vec_drop_safe.contains(name) {
            return;
        }
        let flag = self.fresh_tmp();
        self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
        self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
        self.fctx.vec_drops.push(VecDrop::Vec {
            alloca: alloca.to_string(),
            flag: flag.clone(),
        });
        self.emitln(&format!("  store i1 true, i1* {flag}"));
    }

    /// Emit the pending Vec deallocations (LIFO). Safe to emit on every return
    /// path: the drop flag has already been cleared on paths that ran this.
    pub(crate) fn compile_vec_drops(&mut self) -> Result<(), String> {
        if self.fctx.vec_drops.is_empty() {
            return Ok(());
        }
        let drops = self.fctx.vec_drops.clone();
        self.emit_vec_drops(&drops)
    }

    /// Emit drops for the entries added since `start` without removing them.
    /// Used at loop-body/scope exits: the same compiled drop code runs each
    /// iteration, and the function-return pass still sees the entries for
    /// paths that leave the scope early (break/continue/return).
    pub(crate) fn compile_vec_drops_since(&mut self, start: usize) -> Result<(), String> {
        if self.fctx.vec_drops.len() <= start {
            return Ok(());
        }
        let drops = self.fctx.vec_drops[start..].to_vec();
        self.emit_vec_drops(&drops)
    }

    /// Emit and REMOVE the entries added since `start` (match-arm scope).
    /// The arm slot only dominates the arm's own exit, so it must not leak
    /// into the enclosing loop/function drop list.
    pub(crate) fn compile_vec_drops_scope(&mut self, start: usize) -> Result<(), String> {
        if self.fctx.vec_drops.len() <= start {
            return Ok(());
        }
        let drops = self.fctx.vec_drops[start..].to_vec();
        self.emit_vec_drops(&drops)?;
        self.fctx.vec_drops.truncate(start);
        Ok(())
    }

    fn emit_vec_drops(&mut self, drops: &[VecDrop]) -> Result<(), String> {
        for drop in drops.iter().rev() {
            let flag = drop.flag();
            let armed = self.fresh_tmp();
            let do_block = self.fresh_block("vecdrop_do");
            let skip_block = self.fresh_block("vecdrop_skip");
            self.emitln(&format!("  {armed} = load i1, i1* {flag}"));
            self.emitln(&format!("  br i1 {armed}, label %{do_block}, label %{skip_block}"));
            self.emitln(&format!("\n{do_block}:"));
            let data_ptr = match drop {
                VecDrop::Vec { alloca, .. } => {
                    let data_gep = self.fresh_tmp();
                    let data_ptr = self.fresh_tmp();
                    self.emitln(&format!(
                        "  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {alloca}, i32 0, i32 0"
                    ));
                    self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                    data_ptr
                }
                VecDrop::Str { alloca, .. } => {
                    let data_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {data_ptr} = load i8*, i8** {alloca}"));
                    data_ptr
                }
                VecDrop::Ptr { data_alloca, .. } => {
                    let data_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_alloca}"));
                    data_ptr
                }
                VecDrop::Aggregate {
                    struct_alloca,
                    struct_ty,
                    field_index,
                    ..
                } => {
                    let field_ptr = self.fresh_tmp();
                    self.emitln(&format!(
                        "  {field_ptr} = getelementptr {struct_ty}, {struct_ty}* {struct_alloca}, i32 0, i32 {field_index}"
                    ));
                    let data_gep = self.fresh_tmp();
                    let data_ptr = self.fresh_tmp();
                    self.emitln(&format!(
                        "  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {field_ptr}, i32 0, i32 0"
                    ));
                    self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                    data_ptr
                }
            };
            // Arena-aware: block-fn Vecs live on the guard arena and are
            // reclaimed wholesale; main-heap Vecs are freed here.
            self.emitln(&format!("  call void @xiom_guard_free(i8* {data_ptr})"));
            self.emitln(&format!("  store i1 false, i1* {flag}"));
            self.emitln(&format!("  br label %{skip_block}"));
            self.emitln(&format!("\n{skip_block}:"));
        }
        Ok(())
    }
}
