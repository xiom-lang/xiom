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

/// Resolve type aliases to a canonical name for field-type tests.
fn resolve_alias_name(types: &TypeContext, ty: &str) -> String {
    let mut cur = ty.trim().to_string();
    let mut seen = HashSet::new();
    while let Some(next) = types.type_aliases.get(&cur) {
        if !seen.insert(cur.clone()) {
            break;
        }
        cur = next.clone();
    }
    cur
}

/// `Vec[T]` -> `T`.
fn vec_inner_type(ty: &str) -> Option<&str> {
    let ty = ty.trim();
    if ty.starts_with("Vec[") && ty.ends_with(']') {
        Some(ty[4..ty.len() - 1].trim())
    } else {
        None
    }
}

/// `Map[K, V]` -> `(K, V)`, splitting only on the top-level comma.
fn map_kv_types(ty: &str) -> Option<(&str, &str)> {
    let ty = ty.trim();
    if !ty.starts_with("Map[") || !ty.ends_with(']') {
        return None;
    }
    let inner = &ty[4..ty.len() - 1];
    let mut depth = 0usize;
    for (i, ch) in inner.char_indices() {
        match ch {
            '[' | '(' => depth += 1,
            ']' | ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                return Some((inner[..i].trim(), inner[i + 1..].trim()));
            }
            _ => {}
        }
    }
    None
}

/// Deterministic symbol for the generated RECURSIVE drop helper (parse-owned
/// tree: containers + freshly allocated String payloads).
fn enum_tree_drop_helper_name(enum_key: &str) -> String {
    enum_tree_helper_name(enum_key, "__xiom_enum_tree_drop_")
}

/// R4: symbol for the CONTAINER-ONLY helper (builder/stringify-owned tree:
/// Vec/Map data buffers only; every Str leaf is borrowed and stays alive).
pub(crate) fn enum_tree_container_drop_helper_name(enum_key: &str) -> String {
    enum_tree_helper_name(enum_key, "__xiom_enum_tree_containers_")
}

fn enum_tree_helper_name(enum_key: &str, prefix: &str) -> String {
    let mut out = String::from(prefix);
    for ch in enum_key.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    out
}

/// Symbol for the recursive owned-struct payload helper. `free_strings`
/// selects the R3 (fresh Str leaves) or R4 (container-only) provenance.
fn struct_payload_helper_name(struct_key: &str, free_strings: bool) -> String {
    let prefix = if free_strings {
        "__xiom_struct_tree_drop_"
    } else {
        "__xiom_struct_tree_containers_"
    };
    enum_tree_helper_name(struct_key, prefix)
}

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
    /// Locals/params known to be XIOM `Str`. Used to recognize a
    /// `Str + Str` concat as a freshly allocated buffer even when neither
    /// operand is a literal or a known Str-returning call (e.g.
    /// `resp = resp + body` in server_build_response). Without this the
    /// assignment was treated as an alias, disabling the replaced-buffer
    /// free and leaking every intermediate concat.
    str_typed: HashSet<String>,
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
    /// Match-arm payload names bound from a BOXED `Vec` removal
    /// (`pop`/`remove`/`swap_remove` on an owned `Vec[Vec[T]]`). The Option
    /// payload is an i64 handle to a malloc'd `%struct.Vec` header, so the
    /// arm-scope drop must free both the data buffer and the header box.
    boxed_vec_payload: HashSet<String>,
    /// Boxed payload name -> removal container name. Reclamation is only
    /// allowed when the CONTAINER is still a private owned drop candidate
    /// (single declaration, no escape/move/return); an escaped container may
    /// share the removed element with a callee copy.
    boxed_vec_container: HashMap<String, String>,
    /// Leaf names of `#[owns_str]` functions proven to return a freshly
    /// allocated Str. A binding from a DIRECT call result is a drop candidate.
    fresh_str: HashSet<String>,
    /// Leaf names of `#[owns_return]` aggregate functions (frames).
    fresh_aggregate: HashSet<String>,
    /// Leaf -> enum type for `#[owns_return]` Result/Option<named enum>
    /// functions (json_parse). A direct `let p = call` binding owns the tree.
    fresh_enum_tree: HashMap<String, String>,
    /// Leaf -> (named struct type, free_strings) for contracted
    /// `#[owns_return]` / `#[consumes_agg]` Result/Option<named struct>
    /// functions (server_parse_request). A direct `let r = call` binding owns
    /// the Some/Ok payload.
    fresh_struct_payload: HashMap<String, (String, bool)>,
    /// Enum type key/leaf -> variant names. A returned enum-variant
    /// construction (`Type.Variant(value)`) MOVES its by-value arguments into
    /// the returned aggregate; those locals must not get a scope-exit drop.
    enum_ctors: Arc<HashMap<String, HashSet<String>>>,
    /// Bare enum-variant constructor names (no `Type.` prefix), matching the
    /// codegen's bare-constructor resolution.
    bare_enum_ctors: Arc<HashSet<String>>,
    /// Locals bound DIRECTLY from a fresh-enum-tree call.
    enum_tree_decl: HashMap<String, String>,
    /// Locals bound DIRECTLY from a contracted Result/Option<named struct>
    /// call: name -> (struct type, free_strings).
    struct_payload_decl: HashMap<String, (String, bool)>,
    /// Locals bound from a fresh aggregate return; their Vec fields drop.
    aggregate: HashSet<String>,
    /// Locals bound from a struct LITERAL constructed here that owns at least
    /// one fresh Vec/Str field. Only these can prove "the aggregate was
    /// constructed in this function" for field/whole-struct overwrite reclaim.
    aggregate_literal: HashSet<String>,
    /// Aggregate literal local -> Vec field names initialized from a fresh
    /// Vec (ctor / fresh-return call) at the binding. Field overwrite frees
    /// are restricted to these leaves; a borrowed field is never freed.
    aggregate_vec_fields: HashMap<String, Vec<String>>,
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
    /// `#[consumes_str]` callees: leaf -> Str parameter indices. A contracted
    /// call MOVES its fresh Str args into the `&mut` aggregate's owned
    /// `Vec[Str]` fields (Map/Set put/set/add).
    consumes_str_callees: Arc<HashMap<String, Vec<usize>>>,
    /// `#[borrows_vec]` callees: Vec args may be mutated/read during the call
    /// but are not retained; the caller keeps the drop.
    borrows_vec_callees: Arc<HashSet<String>>,
    /// `#[borrows_str]` callees: leaf -> by-value Str parameter indices. The
    /// callee does not retain those arguments, so the caller keeps ownership
    /// (and the scope-exit drop) of its own Str local.
    borrows_str_callees: Arc<HashMap<String, Vec<usize>>>,
    /// Fresh aggregate locals that received only contracted + fresh Str
    /// arguments. Their `Vec[Str]` field elements are freed at scope exit.
    str_vec_owned: HashSet<String>,
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
    /// Single-declaration owned Vec locals reassigned with a fresh/owned Vec.
    /// The old data pointer is freed at each overwrite; the final one at exit.
    vec_rebind: HashSet<String>,
}

pub(crate) struct OwnedLocals {
    pub vec: HashSet<String>,
    pub str: HashSet<String>,
    pub aggregate: HashSet<String>,
    /// Aggregate local -> freshly-owned Str field names.
    pub aggregate_str: HashMap<String, Vec<String>>,
    /// Aggregate literal locals provably uniquely owned for field/whole-struct
    /// overwrite reclamation (single decl, never aliased/moved/returned).
    pub agg_overwrite: HashSet<String>,
    /// Aggregate literal local -> fresh Vec field names owned at the binding.
    pub aggregate_vec_fields: HashMap<String, Vec<String>>,
    /// Local bound directly from a contracted enum-tree call -> enum type.
    pub enum_tree: HashMap<String, String>,
    /// Local bound directly from a contracted Result/Option<named struct>
    /// call -> (struct type, free_strings). Reclaimed at scope exit through a
    /// metadata-generated recursive field-drop helper (Some/Ok only).
    pub struct_payload: HashMap<String, (String, bool)>,
    /// Match-arm payloads bound from a boxed `Vec` removal on an owned
    /// `Vec[Vec[T]]` container: free the element data AND the header box.
    pub boxed_vec_payload: HashSet<String>,
    /// Container local -> (element XIOM type, element Vec field names) for
    /// containers whose elements are all owned and never reordered/removed.
    pub elem: HashMap<String, (String, Vec<String>)>,
    /// Single-declaration Str locals eligible for replaced-buffer
    /// reclamation on fresh reassignment (REL-20261028).
    pub str_rebind: HashSet<String>,
    /// Rebind locals that are not returned: their final owned buffer is freed
    /// at scope exit (the replaced buffers are already freed at each rebind).
    pub str_rebind_exit: HashSet<String>,
    /// Single-declaration locals never referenced after their binding. Used to
    /// free a dead boxed `remove`/`pop` result at the binding site.
    pub unused: HashSet<String>,
    /// Fresh aggregates whose owned `Vec[Str]` fields were populated only by
    /// `#[consumes_str]` calls with fresh Str args. Their elements are freed.
    pub aggregate_str_vec: HashSet<String>,
    /// Single-declaration owned Vec locals reassigned with a fresh/owned Vec.
    /// The old data is freed at each overwrite, the final data at scope exit.
    pub vec_rebind: HashSet<String>,
}

/// Returns the local names that are owned and private for the function body.
pub(crate) fn collect_owned_vec_locals(
    block: &Block,
    fresh: &HashSet<String>,
    fresh_payload: &HashSet<String>,
    fresh_aggregate: &HashSet<String>,
    fresh_enum_tree: &HashMap<String, String>,
    fresh_struct_payload: &HashMap<String, (String, bool)>,
    fresh_str: &HashSet<String>,
    str_params: &HashSet<String>,
    trust_unsafe: bool,
    elem_type_vec_fields: &Arc<HashMap<String, Vec<String>>>,
    known_nonretaining: &Arc<HashSet<String>>,
    retaining_callees: &Arc<HashSet<String>>,
    byval_vec_readers: &Arc<HashSet<String>>,
    consumes_str_callees: &Arc<HashMap<String, Vec<usize>>>,
    borrows_vec_callees: &Arc<HashSet<String>>,
    borrows_str_callees: &Arc<HashMap<String, Vec<usize>>>,
    by_value_vec_params: &HashSet<String>,
    enum_ctors: &Arc<HashMap<String, HashSet<String>>>,
    bare_enum_ctors: &Arc<HashSet<String>>,
) -> OwnedLocals {
    let mut state = ScanState::default();
    state.by_value_vec_params = by_value_vec_params.clone();
    state.enum_ctors = enum_ctors.clone();
    state.bare_enum_ctors = bare_enum_ctors.clone();
    state.fresh = fresh.clone();
    state.fresh_payload = fresh_payload.clone();
    state.fresh_aggregate = fresh_aggregate.clone();
    state.fresh_enum_tree = fresh_enum_tree.clone();
    state.fresh_struct_payload = fresh_struct_payload.clone();
    state.fresh_str = fresh_str.clone();
    state.str_typed = str_params.clone();
    state.trust_unsafe = trust_unsafe;
    state.elem_type_vec_fields = elem_type_vec_fields.clone();
    state.known_nonretaining = known_nonretaining.clone();
    state.retaining_callees = retaining_callees.clone();
    state.byval_vec_readers = byval_vec_readers.clone();
    state.consumes_str_callees = consumes_str_callees.clone();
    state.borrows_vec_callees = borrows_vec_callees.clone();
    state.borrows_str_callees = borrows_str_callees.clone();
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
    // Locals declared once and never referenced again. A dead
    // `let gone = v.remove(i)` result is freed at the binding site (the
    // boxed element is not reachable after this point); any later use marks
    // the name escaping and disqualifies it.
    let unused_locals: HashSet<String> = state
        .decls
        .iter()
        .filter(|(name, info)| {
            info.decls == 1
                && !state.escaping.contains(*name)
                && !state.moved.contains(*name)
                && !state.returned.contains(*name)
        })
        .map(|(name, _)| name.clone())
        .collect();
    // Contract proof: a fresh aggregate whose only uses were contracted
    // `#[consumes_str]` calls (with fresh Str args) or safe borrow reads. Any
    // escape/move/return disqualifies the element free (leak-only).
    let aggregate_str_vec: HashSet<String> = state
        .str_vec_owned
        .into_iter()
        .filter(|name| {
            single_decl.contains(name)
                && !state.escaping.contains(name)
                && !state.moved.contains(name)
                && !state.returned.contains(name)
        })
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
    // Boxed-removal arm payloads are only reclaimed when the SAME proof that
    // makes the local a Vec candidate holds: single owned binding, no escape,
    // no move, no return. A leaked payload stays leak-only (never a UAF).
    let boxed_vec_payload: HashSet<String> = state
        .boxed_vec_payload
        .iter()
        .filter(|name| {
            vec.contains(*name)
                && single_decl.contains(*name)
                && !state.escaping.contains(*name)
                && !state.moved.contains(*name)
                && !state.returned.contains(*name)
                && state.boxed_vec_container.get(*name).map_or(false, |container| {
                    single_decl.contains(container)
                        && !state.escaping.contains(container)
                        && !state.moved.contains(container)
                        && !state.returned.contains(container)
                })
        })
        .cloned()
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
    // Vec overwrite-drop: only plain Vec locals (no element-payload tag, so
    // the overwrite free cannot orphan element payloads). The local must be
    // owned, single-decl, non-escaping/moved/returned.
    let vec_rebind: HashSet<String> = state
        .vec_rebind
        .into_iter()
        .filter(|name| {
            single_decl.contains(name)
                && vec.contains(name)
                && !elem.contains_key(name)
                && !state.escaping.contains(name)
                && !state.moved.contains(name)
                && !state.returned.contains(name)
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
    // Field/whole-struct overwrite reclaim (2117): only a struct literal
    // CONSTRUCTED here, declared once, and never copied as a whole value,
    // returned, moved, or field-aliased. `escaping` covers every non-destination
    // mention (a field read into a binding, a by-value argument, an alias).
    let agg_overwrite: HashSet<String> = state
        .aggregate_literal
        .iter()
        .filter(|name| {
            single_decl.contains(*name)
                && !state.escaping.contains(*name)
                && !state.moved.contains(*name)
                && !state.returned.contains(*name)
        })
        .cloned()
        .collect();
    let aggregate_vec_fields: HashMap<String, Vec<String>> = state
        .aggregate_vec_fields
        .into_iter()
        .filter(|(name, _)| agg_overwrite.contains(name))
        .collect();
    OwnedLocals {
        agg_overwrite,
        aggregate_vec_fields,
        boxed_vec_payload,
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
        enum_tree: state
            .enum_tree_decl
            .into_iter()
            .filter(|(name, _)| {
                single_decl.contains(name)
                    && !state.escaping.contains(name)
                    && !state.moved.contains(name)
                    && !state.returned.contains(name)
            })
            .collect(),
        struct_payload: state
            .struct_payload_decl
            .into_iter()
            .filter(|(name, _)| {
                single_decl.contains(name)
                    && !state.escaping.contains(name)
                    && !state.moved.contains(name)
                    && !state.returned.contains(name)
            })
            .collect(),
        elem,
        str_rebind,
        str_rebind_exit,
        vec_rebind,
        unused: unused_locals,
        aggregate_str_vec,
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

/// `#[owns_str]` call-return provenance: a DIRECT call to a leaf proven to
/// return a freshly allocated Str. Nothing else (block/if/match wrappers,
/// field reads, bare identifiers) is accepted, so a borrowed or literal Str
/// cannot inherit the drop.
pub(crate) fn is_fresh_str_call(expr: &Expr, fresh: &HashSet<String>) -> bool {
    match expr {
        Expr::Paren(inner, _) => is_fresh_str_call(inner, fresh),
        Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
            call_leaf(func).map_or(false, |leaf| fresh.contains(leaf))
        }
        _ => false,
    }
}

/// Fresh Str initializer OR direct `#[owns_str]` call return. Used at a
/// binding/rebind site to decide whether the local owns a heap buffer.
pub(crate) fn is_fresh_str_value(expr: &Expr, fresh: &HashSet<String>) -> bool {
    is_fresh_str_init(expr) || is_fresh_str_call(expr, fresh)
}

/// Scan-aware Str-operand test: additionally recognizes a local/param known
/// to be `Str` (see `ScanState::str_typed`). `Str + Str` always lowers to
/// `xiom_str_concat`, which allocates a NEW buffer, so a concat of two known
/// Str values is a fresh owned Str.
fn str_operand_scan(expr: &Expr, state: &ScanState) -> bool {
    match expr {
        Expr::Str(..) => true,
        Expr::Paren(inner, _) => str_operand_scan(inner, state),
        Expr::Ident(id) => state.str_typed.contains(&id.name),
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
                        | "str_repeat"
                        | "str_join"
                        | "str_replace"
                        | "str_slice"
                )
        ),
        _ => false,
    }
}

/// `is_fresh_str_value` plus the scan-proven `Str + Str` concat shape.
fn is_fresh_str_value_for_scan(expr: &Expr, state: &ScanState) -> bool {
    if is_fresh_str_value(expr, &state.fresh_str) {
        return true;
    }
    match expr {
        Expr::Paren(inner, _) => is_fresh_str_value_for_scan(inner, state),
        Expr::Binary(left, BinOp::Add, right, _) => {
            str_operand_scan(left, state) || str_operand_scan(right, state)
        }
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

/// A direct struct-literal field initializer that yields a freshly allocated
/// Vec. Bare identifiers/projections are intentionally excluded: the field
/// store is shallow, so those would alias another owner.
fn fresh_owned_vec_field(expr: &Expr, state: &ScanState) -> bool {
    is_fresh_vec_init(expr) || is_fresh_vec_call(expr, &state.fresh)
}

fn agg_owned_vec_field(state: &ScanState, name: &str, field: &str) -> bool {
    state
        .aggregate_vec_fields
        .get(name)
        .map_or(false, |fs| fs.iter().any(|f| f == field))
}

fn agg_owned_str_field(state: &ScanState, name: &str, field: &str) -> bool {
    state
        .aggregate_str_fields
        .get(name)
        .map_or(false, |fs| fs.iter().any(|f| f == field))
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
            // A whole-value copy (`var out = r`) shares r's field headers.
            // Transfer ownership to the copy and disarm r's aggregate drops:
            // the copy is not a drop site, so this is leak-only (never a UAF).
            if let Expr::Ident(src) = value {
                if state.aggregate_literal.contains(&src.name) {
                    state.moved.insert(src.name.clone());
                }
            }
            // REL-20261027-1717-engine: a local struct literal that owns a
            // fresh Vec field is an aggregate drop site. The source Vec local
            // is MOVED into the field (do not drop it separately); a fresh
            // Vec field expression is owned by the literal.
            if let Expr::Struct(_, fields, spread, _) = value {
                if spread.is_none() {
                    let owns_vec = struct_literal_owns_vec(fields, state);
                    // 2117: record WHICH Vec fields were freshly initialized
                    // here. The field store is shallow, so only these leaves
                    // may be freed on a later field overwrite; a borrowed /
                    // projected field is never in this list.
                    let owned_vec_fields: Vec<String> = fields
                        .iter()
                        .filter(|(_, fv)| fresh_owned_vec_field(fv, state))
                        .map(|(fname, _)| fname.name.clone())
                        .collect();
                    // m375: a struct literal whose Str field is a FRESH Str
                    // owns that buffer. Track the exact field names so the
                    // aggregate drop frees only proven-fresh leaves; a
                    // literal/borrowed Str field in the same struct is left
                    // alone. Moving a fresh Str local into the field disarms
                    // its own plain-Str drop.
                    let mut owned_str_fields: Vec<String> = Vec::new();
                    for (fname, fval) in fields {
                        let fresh_field = is_fresh_str_value_for_scan(fval, state)
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
                    // 2117 candidate: a literal constructed here with at
                    // least one freshly owned Vec/Str leaf. This is kept
                    // separate from `aggregate` (which also covers
                    // `#[owns_return]` calls) so overwrite reclaim stays
                    // restricted to values this function actually built.
                    if !owned_vec_fields.is_empty() || !owned_str_fields.is_empty() {
                        state.aggregate_literal.insert(name.name.clone());
                    }
                    if !owned_str_fields.is_empty() {
                        state
                            .aggregate_str_fields
                            .insert(name.name.clone(), owned_str_fields);
                    }
                    if !owned_vec_fields.is_empty() {
                        state
                            .aggregate_vec_fields
                            .insert(name.name.clone(), owned_vec_fields);
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
            // R3: a local bound DIRECTLY from a contracted enum-tree call
            // (json_parse) owns the returned tree. Only this provenance is
            // accepted; `json_string`/`json_get`/match arms stay leak-only.
            if let Some(enum_name) = fresh_enum_tree_call(value, &state.fresh_enum_tree) {
                state
                    .enum_tree_decl
                    .insert(name.name.clone(), enum_name);
            }
            // R3/R4: a local bound DIRECTLY from a contracted
            // Result/Option<named struct> call owns the Some/Ok payload.
            if let Some(payload) = fresh_struct_payload_call(value, &state.fresh_struct_payload) {
                state
                    .struct_payload_decl
                    .insert(name.name.clone(), payload);
            }
            let info = state.decls.entry(name.name.clone()).or_insert_with(|| VecDeclInfo {
                decls: 0,
                all_fresh: true,
            });
            info.decls += 1;
            info.all_fresh &= fresh;
            let fresh_str = is_fresh_str_value_for_scan(value, state);
            let sinfo = state
                .str_decls
                .entry(name.name.clone())
                .or_insert_with(|| VecDeclInfo {
                    decls: 0,
                    all_fresh: true,
                });
            sinfo.decls += 1;
            sinfo.all_fresh &= fresh_str;
            // Track Str-typed locals/params so a later `resp = resp + body`
            // recognizes the concat operand as Str (both sides of the concat
            // are proven Str, so `xiom_str_concat` allocates a new buffer).
            let annotated_str = matches!(
                ty.as_deref(),
                Some(Type::Named(id, _)) if id.name == "Str" || id.name.ends_with(".Str")
            );
            if annotated_str || fresh_str || matches!(value, Expr::Str(..)) {
                state.str_typed.insert(name.name.clone());
            } else {
                state.str_typed.remove(&name.name);
            }
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
                            && !is_fresh_str_value_for_scan(value, state)
                        {
                            // A fresh Str RHS is itself an owned buffer, so
                            // keeping the exit drop armed is safe (it frees
                            // the FINAL field). A borrowed/alias RHS still
                            // disarms the whole exit drop (leak-only).
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
                    is_fresh_str_value_for_scan(value, state)
                        && state
                            .decls
                            .get(&id.name)
                            .map_or(false, |info| info.decls == 1)
                }
                _ => false,
            };
            // Vec overwrite-drop (REL-20261101-2117): a single-declaration
            // owned Vec local reassigned with a fresh/owned Vec can free the
            // REPLACED data pointer at the assignment. The exit drop is
            // replaced by a guarded pointer drop (final value). A non-fresh
            // assignment disqualifies the local (leak-only) via escaping.
            let fresh_vec_rebind = match place {
                Expr::Ident(id) => {
                    state.decls.get(&id.name).map_or(false, |info| {
                        info.decls == 1 && info.all_fresh
                    }) && (is_fresh_vec_init(value)
                        || is_fresh_vec_call(value, &state.fresh)
                        || is_owned_copy_vec_init(value, &state.by_value_vec_params))
                }
                _ => false,
            };
            // 2117 whole-struct overwrite: a literal-bound aggregate replaced
            // by a fresh `#[owns_return]` aggregate call. The contract proves
            // the new value owns every Vec field, so the exit drop stays sound.
            let agg_whole_rebind = match place {
                Expr::Ident(id) => {
                    state.aggregate_literal.contains(&id.name)
                        && is_fresh_aggregate_call(value, &state.fresh_aggregate)
                }
                _ => false,
            };
            // 2117 field overwrite: `d.v = fresh`. Only a leaf proven owned at
            // the literal binding, and only a self-contained fresh RHS. A
            // whole-value copy of the source is marked moved just below.
            let agg_field_rebind = match place {
                Expr::Field(base, field, _) => match base.as_ref() {
                    Expr::Ident(id) => {
                        if agg_owned_vec_field(state, &id.name, &field.name) {
                            is_fresh_vec_init(value) || is_fresh_vec_call(value, &state.fresh)
                        } else if agg_owned_str_field(state, &id.name, &field.name) {
                            is_fresh_str_value_for_scan(value, state)
                        } else {
                            false
                        }
                    }
                    _ => false,
                },
                _ => false,
            };
            // `d = other` copies the header; transfer ownership to the
            // destination copy and disarm the source's aggregate drops.
            if let Expr::Ident(src) = value {
                if state.aggregate_literal.contains(&src.name) {
                    state.moved.insert(src.name.clone());
                }
            }
            if let Expr::Ident(id) = place {
                if fresh_str_rebind {
                    state.str_rebind.insert(id.name.clone());
                } else if fresh_vec_rebind {
                    state.vec_rebind.insert(id.name.clone());
                } else if agg_whole_rebind {
                    // The aggregate stays uniquely owned; codegen frees the
                    // replaced owned fields before the store.
                } else {
                    state.escaping.insert(id.name.clone());
                }
            } else if agg_field_rebind {
                // Destination store through a private aggregate slot: the
                // base is neither read nor copied, so it does not escape.
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
            // a scope-exit drop. The same applies to a BOXED removal payload
            // (`match outer.pop() { Some(v) => .. }` on an owned Vec[Vec[T]]):
            // the removed element left the container's len and owns its buffer.
            let boxed_removal = boxed_vec_removal_container(scrutinee, state);
            if boxed_removal.is_some()
                || is_fresh_payload_call(scrutinee, &state.fresh_payload)
                || is_fresh_payload_local(scrutinee, &state.fresh_payload_locals)
            {
                for arm in arms {
                    if let Some(name) = payload_binding_name(&arm.pattern) {
                        let info = state.decls.entry(name.clone()).or_insert_with(|| VecDeclInfo {
                            decls: 0,
                            all_fresh: true,
                        });
                        info.decls += 1;
                        info.all_fresh = true;
                        if let Some(container) = &boxed_removal {
                            state.boxed_vec_payload.insert(name.clone());
                            state
                                .boxed_vec_container
                                .insert(name, container.clone());
                        }
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
        Stmt::Destructure(_, value, _, _) => scan_expr(value, UseCtx::Escape, state),
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
    // A block-valued expression in a RETURN/transfer context (`return
    // if .. { a } else { b }`, `match .. { Ok(x) => { .. Ok(x) } }`) has the
    // same ownership semantics as a function-body tail: every taken branch's
    // tail value is moved out. Propagate the transfer so `mark_returned_roots`
    // sees the moved local. Without this a block arm's `Ok(request)` left
    // `request` in `str_rebind` while it was NOT recorded as returned, so
    // codegen freed the final buffer at scope exit AFTER storing it into the
    // returned payload (REL-20261104 Str UAF: stdlib http_build_request,
    // orbitdb schema_row).
    let tail_is_return = state.in_return;
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
            // A header-only use of an owned aggregate field (`d.v.len()`,
            // `d.v.push(..)`) does not copy the Vec header. SafeBase is only
            // used for the receiver of a known non-view Vec method, so the
            // aggregate stays non-escaping and overwrite reclaim is allowed.
            if ctx == UseCtx::SafeBase {
                if let Expr::Ident(bid) = base.as_ref() {
                    if agg_owned_vec_field(state, &bid.name, &field.name) {
                        scan_expr(base, UseCtx::SafeBase, state);
                        return;
                    }
                }
            }
            // Scalar tag reads on an owned Result/Option/Vec local. `is_err`
            // / `is_ok` / `is_some` / `is_none` read only the discriminant, so
            // they cannot alias the payload or its buffers.
            let safe_read = matches!(
                field.name.as_str(),
                "len" | "capacity" | "is_empty" | "is_err" | "is_ok" | "is_some" | "is_none"
            );
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
                        // Removal/reorder methods keep the REMAINING elements
                        // owned: `remove`/`swap_remove`/`pop` move one element
                        // out (its payload is handled at the removal site when
                        // the result is dead), `sort`/`reverse`/`swap` reorder,
                        // and `retain`/`dedup`/`truncate`/`clear` drop elements
                        // without returning them (those leak conservatively).
                        // The container's exit loop only walks the CURRENT len,
                        // so it never frees a removed element twice.
                        const ELEM_REMOVAL_ORDER_ONLY: &[&str] = &[
                            "remove", "swap_remove", "pop", "sort", "sort_by", "reverse",
                            "retain", "dedup", "truncate", "clear", "swap", "shrink_to_fit",
                        ];
                        if !ELEM_READ_ONLY.contains(&m)
                            && m != "push"
                            && m != "insert"
                            && !ELEM_REMOVAL_ORDER_ONLY.contains(&m)
                        {
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
                                                            is_fresh_str_value(fval, &state.fresh_str)
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
                                        && is_fresh_str_value(inner, &state.fresh_str) =>
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
            // REL-20261031-2337-compiler: `#[consumes_str]` (Map/Set
            // put/set/add) moves fresh Str args into the `&mut` aggregate's
            // owned Vec[Str] fields. Accept only a fresh `#[owns_return]`
            // aggregate whose Str args are fresh; mark both sides and record
            // the aggregate for element drops. Any other use (escape, return,
            // unknown call) disqualifies it at finalize (leak-only).
            let consumes_leaf = match func.as_ref() {
                Expr::Ident(id) => Some(id.name.rsplit('.').next().unwrap_or(&id.name)),
                Expr::Field(_, method, _) => Some(method.name.as_str()),
                _ => None,
            };
            let mut consumed_str_indices: HashSet<usize> = HashSet::new();
            let mut consumed_agg_index: Option<usize> = None;
            if let Some(indices) = consumes_leaf
                .and_then(|leaf| state.consumes_str_callees.get(leaf).cloned())
            {
                if let Some((agg_idx, agg_name)) = args.iter().enumerate().find_map(|(i, a)| {
                    match a {
                        Expr::Ref(inner, _) | Expr::MutRef(inner, _) => match inner.as_ref() {
                            Expr::Ident(id) if state.aggregate.contains(&id.name) => {
                                Some((i, id.name.clone()))
                            }
                            _ => None,
                        },
                        _ => None,
                    }
                }) {
                    let all_fresh = indices.iter().all(|&i| {
                        args.get(i).map_or(false, |a| {
                            is_fresh_str_init(a)
                                || matches!(a, Expr::Ident(id)
                                    if state.str_decls.get(&id.name).map_or(false, |d| d.all_fresh)
                                        && !state.escaping.contains(&id.name)
                                        && !state.moved.contains(&id.name)
                                        && state.decl_block.get(&id.name)
                                            == Some(&state.cur_block))
                        })
                    });
                    if all_fresh {
                        state.str_vec_owned.insert(agg_name);
                        consumed_agg_index = Some(agg_idx);
                        for &i in &indices {
                            consumed_str_indices.insert(i);
                            if let Some(Expr::Ident(id)) = args.get(i) {
                                state.moved.insert(id.name.clone());
                            }
                        }
                    }
                }
            }
            // `#[borrows_vec]` contract: the callee may mutate/upload through
            // a Vec reference and read a by-value header, but it never retains
            // the header. The caller keeps ownership and drops at scope exit.
            // This is the engine per-frame Vulkan UI batch fix (REL-20261101-
            // 0332-engine: ~2.9 MB/frame). It deliberately OVERRIDES the
            // `&mut` => retaining classification; the attribute is the proof.
            let borrows_vec_callee = match func.as_ref() {
                Expr::Ident(id) => state
                    .borrows_vec_callees
                    .contains(id.name.rsplit('.').next().unwrap_or(&id.name)),
                Expr::Field(_, method, _) => {
                    state.borrows_vec_callees.contains(&method.name)
                }
                _ => false,
            };
            // `#[borrows_str]` contract (R2): the callee does not retain a
            // by-value Str argument (json_parse reads its input and builds a
            // new tree). The argument stays owned by the caller; only the
            // listed Str parameter positions take this path.
            let borrows_str_indices = match func.as_ref() {
                Expr::Ident(id) => state
                    .borrows_str_callees
                    .get(id.name.rsplit('.').next().unwrap_or(&id.name))
                    .cloned(),
                Expr::Field(_, method, _) => {
                    state.borrows_str_callees.get(&method.name).cloned()
                }
                _ => None,
            };
            for (arg_index, arg) in args.iter().enumerate() {
                if consumed_agg_index == Some(arg_index)
                    || consumed_str_indices.contains(&arg_index)
                {
                    continue;
                }
                if safe_str_callee {
                    scan_expr(arg, UseCtx::SafeBase, state);
                    continue;
                }
                if borrows_str_indices
                    .as_ref()
                    .map_or(false, |idx| idx.contains(&arg_index))
                {
                    // Read-only during the call; not retained. The caller's
                    // Str local keeps its scope-exit drop (a literal/borrowed
                    // argument has no drop and is never freed).
                    scan_borrow_arg(arg, state);
                    continue;
                }
                if borrows_vec_callee {
                    if let Expr::Ref(inner, _) | Expr::MutRef(inner, _) = arg {
                        // Mutated/uploaded during the call only; not retained.
                        scan_borrow_arg(inner, state);
                        continue;
                    }
                    if let Expr::Struct(_, _, spread, _) = arg {
                        if spread.is_none() {
                            // Inline frame/command literal passed by value:
                            // keep the owned field locals' scope drops.
                            scan_borrows_vec_literal(arg, state);
                            continue;
                        }
                    }
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
                    && (!state.in_unsafe
                        || (state.trust_unsafe && state.in_trusted_unsafe))
                    && (borrows_vec_callee
                        || (!callee_retains
                            && match func.as_ref() {
                                Expr::Ident(id) => state.byval_vec_readers.contains(
                                    id.name.rsplit('.').next().unwrap_or(&id.name),
                                ),
                                Expr::Field(_, method, _) => {
                                    state.byval_vec_readers.contains(&method.name)
                                }
                                _ => false,
                            }));
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
        Expr::BlockExpr(block, _) => scan_block(block, state, tail_is_return),
        Expr::Closure(_, _, body, _) => mark_block_mentions(body, state),
        Expr::PipeClosure(_, body, _) => scan_expr(body, UseCtx::Escape, state),
        Expr::If(cond, then_block, elifs, else_block, _) => {
            scan_expr(cond, UseCtx::Escape, state);
            scan_block(then_block, state, tail_is_return);
            for (elif_cond, elif_block) in elifs {
                scan_expr(elif_cond, UseCtx::Escape, state);
                scan_block(elif_block, state, tail_is_return);
            }
            if let Some(else_block) = else_block {
                scan_block(else_block, state, tail_is_return);
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            scan_expr(scrutinee, UseCtx::Escape, state);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    scan_expr(guard, UseCtx::Escape, state);
                }
                match &arm.body {
                    MatchBody::Block(block) => scan_block(block, state, tail_is_return),
                    MatchBody::Expr(inner) => {
                        if tail_is_return {
                            mark_returned_roots(inner, state);
                        }
                        scan_expr(inner, UseCtx::Escape, state);
                    }
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
                scan_block(body, state, tail_is_return);
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

pub(crate) fn call_leaf(expr: &Expr) -> Option<&str> {
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

/// Call to an `#[owns_return]` Result/Option<named enum> function. Returns
/// the enum type name. Only a DIRECT call is accepted here: a block/if/match
/// wrapped call, a literal constructor, or a field access is not provably the
/// contracted tree.
fn fresh_enum_tree_call(expr: &Expr, fresh: &HashMap<String, String>) -> Option<String> {
    match expr {
        Expr::Paren(inner, _) => fresh_enum_tree_call(inner, fresh),
        Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
            call_leaf(func).and_then(|leaf| fresh.get(leaf).cloned())
        }
        _ => None,
    }
}

/// Call to a contracted `#[owns_return]` / `#[consumes_agg]`
/// Result/Option<named struct> function. Returns (struct type, free_strings).
/// Only a DIRECT call is accepted: a block/if/match wrapped call, a literal
/// constructor, or a field access is not provably the contracted payload.
fn fresh_struct_payload_call(
    expr: &Expr,
    fresh: &HashMap<String, (String, bool)>,
) -> Option<(String, bool)> {
    match expr {
        Expr::Paren(inner, _) => fresh_struct_payload_call(inner, fresh),
        Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
            call_leaf(func).and_then(|leaf| fresh.get(leaf).cloned())
        }
        _ => None,
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

/// `pop`/`remove`/`swap_remove` on a container whose prepass element tag proved
/// every element is an owned `Vec` header. The result Option payload is a
/// malloc'd `%struct.Vec` box (a copy of the removed element), and the removed
/// slot is outside the container's current len, so the arm binding owns it.
fn boxed_vec_removal_container(expr: &Expr, state: &ScanState) -> Option<String> {
    let (func, _args) = match expr {
        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => (func, args),
        Expr::Paren(inner, _) => return boxed_vec_removal_container(inner, state),
        _ => return None,
    };
    let Expr::Field(recv, method, _) = func.as_ref() else {
        return None;
    };
    if !matches!(method.name.as_str(), "pop" | "remove" | "swap_remove") {
        return None;
    }
    let Expr::Ident(id) = recv.as_ref() else {
        return None;
    };
    // The element tag must still be VALID: a disqualified container
    // (`elem_unsafe`) may hold a borrowed header whose buffer this function
    // must not free.
    if state.elem_unsafe.contains(&id.name)
        || !state
            .elem_candidate
            .get(&id.name)
            .map_or(false, |elem| elem.starts_with("Vec["))
    {
        return None;
    }
    Some(id.name.clone())
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

/// Scan an inline aggregate literal passed by value to a `#[borrows_vec]`
/// callee. The callee reads the fields during the call but does not retain
/// them, so owned Vec/aggregate LOCALS used as field values keep their
/// scope-exit drop (the literal temp is a shallow copy; the caller owns the
/// backing buffers). Nested struct literals are walked the same way
/// (`RhiCmdList{ draws: vp_draws }`). A fresh inline expression has no local
/// drop and stays leak-only.
fn scan_borrows_vec_literal(expr: &Expr, state: &mut ScanState) {
    match expr {
        Expr::Struct(_, fields, spread, _) if spread.is_none() => {
            for (_, value) in fields {
                scan_borrows_vec_literal(value, state);
            }
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
            for elem in elems {
                scan_borrows_vec_literal(elem, state);
            }
        }
        Expr::Paren(inner, _) => scan_borrows_vec_literal(inner, state),
        Expr::Ident(id)
            if state.decls.get(&id.name).map_or(false, |info| info.all_fresh)
                || state.aggregate.contains(&id.name) =>
        {
            // Owned local: keep its drop; the callee does not retain it.
            scan_expr(expr, UseCtx::SafeBase, state);
        }
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
        // `Type.Variant(value)` (and its bare form) constructs the returned
        // enum aggregate; `value` is MOVED into it. Without this a local Str
        // accumulator returned through an enum variant kept its scope-exit
        // drop, freeing the buffer the caller now owns (REL-20261104:
        // orbitdb `schema_row` text values). A false positive is leak-only.
        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
            let ctor = match func.as_ref() {
                Expr::Field(recv, variant, _) => match recv.as_ref() {
                    Expr::Ident(ty) => state
                        .enum_ctors
                        .get(&ty.name)
                        .map_or(false, |vs| vs.contains(&variant.name)),
                    _ => false,
                },
                Expr::Ident(id) => state.bare_enum_ctors.contains(&id.name),
                _ => false,
            };
            if ctor {
                for arg in args {
                    mark_returned_roots(arg, state);
                }
            }
        }
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
        Stmt::Destructure(_, value, _, _) => mark_expr_mentions(value, state),
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
    safe_boxed_vec: HashSet<String>,
    safe_str: HashSet<String>,
    safe_aggregate: HashSet<String>,
    safe_aggregate_str: HashMap<String, Vec<String>>,
    safe_agg_overwrite: HashSet<String>,
    safe_aggregate_vec_owned: HashMap<String, Vec<String>>,
    safe_aggregate_str_vec: HashSet<String>,
    safe_enum_tree: HashMap<String, String>,
    safe_struct_payload: HashMap<String, (String, bool)>,
    safe_elem: HashMap<String, (String, Vec<String>)>,
    safe_str_rebind: HashSet<String>,
    safe_str_rebind_exit: HashSet<String>,
    unused: HashSet<String>,
    safe_vec_rebind: HashSet<String>,
    str_rebind_state: HashMap<String, (String, String)>,
    vec_rebind_alloca: HashMap<String, String>,
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
        str_params: &HashSet<String>,
    ) {
        self.fctx.by_value_vec_params = by_value_vec_params.clone();
        let fresh = self.fresh_vec_returns.clone();
        let fresh_payload = self.fresh_payload_returns.clone();
        let fresh_aggregate = self.fresh_aggregate_returns.clone();
        let fresh_enum_tree = self.fresh_enum_tree_returns.clone();
        let fresh_struct_payload = self.fresh_struct_payload_returns.clone();
        let fresh_str = self.fresh_str_returns.clone();
        // Program-global sets, built once in build_drop_caches(); sharing them
        // by Arc keeps the per-function prepass O(body), not O(catalog). The
        // engine injects ~12k decls; rebuilding/cloning the catalog per
        // function made a full editor compile take hours.
        let elem_type_vec_fields = self.elem_type_vec_fields.clone();
        let known_nonretaining = self.known_nonretaining.clone();
        let retaining_callees = self.retaining_callees.clone();
        let byval_vec_readers = self.byval_vec_readers.clone();
        let consumes_str_callees = self.consumes_str_callees.clone();
        let borrows_vec_callees = self.borrows_vec_callees.clone();
        let borrows_str_callees = self.borrows_str_callees.clone();
        let owned = collect_owned_vec_locals(
            body,
            &fresh,
            &fresh_payload,
            &fresh_aggregate,
            &fresh_enum_tree,
            &fresh_struct_payload,
            &fresh_str,
            str_params,
            trust_unsafe,
            &elem_type_vec_fields,
            &known_nonretaining,
            &retaining_callees,
            &byval_vec_readers,
            &consumes_str_callees,
            &borrows_vec_callees,
            &borrows_str_callees,
            by_value_vec_params,
            &self.enum_ctors,
            &self.bare_enum_ctors,
        );
        self.fctx.vec_drop_safe = owned.vec;
        self.fctx.boxed_vec_payload_safe = owned.boxed_vec_payload;
        self.fctx.str_drop_safe = owned.str;
        self.fctx.aggregate_drop_safe = owned.aggregate;
        self.fctx.aggregate_str_drop_safe = owned.aggregate_str;
        self.fctx.agg_overwrite_safe = owned.agg_overwrite;
        self.fctx.aggregate_vec_owned = owned.aggregate_vec_fields;
        self.fctx.enum_tree = owned.enum_tree;
        self.fctx.struct_payload_tree = owned.struct_payload;
        self.fctx.aggregate_str_vec_safe = owned.aggregate_str_vec;
        self.fctx.elem_drop_safe = owned.elem;
        self.fctx.str_rebind_safe = owned.str_rebind;
        self.fctx.str_rebind_exit = owned.str_rebind_exit;
        self.fctx.unused_locals = owned.unused;
        self.fctx.vec_rebind_safe = owned.vec_rebind;
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
        let (vec, payload, aggregate, enum_tree, struct_payload, agg_returns, agg_callees, str_returns) =
            crate::fresh_return::collect_fresh_ownership(program);
        if std::env::var("XIOM_DEBUG_OWNERSHIP").is_ok() {
            eprintln!(
                "ownership: vec={} payload={} aggregate={} enum_tree={} struct_payload={} consumes_agg_returns={} consumes_agg_callees={} read_at_vec={} read_at_payload={} make_frame_aggregate={}",
                vec.len(),
                payload.len(),
                aggregate.len(),
                enum_tree.len(),
                struct_payload.len(),
                agg_returns.len(),
                agg_callees.len(),
                vec.contains("read_at"),
                payload.contains("read_at"),
                aggregate.contains("make_frame")
            );
        }
        self.fresh_vec_returns = vec;
        self.fresh_payload_returns = payload;
        self.fresh_aggregate_returns = aggregate;
        self.fresh_enum_tree_returns = enum_tree;
        self.fresh_struct_payload_returns = struct_payload;
        self.fresh_str_returns = str_returns;
        self.consumes_agg_returns = agg_returns;
        self.consumes_agg_callees = agg_callees;
        self.retaining_callees = Arc::new(crate::fresh_return::collect_retaining_callees(program));
        self.byval_vec_readers = Arc::new(crate::fresh_return::collect_byval_vec_readers(program));
        self.consumes_str_callees =
            Arc::new(crate::fresh_return::collect_str_consuming_callees(program));
        self.borrows_vec_callees =
            Arc::new(crate::fresh_return::collect_borrows_vec_callees(program));
        self.borrows_str_callees =
            Arc::new(crate::fresh_return::collect_borrows_str_callees(program));
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
        // Enum-variant constructors are aggregate transfers: a returned
        // `Type.Variant(value)` moves `value` into the returned enum. Index
        // both the registry key and the bare leaf; the leaf union is a safe
        // over-approximation (a false positive only disables a drop = leak).
        let mut enum_ctors: HashMap<String, HashSet<String>> = HashMap::new();
        let mut bare_enum_ctors: HashSet<String> = HashSet::new();
        for (key, vars) in self.types.enum_variants.entries() {
            let names: HashSet<String> = vars.iter().map(|(v, _)| v.clone()).collect();
            enum_ctors.entry(key.clone()).or_default().extend(names.clone());
            let leaf = key.rsplit('.').next().unwrap_or(&key).to_string();
            enum_ctors.entry(leaf).or_default().extend(names);
            for (variant, _) in vars {
                bare_enum_ctors.insert(variant);
            }
        }
        self.enum_ctors = Arc::new(enum_ctors);
        self.bare_enum_ctors = Arc::new(bare_enum_ctors);
    }

    /// Reset per-function drop state. Called at every function-body start.
    pub(crate) fn clear_vec_drops(&mut self) {
        self.fctx.vec_drops.clear();
        self.fctx.vec_drop_safe.clear();
        self.fctx.boxed_vec_payload_safe.clear();
        self.fctx.str_drop_safe.clear();
        self.fctx.aggregate_drop_safe.clear();
        self.fctx.aggregate_str_drop_safe.clear();
        self.fctx.agg_overwrite_safe.clear();
        self.fctx.aggregate_vec_owned.clear();
        self.fctx.aggregate_str_vec_safe.clear();
        self.fctx.enum_tree.clear();
        self.fctx.struct_payload_tree.clear();
        self.fctx.elem_drop_safe.clear();
        self.fctx.str_rebind_safe.clear();
        self.fctx.str_rebind_exit.clear();
        self.fctx.unused_locals.clear();
        self.fctx.vec_rebind_safe.clear();
        self.fctx.vec_rebind_alloca.clear();
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
            safe_boxed_vec: std::mem::take(&mut self.fctx.boxed_vec_payload_safe),
            safe_str: std::mem::take(&mut self.fctx.str_drop_safe),
            safe_aggregate: std::mem::take(&mut self.fctx.aggregate_drop_safe),
            safe_aggregate_str: std::mem::take(&mut self.fctx.aggregate_str_drop_safe),
            safe_agg_overwrite: std::mem::take(&mut self.fctx.agg_overwrite_safe),
            safe_aggregate_vec_owned: std::mem::take(&mut self.fctx.aggregate_vec_owned),
            safe_aggregate_str_vec: std::mem::take(&mut self.fctx.aggregate_str_vec_safe),
            safe_enum_tree: std::mem::take(&mut self.fctx.enum_tree),
            safe_struct_payload: std::mem::take(&mut self.fctx.struct_payload_tree),
            safe_elem: std::mem::take(&mut self.fctx.elem_drop_safe),
            safe_str_rebind: std::mem::take(&mut self.fctx.str_rebind_safe),
            safe_str_rebind_exit: std::mem::take(&mut self.fctx.str_rebind_exit),
            unused: std::mem::take(&mut self.fctx.unused_locals),
            safe_vec_rebind: std::mem::take(&mut self.fctx.vec_rebind_safe),
            str_rebind_state: std::mem::take(&mut self.fctx.str_rebind_state),
            vec_rebind_alloca: std::mem::take(&mut self.fctx.vec_rebind_alloca),
            by_value_vec_params: std::mem::take(&mut self.fctx.by_value_vec_params),
            entry_prelude: std::mem::take(&mut self.local.entry_prelude),
        }
    }

    pub(crate) fn resume_vec_drops(&mut self, saved: VecDropScope) {
        self.fctx.vec_drops = saved.drops;
        self.fctx.vec_drop_safe = saved.safe;
        self.fctx.boxed_vec_payload_safe = saved.safe_boxed_vec;
        self.fctx.str_drop_safe = saved.safe_str;
        self.fctx.aggregate_drop_safe = saved.safe_aggregate;
        self.fctx.aggregate_str_drop_safe = saved.safe_aggregate_str;
        self.fctx.agg_overwrite_safe = saved.safe_agg_overwrite;
        self.fctx.aggregate_vec_owned = saved.safe_aggregate_vec_owned;
        self.fctx.aggregate_str_vec_safe = saved.safe_aggregate_str_vec;
        self.fctx.enum_tree = saved.safe_enum_tree;
        self.fctx.struct_payload_tree = saved.safe_struct_payload;
        self.fctx.elem_drop_safe = saved.safe_elem;
        self.fctx.str_rebind_safe = saved.safe_str_rebind;
        self.fctx.str_rebind_exit = saved.safe_str_rebind_exit;
        self.fctx.unused_locals = saved.unused;
        self.fctx.vec_rebind_safe = saved.safe_vec_rebind;
        self.fctx.str_rebind_state = saved.str_rebind_state;
        self.fctx.vec_rebind_alloca = saved.vec_rebind_alloca;
        self.fctx.by_value_vec_params = saved.by_value_vec_params;
        self.local.entry_prelude = saved.entry_prelude;
    }

    /// The element type when `value` is a `remove`/`swap_remove`/`pop` call on
    /// a container whose element tag survived the prepass (so every element is
    /// owned). `Some("Vec[...]")` means the erased Option payload is a boxed
    /// `%struct.Vec` header.
    pub(crate) fn boxed_removal_vec_payload(&self, value: &Expr) -> Option<String> {
        let (func, _args) = match value {
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => (func, args),
            Expr::Paren(inner, _) => return self.boxed_removal_vec_payload(inner),
            _ => return None,
        };
        let Expr::Field(recv, method, _) = func.as_ref() else {
            return None;
        };
        if !matches!(method.name.as_str(), "remove" | "swap_remove" | "pop") {
            return None;
        }
        let Expr::Ident(id) = recv.as_ref() else {
            return None;
        };
        let (elem_ty, _) = self.fctx.elem_drop_safe.get(&id.name)?.clone();
        // Vec-of-Vec only: the Option payload is a boxed %struct.Vec header
        // (element size 32). Str/struct payloads are a separate slice.
        if elem_ty.starts_with("Vec[") {
            Some(elem_ty)
        } else {
            None
        }
    }

    /// Free the boxed element of a provably-DEAD `remove`/`pop` result: its
    /// Vec data buffer and the malloc'd element header. The container's exit
    /// drop only walks the current len, so the removed slot is never freed a
    /// second time (m375 p_remove).
    pub(crate) fn emit_dead_boxed_vec_payload(&mut self, option_alloca: &str) {
        let tag_gep = self.fresh_tmp();
        let tag = self.fresh_tmp();
        let is_some = self.fresh_tmp();
        let free_blk = self.fresh_block("dead_remove_free");
        let after_blk = self.fresh_block("dead_remove_after");
        self.emitln(&format!(
            "  {tag_gep} = getelementptr %struct.Option, %struct.Option* {option_alloca}, i32 0, i32 0"
        ));
        self.emitln(&format!("  {tag} = load i64, i64* {tag_gep}"));
        self.emitln(&format!("  {is_some} = icmp ne i64 {tag}, 0"));
        self.emitln(&format!(
            "  br i1 {is_some}, label %{free_blk}, label %{after_blk}"
        ));
        self.emitln(&format!("\n{free_blk}:"));
        let pay_gep = self.fresh_tmp();
        let pay = self.fresh_tmp();
        let boxed = self.fresh_tmp();
        self.emitln(&format!(
            "  {pay_gep} = getelementptr %struct.Option, %struct.Option* {option_alloca}, i32 0, i32 1"
        ));
        self.emitln(&format!("  {pay} = load i64, i64* {pay_gep}"));
        self.emitln(&format!("  {boxed} = inttoptr i64 {pay} to %struct.Vec*"));
        let dgp = self.fresh_tmp();
        let dp = self.fresh_tmp();
        self.emitln(&format!(
            "  {dgp} = getelementptr %struct.Vec, %struct.Vec* {boxed}, i32 0, i32 0"
        ));
        self.emitln(&format!("  {dp} = load i8*, i8** {dgp}"));
        self.emitln(&format!("  call void @xiom_guard_free(i8* {dp})"));
        let box_i8 = self.fresh_tmp();
        self.emitln(&format!("  {box_i8} = bitcast %struct.Vec* {boxed} to i8*"));
        self.emitln(&format!("  call void @xiom_guard_free(i8* {box_i8})"));
        self.emitln(&format!("  br label %{after_blk}"));
        self.emitln(&format!("\n{after_blk}:"));
    }

    /// Named result of `collect_owned_vec_locals` for this function, consulted
    /// before emitting a binding's drop glue.
    pub(crate) fn unused_local(&self, name: &str) -> bool {
        self.fctx.unused_locals.contains(name)
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
        // Vec overwrite-drop: remember the header slot; the normal Vec drop
        // below frees the FINAL header data at exit, and each assignment
        // frees the CURRENT header data before overwriting it.
        if self.fctx.vec_rebind_safe.contains(name) {
            self.fctx
                .vec_rebind_alloca
                .insert(name.to_string(), alloca.to_string());
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
            || !is_fresh_str_value(init, &self.fresh_str_returns)
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
        self.emit_free_str_ptr(&ptr);
        self.emitln(&format!("  br label %{after_block}"));
        self.emitln(&format!("\n{after_block}:"));
    }

    /// Free an owned Str pointer, skipping null and EMPTY strings. An empty
    /// Str may be the shared `""` literal (a contracted callee such as
    /// `json_escape` returns it for empty input); freeing a literal is
    /// invalid. Skipping empty strings is leak-only and matches the
    /// enum-tree helper's existing guard. `ptr` is an SSA i8* value, not a
    /// slot; the caller's code continues in the emitted `after` block.
    pub(crate) fn emit_free_str_ptr(&mut self, ptr: &str) {
        let is_null = self.fresh_tmp();
        let chk = self.fresh_block("strfree_chk");
        let do_free = self.fresh_block("strfree_do");
        let after = self.fresh_block("strfree_after");
        self.emitln(&format!("  {is_null} = icmp eq i8* {ptr}, null"));
        self.emitln(&format!("  br i1 {is_null}, label %{after}, label %{chk}"));
        self.emitln(&format!("\n{chk}:"));
        let len = self.fresh_tmp();
        let empty = self.fresh_tmp();
        self.emitln(&format!("  {len} = call i64 @xiom_str_len(i8* {ptr})"));
        self.emitln(&format!("  {empty} = icmp eq i64 {len}, 0"));
        self.emitln(&format!("  br i1 {empty}, label %{after}, label %{do_free}"));
        self.emitln(&format!("\n{do_free}:"));
        self.emitln(&format!("  call void @xiom_guard_free(i8* {ptr})"));
        self.emitln(&format!("  br label %{after}"));
        self.emitln(&format!("\n{after}:"));
    }

    /// At a fresh Str binding, start tracking its owned buffer so the first
    /// reassignment can free it. Literal/alias initializers start untracked
    /// (the flag stays false) and arm on the first fresh assignment.
    pub(crate) fn note_str_rebind_binding(&mut self, name: &str, init: &Expr, alloca: &str) {
        if !self.fctx.str_rebind_safe.contains(name)
            || !is_fresh_str_value(init, &self.fresh_str_returns)
        {
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
        let fresh = is_fresh_str_value(value, &self.fresh_str_returns)
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

    /// Before storing a fresh/owned Vec into a rebind local, free the data
    /// pointer that the local's CURRENT header owns. Reading the header (not
    /// a captured slot) is essential: `push` may have realloc'd and freed the
    /// old buffer since the binding; a stale captured pointer is a
    /// double-free (orbitdb hybrid crash 0xC0000374, REL-20261102-1605).
    pub(crate) fn emit_vec_rebind_free_old(&mut self, name: &str) {
        if !self.fctx.vec_rebind_safe.contains(name) {
            return;
        }
        let Some(alloca) = self.fctx.vec_rebind_alloca.get(name).cloned() else {
            return;
        };
        let dgp = self.fresh_tmp();
        let dp = self.fresh_tmp();
        self.emitln(&format!(
            "  {dgp} = getelementptr %struct.Vec, %struct.Vec* {alloca}, i32 0, i32 0"
        ));
        self.emitln(&format!("  {dp} = load i8*, i8** {dgp}"));
        self.emitln(&format!("  call void @xiom_guard_free(i8* {dp})"));
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
        for (idx, field) in meta.fields.iter().enumerate() {
            if self.field_llvm_type(&type_name, idx) != "%struct.Vec" {
                continue;
            }
            // Contracted Map/Set: this Vec[Str] field's elements are owned by
            // the aggregate (the #[consumes_str] call moved them in). Free
            // each string before the field buffer.
            let elem_is_str = self.fctx.aggregate_str_vec_safe.contains(name)
                && resolve_alias_name(&self.types, &field.1) == "Vec[Str]";
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
                elem_is_str,
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

    /// 2117: a struct-literal local whose aggregate slot must be hoisted so
    /// the field GEPs registered for its drop dominate every scope exit.
    pub(crate) fn agg_overwrite_candidate(&self, name: &str, llvm_ty: &str) -> bool {
        self.fctx.agg_overwrite_safe.contains(name) && llvm_ty.starts_with("%struct.")
    }

    /// Resolve the type metadata for a `%struct.X` LLVM type (exact key, then
    /// a unique dotted-leaf match).
    fn struct_meta_for(&self, type_name: &str) -> Option<crate::context::TypeMeta> {
        self.types.type_meta.get(&type_name.to_string()).or_else(|| {
            self.types
                .type_meta
                .entries()
                .into_iter()
                .find(|(k, _)| k.ends_with(&format!(".{type_name}")))
                .map(|(_, meta)| meta)
        })
    }

    /// 2117: register the scope-exit drop for a literal-bound aggregate that
    /// is NOT already covered by the existing `aggregate_drop_safe` machinery
    /// (that path only sees ctor-valued fields). Only the leaves proven freshly
    /// owned at the binding are registered; a borrowed field is never freed.
    pub(crate) fn register_agg_overwrite_drops(
        &mut self,
        name: &str,
        alloca: &str,
        llvm_ty: &str,
    ) {
        if !self.agg_overwrite_candidate(name, llvm_ty) {
            return;
        }
        // The existing aggregate path already registered this binding (it
        // covers all Vec fields + fresh Str leaves). Never duplicate a drop.
        if self.fctx.aggregate_drop_safe.contains(name) {
            return;
        }
        let type_name = llvm_ty.trim_start_matches("%struct.").to_string();
        let Some(meta) = self.struct_meta_for(&type_name) else {
            return;
        };
        let vec_fields = self
            .fctx
            .aggregate_vec_owned
            .get(name)
            .cloned()
            .unwrap_or_default();
        let str_fields = self
            .fctx
            .aggregate_str_drop_safe
            .get(name)
            .cloned()
            .unwrap_or_default();
        for (idx, field) in meta.fields.iter().enumerate() {
            let fty = self.field_llvm_type(&type_name, idx);
            if fty == "%struct.Vec" && vec_fields.iter().any(|f| f == &field.0) {
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
                    elem_is_str: false,
                });
                self.emitln(&format!("  store i1 true, i1* {flag}"));
            } else if fty == "i8*" && str_fields.iter().any(|f| f == &field.0) {
                let field_ptr = self.fresh_tmp();
                self.local.entry_prelude.push(format!(
                    "  {field_ptr} = getelementptr {llvm_ty}, {llvm_ty}* {alloca}, i32 0, i32 {idx}"
                ));
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

    /// 2117: before `d.<field> = fresh`, free the OLD owned field data. The
    /// field pointer is the store destination (already computed); the RHS has
    /// been materialized. Only a leaf proven owned at the literal binding and
    /// a self-contained fresh RHS qualify.
    pub(crate) fn emit_agg_field_free_old(
        &mut self,
        agg: &str,
        field: &str,
        field_ptr: &str,
        field_llvm_ty: &str,
        value: &Expr,
    ) {
        if !self.fctx.agg_overwrite_safe.contains(agg) {
            return;
        }
        if field_llvm_ty == "%struct.Vec" {
            if !self
                .fctx
                .aggregate_vec_owned
                .get(agg)
                .map_or(false, |fs| fs.iter().any(|f| f == field))
                || !(is_fresh_vec_init(value)
                    || is_fresh_vec_call(value, &self.fresh_vec_returns))
            {
                return;
            }
            let dgp = self.fresh_tmp();
            let dp = self.fresh_tmp();
            self.emitln(&format!(
                "  {dgp} = getelementptr %struct.Vec, %struct.Vec* {field_ptr}, i32 0, i32 0"
            ));
            self.emitln(&format!("  {dp} = load i8*, i8** {dgp}"));
            self.emitln(&format!("  call void @xiom_guard_free(i8* {dp})"));
        } else if field_llvm_ty == "i8*" {
            if !self
                .fctx
                .aggregate_str_drop_safe
                .get(agg)
                .map_or(false, |fs| fs.iter().any(|f| f == field))
                || !is_fresh_str_value(value, &self.fresh_str_returns)
            {
                return;
            }
            let dp = self.fresh_tmp();
            self.emitln(&format!("  {dp} = load i8*, i8** {field_ptr}"));
            self.emit_free_str_ptr(&dp);
        }
    }

    /// 2117: before `d = fresh_aggregate_call`, free the OLD aggregate's owned
    /// Vec/Str fields. Only the leaves proven owned at the literal binding are
    /// freed; a borrowed/projected field is left untouched.
    pub(crate) fn emit_agg_overwrite_free_old(
        &mut self,
        name: &str,
        alloca: &str,
        llvm_ty: &str,
        value: &Expr,
    ) {
        if !self.fctx.agg_overwrite_safe.contains(name)
            || !is_fresh_aggregate_call(value, &self.fresh_aggregate_returns)
            || !llvm_ty.starts_with("%struct.")
        {
            return;
        }
        let type_name = llvm_ty.trim_start_matches("%struct.").to_string();
        let Some(meta) = self.struct_meta_for(&type_name) else {
            return;
        };
        let vec_fields = self
            .fctx
            .aggregate_vec_owned
            .get(name)
            .cloned()
            .unwrap_or_default();
        let str_fields = self
            .fctx
            .aggregate_str_drop_safe
            .get(name)
            .cloned()
            .unwrap_or_default();
        for (idx, field) in meta.fields.iter().enumerate() {
            let fty = self.field_llvm_type(&type_name, idx);
            if fty == "%struct.Vec" && vec_fields.iter().any(|f| f == &field.0) {
                let field_ptr = self.fresh_tmp();
                self.emitln(&format!(
                    "  {field_ptr} = getelementptr {llvm_ty}, {llvm_ty}* {alloca}, i32 0, i32 {idx}"
                ));
                let dgp = self.fresh_tmp();
                let dp = self.fresh_tmp();
                self.emitln(&format!(
                    "  {dgp} = getelementptr %struct.Vec, %struct.Vec* {field_ptr}, i32 0, i32 0"
                ));
                self.emitln(&format!("  {dp} = load i8*, i8** {dgp}"));
                self.emitln(&format!("  call void @xiom_guard_free(i8* {dp})"));
            } else if fty == "i8*" && str_fields.iter().any(|f| f == &field.0) {
                let field_ptr = self.fresh_tmp();
                self.emitln(&format!(
                    "  {field_ptr} = getelementptr {llvm_ty}, {llvm_ty}* {alloca}, i32 0, i32 {idx}"
                ));
                let dp = self.fresh_tmp();
                self.emitln(&format!("  {dp} = load i8*, i8** {field_ptr}"));
                self.emit_free_str_ptr(&dp);
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

    /// Register a match-arm binding of a BOXED Vec removal payload
    /// (`pop`/`remove`/`swap_remove` on an owned `Vec[Vec[T]]`). `header_ptr`
    /// is the `%struct.Vec*` unboxed from the Option payload at the binding.
    /// The pointer is captured into a hoisted slot so the arm/loop/return drop
    /// never loads a non-dominating arm value. The drop frees the element's
    /// data buffer and then the malloc'd header box.
    pub(crate) fn register_boxed_vec_binding(&mut self, name: &str, header_ptr: &str) {
        if !self.fctx.vec_drop_safe.contains(name)
            || !self.fctx.boxed_vec_payload_safe.contains(name)
        {
            return;
        }
        let header_slot = self.fresh_tmp();
        self.local
            .hoisted_allocas
            .push((header_slot.clone(), "%struct.Vec*".to_string()));
        self.local
            .entry_prelude
            .push(format!("  store %struct.Vec* null, %struct.Vec** {header_slot}"));
        self.emitln(&format!(
            "  store %struct.Vec* {header_ptr}, %struct.Vec** {header_slot}"
        ));
        let flag = self.fresh_tmp();
        self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
        self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
        self.fctx.vec_drops.push(VecDrop::BoxedVec {
            header_alloca: header_slot,
            flag: flag.clone(),
        });
        self.emitln(&format!("  store i1 true, i1* {flag}"));
    }

    /// Register a local bound directly from a contracted
    /// `#[owns_return]` Result/Option<named enum> call (json_parse). The
    /// Result payload's tree is reclaimed at scope exit by a recursive helper
    /// generated from the enum metadata. Nothing is registered unless the
    /// prepass proved this exact provenance and the local is private.
    pub(crate) fn register_enum_tree_drop(&mut self, name: &str, alloca: &str, llvm_ty: &str) {
        let Some(enum_name) = self.fctx.enum_tree.get(name).cloned() else {
            return;
        };
        let Some(result_type) = llvm_ty.strip_prefix("%struct.") else {
            return;
        };
        if !(result_type.starts_with("Result__") || result_type.starts_with("Option__")) {
            return;
        }
        let Some(enum_key) = self.resolve_enum_key(&enum_name) else {
            return;
        };
        let Some(meta) = self.types.type_meta.get(&result_type.to_string()) else {
            return;
        };
        let Some(payload_idx) = meta.fields.iter().position(|(n, _)| n == "value") else {
            return;
        };
        let Some(disc_idx) = meta
            .fields
            .iter()
            .position(|(n, _)| n == "discriminant")
        else {
            return;
        };
        let enum_llvm_ty = format!("%struct.{enum_key}");
        let helper = enum_tree_drop_helper_name(&enum_key);
        let flag = self.fresh_tmp();
        self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
        self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
        // The binding alloca may be declared inside a nested block, but the
        // drop is emitted at function exit. Keep an entry-block slot that
        // holds the binding pointer; the drop loads it only when the flag is
        // armed, so dominance holds without moving the value alloca.
        let ptr_slot = self.fresh_tmp();
        self.local
            .hoisted_allocas
            .push((ptr_slot.clone(), format!("{llvm_ty}*")));
        self.local
            .entry_prelude
            .push(format!("  store {llvm_ty}* null, {llvm_ty}** {ptr_slot}"));
        self.fctx.vec_drops.push(VecDrop::EnumTree {
            ptr_slot: ptr_slot.clone(),
            flag: flag.clone(),
            result_llvm_ty: llvm_ty.to_string(),
            disc_idx: disc_idx as u32,
            payload_idx: payload_idx as u32,
            enum_llvm_ty,
            helper,
        });
        self.emitln(&format!(
            "  store {llvm_ty}* {alloca}, {llvm_ty}** {ptr_slot}"
        ));
        self.emitln(&format!("  store i1 true, i1* {flag}"));
    }

    /// Register a local bound directly from a contracted
    /// `#[owns_return]` / `#[consumes_agg]` Result/Option<named struct> call.
    /// The Some/Ok payload struct is reclaimed at scope exit by a
    /// metadata-generated recursive field-drop helper. Nothing is registered
    /// unless the prepass proved this exact provenance AND the binding LLVM
    /// type is a CONCRETE Option__/Result__ wrapper whose `value` field is an
    /// inline named struct. A boxed/erased payload or any other shape stays
    /// leak-only (never a wrong-layout free).
    pub(crate) fn register_struct_payload_drop(
        &mut self,
        name: &str,
        alloca: &str,
        llvm_ty: &str,
    ) {
        let Some((_struct_name, free_strings)) =
            self.fctx.struct_payload_tree.get(name).cloned()
        else {
            return;
        };
        let Some(result_type) = llvm_ty.strip_prefix("%struct.") else {
            return;
        };
        if !(result_type.starts_with("Result__") || result_type.starts_with("Option__")) {
            return;
        }
        let Some(meta) = self.types.type_meta.get(&result_type.to_string()) else {
            return;
        };
        let Some(payload_idx) = meta.fields.iter().position(|(n, _)| n == "value") else {
            return;
        };
        let Some(disc_idx) = meta
            .fields
            .iter()
            .position(|(n, _)| n == "discriminant")
        else {
            return;
        };
        let payload_llvm_ty = self.field_llvm_type(result_type, payload_idx);
        let Some(payload_key) = payload_llvm_ty.strip_prefix("%struct.") else {
            return;
        };
        // Only an INLINE named struct payload. Builtin containers, enums,
        // nested Option/Result, and empty sentinels have their own provenance
        // (or none); leave them leak-only rather than guessing a layout.
        if matches!(
            payload_key,
            "Vec" | "Map" | "Set" | "Option" | "Result" | "Str" | "Slice"
        ) || payload_key.starts_with("Option__")
            || payload_key.starts_with("Result__")
            || payload_key.starts_with("_Anon__")
            || self.types.enum_variants.contains_key(&payload_key.to_string())
            || !self.types.type_meta.contains_key(&payload_key.to_string())
        {
            return;
        }
        let helper = struct_payload_helper_name(payload_key, free_strings);
        // Record the EXACT payload key for module-end helper emission, so the
        // generated signature and this call site can never disagree.
        self.struct_payload_helpers
            .insert((payload_key.to_string(), free_strings));
        let flag = self.fresh_tmp();
        self.local.hoisted_allocas.push((flag.clone(), "i1".to_string()));
        self.local.entry_prelude.push(format!("  store i1 false, i1* {flag}"));
        // The binding alloca may be declared inside a nested block, but the
        // drop can be emitted at function exit. Keep an entry-block slot that
        // holds the binding pointer; the drop loads it only when the flag is
        // armed, so dominance holds without moving the value alloca.
        let ptr_slot = self.fresh_tmp();
        self.local
            .hoisted_allocas
            .push((ptr_slot.clone(), format!("{llvm_ty}*")));
        self.local
            .entry_prelude
            .push(format!("  store {llvm_ty}* null, {llvm_ty}** {ptr_slot}"));
        self.fctx.vec_drops.push(VecDrop::StructPayload {
            ptr_slot: ptr_slot.clone(),
            flag: flag.clone(),
            result_llvm_ty: llvm_ty.to_string(),
            disc_idx: disc_idx as u32,
            payload_idx: payload_idx as u32,
            payload_llvm_ty,
            helper,
        });
        self.emitln(&format!(
            "  store {llvm_ty}* {alloca}, {llvm_ty}** {ptr_slot}"
        ));
        self.emitln(&format!("  store i1 true, i1* {flag}"));
    }

    /// Resolve a possibly-bare enum name to its registry key (exact first,
    /// then a unique leaf match). Returns None when ambiguous or unknown.
    pub(crate) fn resolve_enum_key(&self, name: &str) -> Option<String> {
        if self.types.enum_variants.contains_key(&name.to_string()) {
            return Some(name.to_string());
        }
        let leaf = name.rsplit('.').next().unwrap_or(name);
        let mut found: Option<String> = None;
        for key in self.types.enum_variants.keys() {
            let key_leaf = key.rsplit('.').next().unwrap_or(&key);
            if key_leaf == leaf {
                if found.is_some() {
                    return None;
                }
                found = Some(key);
            }
        }
        found
    }

    /// Emit one recursive drop helper per contracted enum tree type. Called at
    /// module end (after every type definition), once per enum, so parallel
    /// per-function emitters cannot duplicate the definition.
    pub(crate) fn emit_enum_tree_drop_helpers(&mut self) {
        // R3: parse-owned trees free containers AND freshly allocated Str
        // payloads.
        let mut names: Vec<String> = self.fresh_enum_tree_returns.values().cloned().collect();
        names.sort();
        names.dedup();
        for name in names {
            let Some(key) = self.resolve_enum_key(&name) else {
                continue;
            };
            if let Some(def) = self.build_enum_tree_drop_helper(&key, true) {
                self.output.push('\n');
                self.output.push_str(&def);
            }
        }
        // R4: builder/stringify-owned trees free CONTAINERS only. Literal keys
        // and borrowed values (`json_string("wave-1704")` stores its argument)
        // must never be freed.
        let mut containers: Vec<String> =
            self.consumes_agg_returns.values().cloned().collect();
        containers.sort();
        containers.dedup();
        for name in containers {
            let Some(key) = self.resolve_enum_key(&name) else {
                continue;
            };
            if let Some(def) = self.build_enum_tree_drop_helper(&key, false) {
                self.output.push('\n');
                self.output.push_str(&def);
            }
        }
    }

    /// Emit one recursive field-drop helper per struct payload type requested
    /// by `register_struct_payload_drop`. Runs at module end (all type
    /// definitions + runtime declares precede it), once per (type,
    /// free_strings) pair. Reachable nested struct/tuple element types are
    /// pulled in transitively.
    pub(crate) fn emit_struct_payload_drop_helpers(&mut self) {
        let mut built: HashMap<(String, bool), String> = HashMap::new();
        let mut stack: Vec<(String, bool)> = self.struct_payload_helpers.iter().cloned().collect();
        while let Some(key) = stack.pop() {
            if built.contains_key(&key) {
                continue;
            }
            let Some((def, deps)) = self.build_struct_payload_drop_helper(&key.0, key.1) else {
                continue;
            };
            for dep in deps {
                if !built.contains_key(&dep) {
                    stack.push(dep);
                }
            }
            built.insert(key, def);
        }
        let mut keys: Vec<(String, bool)> = built.keys().cloned().collect();
        keys.sort();
        for key in keys {
            if let Some(def) = built.get(&key) {
                self.output.push('\n');
                self.output.push_str(def);
            }
        }
    }

    /// Build the recursive field-drop helper for one named struct/tuple type.
    /// Returns the definition text plus the nested (type, free_strings)
    /// helpers it calls. The helper frees exactly the owned leaves for the
    /// provenance: `free_strings=true` (R3) frees Str leaves and container
    /// contents; `false` (R4) frees container buffers only.
    fn build_struct_payload_drop_helper(
        &self,
        struct_key: &str,
        free_strings: bool,
    ) -> Option<(String, Vec<(String, bool)>)> {
        let meta = self.struct_meta_for(struct_key)?;
        let llvm_ty = format!("%struct.{struct_key}");
        let helper = struct_payload_helper_name(struct_key, free_strings);
        let mut c = 0usize;
        let mut deps: Vec<(String, bool)> = Vec::new();
        let mut out = String::new();
        out.push_str(&format!(
            "define internal void @{helper}({llvm_ty}* %node) {{\nentry:\n"
        ));
        for (idx, (_fname, fty)) in meta.fields.iter().enumerate() {
            let fllvm = self.field_llvm_type(struct_key, idx);
            let field_ptr = format!("%sf{}", c);
            c += 1;
            out.push_str(&format!(
                "  {field_ptr} = getelementptr {llvm_ty}, {llvm_ty}* %node, i32 0, i32 {idx}\n"
            ));
            self.emit_type_value_drop(
                &mut out,
                &mut c,
                fty,
                &fllvm,
                &field_ptr,
                free_strings,
                &mut deps,
                0,
            );
        }
        out.push_str("  ret void\n}\n");
        Some((out, deps))
    }

    /// Emit code freeing the owned parts of the value at `ptr` (typed
    /// `{llvm_ty}*`). `xiom_ty` is the declared XIOM spelling (keeps generic
    /// args for Vec/Map elements). Provenance-strict: borrowed/literal Str
    /// leaves are never freed; unknown/enum/nested-Option layouts stay
    /// leak-only.
    fn emit_type_value_drop(
        &self,
        out: &mut String,
        c: &mut usize,
        xiom_ty: &str,
        llvm_ty: &str,
        ptr: &str,
        free_strings: bool,
        deps: &mut Vec<(String, bool)>,
        depth: usize,
    ) {
        if depth > 6 {
            return;
        }
        let resolved = resolve_alias_name(&self.types, xiom_ty);
        let leaf = resolved.rsplit('.').next().unwrap_or(&resolved);
        if leaf == "Str" {
            if !free_strings {
                return;
            }
            if llvm_ty == "i8*" {
                let raw = format!("%sr{}", *c);
                *c += 1;
                out.push_str(&format!("  {raw} = load i8*, i8** {ptr}\n"));
                Self::emit_str_free_value_inline(out, c, &raw);
            } else if llvm_ty == "i64" {
                let raw = format!("%sr{}", *c);
                *c += 1;
                let p = format!("%sp{}", *c);
                *c += 1;
                out.push_str(&format!("  {raw} = load i64, i64* {ptr}\n"));
                out.push_str(&format!("  {p} = inttoptr i64 {raw} to i8*\n"));
                Self::emit_str_free_value_inline(out, c, &p);
            }
            return;
        }
        // Option/Result fields are a separate (boxed/inline) shape; keep them
        // leak-only rather than guess the payload layout here.
        if resolved.starts_with("Option[") || resolved.starts_with("Result[") {
            return;
        }
        if let Some(inner) = vec_inner_type(&resolved) {
            if llvm_ty != "%struct.Vec" {
                return;
            }
            self.emit_vec_value_drop(
                out,
                c,
                inner,
                ptr,
                free_strings,
                deps,
                depth + 1,
            );
            return;
        }
        if let Some((k, v)) = map_kv_types(&resolved) {
            if llvm_ty != "%struct.Map" {
                return;
            }
            self.emit_map_value_drop(out, c, k, v, ptr, free_strings, deps, depth + 1);
            return;
        }
        // An inline named struct/tuple: recurse through its generated helper.
        if let Some(key) = llvm_ty.strip_prefix("%struct.") {
            if matches!(key, "Vec" | "Map" | "Set" | "Option" | "Result" | "Str" | "Slice")
                || key.starts_with("Option__")
                || key.starts_with("Result__")
                || key.starts_with("_Anon__")
                || self.types.enum_variants.contains_key(&key.to_string())
                || !self.types.type_meta.contains_key(&key.to_string())
            {
                return;
            }
            // Only emit a call when the nested type actually owns something.
            if !self.type_value_needs_drop(xiom_ty, llvm_ty, free_strings, depth + 1) {
                return;
            }
            deps.push((key.to_string(), free_strings));
            let helper = struct_payload_helper_name(key, free_strings);
            out.push_str(&format!("  call void @{helper}({llvm_ty}* {ptr})\n"));
        }
    }

    /// Does a value of this type own anything the helper would free? Used to
    /// avoid emitting element loops / recursive calls for scalar-only types.
    fn type_value_needs_drop(
        &self,
        xiom_ty: &str,
        llvm_ty: &str,
        free_strings: bool,
        depth: usize,
    ) -> bool {
        if depth > 6 {
            return false;
        }
        let resolved = resolve_alias_name(&self.types, xiom_ty);
        let leaf = resolved.rsplit('.').next().unwrap_or(&resolved);
        if leaf == "Str" {
            return free_strings;
        }
        if resolved.starts_with("Option[") || resolved.starts_with("Result[") {
            return false;
        }
        if vec_inner_type(&resolved).is_some() {
            return llvm_ty == "%struct.Vec";
        }
        if map_kv_types(&resolved).is_some() {
            return llvm_ty == "%struct.Map";
        }
        let Some(key) = llvm_ty.strip_prefix("%struct.") else {
            return false;
        };
        if matches!(key, "Vec" | "Map" | "Set" | "Option" | "Result" | "Str" | "Slice")
            || key.starts_with("Option__")
            || key.starts_with("Result__")
            || key.starts_with("_Anon__")
            || self.types.enum_variants.contains_key(&key.to_string())
        {
            return false;
        }
        let Some(meta) = self.struct_meta_for(key) else {
            return false;
        };
        for (idx, (_fname, fty)) in meta.fields.iter().enumerate() {
            let fllvm = self.field_llvm_type(key, idx);
            if self.type_value_needs_drop(fty, &fllvm, free_strings, depth + 1) {
                return true;
            }
        }
        false
    }

    /// Free a Vec value's owned element payloads (when proven) and then its
    /// data buffer. `vec_ptr` is a `%struct.Vec*`.
    fn emit_vec_value_drop(
        &self,
        out: &mut String,
        c: &mut usize,
        elem_xiom: &str,
        vec_ptr: &str,
        free_strings: bool,
        deps: &mut Vec<(String, bool)>,
        depth: usize,
    ) {
        let vec_meta = self.types.type_meta.get(&"Vec".to_string());
        let field_idx = |name: &str, fallback: u32| -> u32 {
            vec_meta
                .as_ref()
                .and_then(|m| m.fields.iter().position(|(n, _)| n == name))
                .map(|i| i as u32)
                .unwrap_or(fallback)
        };
        let data_idx = field_idx("data", 0);
        let len_idx = field_idx("len", 1);
        let esz_idx = field_idx("elem_size", 3);
        let data_slot = format!("%vd{}", *c);
        *c += 1;
        let data = format!("%vdp{}", *c);
        *c += 1;
        out.push_str(&format!(
            "  {data_slot} = getelementptr %struct.Vec, %struct.Vec* {vec_ptr}, i32 0, i32 {data_idx}\n"
        ));
        out.push_str(&format!("  {data} = load i8*, i8** {data_slot}\n"));
        let elem_llvm = self
            .llvm_type_for(elem_xiom)
            .unwrap_or_else(|_| "i64".to_string());
        if self.type_value_needs_drop(elem_xiom, &elem_llvm, free_strings, depth) {
            let len_slot = format!("%vl{}", *c);
            *c += 1;
            let len = format!("%vln{}", *c);
            *c += 1;
            let esz_slot = format!("%ve{}", *c);
            *c += 1;
            let esz = format!("%vev{}", *c);
            *c += 1;
            out.push_str(&format!(
                "  {len_slot} = getelementptr %struct.Vec, %struct.Vec* {vec_ptr}, i32 0, i32 {len_idx}\n"
            ));
            out.push_str(&format!("  {len} = load i64, i64* {len_slot}\n"));
            out.push_str(&format!(
                "  {esz_slot} = getelementptr %struct.Vec, %struct.Vec* {vec_ptr}, i32 0, i32 {esz_idx}\n"
            ));
            out.push_str(&format!("  {esz} = load i64, i64* {esz_slot}\n"));
            let idx_slot = format!("%vi{}", *c);
            *c += 1;
            let cond = format!("vd_cond{}", *c);
            *c += 1;
            let body = format!("vd_body{}", *c);
            *c += 1;
            let done_loop = format!("vd_done{}", *c);
            *c += 1;
            let esz_ok = format!("%veo{}", *c);
            *c += 1;
            out.push_str(&format!("  {idx_slot} = alloca i64\n"));
            out.push_str(&format!("  store i64 0, i64* {idx_slot}\n"));
            out.push_str(&format!("  {esz_ok} = icmp ne i64 {esz}, 0\n"));
            out.push_str(&format!(
                "  br i1 {esz_ok}, label %{cond}, label %{done_loop}\n"
            ));
            out.push_str(&format!("\n{cond}:\n"));
            let i = format!("%vii{}", *c);
            *c += 1;
            let more = format!("%vim{}", *c);
            *c += 1;
            out.push_str(&format!("  {i} = load i64, i64* {idx_slot}\n"));
            out.push_str(&format!("  {more} = icmp ult i64 {i}, {len}\n"));
            out.push_str(&format!(
                "  br i1 {more}, label %{body}, label %{done_loop}\n"
            ));
            out.push_str(&format!("\n{body}:\n"));
            let off = format!("%vio{}", *c);
            *c += 1;
            let ep = format!("%vip{}", *c);
            *c += 1;
            let epc = format!("%vic{}", *c);
            *c += 1;
            out.push_str(&format!("  {off} = mul i64 {i}, {esz}\n"));
            out.push_str(&format!("  {ep} = getelementptr i8, i8* {data}, i64 {off}\n"));
            out.push_str(&format!("  {epc} = bitcast i8* {ep} to {elem_llvm}*\n"));
            self.emit_type_value_drop(
                out,
                c,
                elem_xiom,
                &elem_llvm,
                &epc,
                free_strings,
                deps,
                depth + 1,
            );
            let next = format!("%vin{}", *c);
            *c += 1;
            out.push_str(&format!("  {next} = add i64 {i}, 1\n"));
            out.push_str(&format!("  store i64 {next}, i64* {idx_slot}\n"));
            out.push_str(&format!("  br label %{cond}\n"));
            out.push_str(&format!("\n{done_loop}:\n"));
        }
        out.push_str(&format!("  call void @xiom_guard_free(i8* {data})\n"));
    }

    /// Free a Map value's keys/values buffers (and, for R3, fresh Str
    /// elements) plus any owned nested payloads. `map_ptr` is `%struct.Map*`.
    fn emit_map_value_drop(
        &self,
        out: &mut String,
        c: &mut usize,
        k_xiom: &str,
        v_xiom: &str,
        map_ptr: &str,
        free_strings: bool,
        deps: &mut Vec<(String, bool)>,
        depth: usize,
    ) {
        let Some(map_meta) = self.types.type_meta.get(&"Map".to_string()) else {
            return;
        };
        let Some(kidx) = map_meta.fields.iter().position(|(n, _)| n == "keys") else {
            return;
        };
        let Some(vidx) = map_meta.fields.iter().position(|(n, _)| n == "values") else {
            return;
        };
        let keys_vec = format!("%mk{}", *c);
        *c += 1;
        let values_vec = format!("%mv{}", *c);
        *c += 1;
        out.push_str(&format!(
            "  {keys_vec} = getelementptr %struct.Map, %struct.Map* {map_ptr}, i32 0, i32 {kidx}\n"
        ));
        out.push_str(&format!(
            "  {values_vec} = getelementptr %struct.Map, %struct.Map* {map_ptr}, i32 0, i32 {vidx}\n"
        ));
        self.emit_vec_value_drop(out, c, k_xiom, &keys_vec, free_strings, deps, depth);
        self.emit_vec_value_drop(out, c, v_xiom, &values_vec, free_strings, deps, depth);
    }

    /// Free an `i8*` SSA value, skipping null and EMPTY strings (an empty Str
    /// may be the shared `""` literal). Mirrors the codegen emitter, but writes
    /// into a helper body where `self.emitln` is not available.
    fn emit_str_free_value_inline(out: &mut String, c: &mut usize, ptr: &str) {
        let is_null = format!("%sn{}", *c);
        *c += 1;
        let chk = format!("sfchk{}", *c);
        *c += 1;
        let do_free = format!("sffree{}", *c);
        *c += 1;
        let after = format!("sfafter{}", *c);
        *c += 1;
        out.push_str(&format!("  {is_null} = icmp eq i8* {ptr}, null\n"));
        out.push_str(&format!(
            "  br i1 {is_null}, label %{after}, label %{chk}\n"
        ));
        out.push_str(&format!("\n{chk}:\n"));
        let len = format!("%sl{}", *c);
        *c += 1;
        let empty = format!("%se{}", *c);
        *c += 1;
        out.push_str(&format!("  {len} = call i64 @xiom_str_len(i8* {ptr})\n"));
        out.push_str(&format!("  {empty} = icmp eq i64 {len}, 0\n"));
        out.push_str(&format!(
            "  br i1 {empty}, label %{after}, label %{do_free}\n"
        ));
        out.push_str(&format!("\n{do_free}:\n"));
        out.push_str(&format!("  call void @xiom_guard_free(i8* {ptr})\n"));
        out.push_str(&format!("  br label %{after}\n"));
        out.push_str(&format!("\n{after}:\n"));
    }

    /// Build a tag-aware recursive drop helper from the enum registry:
    /// discriminant -> variant payload slots. String payloads free their heap
    /// buffer (skipping the empty literal), Vec[Self] recurses elements before
    /// freeing the data buffer, and Map[Str, Self] frees each key string and
    /// recurses each value before freeing both buffers. Every field index and
    /// element stride comes from `type_meta`, not a hardcoded offset.
    /// `free_strings` selects the ownership provenance: `true` for R3
    /// parse-owned trees (String payloads are freshly allocated), `false` for
    /// R4 builder/stringify-owned trees (String payloads are borrowed; only
    /// Vec/Map data buffers are owned).
    fn build_enum_tree_drop_helper(&self, enum_key: &str, free_strings: bool) -> Option<String> {
        let enum_ty = format!("%struct.{enum_key}");
        let variants = self.types.enum_variants.get(&enum_key.to_string())?;
        let meta = self.types.type_meta.get(&enum_key.to_string())?;
        let vtypes = self.types.enum_variant_field_types.get(&enum_key.to_string());
        let enum_leaf = enum_key.rsplit('.').next().unwrap_or(enum_key);
        let helper = if free_strings {
            enum_tree_drop_helper_name(enum_key)
        } else {
            enum_tree_container_drop_helper_name(enum_key)
        };
        let mut c = 0usize;
        let mut out = String::new();
        out.push_str(&format!(
            "define internal void @{helper}({enum_ty}* %node) {{\nentry:\n"
        ));
        out.push_str(&format!(
            "  %hdisc = getelementptr {enum_ty}, {enum_ty}* %node, i32 0, i32 0\n"
        ));
        out.push_str("  %hdiscv = load i64, i64* %hdisc\n");
        let mut cases: Vec<(usize, &str)> = Vec::new();
        for (vi, (variant, decl_fields)) in variants.iter().enumerate() {
            let declared: Vec<String> = vtypes
                .as_ref()
                .and_then(|vts| vts.iter().find(|(v, _)| v == variant))
                .map(|(_, ts)| ts.clone())
                .unwrap_or_else(|| {
                    decl_fields
                        .iter()
                        .map(|f| {
                            meta.fields
                                .iter()
                                .find(|(n, _)| n == f)
                                .map(|(_, t)| t.clone())
                                .unwrap_or_default()
                        })
                        .collect()
                });
            let droppable = decl_fields
                .iter()
                .zip(declared.iter())
                .any(|(f, t)| self.enum_field_needs_drop(f, t, enum_leaf, free_strings));
            if droppable {
                cases.push((vi, variant.as_str()));
            }
        }
        if cases.is_empty() {
            out.push_str("  ret void\n}\n");
            return Some(out);
        }
        out.push_str("  switch i64 %hdiscv, label %hdone [\n");
        for (vi, _) in &cases {
            out.push_str(&format!("    i64 {vi}, label %hcase{vi}\n"));
        }
        out.push_str("  ]\n");
        for (vi, variant) in &cases {
            let decl_fields = &variants[*vi].1;
            let declared: Vec<String> = vtypes
                .as_ref()
                .and_then(|vts| vts.iter().find(|(v, _)| v == variant))
                .map(|(_, ts)| ts.clone())
                .unwrap_or_else(|| {
                    decl_fields
                        .iter()
                        .map(|f| {
                            meta.fields
                                .iter()
                                .find(|(n, _)| n == f)
                                .map(|(_, t)| t.clone())
                                .unwrap_or_default()
                        })
                        .collect()
                });
            out.push_str(&format!("\nhcase{vi}:\n"));
            for (fi, fname) in decl_fields.iter().enumerate() {
                let Some(fidx) = meta.fields.iter().position(|(n, _)| n == fname) else {
                    continue;
                };
                let dty = declared.get(fi).cloned().unwrap_or_default();
                let fllvm = self.field_llvm_type(enum_key, fidx);
                self.emit_enum_field_drop(
                    &mut out,
                    &mut c,
                    &enum_ty,
                    fidx as u32,
                    &dty,
                    &fllvm,
                    enum_leaf,
                    &helper,
                    free_strings,
                );
            }
            out.push_str("  br label %hdone\n");
        }
        out.push_str("\nhdone:\n  ret void\n}\n");
        Some(out)
    }

    fn enum_field_needs_drop(
        &self,
        _fname: &str,
        dty: &str,
        enum_leaf: &str,
        free_strings: bool,
    ) -> bool {
        let leaf = dty.rsplit('.').next().unwrap_or(dty);
        if leaf == "Str" {
            // A builder/stringify-owned String leaf is borrowed (literal key
            // or `json_string` argument). Only parse-owned trees free it.
            return free_strings;
        }
        if let Some(inner) = vec_inner_type(dty) {
            let ileaf = inner.rsplit('.').next().unwrap_or(inner);
            return ileaf == "Str" || ileaf == enum_leaf;
        }
        if let Some((k, v)) = map_kv_types(dty) {
            let kleaf = k.rsplit('.').next().unwrap_or(k);
            let vleaf = v.rsplit('.').next().unwrap_or(v);
            return kleaf == "Str" || vleaf == enum_leaf || vleaf.eq_ignore_ascii_case(enum_leaf);
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_enum_field_drop(
        &self,
        out: &mut String,
        c: &mut usize,
        enum_ty: &str,
        fidx: u32,
        dty: &str,
        fllvm: &str,
        enum_leaf: &str,
        helper: &str,
        free_strings: bool,
    ) {
        let leaf = dty.rsplit('.').next().unwrap_or(dty);
        if leaf == "Str" {
            if free_strings {
                self.emit_enum_str_free(out, c, enum_ty, fidx, fllvm);
            }
            return;
        }
        if let Some(inner) = vec_inner_type(dty) {
            if fllvm != "%struct.Vec" {
                // A boxed/erased container slot is not this enum's layout;
                // leave it leak-only instead of guessing a box free.
                return;
            }
            let ileaf = inner.rsplit('.').next().unwrap_or(inner);
            let recurse = ileaf == enum_leaf;
            let str_elems = ileaf == "Str";
            if !recurse && !str_elems {
                return;
            }
            let vec_ptr = format!("%hv{}", *c);
            *c += 1;
            out.push_str(&format!(
                "  {vec_ptr} = getelementptr {enum_ty}, {enum_ty}* %node, i32 0, i32 {fidx}\n"
            ));
            // Vec[Str] is always an owned data buffer. Elements are freed only
            // for parse-owned trees; builder keys/values are borrowed.
            self.emit_vec_tree_free(
                out,
                c,
                enum_ty,
                &vec_ptr,
                recurse,
                helper,
                free_strings,
            );
            return;
        }
        if let Some((k, v)) = map_kv_types(dty) {
            if fllvm != "%struct.Map" {
                return;
            }
            let kleaf = k.rsplit('.').next().unwrap_or(k);
            let vleaf = v.rsplit('.').next().unwrap_or(v);
            let keys_str = kleaf == "Str";
            let recurse_values = vleaf == enum_leaf;
            if !keys_str && !recurse_values {
                return;
            }
            let map_ptr = format!("%hm{}", *c);
            *c += 1;
            out.push_str(&format!(
                "  {map_ptr} = getelementptr {enum_ty}, {enum_ty}* %node, i32 0, i32 {fidx}\n"
            ));
            let Some(map_meta) = self.types.type_meta.get(&"Map".to_string()) else {
                return;
            };
            let Some(kidx) = map_meta.fields.iter().position(|(n, _)| n == "keys") else {
                return;
            };
            let Some(vidx) = map_meta.fields.iter().position(|(n, _)| n == "values") else {
                return;
            };
            let keys_vec = format!("%hkv{}", *c);
            *c += 1;
            out.push_str(&format!(
                "  {keys_vec} = getelementptr %struct.Map, %struct.Map* {map_ptr}, i32 0, i32 {kidx}\n"
            ));
            let values_vec = format!("%hvv{}", *c);
            *c += 1;
            out.push_str(&format!(
                "  {values_vec} = getelementptr %struct.Map, %struct.Map* {map_ptr}, i32 0, i32 {vidx}\n"
            ));
            if keys_str {
                // Parse-owned: free each freshly parsed key. Builder-owned:
                // keys are borrowed literals; free only the keys buffer.
                self.emit_vec_tree_free(
                    out,
                    c,
                    enum_ty,
                    &keys_vec,
                    false,
                    helper,
                    free_strings,
                );
            }
            if recurse_values {
                self.emit_vec_tree_free(
                    out,
                    c,
                    enum_ty,
                    &values_vec,
                    true,
                    helper,
                    free_strings,
                );
            }
        }
    }

    fn emit_enum_str_free(
        &self,
        out: &mut String,
        c: &mut usize,
        enum_ty: &str,
        fidx: u32,
        fllvm: &str,
    ) {
        let slot = format!("%hs{}", *c);
        *c += 1;
        out.push_str(&format!(
            "  {slot} = getelementptr {enum_ty}, {enum_ty}* %node, i32 0, i32 {fidx}\n"
        ));
        let raw = format!("%hr{}", *c);
        *c += 1;
        let after = format!("hstr_after{}", *c);
        *c += 1;
        let chk = format!("hstr_chk{}", *c);
        *c += 1;
        let freed = format!("hstr_free{}", *c);
        *c += 1;
        if fllvm == "i64" {
            out.push_str(&format!("  {raw} = load i64, i64* {slot}\n"));
            let isnull = format!("%hn{}", *c);
            *c += 1;
            out.push_str(&format!("  {isnull} = icmp eq i64 {raw}, 0\n"));
            out.push_str(&format!(
                "  br i1 {isnull}, label %{after}, label %{chk}\n"
            ));
            out.push_str(&format!("\n{chk}:\n"));
            let ptr = format!("%hp{}", *c);
            *c += 1;
            out.push_str(&format!("  {ptr} = inttoptr i64 {raw} to i8*\n"));
            let slen = format!("%hl{}", *c);
            *c += 1;
            out.push_str(&format!("  {slen} = call i64 @xiom_str_len(i8* {ptr})\n"));
            let empty = format!("%he{}", *c);
            *c += 1;
            out.push_str(&format!("  {empty} = icmp eq i64 {slen}, 0\n"));
            out.push_str(&format!(
                "  br i1 {empty}, label %{after}, label %{freed}\n"
            ));
            out.push_str(&format!("\n{freed}:\n"));
            out.push_str(&format!("  call void @xiom_guard_free(i8* {ptr})\n"));
            out.push_str(&format!("  br label %{after}\n"));
            out.push_str(&format!("\n{after}:\n"));
        } else if fllvm == "i8*" {
            out.push_str(&format!("  {raw} = load i8*, i8** {slot}\n"));
            let isnull = format!("%hn{}", *c);
            *c += 1;
            out.push_str(&format!("  {isnull} = icmp eq i8* {raw}, null\n"));
            out.push_str(&format!(
                "  br i1 {isnull}, label %{after}, label %{chk}\n"
            ));
            out.push_str(&format!("\n{chk}:\n"));
            let slen = format!("%hl{}", *c);
            *c += 1;
            out.push_str(&format!("  {slen} = call i64 @xiom_str_len(i8* {raw})\n"));
            let empty = format!("%he{}", *c);
            *c += 1;
            out.push_str(&format!("  {empty} = icmp eq i64 {slen}, 0\n"));
            out.push_str(&format!(
                "  br i1 {empty}, label %{after}, label %{freed}\n"
            ));
            out.push_str(&format!("\n{freed}:\n"));
            out.push_str(&format!("  call void @xiom_guard_free(i8* {raw})\n"));
            out.push_str(&format!("  br label %{after}\n"));
            out.push_str(&format!("\n{after}:\n"));
        }
    }

    /// Emit the element loop (if any) and the data-buffer free for one
    /// `%struct.Vec*` field.
    /// - `recurse`: each element is the same enum; call `helper` on it.
    /// - `!recurse && free_strings`: each element is a parse-owned `Str`.
    /// - `!recurse && !free_strings`: elements are borrowed (builder keys or
    ///   literal values); only the data buffer is freed.
    fn emit_vec_tree_free(
        &self,
        out: &mut String,
        c: &mut usize,
        enum_ty: &str,
        vec_ptr: &str,
        recurse: bool,
        helper: &str,
        free_strings: bool,
    ) {
        let data = format!("%hvd{}", *c);
        *c += 1;
        let len = format!("%hvl{}", *c);
        *c += 1;
        let esz = format!("%hve{}", *c);
        *c += 1;
        // Vec header field indices come from the same type registry the rest
        // of the compiler uses; never a hardcoded memory offset.
        let vec_meta = self.types.type_meta.get(&"Vec".to_string());
        let field_idx = |name: &str, fallback: u32| -> u32 {
            vec_meta
                .as_ref()
                .and_then(|m| m.fields.iter().position(|(n, _)| n == name))
                .map(|i| i as u32)
                .unwrap_or(fallback)
        };
        let data_idx = field_idx("data", 0);
        let len_idx = field_idx("len", 1);
        let esz_idx = field_idx("elem_size", 3);
        let data_slot = format!("%hvds{}", *c);
        *c += 1;
        out.push_str(&format!(
            "  {data_slot} = getelementptr %struct.Vec, %struct.Vec* {vec_ptr}, i32 0, i32 {data_idx}\n"
        ));
        out.push_str(&format!("  {data} = load i8*, i8** {data_slot}\n"));
        let len_slot = format!("%hvls{}", *c);
        *c += 1;
        out.push_str(&format!(
            "  {len_slot} = getelementptr %struct.Vec, %struct.Vec* {vec_ptr}, i32 0, i32 {len_idx}\n"
        ));
        out.push_str(&format!("  {len} = load i64, i64* {len_slot}\n"));
        let esz_slot = format!("%hves{}", *c);
        *c += 1;
        out.push_str(&format!(
            "  {esz_slot} = getelementptr %struct.Vec, %struct.Vec* {vec_ptr}, i32 0, i32 {esz_idx}\n"
        ));
        out.push_str(&format!("  {esz} = load i64, i64* {esz_slot}\n"));
        let idx = format!("%hvi{}", *c);
        *c += 1;
        let cond = format!("hvc{}", *c);
        *c += 1;
        let body = format!("hvb{}", *c);
        *c += 1;
        let done = format!("hvd{}", *c);
        *c += 1;
        let esz_ok = format!("%hveo{}", *c);
        *c += 1;
        // Builder/stringify-owned Vec[Str] or Vec[borrowed]: the elements are
        // not owned, so only the data buffer is reclaimed. No element walk.
        if !recurse && !free_strings {
            out.push_str(&format!("  br label %{done}\n"));
            out.push_str(&format!("\n{done}:\n"));
            out.push_str(&format!("  call void @xiom_guard_free(i8* {data})\n"));
            return;
        }
        out.push_str(&format!("  {idx} = alloca i64\n"));
        out.push_str(&format!("  store i64 0, i64* {idx}\n"));
        out.push_str(&format!("  {esz_ok} = icmp ne i64 {esz}, 0\n"));
        out.push_str(&format!(
            "  br i1 {esz_ok}, label %{cond}, label %{done}\n"
        ));
        out.push_str(&format!("\n{cond}:\n"));
        let i = format!("%hvii{}", *c);
        *c += 1;
        let more = format!("%hvm{}", *c);
        *c += 1;
        out.push_str(&format!("  {i} = load i64, i64* {idx}\n"));
        out.push_str(&format!("  {more} = icmp ult i64 {i}, {len}\n"));
        out.push_str(&format!(
            "  br i1 {more}, label %{body}, label %{done}\n"
        ));
        out.push_str(&format!("\n{body}:\n"));
        let off = format!("%hvo{}", *c);
        *c += 1;
        let ep = format!("%hvp{}", *c);
        *c += 1;
        out.push_str(&format!("  {off} = mul i64 {i}, {esz}\n"));
        out.push_str(&format!(
            "  {ep} = getelementptr i8, i8* {data}, i64 {off}\n"
        ));
        if recurse {
            let epc = format!("%hvc2{}", *c);
            *c += 1;
            out.push_str(&format!("  {epc} = bitcast i8* {ep} to {enum_ty}*\n"));
            out.push_str(&format!(
                "  call void @{helper}({enum_ty}* {epc})\n"
            ));
        } else {
            let epc = format!("%hvc2{}", *c);
            *c += 1;
            let sp = format!("%hvs{}", *c);
            *c += 1;
            out.push_str(&format!("  {epc} = bitcast i8* {ep} to i8**\n"));
            out.push_str(&format!("  {sp} = load i8*, i8** {epc}\n"));
            let isnull = format!("%hsn{}", *c);
            *c += 1;
            let chk = format!("hsc{}", *c);
            *c += 1;
            let freed = format!("hsf{}", *c);
            *c += 1;
            let adv = format!("hsa{}", *c);
            *c += 1;
            out.push_str(&format!("  {isnull} = icmp eq i8* {sp}, null\n"));
            out.push_str(&format!(
                "  br i1 {isnull}, label %{adv}, label %{chk}\n"
            ));
            out.push_str(&format!("\n{chk}:\n"));
            let slen = format!("%hsl{}", *c);
            *c += 1;
            out.push_str(&format!("  {slen} = call i64 @xiom_str_len(i8* {sp})\n"));
            let empty = format!("%hse{}", *c);
            *c += 1;
            out.push_str(&format!("  {empty} = icmp eq i64 {slen}, 0\n"));
            out.push_str(&format!(
                "  br i1 {empty}, label %{adv}, label %{freed}\n"
            ));
            out.push_str(&format!("\n{freed}:\n"));
            out.push_str(&format!("  call void @xiom_guard_free(i8* {sp})\n"));
            out.push_str(&format!("  br label %{adv}\n"));
            out.push_str(&format!("\n{adv}:\n"));
        }
        let next = format!("%hvn{}", *c);
        *c += 1;
        out.push_str(&format!("  {next} = add i64 {i}, 1\n"));
        out.push_str(&format!("  store i64 {next}, i64* {idx}\n"));
        out.push_str(&format!("  br label %{cond}\n"));
        out.push_str(&format!("\n{done}:\n"));
        out.push_str(&format!("  call void @xiom_guard_free(i8* {data})\n"));
    }

    /// Free each owned element payload of a Vec before its data buffer.
    /// `vec_ptr` is the `%struct.Vec*` header; `data_ptr` is its loaded data
    /// pointer. Used for plain `Vec[T]` element tags and for contracted
    /// Map/Set `Vec[Str]` aggregate fields.
    fn emit_elem_free_loop(
        &mut self,
        data_ptr: &str,
        vec_ptr: &str,
        e: &crate::context::ElemDrop,
    ) {
        let len_gep = self.fresh_tmp();
        let len = self.fresh_tmp();
        self.emitln(&format!(
            "  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_ptr}, i32 0, i32 1"
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
        self.emitln(&format!("  br i1 {more}, label %{body}, label %{done}"));
        self.emitln(&format!("\n{body}:"));
        let off = self.fresh_tmp();
        let ep = self.fresh_tmp();
        let epc = self.fresh_tmp();
        self.emitln(&format!("  {off} = mul i64 {i}, {}", e.stride));
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
            self.emit_free_str_ptr(&dp);
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
                    let dp = self.fresh_tmp();
                    self.emitln(&format!("  {dp} = load i8*, i8** {cur}"));
                    self.emit_free_str_ptr(&dp);
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

    /// Emit the pending Vec deallocations (LIFO). Safe to emit on every return
    /// path: the drop flag has already been cleared on paths that ran this.
    pub(crate) fn compile_vec_drops(&mut self) -> Result<(), String> {        if self.fctx.vec_drops.is_empty() {
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
            if let VecDrop::EnumTree {
                ptr_slot,
                result_llvm_ty,
                disc_idx,
                payload_idx,
                enum_llvm_ty,
                helper,
                ..
            } = drop
            {
                // Only walk the payload when the Result is Ok/Some. An Err
                // payload slot is zero-initialized, but the tag check keeps a
                // non-contracted constructor from ever presenting a bogus
                // tree to the helper.
                let binding_ptr = self.fresh_tmp();
                self.emitln(&format!(
                    "  {binding_ptr} = load {result_llvm_ty}*, {result_llvm_ty}** {ptr_slot}"
                ));
                let disc_slot = self.fresh_tmp();
                let disc = self.fresh_tmp();
                let is_ok = self.fresh_tmp();
                let tree_do = self.fresh_block("enumtree_do");
                let tree_join = self.fresh_block("enumtree_join");
                self.emitln(&format!(
                    "  {disc_slot} = getelementptr {result_llvm_ty}, {result_llvm_ty}* {binding_ptr}, i32 0, i32 {disc_idx}"
                ));
                self.emitln(&format!("  {disc} = load i64, i64* {disc_slot}"));
                self.emitln(&format!("  {is_ok} = icmp eq i64 {disc}, 1"));
                self.emitln(&format!(
                    "  br i1 {is_ok}, label %{tree_do}, label %{tree_join}"
                ));
                self.emitln(&format!("\n{tree_do}:"));
                let payload_ptr = self.fresh_tmp();
                self.emitln(&format!(
                    "  {payload_ptr} = getelementptr {result_llvm_ty}, {result_llvm_ty}* {binding_ptr}, i32 0, i32 {payload_idx}"
                ));
                self.emitln(&format!(
                    "  call void @{helper}({enum_llvm_ty}* {payload_ptr})"
                ));
                self.emitln(&format!("  br label %{tree_join}"));
                self.emitln(&format!("\n{tree_join}:"));
                self.emitln(&format!("  store i1 false, i1* {flag}"));
                self.emitln(&format!("  br label %{skip_block}"));
                self.emitln(&format!("\n{skip_block}:"));
                continue;
            }
            if let VecDrop::StructPayload {
                ptr_slot,
                result_llvm_ty,
                disc_idx,
                payload_idx,
                payload_llvm_ty,
                helper,
                ..
            } = drop
            {
                // Only walk the payload when the Result/Option is Ok/Some.
                // None/Err has no owned payload, and a zero-initialized slot
                // must never be presented to the helper.
                let binding_ptr = self.fresh_tmp();
                self.emitln(&format!(
                    "  {binding_ptr} = load {result_llvm_ty}*, {result_llvm_ty}** {ptr_slot}"
                ));
                let disc_slot = self.fresh_tmp();
                let disc = self.fresh_tmp();
                let is_ok = self.fresh_tmp();
                let tree_do = self.fresh_block("structpayload_do");
                let tree_join = self.fresh_block("structpayload_join");
                self.emitln(&format!(
                    "  {disc_slot} = getelementptr {result_llvm_ty}, {result_llvm_ty}* {binding_ptr}, i32 0, i32 {disc_idx}"
                ));
                self.emitln(&format!("  {disc} = load i64, i64* {disc_slot}"));
                self.emitln(&format!("  {is_ok} = icmp eq i64 {disc}, 1"));
                self.emitln(&format!(
                    "  br i1 {is_ok}, label %{tree_do}, label %{tree_join}"
                ));
                self.emitln(&format!("\n{tree_do}:"));
                let payload_ptr = self.fresh_tmp();
                self.emitln(&format!(
                    "  {payload_ptr} = getelementptr {result_llvm_ty}, {result_llvm_ty}* {binding_ptr}, i32 0, i32 {payload_idx}"
                ));
                self.emitln(&format!(
                    "  call void @{helper}({payload_llvm_ty}* {payload_ptr})"
                ));
                self.emitln(&format!("  br label %{tree_join}"));
                self.emitln(&format!("\n{tree_join}:"));
                self.emitln(&format!("  store i1 false, i1* {flag}"));
                self.emitln(&format!("  br label %{skip_block}"));
                self.emitln(&format!("\n{skip_block}:"));
                continue;
            }
            if let VecDrop::BoxedVec { header_alloca, .. } = drop {
                // Boxed removal payload: `header_alloca` holds the malloc'd
                // `%struct.Vec*` (a copy of the removed element header). The
                // container shrank its len, so this buffer is unreachable from
                // the container's own exit drop. Free data, then the box.
                let boxed = self.fresh_tmp();
                self.emitln(&format!(
                    "  {boxed} = load %struct.Vec*, %struct.Vec** {header_alloca}"
                ));
                let null = self.fresh_tmp();
                let free_blk = self.fresh_block("boxedvec_free");
                let done_blk = self.fresh_block("boxedvec_done");
                self.emitln(&format!("  {null} = icmp eq %struct.Vec* {boxed}, null"));
                self.emitln(&format!(
                    "  br i1 {null}, label %{done_blk}, label %{free_blk}"
                ));
                self.emitln(&format!("\n{free_blk}:"));
                let data_gep = self.fresh_tmp();
                let data_ptr = self.fresh_tmp();
                self.emitln(&format!(
                    "  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {boxed}, i32 0, i32 0"
                ));
                self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                self.emitln(&format!("  call void @xiom_guard_free(i8* {data_ptr})"));
                let box_i8 = self.fresh_tmp();
                self.emitln(&format!("  {box_i8} = bitcast %struct.Vec* {boxed} to i8*"));
                self.emitln(&format!("  call void @xiom_guard_free(i8* {box_i8})"));
                self.emitln(&format!("  br label %{done_blk}"));
                self.emitln(&format!("\n{done_blk}:"));
                self.emitln(&format!("  store i1 false, i1* {flag}"));
                self.emitln(&format!("  br label %{skip_block}"));
                self.emitln(&format!("\n{skip_block}:"));
                continue;
            }
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
                        self.emit_elem_free_loop(&data_ptr, alloca, e);
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
                VecDrop::Aggregate {
                    data_gep,
                    elem_is_str,
                    ..
                } => {
                    let data_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                    if *elem_is_str {
                        // Contracted Map/Set Vec[Str] field: the elements are
                        // owned Str payloads. Free them before the buffer.
                        let vec_ptr = self.fresh_tmp();
                        self.emitln(&format!(
                            "  {vec_ptr} = bitcast i8** {data_gep} to %struct.Vec*"
                        ));
                        let e = crate::context::ElemDrop {
                            llvm_ty: "i8*".to_string(),
                            stride: 8,
                            elem_is_vec: false,
                            elem_is_str: true,
                            paths: Vec::new(),
                        };
                        self.emit_elem_free_loop(&data_ptr, &vec_ptr, &e);
                    }
                    data_ptr
                }
                VecDrop::BoxedVec { .. } => unreachable!("BoxedVec handled above"),
                VecDrop::EnumTree { .. } => unreachable!("EnumTree handled above"),
                VecDrop::StructPayload { .. } => {
                    unreachable!("StructPayload handled above")
                }
            };
            // Arena-aware: block-fn Vecs live on the guard arena and are
            // reclaimed wholesale; main-heap Vecs are freed here. A Str drop
            // is guarded against null/empty (an empty literal may be shared).
            let is_str_ptr = matches!(drop, VecDrop::Str { .. } | VecDrop::Ptr { .. });
            if is_str_ptr {
                self.emit_free_str_ptr(&data_ptr);
            } else {
                self.emitln(&format!("  call void @xiom_guard_free(i8* {data_ptr})"));
            }
            self.emitln(&format!("  store i1 false, i1* {flag}"));
            self.emitln(&format!("  br label %{skip_block}"));
            self.emitln(&format!("\n{skip_block}:"));
        }
        Ok(())
    }
}
