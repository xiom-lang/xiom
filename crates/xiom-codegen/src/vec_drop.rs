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
use crate::context::{TypeContext, VecDrop};
use xiom_ast::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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
    /// Element-drop ownership tag (container local -> element XIOM type).
    /// A candidate starts empty (`Vec[T].new`/`with_capacity`); it stays a
    /// candidate only while every pushed element is provably owned and no
    /// removal/alias use can invalidate that.
    elem_candidate: HashMap<String, String>,
    /// Container local -> names of the element struct's Vec fields.
    elem_vec_fields: HashMap<String, Vec<String>>,
    /// Containers that lost element ownership (removal, escaping element
    /// reads, duplicate pushes, borrows, unknown methods, ...).
    elem_unsafe: HashSet<String>,
    /// Per-container set of aggregate identifiers already pushed (a second
    /// push of the same header would alias one buffer twice).
    elem_pushed: HashMap<String, HashSet<String>>,
    /// Block id in which each local was declared (a pushed aggregate must be
    /// declared in the same block as the push, so a loop body re-creates both).
    decl_block: HashMap<String, usize>,
    cur_block: usize,
    next_block: usize,
    /// By-value `Vec[T]` parameters of the current function. vec_abi deep-
    /// copies them at the call boundary, so a local bound from one owns its
    /// buffer and can be dropped (m375 / REL-20261030-2357-engine).
    by_value_vec_params: HashSet<String>,
    /// Aggregate local -> Str field names proven fresh at the binding site.
    /// Only these leaves are freed; borrowed/literal Str fields stay alive.
    aggregate_str_fields: HashMap<String, Vec<String>>,
    /// Aggregate locals whose owned Str field was reassigned after binding.
    /// Rebind codegen for fields is not implemented; disabling the exit drop
    /// is a leak, not a UAF (production-safe conservative choice).
    aggregate_str_unsafe: HashSet<String>,
    /// Precomputed type-name variant -> Vec field names (plain structs only).
    elem_type_vec_fields: Arc<HashMap<String, Vec<String>>>,
    /// Callee leaf names whose return type is a scalar/void: only these may
    /// treat a `&` argument as non-retaining. A callee returning a struct or
    /// a Vec can store the borrowed header in its result (huffman
    /// `_tree_from_lengths`), so the borrow MUST be treated as escaping.
    known_nonretaining: Arc<HashSet<String>>,
    /// Callee leaf names with at least one `&mut` parameter. Such a callee can
    /// store a by-value aggregate argument through the reference
    /// (`sim_send_a -> _enqueue(l, p) -> l.q_ab.push(p)`); passing the
    /// aggregate to it transfers ownership even when the callee returns void.
    retaining_callees: Arc<HashSet<String>>,
    /// Callee leaf names whose by-value `Vec` parameters are only READ
    /// (len/is_empty/capacity/count). A by-value Vec argument to such a
    /// callee is a borrowed header: the caller keeps ownership and frees the
    /// buffer at its own scope exit (m375 p_panel_move / p_vecrebind).
    byval_vec_readers: Arc<HashSet<String>>,
    /// Str locals with at least one fresh assignment/declaration.
    str_rebind: HashSet<String>,
    /// Str locals used in an escaping context OTHER than `return` (an alias
    /// may outlive the replaced buffer, so rebind reclamation is disabled).
    str_alias_escaped: HashSet<String>,
    /// True while scanning a `return` expression (transfers the CURRENT
    /// value; older replaced buffers are still safe to reclaim).
    in_return: bool,
    /// While scanning the RHS of a fresh Str assignment, the target name is
    /// read by the concat but does not alias the NEW buffer.
    rebind_target: Option<String>,
}

pub(crate) struct OwnedLocals {
    pub vec: HashSet<String>,
    pub str: HashSet<String>,
    pub aggregate: HashSet<String>,
    /// Aggregate local -> freshly-owned Str field names.
    pub aggregate_str: HashMap<String, Vec<String>>,
    /// Container local -> (element XIOM type, element Vec field names) for
    /// containers whose elements are all owned and never reordered/removed.
    pub elem: HashMap<String, (String, Vec<String>)>,
    /// Single-declaration Str locals eligible for replaced-buffer
    /// reclamation on fresh reassignment (REL-20261028).
    pub str_rebind: HashSet<String>,
    /// Rebind locals that are not returned: their final owned buffer is freed
    /// at scope exit (the replaced buffers are already freed at each rebind).
    pub str_rebind_exit: HashSet<String>,
}

/// Returns the local names that are owned and private for the function body.
pub(crate) fn collect_owned_vec_locals(
    block: &Block,
    fresh: &HashSet<String>,
    fresh_payload: &HashSet<String>,
    fresh_aggregate: &HashSet<String>,
    trust_unsafe: bool,
    elem_type_vec_fields: &Arc<HashMap<String, Vec<String>>>,
    known_nonretaining: &Arc<HashSet<String>>,
    retaining_callees: &Arc<HashSet<String>>,
    byval_vec_readers: &Arc<HashSet<String>>,
    by_value_vec_params: &HashSet<String>,
) -> OwnedLocals {
    let mut state = ScanState::default();
    state.by_value_vec_params = by_value_vec_params.clone();
    state.fresh = fresh.clone();
    state.fresh_payload = fresh_payload.clone();
    state.fresh_aggregate = fresh_aggregate.clone();
    state.trust_unsafe = trust_unsafe;
    state.elem_type_vec_fields = elem_type_vec_fields.clone();
    state.known_nonretaining = known_nonretaining.clone();
    state.retaining_callees = retaining_callees.clone();
    state.byval_vec_readers = byval_vec_readers.clone();
    scan_block(block, &mut state, true);
    // REL-20261028-1641 follow-up: the function body's trailing expression is
    // its IMPLICIT return value. `scan_block` sees it as a plain expression
    // (UseCtx::Escape), so `Ok(conn)` / `Some(conn)` never marked `conn`
    // returned and the aggregate exit drop freed the buffer the caller now
    // owns (`p_borrowed_mut_grown_vec_uaf`, p_wave162 row 12 UAF). Treat the
    // tail exactly like `return <expr>` for ownership transfer.
    match block.stmts.last() {
        Some(StmtOrExpr::Expr(expr)) => mark_returned_roots(expr, &mut state),
        Some(StmtOrExpr::Stmt(Stmt::Expr(expr, _))) => mark_returned_roots(expr, &mut state),
        _ => {}
    }
    // Element drops are per-DECLARATION: a shadowed name with a second,
    // different binding must not inherit the first one's tag.
    let single_decl: HashSet<String> = state
        .decls
        .iter()
        .filter(|(_, info)| info.decls == 1)
        .map(|(name, _)| name.clone())
        .collect();
    let vec: HashSet<String> = state
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
    let mut elem = HashMap::new();
    for (name, elem_ty) in state.elem_candidate {
        if !single_decl.contains(&name)
            || !vec.contains(&name)
            || state.elem_unsafe.contains(&name)
            || state.escaping.contains(&name)
            || state.moved.contains(&name)
        {
            continue;
        }
        // Vec-of-Vec: the inner Vec local was moved into the container. If it
        // is used/aliased later, the element drop could free a live alias.
        let is_vec_elem = state
            .elem_vec_fields
            .get(&name)
            .map_or(false, |f| f.len() == 1 && f[0] == "__vec");
        if is_vec_elem {
            if let Some(pushed) = state.elem_pushed.get(&name) {
                if pushed.iter().any(|p| state.escaping.contains(p)) {
                    continue;
                }
            }
        }
        if let Some(fields) = state.elem_vec_fields.get(&name) {
            elem.insert(name, (elem_ty, fields.clone()));
        }
    }
    let str_rebind: HashSet<String> = state
        .str_rebind
        .into_iter()
        .filter(|name| {
            let keep = single_decl.contains(name)
                && !state.str_alias_escaped.contains(name)
                && !state.moved.contains(name);
            keep
        })
        .collect();
    // A rebind local whose final value is NOT returned still owns the last
    // fresh buffer at scope exit. Free it there; returned locals transfer the
    // buffer to the caller (and must keep no exit drop).
    let str_rebind_exit: HashSet<String> = str_rebind
        .iter()
        .filter(|name| !state.returned.contains(*name))
        .cloned()
        .collect();
    OwnedLocals {
        vec,
        str,
        aggregate: state
            .aggregate
            .into_iter()
            .filter(|n| !state.returned.contains(n) && !state.moved.contains(n))
            .collect(),
        aggregate_str: state
            .aggregate_str_fields
            .into_iter()
            .filter(|(n, _)| {
                !state.returned.contains(n)
                    && !state.moved.contains(n)
                    && !state.aggregate_str_unsafe.contains(n)
            })
            .collect(),
        elem,
        str_rebind,
        str_rebind_exit,
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
                "int_to_string"
                    | "float_to_string"
                    | "bool_to_string"
                    | "str_concat"
                    | "str_repeat"
                    | "str_join"
                    | "str_replace"
            ),
            Expr::Ident(id) => matches!(
                id.name.as_str(),
                "int_to_string"
                    | "float_to_string"
                    | "bool_to_string"
                    | "str_concat"
                    | "str_repeat"
                    | "str_join"
                    | "str_replace"
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

/// REL-20261027-2034-engine: a Vec binding whose RHS is a FIELD or ELEMENT
/// projection is deep-copied by codegen (`vec_abi::vec_store_needs_owned_copy`
/// -> `emit_vec_value_deep_copy`), so the new local uniquely owns the copy.
/// `is_fresh_vec_init` is deliberately NOT widened: `fresh_return.rs` uses it
/// to prove that a FUNCTION returns a fresh buffer, and a projection return
/// aliases the source. This predicate is only for binding ownership.
///
/// A bare identifier (`var b = a`) is excluded on purpose: the m368 drop lock
/// keeps identifier copies as leak-only controls, and the 2034 defect is the
/// param-field projection (`var v = b.data`). Widening that later needs its
/// own evidence that every codegen path takes the owned copy.
pub(crate) fn is_owned_copy_vec_init(
    expr: &Expr,
    by_value_vec_params: &HashSet<String>,
) -> bool {
    match expr {
        Expr::Index(..) | Expr::Field(..) => true,
        // m375: vec_abi deep-copies a by-value Vec parameter at the call
        // boundary, so `var b = bytes` / `var uiv = f_uiv` owns the copy and
        // is a valid drop site. Only parameters of THIS function count.
        Expr::Ident(id) if by_value_vec_params.contains(&id.name) => true,
        Expr::Paren(inner, _) => is_owned_copy_vec_init(inner, by_value_vec_params),
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

/// Element-drop ownership: an EMPTY Vec constructor (`Vec[T].new()` /
/// `Vec[T].with_capacity(n)`). Only these can be proven fully populated by
/// the pushes the scan sees; `with_len` elements are uninitialized.
fn is_empty_vec_ctor(value: &Expr) -> bool {
    match value {
        Expr::Paren(inner, _) => is_empty_vec_ctor(inner),
        Expr::Call(func, args, _) => match func.as_ref() {
            Expr::Field(obj, method, _) => {
                let ctor = method.name == "new"
                    || (method.name == "with_capacity" && args.len() <= 1);
                ctor && matches!(
                    obj.as_ref(),
                    Expr::Index(base, _, _)
                        if matches!(base.as_ref(), Expr::Ident(b) if b.name == "Vec")
                )
            }
            _ => false,
        },
        _ => false,
    }
}

/// XIOM type name of a Vec local's element: the declaration annotation when
/// present, else the `Vec[T].new()/with_capacity()` constructor spelling.
fn vec_element_name(ty: Option<&Type>, value: &Expr) -> Option<String> {
    if let Some(t) = ty {
        if let Some(elem) = IrEmitter::vec_elem_from_type_annotation(t) {
            return Some(elem);
        }
    }
    match value {
        Expr::Paren(inner, _) => vec_element_name(None, inner),
        Expr::Call(func, _, _) => match func.as_ref() {
            Expr::Field(obj, method, _)
                if method.name == "new" || method.name == "with_capacity" =>
            {
                match obj.as_ref() {
                    Expr::Index(base, elem, _)
                        if matches!(base.as_ref(), Expr::Ident(b) if b.name == "Vec") =>
                    {
                        // Nested element types (`Vec[Vec[UInt8]]`) render to
                        // "Vec[UInt8]"; plain structs keep their name.
                        type_expr_name(elem)
                    }
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// Render a type expression (`Ident` / `Index`) to its source spelling.
fn type_expr_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(id) => Some(id.name.clone()),
        Expr::Paren(inner, _) => type_expr_name(inner),
        Expr::Index(base, elem, _) => Some(format!(
            "{}[{}]",
            type_expr_name(base)?,
            type_expr_name(elem)?
        )),
        _ => None,
    }
}

/// Vec field names of a plain element struct. Ambiguous same-leaf names are
/// rejected (conservative leak, never a wrong layout).
fn elem_vec_fields_for(elem_ty: &str, state: &ScanState) -> Option<Vec<String>> {
    if let Some(v) = state.elem_type_vec_fields.get(elem_ty) {
        return Some(v.clone());
    }
    let suffix = format!(".{elem_ty}");
    let mut found: Option<&Vec<String>> = None;
    for (key, fields) in state.elem_type_vec_fields.iter() {
        if key.ends_with(&suffix) {
            if found.is_some() {
                return None;
            }
            found = Some(fields);
        }
    }
    found.cloned()
}

/// A struct literal owns a Vec if any (possibly nested) field is a fresh Vec
/// initializer, an owned deep-copy projection, or a fresh Vec local that is
/// MOVED into the literal. Recursive so `Outer{ inner: Inner{ data: xs } }`
/// marks `xs` moved; otherwise both the literal aggregate and `xs` would free
/// the same buffer.
fn struct_literal_owns_vec(fields: &[(Ident, Expr)], state: &mut ScanState) -> bool {
    let mut owns = false;
    for (_, fv) in fields {
        match fv {
            Expr::Ident(id) => {
                // REL-20261029-1324: an aggregate local that already owns a
                // Vec field is MOVED into this literal exactly like a fresh Vec
                // local. The literal's field copy is shallow (header only), so
                // leaving the source aggregate in the drop set frees the buffer
                // the new aggregate still points at. This is the multi-level
                // `Store{..} -> Tree{ store: store } -> return Some(..)` shape
                // (0.2.1 carve `snap_load_v2`).
                let owned_aggregate = state.aggregate.contains(&id.name);
                let owned_vec = state
                    .decls
                    .get(&id.name)
                    .map_or(false, |i| i.decls > 0 && i.all_fresh);
                if (owned_aggregate || owned_vec)
                    && !state.escaping.contains(&id.name)
                    && !state.moved.contains(&id.name)
                {
                    state.moved.insert(id.name.clone());
                    owns = true;
                }
            }
            Expr::Field(base, _, _) => {
                // REL-2110 u4: a Vec field projected out of an aggregate local
                // (`done.payload = taken.payload`) is shallow-copied into the
                // literal. The source aggregate must not drop that field, or
                // the literal/result points at freed storage. Moving the whole
                // source is conservative (unused fields leak, never UAF).
                if let Expr::Ident(bid) = base.as_ref() {
                    if state.aggregate.contains(&bid.name)
                        && !state.escaping.contains(&bid.name)
                        && !state.moved.contains(&bid.name)
                    {
                        state.moved.insert(bid.name.clone());
                        owns = true;
                    }
                }
            }
            expr if is_fresh_vec_init(expr)
                || is_owned_copy_vec_init(expr, &state.by_value_vec_params) => {
                owns = true;
            }
            Expr::Struct(_, inner, spread, _) if spread.is_none() => {
                if struct_literal_owns_vec(inner, state) {
                    owns = true;
                }
            }
            _ => {}
        }
    }
    owns
}

/// Resolve a dotted literal path (`inner.data`) through nested struct
/// literals.
fn literal_path_value<'a>(fields: &'a [(Ident, Expr)], path: &str) -> Option<&'a Expr> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = fields;
    for (i, part) in parts.iter().enumerate() {
        let value = &cur.iter().find(|(name, _)| name.name == *part)?.1;
        if i + 1 == parts.len() {
            return Some(value);
        }
        match value {
            Expr::Struct(_, inner, None, _) => cur = inner.as_slice(),
            _ => return None,
        }
    }
    None
}

fn scan_block(block: &Block, state: &mut ScanState, tail_is_return: bool) {
    // Element-drop ownership: a pushed aggregate must be declared in the
    // SAME block as the push. Both are then re-created per loop iteration;
    // an outer declaration pushed inside a loop would push one aliasing
    // header many times.
    let id = state.next_block;
    state.next_block += 1;
    let saved_block = state.cur_block;
    state.cur_block = id;
    let last = block.stmts.len().saturating_sub(1);
    for (idx, item) in block.stmts.iter().enumerate() {
        match item {
            StmtOrExpr::Stmt(stmt) => scan_stmt(stmt, state),
            StmtOrExpr::Expr(expr) => {
                // A function body's TAIL expression is a return transfer.
                // Scanning it as an ordinary escape used to mark a returned
                // local `result` as alias-escaped, disabling Str rebind
                // reclamation inside stdlib builders (str_repeat O(log n)
                // concat loop leaked every intermediate buffer).
                if tail_is_return && idx == last {
                    mark_returned_roots(expr, state);
                    let saved = state.in_return;
                    state.in_return = true;
                    scan_expr(expr, UseCtx::Escape, state);
                    state.in_return = saved;
                } else {
                    scan_expr(expr, UseCtx::Escape, state);
                }
            }
        }
    }
    state.cur_block = saved_block;
}

fn scan_stmt(stmt: &Stmt, state: &mut ScanState) {
    match stmt {
        Stmt::Let(name, ty, value, _) | Stmt::Var(name, ty, value, _) => {
            state.decl_block.insert(name.name.clone(), state.cur_block);
            // REL-20261027-1717-engine: a local struct literal that owns a
            // fresh Vec field is an aggregate drop site. The source Vec local
            // is MOVED into the field (do not drop it separately); a fresh
            // Vec field expression is owned by the literal.
            if let Expr::Struct(_, fields, spread, _) = value {
                if spread.is_none() {
                    let owns_vec = struct_literal_owns_vec(fields, state);
                    // m375: a struct literal whose Str field is a FRESH Str
                    // owns that buffer. Track the exact field names so the
                    // aggregate drop frees only proven-fresh leaves; a
                    // literal/borrowed Str field in the same struct is left
                    // alone. Moving a fresh Str local into the field disarms
                    // its own plain-Str drop.
                    let mut owned_str_fields: Vec<String> = Vec::new();
                    for (fname, fval) in fields {
                        let fresh_field = is_fresh_str_init(fval)
                            || matches!(
                                fval,
                                Expr::Ident(id)
                                    if state
                                        .str_decls
                                        .get(&id.name)
                                        .map_or(false, |d| d.all_fresh)
                            );
                        if fresh_field {
                            owned_str_fields.push(fname.name.clone());
                            if let Expr::Ident(id) = fval {
                                if state
                                    .str_decls
                                    .get(&id.name)
                                    .map_or(false, |d| d.all_fresh)
                                {
                                    state.moved.insert(id.name.clone());
                                }
                            }
                        }
                    }
                    if owns_vec || !owned_str_fields.is_empty() {
                        state.aggregate.insert(name.name.clone());
                    }
                    if !owned_str_fields.is_empty() {
                        state
                            .aggregate_str_fields
                            .insert(name.name.clone(), owned_str_fields);
                    }
                }
            }
            // REL-20261027-2034-engine: an aliasing RHS (`b.data`, `v`, `xs[0]`)
            // is deep-copied at the binding, so the local owns its own buffer
            // and is a valid drop site. Without this the callee's copy of a
            // by-value aggregate param field leaked ~0.47 MB/iter (engine 2034).
            let fresh = is_fresh_vec_init(value)
                || is_fresh_vec_call(value, &state.fresh)
                || is_owned_copy_vec_init(value, &state.by_value_vec_params);
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
            // Element-drop ownership tag: only an EMPTY Vec built by
            // `new`/`with_capacity` can be proven fully populated by owned
            // pushes. `with_len` elements are uninitialized; array literals
            // and call returns have their own construction rules.
            if is_empty_vec_ctor(value) {
                if let Some(elem_ty) = vec_element_name(ty.as_deref(), value) {
                    // `Vec[Vec[T]]`: free each element's inner buffer. The
                    // sentinel field list keeps the tag shape uniform.
                    let fields = if elem_ty.starts_with("Vec[") {
                        Some(vec!["__vec".to_string()])
                    } else {
                        elem_vec_fields_for(&elem_ty, state)
                    };
                    if fields.as_ref().map_or(false, |f| !f.is_empty()) {
                        state.elem_candidate.insert(name.name.clone(), elem_ty);
                        state.elem_vec_fields.insert(name.name.clone(), fields.unwrap());
                    }
                }
            }
            scan_expr(value, UseCtx::Escape, state);
        }
        Stmt::Assign(place, value, _) => {
            // A reassigned owned Str FIELD (or whole aggregate) needs
            // field-rebind codegen to free the old buffer safely. Until that
            // exists, disable the exit drop: a leak is preferable to freeing
            // a pointer that may now be borrowed/literal.
            match place {
                Expr::Field(base, field, _) => {
                    if let Expr::Ident(id) = base.as_ref() {
                        if state
                            .aggregate_str_fields
                            .get(&id.name)
                            .map_or(false, |fs| fs.iter().any(|f| f == &field.name))
                        {
                            state.aggregate_str_unsafe.insert(id.name.clone());
                        }
                    }
                }
                Expr::Ident(id) if state.aggregate_str_fields.contains_key(&id.name) => {
                    state.aggregate_str_unsafe.insert(id.name.clone());
                }
                _ => {}
            }
            // REL-20261028: `s = s + x` / `s = int_to_string(..)` replaces an
            // owned Str buffer. The replaced buffer is dead once the fresh
            // RHS exists, so codegen may free it (guarded by a per-local
            // owned flag). This only applies when the name has one
            // declaration and is never alias-escaped (return is a transfer,
            // not an alias). Other rebinds keep the historical conservative
            // escape.
            let fresh_str_rebind = match place {
                Expr::Ident(id) => {
                    is_fresh_str_init(value)
                        && state
                            .decls
                            .get(&id.name)
                            .map_or(false, |info| info.decls == 1)
                }
                _ => false,
            };
            if let Expr::Ident(id) = place {
                if fresh_str_rebind {
                    state.str_rebind.insert(id.name.clone());
                } else {
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
                }
            } else {
                scan_expr(place, UseCtx::Escape, state);
            }
            let saved_target = state.rebind_target.clone();
            if fresh_str_rebind {
                if let Expr::Ident(id) = place {
                    state.rebind_target = Some(id.name.clone());
                }
            }
            scan_expr(value, UseCtx::Escape, state);
            state.rebind_target = saved_target;
        }
        Stmt::Return(Some(expr), _) => {
            // Returning an aggregate transfers ownership to the caller; its
            // Vec fields must not be freed by this function's exit drop.
            mark_returned_roots(expr, state);
            // Fresh-return analysis: returning the local transfers its owned
            // buffer to the caller; do not count that use as an escape.
            let saved = state.in_return;
            state.in_return = true;
            if !state
                .skip_return_local
                .as_deref()
                .map_or(false, |name| is_returned_local(expr, name))
            {
                scan_expr(expr, UseCtx::Escape, state);
            }
            state.in_return = saved;
        }
        Stmt::Return(None, _) | Stmt::Break(..) | Stmt::Continue(..) | Stmt::Debugger(_) => {}
        Stmt::Expr(expr, _) => scan_expr(expr, UseCtx::Escape, state),
        Stmt::If(cond, then_block, elifs, else_block, _) => {
            scan_expr(cond, UseCtx::Escape, state);
            scan_block(then_block, state, false);
            for (elif_cond, elif_block) in elifs {
                scan_expr(elif_cond, UseCtx::Escape, state);
                scan_block(elif_block, state, false);
            }
            if let Some(else_block) = else_block {
                scan_block(else_block, state, false);
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
                    MatchBody::Block(block) => scan_block(block, state, false),
                    MatchBody::Expr(expr) => scan_expr(expr, UseCtx::Escape, state),
                }
            }
        }
        Stmt::While(cond, body, invariant, _, _) => {
            scan_expr(cond, UseCtx::Escape, state);
            if let Some(inv) = invariant {
                scan_expr(inv, UseCtx::Escape, state);
            }
            scan_block(body, state, false);
        }
        Stmt::For(_, iter, body, _, _) => {
            scan_expr(iter, UseCtx::Escape, state);
            scan_block(body, state, false);
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
                // A return transfers the CURRENT value; older replaced
                // buffers are still safe to free. Any other escape (alias,
                // argument, store, capture) can outlive the old buffer.
                if !state.in_return
                    && state.rebind_target.as_deref() != Some(id.name.as_str())
                {
                    state.str_alias_escaped.insert(id.name.clone());
                }
            }
        }
        Expr::Paren(inner, _) => scan_expr(inner, ctx, state),
        Expr::Field(base, field, _) => {
            // Element field read: a Vec field can escape the element (and its
            // buffer), so only SCALAR fields of an element are non-escaping.
            if let Expr::Index(b, _, _) = base.as_ref() {
                if let Expr::Ident(bid) = b.as_ref() {
                    if let Some(vf) = state.elem_vec_fields.get(&bid.name) {
                        let direct = vf.iter().any(|f| f == &field.name);
                        // `inner` in a path `inner.data`: the struct field
                        // itself can be aliased/read, so only a header-only
                        // safe context is allowed.
                        let nested = vf
                            .iter()
                            .any(|p| p.starts_with(&format!("{}.", field.name)));
                        if direct || nested {
                            // Header-only use of an element's Vec field
                            // (`c5[0].a.len()`, `.push(..)`) is safe: the
                            // container drop re-reads the field at exit. A
                            // transfer/escape context is not.
                            let inner = if ctx == UseCtx::SafeBase {
                                UseCtx::SafeBase
                            } else {
                                UseCtx::Escape
                            };
                            scan_expr(base, inner, state);
                        } else {
                            scan_expr(base, UseCtx::SafeBase, state);
                        }
                        return;
                    }
                }
            }
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
            // Element-drop ownership: methods that remove/reorder/replace
            // elements invalidate "every element is still the owned one we
            // pushed"; a push/insert is accepted only with an owned element.
            if let Expr::Field(recv, method, _) = func.as_ref() {
                if let Expr::Ident(rid) = recv.as_ref() {
                    let rname = rid.name.clone();
                    if state.elem_candidate.contains_key(&rname) {
                        let m = method.name.as_str();
                        const ELEM_READ_ONLY: &[&str] = &[
                            "len", "is_empty", "capacity", "contains", "index_of", "count",
                            "get", "first", "last", "peek", "binary_search",
                        ];
                        if !ELEM_READ_ONLY.contains(&m) && m != "push" && m != "insert" {
                            state.elem_unsafe.insert(rname.clone());
                        } else if m == "push" || m == "insert" {
                            let elem_arg = if m == "push" { args.first() } else { args.last() };
                            let needed = state
                                .elem_vec_fields
                                .get(&rname)
                                .cloned()
                                .unwrap_or_default();
                            let owned = match elem_arg {
                                Some(Expr::Ident(id)) => {
                                    let is_str_elem = state
                                        .elem_candidate
                                        .get(&rname)
                                        .map_or(false, |t| t == "Str");
                                    let owned_local = if is_str_elem {
                                        // Vec[Str]: the pushed Str local must
                                        // itself be fresh/owned; ownership
                                        // transfers to the container.
                                        state
                                            .str_decls
                                            .get(&id.name)
                                            .map_or(false, |info| {
                                                info.decls > 0 && info.all_fresh
                                            })
                                            && !state.escaping.contains(&id.name)
                                            && !state.moved.contains(&id.name)
                                    } else {
                                        state.aggregate.contains(&id.name)
                                            || (state.decls.get(&id.name).map_or(false, |info| {
                                                info.decls > 0 && info.all_fresh
                                            }) && !state.escaping.contains(&id.name)
                                                && !state.moved.contains(&id.name))
                                    };
                                    let str_leaves_owned = needed.iter().all(|path| {
                                        match path.strip_prefix("str:") {
                                            // A pushed aggregate owns a Str leaf
                                            // only when that leaf was proven
                                            // fresh at its own binding site.
                                            Some(leaf) => state
                                                .aggregate_str_fields
                                                .get(&id.name)
                                                .map_or(false, |fs| {
                                                    fs.iter().any(|f| f == leaf)
                                                }),
                                            None => true,
                                        }
                                    });
                                    let ok = owned_local
                                        && str_leaves_owned
                                        && state.decl_block.get(&id.name)
                                            == Some(&state.cur_block)
                                        && state
                                            .elem_pushed
                                            .entry(rname.clone())
                                            .or_default()
                                            .insert(id.name.clone());
                                    if ok && is_str_elem {
                                        state.moved.insert(id.name.clone());
                                    }
                                    ok
                                }
                                Some(Expr::Struct(lit, fields, spread, _)) => {
                                    if spread.is_some() {
                                        false
                                    } else {
                                        let elem_ty = state
                                            .elem_candidate
                                            .get(&rname)
                                            .cloned()
                                            .unwrap_or_default();
                                        let same = lit.name == elem_ty
                                            || lit.name.ends_with(&format!(".{elem_ty}"))
                                            || elem_ty.ends_with(&format!(".{}", lit.name));
                                        same
                                            && needed.iter().all(|path| {
                                                match path.strip_prefix("str:") {
                                                    Some(leaf) => literal_path_value(fields, leaf)
                                                        .map_or(false, |fval| {
                                                            is_fresh_str_init(fval)
                                                                || matches!(
                                                                    fval,
                                                                    Expr::Ident(id)
                                                                        if state
                                                                            .str_decls
                                                                            .get(&id.name)
                                                                            .map_or(false, |d| d.all_fresh)
                                                                            && !state.escaping.contains(&id.name)
                                                                            && !state.moved.contains(&id.name)
                                                                            && state.decl_block.get(&id.name)
                                                                                == Some(&state.cur_block)
                                                                )
                                                        }),
                                                    None => literal_path_value(fields, path).map_or(
                                                        false,
                                                        |fval| {
                                                            is_fresh_vec_init(fval)
                                                                || is_owned_copy_vec_init(fval, &state.by_value_vec_params)
                                                        },
                                                    ),
                                                }
                                            })
                                    }
                                }
                                // `Vec[Vec[T]]`: an inline fresh inner Vec
                                // (`rs.push(Vec[UInt8].with_len(n))`).
                                Some(inner) if is_fresh_vec_init(inner) => true,
                                // `Vec[Str]`: an inline fresh Str expression.
                                Some(inner)
                                    if state
                                        .elem_candidate
                                        .get(&rname)
                                        .map_or(false, |t| t == "Str")
                                        && is_fresh_str_init(inner) =>
                                {
                                    true
                                }
                                _ => false,
                            };
                            if !owned {
                                state.elem_unsafe.insert(rname.clone());
                            } else if let Some(Expr::Struct(_, fields, spread, _)) = elem_arg {
                                // A fresh Str LOCAL moved into the element now
                                // belongs to the container. Mark it moved so no
                                // exit drop frees it a second time; the element
                                // drop owns it.
                                if spread.is_none() {
                                    for path in &needed {
                                        if let Some(leaf) = path.strip_prefix("str:") {
                                            if let Some(Expr::Ident(id)) =
                                                literal_path_value(fields, leaf)
                                            {
                                                state.moved.insert(id.name.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // A method call ON an element (`rs[i].take()`) may move the
                // element's Vec fields out; conservative. For a Vec-of-Vec
                // element, header-only reads (`outer[0].len()`) are safe.
                if let Expr::Index(b, _, _) = recv.as_ref() {
                    if let Expr::Ident(bid) = b.as_ref() {
                        if state.elem_candidate.contains_key(&bid.name) {
                            let vec_elem = state
                                .elem_vec_fields
                                .get(&bid.name)
                                .map_or(false, |f| f.len() == 1 && f[0] == "__vec");
                            let header_only = matches!(
                                method.name.as_str(),
                                "len" | "is_empty" | "capacity"
                            );
                            if !(vec_elem && header_only) {
                                state.elem_unsafe.insert(bid.name.clone());
                            }
                        }
                    }
                }
            }
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
            // REL-20261029-2110-packages: passing an owned aggregate BY VALUE
            // to a callee that has a `&mut` parameter. The callee can store the
            // shallow-copied Vec headers through that reference
            // (`sim_send_a -> _enqueue(l, p) -> l.q_ab.push(p)`), so ownership
            // moves and the caller's exit drop must be disarmed. Callees with
            // no `&mut` parameter (consume_val/consume_ref-style readers) keep
            // the caller drop, which the memory gates require.
            let callee_retains = match func.as_ref() {
                Expr::Ident(id) => state.retaining_callees.contains(&id.name),
                Expr::Field(_, method, _) => state.retaining_callees.contains(&method.name),
                _ => false,
            };
            // Known Str-producing allocators only READ their Str arguments and
            // return a NEW buffer. Reading `result` in
            // `result = str_concat(result, chunk)` must not disable rebind
            // reclamation; otherwise every intermediate buffer in stdlib
            // builders (str_repeat/join/replace) leaks.
            let safe_str_callee = match func.as_ref() {
                Expr::Ident(id) => matches!(
                    id.name.as_str(),
                    "str_concat" | "str_repeat" | "str_join" | "str_replace"
                ),
                Expr::Field(_, method, _) => matches!(
                    method.name.as_str(),
                    "str_concat" | "str_repeat" | "str_join" | "str_replace"
                ),
                _ => false,
            };
            for arg in args {
                if safe_str_callee {
                    scan_expr(arg, UseCtx::SafeBase, state);
                    continue;
                }
                if callee_retains {
                    if let Expr::Ident(id) = arg {
                        if state.aggregate.contains(&id.name)
                            && !state.moved.contains(&id.name)
                        {
                            state.moved.insert(id.name.clone());
                            continue;
                        }
                    }
                }
                // A borrow of a container lets the callee read/mutate its
                // elements without this scan seeing it; element ownership
                // cannot be proven.
                if let Expr::Ref(inner, _) | Expr::MutRef(inner, _) = arg {
                    if let Expr::Ident(bid) = inner.as_ref() {
                        if state.elem_candidate.contains_key(&bid.name) {
                            state.elem_unsafe.insert(bid.name.clone());
                        }
                    }
                }
                if container_store {
                    if let Expr::Ident(id) = arg {
                        if state.aggregate.contains(&id.name) {
                            state.moved.insert(id.name.clone());
                            continue;
                        }
                        // `Vec[Vec[T]]`: pushing a fresh inner Vec transfers
                        // its buffer to the container (the element drop owns
                        // it). A later use/alias of the local marks escaping
                        // and disqualifies the element tag at finalize.
                        let fresh_vec_local = state
                            .decls
                            .get(&id.name)
                            .map_or(false, |info| info.decls > 0 && info.all_fresh)
                            && !state.escaping.contains(&id.name)
                            && !state.moved.contains(&id.name);
                        if fresh_vec_local {
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
                let borrow_safe = (!state.in_unsafe
                    || (state.trust_unsafe && state.in_trusted_unsafe))
                    && match func.as_ref() {
                        Expr::Ident(id) => {
                            state.known_nonretaining.contains(&id.name)
                        }
                        Expr::Field(_, method, _) => {
                            state.known_nonretaining.contains(&method.name)
                        }
                        _ => false,
                    };
                if borrow_safe {
                    if let Expr::Ref(inner, _) | Expr::MutRef(inner, _) = arg {
                        scan_borrow_arg(inner, state);
                        continue;
                    }
                }
                // m375 / p_panel_move: a by-value Vec local passed to a callee
                // proven to only READ its by-value Vec params keeps the
                // caller's ownership. The param ABI is a shallow header (the
                // callee rebinds it by deep copy when it needs a private one),
                // and the reader never reallocs/retains it, so the caller must
                // still free the buffer. Without this the caller's drop was
                // disarmed by the argument escape and the callee never freed a
                // borrowed param -> 190 KB/iter (p_panel_move).
                let byval_reader = !container_store
                    && !callee_retains
                    && (!state.in_unsafe
                        || (state.trust_unsafe && state.in_trusted_unsafe))
                    && match func.as_ref() {
                        Expr::Ident(id) => state
                            .byval_vec_readers
                            .contains(id.name.rsplit('.').next().unwrap_or(&id.name)),
                        Expr::Field(_, method, _) => {
                            state.byval_vec_readers.contains(&method.name)
                        }
                        _ => false,
                    };
                if byval_reader {
                    if let Expr::Ident(id) = arg {
                        // Only an owned Vec binding (fresh ctor/deep copy) or a
                        // fresh aggregate is safe to keep; the element-tagged
                        // container is excluded because the callee could mutate
                        // elements behind the tag's back.
                        let owned_vec = state
                            .decls
                            .get(&id.name)
                            .map_or(false, |info| info.all_fresh);
                        let owned_aggregate = state.aggregate.contains(&id.name);
                        if (owned_vec || owned_aggregate)
                            && !state.moved.contains(&id.name)
                            && !state.elem_candidate.contains_key(&id.name)
                        {
                            continue;
                        }
                    }
                }
                scan_expr(arg, UseCtx::Escape, state);
            }
        }
        Expr::Index(base, index, _) => {
            // An element projection used in an ESCAPE context (call arg,
            // return, assignment, another literal) can alias the element's
            // Vec payload; element drops need it to stay local.
            if ctx == UseCtx::Escape {
                if let Expr::Ident(bid) = base.as_ref() {
                    if state.elem_candidate.contains_key(&bid.name) {
                        state.elem_unsafe.insert(bid.name.clone());
                    }
                }
            }
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
        Expr::BlockExpr(block, _) => scan_block(block, state, false),
        Expr::Closure(_, _, body, _) => mark_block_mentions(body, state),
        Expr::PipeClosure(_, body, _) => scan_expr(body, UseCtx::Escape, state),
        Expr::If(cond, then_block, elifs, else_block, _) => {
            scan_expr(cond, UseCtx::Escape, state);
            scan_block(then_block, state, false);
            for (elif_cond, elif_block) in elifs {
                scan_expr(elif_cond, UseCtx::Escape, state);
                scan_block(elif_block, state, false);
            }
            if let Some(else_block) = else_block {
                scan_block(else_block, state, false);
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            scan_expr(scrutinee, UseCtx::Escape, state);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    scan_expr(guard, UseCtx::Escape, state);
                }
                match &arm.body {
                    MatchBody::Block(block) => scan_block(block, state, false),
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
                scan_block(body, state, false);
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
            state.str_alias_escaped.insert(id.name.clone());
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
    safe_aggregate_str: HashMap<String, Vec<String>>,
    safe_elem: HashMap<String, (String, Vec<String>)>,
    safe_str_rebind: HashSet<String>,
    safe_str_rebind_exit: HashSet<String>,
    str_rebind_state: HashMap<String, (String, String)>,
    by_value_vec_params: HashSet<String>,
    entry_prelude: Vec<String>,
}

impl IrEmitter {
    /// Build the owned-and-private local set for the function about to be
    /// compiled. Called at function-body start (regular and mono paths).
    pub(crate) fn prepass_owned_vec_locals(
        &mut self,
        body: &Block,
        trust_unsafe: bool,
        by_value_vec_params: &HashSet<String>,
    ) {
        self.fctx.by_value_vec_params = by_value_vec_params.clone();
        let fresh = self.fresh_vec_returns.clone();
        let fresh_payload = self.fresh_payload_returns.clone();
        let fresh_aggregate = self.fresh_aggregate_returns.clone();
        // Program-global sets, built once in build_drop_caches(); sharing them
        // by Arc keeps the per-function prepass O(body), not O(catalog). The
        // engine injects ~12k decls; rebuilding/cloning the catalog per
        // function made a full editor compile take hours.
        let elem_type_vec_fields = self.elem_type_vec_fields.clone();
        let known_nonretaining = self.known_nonretaining.clone();
        let retaining_callees = self.retaining_callees.clone();
        let byval_vec_readers = self.byval_vec_readers.clone();
        let owned = collect_owned_vec_locals(
            body,
            &fresh,
            &fresh_payload,
            &fresh_aggregate,
            trust_unsafe,
            &elem_type_vec_fields,
            &known_nonretaining,
            &retaining_callees,
            &byval_vec_readers,
            by_value_vec_params,
        );
        self.fctx.vec_drop_safe = owned.vec;
        self.fctx.str_drop_safe = owned.str;
        self.fctx.aggregate_drop_safe = owned.aggregate;
        self.fctx.aggregate_str_drop_safe = owned.aggregate_str;
        self.fctx.elem_drop_safe = owned.elem;
        self.fctx.str_rebind_safe = owned.str_rebind;
        self.fctx.str_rebind_exit = owned.str_rebind_exit;
    }

    /// A return type that cannot carry a borrowed Vec header: scalar/void.
    fn is_nonretaining_return(ret: &str) -> bool {
        let r = ret.trim();
        matches!(r, "void" | "float" | "double" | "half" | "fp128")
            || (r.starts_with('i')
                && r.len() > 1
                && r[1..].chars().all(|c| c.is_ascii_digit()))
    }

    /// Names of the Vec fields of every plain struct type, keyed by the full
    /// registry key AND the bare leaf when unambiguous. Element drops are only
    /// enabled for such structs.
    fn elem_type_vec_fields(
        types: &TypeContext,
    ) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let mut ambiguous: HashSet<String> = HashSet::new();
        fn resolve_alias(types: &TypeContext, ty: &str) -> String {
            let mut cur = ty.to_string();
            let mut seen: HashSet<String> = HashSet::new();
            while let Some(next) = types.type_aliases.get(&cur) {
                if !seen.insert(cur.clone()) {
                    break;
                }
                cur = next;
            }
            cur
        }
        fn is_vec_ty(types: &TypeContext, ty: &str) -> bool {
            let cur = resolve_alias(types, ty);
            cur == "Vec" || cur.starts_with("Vec[")
        }
        fn is_str_ty(types: &TypeContext, ty: &str) -> bool {
            resolve_alias(types, ty) == "Str"
        }
        fn is_skippable(k: &str) -> bool {
            k.starts_with("Tuple__")
                || k.starts_with("Option__")
                || k.starts_with("Result__")
                || k.starts_with("_Anon__")
                || k.starts_with("Vec[")
                || k.starts_with("Map[")
                || k.starts_with("Set[")
                || k.starts_with("Option[")
                || k.starts_with("Result[")
        }
        fn struct_key(types: &TypeContext, ty: &str) -> Option<String> {
            let resolved = resolve_alias(types, ty);
            if resolved == "Vec" || resolved.starts_with("Vec[") || is_skippable(&resolved) {
                return None;
            }
            if types.type_meta.contains_key(&resolved) {
                return Some(resolved);
            }
            let suffix = format!(".{resolved}");
            let mut found: Option<String> = None;
            for k in types.type_meta.keys() {
                if k.ends_with(&suffix) {
                    if found.is_some() {
                        return None;
                    }
                    found = Some(k);
                }
            }
            found
        }
        // Dotted paths to every owned Vec inside a struct, including nested
        // structs (`inner.data`). Depth-limited and cycle-safe.
        fn walk(
            types: &TypeContext,
            key: &str,
            prefix: &str,
            depth: usize,
            seen: &mut HashSet<String>,
            out: &mut Vec<String>,
        ) {
            if depth > 8 || !seen.insert(key.to_string()) {
                return;
            }
            if let Some(meta) = types.type_meta.get(&key.to_string()) {
                for (fname, fty) in meta.fields.iter() {
                    let path = if prefix.is_empty() {
                        fname.clone()
                    } else {
                        format!("{prefix}.{fname}")
                    };
                    if is_vec_ty(types, fty) {
                        out.push(path);
                    } else if is_str_ty(types, fty) {
                        // Typed Str leaf of an element: the `str:` prefix tells
                        // the push check to require a fresh/owned Str value and
                        // elem_drop_info to emit a pointer free instead of a
                        // Vec data free. `Vec[UiDrawText]` / `Vec[T{text:Str}]`
                        // leaked every per-element label before this.
                        out.push(format!("str:{path}"));
                    } else if let Some(child) = struct_key(types, fty) {
                        walk(types, &child, &path, depth + 1, seen, out);
                    }
                }
            }
            seen.remove(key);
        }
        for (key, _meta) in types.type_meta.entries() {
            if is_skippable(&key) {
                continue;
            }
            let mut fields: Vec<String> = Vec::new();
            let mut seen: HashSet<String> = HashSet::new();
            walk(types, &key, "", 0, &mut seen, &mut fields);
            fields.sort();
            fields.dedup();
            if fields.is_empty() {
                continue;
            }
            map.insert(key.clone(), fields.clone());
            if let Some(leaf) = key.rsplit('.').next() {
                if leaf != key {
                    if map.contains_key(leaf) {
                        ambiguous.insert(leaf.to_string());
                    } else {
                        map.insert(leaf.to_string(), fields);
                    }
                }
            }
        }
        for (alias, target) in types.type_aliases.entries() {
            if map.contains_key(&alias) {
                continue;
            }
            if let Some(fields) = map.get(&target).cloned() {
                map.insert(alias, fields);
            }
        }
        for name in ambiguous {
            map.remove(&name);
        }
        // Vec[Str]: the element itself is an owned i8*; the `__str` marker
        // tells elem_drop_info to free each element pointer.
        map.insert("Str".to_string(), vec!["__str".to_string()]);
        map
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
        self.retaining_callees = Arc::new(crate::fresh_return::collect_retaining_callees(program));
        self.byval_vec_readers = Arc::new(crate::fresh_return::collect_byval_vec_readers(program));
    }

    /// Build the program-global drop-analysis caches ONCE, after the function
    /// catalog is complete (preassign). The per-function prepass then shares
    /// them by Arc: on the engine program (~12k injected decls) rebuilding and
    /// cloning these per function was O(functions^2) and took hours.
    pub(crate) fn build_drop_caches(&mut self) {
        let mut known: HashSet<String> = HashSet::new();
        let mut may_retain: HashSet<String> = HashSet::new();
        for (name, (_, ret)) in self.types.functions.entries() {
            let leaf = name.rsplit('.').next().unwrap_or(&name).to_string();
            known.insert(leaf.clone());
            if !Self::is_nonretaining_return(&ret) {
                may_retain.insert(leaf);
            }
        }
        self.known_nonretaining =
            Arc::new(known.difference(&may_retain).cloned().collect());
        self.elem_type_vec_fields = Arc::new(Self::elem_type_vec_fields(&self.types));
    }

    /// Reset per-function drop state. Called at every function-body start.
    pub(crate) fn clear_vec_drops(&mut self) {
        self.fctx.vec_drops.clear();
        self.fctx.vec_drop_safe.clear();
        self.fctx.str_drop_safe.clear();
        self.fctx.aggregate_drop_safe.clear();
        self.fctx.aggregate_str_drop_safe.clear();
        self.fctx.elem_drop_safe.clear();
        self.fctx.str_rebind_safe.clear();
        self.fctx.str_rebind_exit.clear();
        self.fctx.str_rebind_state.clear();
        self.fctx.by_value_vec_params.clear();
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
            safe_aggregate_str: std::mem::take(&mut self.fctx.aggregate_str_drop_safe),
            safe_elem: std::mem::take(&mut self.fctx.elem_drop_safe),
            safe_str_rebind: std::mem::take(&mut self.fctx.str_rebind_safe),
            safe_str_rebind_exit: std::mem::take(&mut self.fctx.str_rebind_exit),
            str_rebind_state: std::mem::take(&mut self.fctx.str_rebind_state),
            by_value_vec_params: std::mem::take(&mut self.fctx.by_value_vec_params),
            entry_prelude: std::mem::take(&mut self.local.entry_prelude),
        }
    }

    pub(crate) fn resume_vec_drops(&mut self, saved: VecDropScope) {
        self.fctx.vec_drops = saved.drops;
        self.fctx.vec_drop_safe = saved.safe;
        self.fctx.str_drop_safe = saved.safe_str;
        self.fctx.aggregate_drop_safe = saved.safe_aggregate;
        self.fctx.aggregate_str_drop_safe = saved.safe_aggregate_str;
        self.fctx.elem_drop_safe = saved.safe_elem;
        self.fctx.str_rebind_safe = saved.safe_str_rebind;
        self.fctx.str_rebind_exit = saved.safe_str_rebind_exit;
        self.fctx.str_rebind_state = saved.str_rebind_state;
        self.fctx.by_value_vec_params = saved.by_value_vec_params;
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
                || is_fresh_vec_call(init, &self.fresh_vec_returns)
                // codegen deep-copied an aliasing RHS into this slot, so the
                // local owns the copy (REL-20261027-2034-engine).
                || is_owned_copy_vec_init(init, &self.fctx.by_value_vec_params))
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
        let elem = self.elem_drop_info(name);
        self.fctx.vec_drops.push(VecDrop::Vec {
            alloca: alloca.to_string(),
            flag: flag.clone(),
            elem,
        });
        self.emitln(&format!("  store i1 true, i1* {flag}"));
    }

    /// Register a fresh, non-escaping Str local for reclamation. The pointer
    /// is captured HERE into a hoisted i8* slot so the scope-exit drop never
    /// loads an alloca from a non-dominating if-arm (engine sfr_unique).
    pub(crate)     fn register_str_drop(&mut self, name: &str, alloca: &str, llvm_ty: &str, init: &Expr) {
        // Reassigned Str locals free each replaced buffer at the rebind. If
        // the final value is NOT returned, it is still owned at scope exit:
        // free it through the same guarded slot (m375 str_repeat's `chunk`
        // accumulator leaked one buffer per call). A returned local transfers
        // the final buffer to the caller and must keep no exit drop.
        if self.fctx.str_rebind_safe.contains(name) {
            if self.fctx.str_rebind_exit.contains(name) {
                let (slot, flag) = self.str_rebind_state(name);
                self.fctx.vec_drops.push(VecDrop::Ptr {
                    data_alloca: slot,
                    flag,
                });
            }
            return;
        }
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

    /// Rebind bookkeeping for a Str local: (owned-pointer slot, owned flag).
    /// Created lazily (and hoisted) so loop bodies reuse one flag.
    fn str_rebind_state(&mut self, name: &str) -> (String, String) {
        if let Some(state) = self.fctx.str_rebind_state.get(name) {
            return state.clone();
        }
        let slot = self.fresh_tmp();
        self.local
            .hoisted_allocas
            .push((slot.clone(), "i8*".to_string()));
        self.local
            .entry_prelude
            .push(format!("  store i8* null, i8** {slot}"));
        let flag = self.fresh_tmp();
        self.local
            .hoisted_allocas
            .push((flag.clone(), "i1".to_string()));
        self.local
            .entry_prelude
            .push(format!("  store i1 false, i1* {flag}"));
        self.fctx
            .str_rebind_state
            .insert(name.to_string(), (slot.clone(), flag.clone()));
        (slot, flag)
    }

    fn emit_free_owned_str(&mut self, slot: &str, flag: &str) {
        let free_block = self.fresh_block("str_rebind_free");
        let after_block = self.fresh_block("str_rebind_after");
        let owned = self.fresh_tmp();
        self.emitln(&format!("  {owned} = load i1, i1* {flag}"));
        self.emitln(&format!(
            "  br i1 {owned}, label %{free_block}, label %{after_block}"
        ));
        self.emitln(&format!("\n{free_block}:"));
        let ptr = self.fresh_tmp();
        self.emitln(&format!("  {ptr} = load i8*, i8** {slot}"));
        self.emitln(&format!("  call void @xiom_guard_free(i8* {ptr})"));
        self.emitln(&format!("  br label %{after_block}"));
        self.emitln(&format!("\n{after_block}:"));
    }

    /// At a fresh Str binding, start tracking its owned buffer so the first
    /// reassignment can free it. Literal/alias initializers start untracked
    /// (the flag stays false) and arm on the first fresh assignment.
    pub(crate) fn note_str_rebind_binding(&mut self, name: &str, init: &Expr, alloca: &str) {
        if !self.fctx.str_rebind_safe.contains(name) || !is_fresh_str_init(init) {
            return;
        }
        let (slot, flag) = self.str_rebind_state(name);
        let ptr = self.fresh_tmp();
        self.emitln(&format!("  {ptr} = load i8*, i8** {alloca}"));
        self.emitln(&format!("  store i8* {ptr}, i8** {slot}"));
        self.emitln(&format!("  store i1 true, i1* {flag}"));
    }

    /// On `s = <value>`: free the replaced owned buffer, then track the new
    /// value. A concat (`s = s + x`) allocates a NEW buffer, so freeing the
    /// old one is safe even though the RHS reads `s`. A non-fresh alias that
    /// mentions the target (self-assign) is left leak-only.
    pub(crate) fn update_str_rebind_assign(
        &mut self,
        name: &str,
        value: &Expr,
        store_val: &str,
        val_ty: &str,
    ) {
        if !self.fctx.str_rebind_safe.contains(name) || val_ty != "i8*" {
            return;
        }
        let fresh = is_fresh_str_init(value)
            || matches!(value, Expr::Binary(_, BinOp::Add, _, _));
        let mentions = Self::expr_mentions_any_ident(value, &[name]);
        // Always materialize the state BEFORE deciding to free: the loop body
        // is emitted once, so a first-iteration-only check would omit the
        // free for every later iteration. The flag makes iteration 1 a no-op.
        let (slot, flag) = self.str_rebind_state(name);
        if fresh || !mentions {
            self.emit_free_owned_str(&slot, &flag);
        }
        if fresh {
            self.emitln(&format!("  store i8* {store_val}, i8** {slot}"));
            self.emitln(&format!("  store i1 true, i1* {flag}"));
        } else {
            self.emitln(&format!("  store i8* {store_val}, i8** {slot}"));
            self.emitln(&format!("  store i1 false, i1* {flag}"));
        }
    }

    /// Element drop glue for a container whose prepass ownership tag proved
    /// every element owns its Vec fields. `None` means "leak elements".
    fn elem_drop_info(&self, name: &str) -> Option<crate::context::ElemDrop> {
        let (xiom_ty, validated) = self.fctx.elem_drop_safe.get(name)?.clone();
        // Vec[Str]: each element is an owned i8* buffer.
        if xiom_ty == "Str" {
            return Some(crate::context::ElemDrop {
                llvm_ty: "i8*".to_string(),
                stride: 8,
                elem_is_vec: false,
                elem_is_str: true,
                paths: Vec::new(),
            });
        }
        // Vec-of-Vec: each element is a %struct.Vec header {data,len,cap,
        // elem_size}; free its data pointer. The header is 4 x 8 bytes, and
        // field 0 is `data`. No struct metadata is involved.
        if xiom_ty.starts_with("Vec[") {
            return Some(crate::context::ElemDrop {
                llvm_ty: "%struct.Vec".to_string(),
                stride: Self::vec_slot_size(32),
                elem_is_vec: true,
                elem_is_str: false,
                paths: Vec::new(),
            });
        }
        let llvm_ty = self.llvm_type_for(&xiom_ty).ok()?;
        if !llvm_ty.starts_with("%struct.") {
            return None;
        }
        let key = llvm_ty.trim_start_matches("%struct.").to_string();
        let mut paths: Vec<Vec<(u32, String)>> = Vec::new();
        for path in &validated {
            // `str:` marks a typed Str leaf of the element struct (not a Vec
            // field). The final GEP target is an i8* slot.
            let is_str_leaf = path.starts_with("str:");
            let path = path.strip_prefix("str:").unwrap_or(path.as_str());
            let mut cur_key = key.clone();
            let mut steps: Vec<(u32, String)> = Vec::new();
            let mut ok = true;
            let parts: Vec<&str> = path.split('.').collect();
            for (i, part) in parts.iter().enumerate() {
                let Some(meta) = self.types.type_meta.get(&cur_key) else {
                    ok = false;
                    break;
                };
                let Some(idx) = meta.fields.iter().position(|(n, _)| n == part) else {
                    ok = false;
                    break;
                };
                let fty = self.field_llvm_type(&cur_key, idx);
                steps.push((idx as u32, fty.clone()));
                if i + 1 < parts.len() {
                    match self.resolve_struct_key(&meta.fields[idx].1) {
                        Some(next) => cur_key = next,
                        None => {
                            ok = false;
                            break;
                        }
                    }
                } else if (is_str_leaf && fty != "i8*")
                    || (!is_str_leaf && fty != "%struct.Vec")
                {
                    ok = false;
                }
            }
            if ok && !steps.is_empty() {
                paths.push(steps);
            }
        }
        if paths.is_empty() {
            return None;
        }
        // Intersect with the field paths the prepass validated: never free a
        // Vec field the ownership analysis did not see.
        let size = self.vec_elem_storage_size(&xiom_ty);
        if size <= 0 {
            return None;
        }
        Some(crate::context::ElemDrop {
            llvm_ty,
            stride: Self::vec_slot_size(size),
            elem_is_vec: false,
            elem_is_str: false,
            paths,
        })
    }

    /// Resolve a field's XIOM type to a `type_meta` key (alias + leaf aware).
    fn resolve_struct_key(&self, ty: &str) -> Option<String> {
        let mut cur = ty.to_string();
        let mut seen: HashSet<String> = HashSet::new();
        while let Some(next) = self.types.type_aliases.get(&cur) {
            if !seen.insert(cur.clone()) {
                break;
            }
            cur = next;
        }
        if self.types.type_meta.contains_key(&cur) {
            return Some(cur);
        }
        let suffix = format!(".{cur}");
        let mut found: Option<String> = None;
        for k in self.types.type_meta.keys() {
            if k.ends_with(&suffix) {
                if found.is_some() {
                    return None;
                }
                found = Some(k);
            }
        }
        found
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
            // REL-20261030-1540-bindings (epos): the field GEP must be
            // computed HERE, at registration, where the struct alloca
            // dominates; a GEP emitted at the drop block can be in a
            // non-dominating arm (alloca EposLibrary / field 5). The drop
            // still LOADS the data pointer at drop time, so realloc/growth
            // (p_wave162) is seen.
            // Entry-block GEPs: the binding's alloca is hoisted when it is an
            // aggregate-drop candidate, so these dominate every scope exit.
            let field_ptr = self.fresh_tmp();
            self.local.entry_prelude.push(format!(
                "  {field_ptr} = getelementptr {llvm_ty}, {llvm_ty}* {alloca}, i32 0, i32 {idx}"
            ));
            let data_gep = self.fresh_tmp();
            self.local.entry_prelude.push(format!(
                "  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {field_ptr}, i32 0, i32 0"
            ));
            let flag = self.fresh_tmp();
            self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
            self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
            self.fctx.vec_drops.push(VecDrop::Aggregate {
                data_gep,
                flag: flag.clone(),
            });
            self.emitln(&format!("  store i1 true, i1* {flag}"));
        }
        // m375: free only the Str fields the prepass proved FRESH at the
        // binding site. Borrowed/literal Str fields are not in this list, so
        // a struct can mix owned and borrowed text safely.
        if let Some(str_fields) = self.fctx.aggregate_str_drop_safe.get(name).cloned() {
            for (idx, field) in meta.fields.iter().enumerate() {
                if !str_fields.contains(&field.0) {
                    continue;
                }
                if self.field_llvm_type(&type_name, idx) != "i8*" {
                    continue;
                }
                let field_ptr = self.fresh_tmp();
                self.local.entry_prelude.push(format!(
                    "  {field_ptr} = getelementptr {llvm_ty}, {llvm_ty}* {alloca}, i32 0, i32 {idx}"
                ));
                // Defer the load/free to scope exit: freeing at the binding
                // would UAF a field the caller reads after construction (e.g.
                // UiDrawText.text is drawn after the literal). The slot is
                // re-read at drop time so a later fresh rebind is seen.
                let flag = self.fresh_tmp();
                self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
                self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
                self.fctx.vec_drops.push(VecDrop::Str {
                    alloca: field_ptr,
                    flag: flag.clone(),
                });
                self.emitln(&format!("  store i1 true, i1* {flag}"));
            }
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
        let elem = self.elem_drop_info(name);
        self.fctx.vec_drops.push(VecDrop::Vec {
            alloca: alloca.to_string(),
            flag: flag.clone(),
            elem,
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
                VecDrop::Vec { alloca, elem, .. } => {
                    let data_gep = self.fresh_tmp();
                    let data_ptr = self.fresh_tmp();
                    self.emitln(&format!(
                        "  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {alloca}, i32 0, i32 0"
                    ));
                    self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                    // Element-drop ownership tag: free each element's Vec
                    // payloads BEFORE the element buffer. The header is
                    // re-read here, so pushes/reallocs after the binding are
                    // reflected. Owned elements only (prepass proven).
                    if let Some(e) = elem {
                        let len_gep = self.fresh_tmp();
                        let len = self.fresh_tmp();
                        self.emitln(&format!(
                            "  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {alloca}, i32 0, i32 1"
                        ));
                        self.emitln(&format!("  {len} = load i64, i64* {len_gep}"));
                        let idx_slot = self.fresh_tmp();
                        self.emitln(&format!("  {idx_slot} = alloca i64"));
                        self.emitln(&format!("  store i64 0, i64* {idx_slot}"));
                        let cond = self.fresh_block("vecdrop_elt_cond");
                        let body = self.fresh_block("vecdrop_elt_body");
                        let done = self.fresh_block("vecdrop_elt_done");
                        self.emitln(&format!("  br label %{cond}"));
                        self.emitln(&format!("\n{cond}:"));
                        let i = self.fresh_tmp();
                        let more = self.fresh_tmp();
                        self.emitln(&format!("  {i} = load i64, i64* {idx_slot}"));
                        self.emitln(&format!("  {more} = icmp ult i64 {i}, {len}"));
                        self.emitln(&format!(
                            "  br i1 {more}, label %{body}, label %{done}"
                        ));
                        self.emitln(&format!("\n{body}:"));
                        let off = self.fresh_tmp();
                        let ep = self.fresh_tmp();
                        let epc = self.fresh_tmp();
                        self.emitln(&format!(
                            "  {off} = mul i64 {i}, {}",
                            e.stride
                        ));
                        self.emitln(&format!(
                            "  {ep} = getelementptr i8, i8* {data_ptr}, i64 {off}"
                        ));
                        self.emitln(&format!(
                            "  {epc} = bitcast i8* {ep} to {}*",
                            e.llvm_ty
                        ));
                        if e.elem_is_str {
                            let dp = self.fresh_tmp();
                            self.emitln(&format!("  {dp} = load i8*, i8** {epc}"));
                            self.emitln(&format!("  call void @xiom_guard_free(i8* {dp})"));
                        } else if e.elem_is_vec {
                            let dgp = self.fresh_tmp();
                            let dp = self.fresh_tmp();
                            self.emitln(&format!(
                                "  {dgp} = getelementptr %struct.Vec, %struct.Vec* {epc}, i32 0, i32 0"
                            ));
                            self.emitln(&format!("  {dp} = load i8*, i8** {dgp}"));
                            self.emitln(&format!("  call void @xiom_guard_free(i8* {dp})"));
                        } else {
                            for path in &e.paths {
                                let mut cur = epc.clone();
                                let mut cur_ty = e.llvm_ty.clone();
                                for (idx, fty) in path {
                                    let fp = self.fresh_tmp();
                                    self.emitln(&format!(
                                        "  {fp} = getelementptr {cur_ty}, {cur_ty}* {cur}, i32 0, i32 {idx}"
                                    ));
                                    cur = fp;
                                    cur_ty = fty.clone();
                                }
                                if cur_ty == "i8*" {
                                    // Typed Str leaf: `cur` is the i8** field
                                    // slot; free the owned string pointer.
                                    let dp = self.fresh_tmp();
                                    self.emitln(&format!("  {dp} = load i8*, i8** {cur}"));
                                    self.emitln(&format!("  call void @xiom_guard_free(i8* {dp})"));
                                } else {
                                    let dgp = self.fresh_tmp();
                                    let dp = self.fresh_tmp();
                                    self.emitln(&format!(
                                        "  {dgp} = getelementptr %struct.Vec, %struct.Vec* {cur}, i32 0, i32 0"
                                    ));
                                    self.emitln(&format!("  {dp} = load i8*, i8** {dgp}"));
                                    self.emitln(&format!("  call void @xiom_guard_free(i8* {dp})"));
                                }
                            }
                        }
                        let next = self.fresh_tmp();
                        self.emitln(&format!("  {next} = add i64 {i}, 1"));
                        self.emitln(&format!("  store i64 {next}, i64* {idx_slot}"));
                        self.emitln(&format!("  br label %{cond}"));
                        self.emitln(&format!("\n{done}:"));
                    }
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
                VecDrop::Aggregate { data_gep, .. } => {
                    let data_ptr = self.fresh_tmp();
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
