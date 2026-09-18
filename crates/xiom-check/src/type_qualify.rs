// XIOM -- Type Checker (catalog-injection type qualification)
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// R39 (same-leaf TYPE collision): catalog-injected modules flatten into the
// program as top-level decls, so their struct/enum names used to be injected
// BARE (`Metrics`). Two project modules that declare the same leaf
// (`benchmark.borrow.Metrics` 4 fields vs `benchmark.derive.Metrics` 7
// fields, the bench graph repro) collapsed to ONE `%struct.Metrics`
// definition while the losing module's bodies kept GEPing their own field
// count -- invalid IR (field 4 of a 4-field struct).
//
// This pass is the type analogue of the fn-side module qualification
// (`collect_pub_decls` renames free fns to `module.leaf`): when a type leaf
// is declared by more than one project module, every reference in the
// affected modules is rewritten to the module-qualified name BEFORE the
// decls are flattened and injected. Non-colliding types are untouched, so
// the blast radius is exactly the collision set.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use xiom_ast::*;

/// Injectable type/enum leaves declared in `items` (recursing module
/// wrappers) that participate in the collision qualification.
///
/// Includes generic pub types (`Box[T]`): generic leaves are renamed only
/// when their DECLARED SHAPES conflict (see `declared_type_shapes` and the
/// triage in `collect_external_decls`), because mono/concrete-container keys
/// (`Option__Pair`) are leaf-derived and identical re-declarations must keep
/// the legacy single key. Interfaces have no layout and are not renamed.
pub(crate) fn declared_type_leaves(items: &[TopDecl], out: &mut BTreeSet<String>) {
    for item in items {
        match item {
            TopDecl::Type(td) if td.is_pub || !td.generics.is_empty() => {
                out.insert(td.name.name.clone());
            }
            TopDecl::Enum(ed) if ed.is_pub => {
                out.insert(ed.name.name.clone());
            }
            TopDecl::Module(md) => declared_type_leaves(&md.items, out),
            _ => {}
        }
    }
}

// ============================================================================
// Declaration shapes (collision triage)
// ============================================================================

/// Stable, span-free shape of a type reference (names + structure only).
fn type_shape(ty: &Type) -> String {
    match ty {
        Type::Named(id, args) => {
            if args.is_empty() {
                id.name.clone()
            } else {
                let inner: Vec<String> = args.iter().map(type_shape).collect();
                format!("{}[{}]", id.name, inner.join(","))
            }
        }
        Type::Ref(t) => format!("&{}", type_shape(t)),
        Type::MutRef(t) => format!("&mut {}", type_shape(t)),
        Type::Ptr(t) => format!("*{}", type_shape(t)),
        Type::Option(t) => format!("Option[{}]", type_shape(t)),
        Type::Vec(t) => format!("Vec[{}]", type_shape(t)),
        Type::Slice(t) => format!("Slice[{}]", type_shape(t)),
        Type::Set(t) => format!("Set[{}]", type_shape(t)),
        Type::Result(a, b) => format!("Result[{},{}]", type_shape(a), type_shape(b)),
        Type::Map(a, b) => format!("Map[{},{}]", type_shape(a), type_shape(b)),
        Type::Tuple(ts) => {
            let inner: Vec<String> = ts.iter().map(type_shape).collect();
            format!("({})", inner.join(","))
        }
        Type::Fn(ps, ret) => {
            let inner: Vec<String> = ps.iter().map(type_shape).collect();
            format!("fn({})->{}", inner.join(","), type_shape(ret))
        }
        Type::Array(_, elem) => format!("[{}]", type_shape(elem)),
        Type::AnonStruct(fields) => {
            let inner: Vec<String> = fields.iter().map(|f| type_shape(&f.ty)).collect();
            format!("{{{}}}", inner.join(";"))
        }
        Type::ImplTrait(_) => "impl".to_string(),
        Type::Never => "!".to_string(),
    }
}

/// Layout-ish shape of a struct/type declaration.
pub(crate) fn type_decl_shape(td: &TypeDecl) -> String {
    if let Some(alias) = &td.alias {
        return format!("alias:{}", type_shape(alias));
    }
    let fields: Vec<String> = td.fields.iter().map(|f| type_shape(&f.ty)).collect();
    format!("{{generics:{};fields:{}}}", td.generics.len(), fields.join(";"))
}

/// Layout-ish shape of an enum declaration.
pub(crate) fn enum_decl_shape(ed: &EnumDecl) -> String {
    let variants: Vec<String> = ed
        .variants
        .iter()
        .map(|v| {
            let fields: Vec<String> = v.fields.iter().map(|f| type_shape(&f.ty)).collect();
            format!("{}[{}]", v.name.name, fields.join(","))
        })
        .collect();
    format!("{{generics:{};{} }}", ed.generics.len(), variants.join("|"))
}

/// Injectable type/enum leaves mapped to `(shape, is_generic)`.
pub(crate) fn declared_type_shapes(items: &[TopDecl], out: &mut BTreeMap<String, (String, bool)>) {
    for item in items {
        match item {
            TopDecl::Type(td) if td.is_pub || !td.generics.is_empty() => {
                out.insert(
                    td.name.name.clone(),
                    (type_decl_shape(td), !td.generics.is_empty()),
                );
            }
            TopDecl::Enum(ed) if ed.is_pub => {
                out.insert(ed.name.name.clone(), (enum_decl_shape(ed), false));
            }
            TopDecl::Module(md) => declared_type_shapes(&md.items, out),
            _ => {}
        }
    }
}

/// Every type-name reference in `items` (annotation positions, struct
/// literals, associated-call receivers, method receivers, destructure
/// patterns). Used to keep the user program's own references untouched: a
/// colliding leaf that the program references is never renamed.
pub(crate) fn referenced_type_names(items: &[TopDecl]) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for item in items {
        let mut visit = |id: &mut Ident| {
            // Record the full spelling AND its leaf: a dotted reference
            // (`am.Metrics`) must exclude the leaf from renaming just like a
            // bare one, or the program's own annotation would stop resolving.
            if let Some(leaf) = id.name.rsplit('.').next() {
                names.insert(leaf.to_string());
            }
            names.insert(id.name.clone());
        };
        walk_top_ro(item, &mut visit);
    }
    names
}

/// Rename plan for one module: bare leaf / dotted import paths -> qualified
/// type names. Built only for leaves in the global collision set.
#[derive(Default, Clone)]
pub(crate) struct TypeRenames {
    /// Exact name (bare leaf, alias, or import spelling) -> qualified name.
    exact: HashMap<String, String>,
    /// Import prefix (alias or dotted module path) -> module path.
    prefixes: BTreeMap<String, String>,
    /// Colliding leaf -> its qualified name (resolved per prefix).
    leaf_full: HashMap<String, String>,
}

impl TypeRenames {
    pub(crate) fn is_empty(&self) -> bool {
        self.exact.is_empty() && self.leaf_full.is_empty()
    }

    fn add_exact(&mut self, from: &str, full: &str) {
        if !from.is_empty() && from != full {
            self.exact.insert(from.to_string(), full.to_string());
        }
    }

    fn add_prefix(&mut self, prefix: &str, module: &str) {
        if !prefix.is_empty() && prefix != module {
            self.prefixes.insert(prefix.to_string(), module.to_string());
        }
    }

    fn add_leaf(&mut self, leaf: &str, full: &str) {
        self.leaf_full.insert(leaf.to_string(), full.to_string());
    }

    /// Map a name in a type position. `None` = leave unchanged.
    fn map(&self, name: &str) -> Option<String> {
        if let Some(full) = self.exact.get(name) {
            return Some(full.clone());
        }
        // Dotted reference through an import/alias prefix: longest prefix wins.
        let mut best: Option<(&str, &str)> = None;
        for (prefix, module) in &self.prefixes {
            if name.len() > prefix.len() + 1
                && name.starts_with(prefix.as_str())
                && name.as_bytes()[prefix.len()] == b'.'
            {
                if best.map_or(true, |(bp, _)| prefix.len() > bp.len()) {
                    best = Some((prefix.as_str(), module.as_str()));
                }
            }
        }
        if let Some((prefix, module)) = best {
            let leaf = &name[prefix.len() + 1..];
            if let Some(full) = self.leaf_full.get(leaf) {
                if full == &format!("{module}.{leaf}") {
                    return Some(full.clone());
                }
            }
        }
        None
    }
}

/// Module paths a `use` declaration can bind, plus the item leaf it names.
fn use_binding(ud: &UseDecl) -> (String, String, Option<String>) {
    let full = ud.path.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(".");
    let last = ud.path.last().map(|i| i.name.clone()).unwrap_or_default();
    let alias = ud.alias.as_ref().map(|a| a.name.clone());
    (full, last, alias)
}

/// Build the rename plan for one catalog module.
///
/// * `colliding`: leaf -> set of project modules declaring it (>= 2 owners).
/// * `excluded`: leaves the user program declares or references; never
///   renamed (their references are resolved by normal module registration and
///   first-wins stays for program/catalog overlap).
pub(crate) fn build_renames(
    items: &[TopDecl],
    module: &str,
    colliding: &HashMap<String, BTreeSet<String>>,
    excluded: &BTreeSet<String>,
) -> TypeRenames {
    let mut plan = TypeRenames::default();

    // Own colliding types -> module-qualified.
    let mut own = BTreeSet::new();
    declared_type_leaves(items, &mut own);
    for leaf in &own {
        if excluded.contains(leaf) {
            continue;
        }
        if let Some(owners) = colliding.get(leaf) {
            if owners.contains(module) {
                let full = format!("{module}.{leaf}");
                plan.add_exact(leaf, &full);
                plan.add_leaf(leaf, &full);
            }
        }
    }

    // Imported colliding types: `use other.module;` then `module.Leaf`, or
    // `use other.module.Leaf;` (optionally aliased).
    fn owners_hit(colliding: &HashMap<String, BTreeSet<String>>, module: &str) -> bool {
        colliding.values().any(|owners| owners.contains(module))
    }
    fn walk_uses(
        items: &[TopDecl],
        plan: &mut TypeRenames,
        colliding: &HashMap<String, BTreeSet<String>>,
    ) {
        for item in items {
            match item {
                TopDecl::Use(ud) => {
                    let (full, last, alias) = use_binding(ud);
                    // Item import (`use a.b.Leaf;` / `use a.b.Leaf as L;`).
                    if let Some(owners) = colliding.get(&last) {
                        let owner = ud.path[..ud.path.len().saturating_sub(1)]
                            .iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(".");
                        if owners.contains(&owner) {
                            let target = format!("{owner}.{last}");
                            let bound = alias.clone().unwrap_or_else(|| last.clone());
                            plan.add_exact(&bound, &target);
                            plan.add_leaf(&last, &target);
                        }
                    }
                    // Module import (`use a.b as ab;` or `use a.b;`): the
                    // dotted form binds only if `a.b` owns a colliding leaf.
                    if owners_hit(colliding, &full) {
                        let bound = alias.clone().unwrap_or_else(|| last.clone());
                        plan.add_prefix(&bound, &full);
                        plan.add_prefix(&full, &full);
                        for (leaf, owners) in colliding {
                            if owners.contains(&full) {
                                plan.add_leaf(leaf, &format!("{full}.{leaf}"));
                            }
                        }
                    }
                }
                TopDecl::Module(md) => walk_uses(&md.items, plan, colliding),
                _ => {}
            }
        }
    }
    walk_uses(items, &mut plan, colliding);

    plan
}

/// Deep-clone `items` and rewrite every type reference named by `plan`.
pub(crate) fn qualify_type_refs(items: &[TopDecl], plan: &TypeRenames) -> Vec<TopDecl> {
    let mut out = items.to_vec();
    for item in &mut out {
        walk_top(item, &mut |id: &mut Ident| {
            if let Some(full) = plan.map(&id.name) {
                id.name = full;
            }
        });
    }
    out
}

// ============================================================================
// Type-reference walker. One traversal, two consumers: the mutating
// qualifier and the read-only reference collector.
// ============================================================================

fn walk_type<F: FnMut(&mut Ident)>(ty: &mut Type, visit: &mut F) {
    match ty {
        Type::Named(id, args) => {
            visit(id);
            for a in args {
                walk_type(a, visit);
            }
        }
        Type::Ref(i) | Type::MutRef(i) | Type::Ptr(i) | Type::Option(i) | Type::Vec(i)
        | Type::Slice(i) | Type::Set(i) => walk_type(i, visit),
        Type::Result(a, b) | Type::Map(a, b) => {
            walk_type(a, visit);
            walk_type(b, visit);
        }
        Type::Tuple(elems) => {
            for e in elems {
                walk_type(e, visit);
            }
        }
        Type::Fn(params, ret) => {
            for p in params {
                walk_type(p, visit);
            }
            walk_type(ret, visit);
        }
        Type::Array(size, elem) => {
            walk_expr(size, visit);
            walk_type(elem, visit);
        }
        Type::AnonStruct(fields) => {
            for f in fields {
                walk_type(&mut f.ty, visit);
            }
        }
        Type::ImplTrait(_) | Type::Never => {}
    }
}

fn walk_pattern<F: FnMut(&mut Ident)>(p: &mut Pattern, visit: &mut F) {
    match p {
        Pattern::Struct(id, fields, _) => {
            visit(id);
            for (_, fp) in fields {
                walk_pattern(fp, visit);
            }
        }
        Pattern::Variant(_, _, _) => {}
        Pattern::Tuple(elems, _) | Pattern::Or(elems, _) => {
            for e in elems {
                walk_pattern(e, visit);
            }
        }
        Pattern::Some(inner, _) | Pattern::Ok(inner, _) | Pattern::Err(inner, _) => {
            walk_pattern(inner, visit);
        }
        _ => {}
    }
}

fn walk_block<F: FnMut(&mut Ident)>(block: &mut Block, visit: &mut F) {
    for se in &mut block.stmts {
        match se {
            StmtOrExpr::Stmt(s) => walk_stmt(s, visit),
            StmtOrExpr::Expr(e) => walk_expr(e, visit),
        }
    }
}

fn walk_stmt<F: FnMut(&mut Ident)>(stmt: &mut Stmt, visit: &mut F) {
    match stmt {
        Stmt::Let(_, ty, e, _) | Stmt::Var(_, ty, e, _) => {
            if let Some(t) = ty {
                walk_type(t, visit);
            }
            walk_expr(e, visit);
        }
        Stmt::Assign(a, b, _) => {
            walk_expr(a, visit);
            walk_expr(b, visit);
        }
        Stmt::Return(e, _) => {
            if let Some(e) = e {
                walk_expr(e, visit);
            }
        }
        Stmt::Expr(e, _) => walk_expr(e, visit),
        Stmt::If(c, t, elifs, els, _) => {
            walk_expr(c, visit);
            walk_block(t, visit);
            for (ec, eb) in elifs {
                walk_expr(ec, visit);
                walk_block(eb, visit);
            }
            if let Some(eb) = els {
                walk_block(eb, visit);
            }
        }
        Stmt::Match(e, arms, _) => {
            walk_expr(e, visit);
            for arm in arms {
                walk_pattern(&mut arm.pattern, visit);
                if let Some(g) = &mut arm.guard {
                    walk_expr(g, visit);
                }
                match &mut arm.body {
                    MatchBody::Block(b) => walk_block(b, visit),
                    MatchBody::Expr(e) => walk_expr(e, visit),
                }
            }
        }
        Stmt::While(c, b, inv, _, _) => {
            walk_expr(c, visit);
            walk_block(b, visit);
            if let Some(inv) = inv {
                walk_expr(inv, visit);
            }
        }
        Stmt::For(_, e, b, _, _) => {
            walk_expr(e, visit);
            walk_block(b, visit);
        }
        Stmt::Spawn(b, _, _) | Stmt::Defer(b, _) => walk_block(b, visit),
        Stmt::Destructure(_, e, _) => walk_expr(e, visit),
        Stmt::Asm(asm) => {
            for (_, e) in &mut asm.inputs {
                walk_expr(e, visit);
            }
        }
        Stmt::Assert(c, msg, _) => {
            walk_expr(c, visit);
            if let Some(msg) = msg {
                walk_expr(msg, visit);
            }
        }
        Stmt::Break(..) | Stmt::Continue(..) | Stmt::Debugger(_) => {}
    }
}

/// Visit type-name identifiers inside an expression-position container
/// argument (`Vec[Record]`, `Map[Str, Record]`, nested forms).
fn visit_type_args_in_expr<F: FnMut(&mut Ident)>(expr: &mut Expr, visit: &mut F) {
    match expr {
        Expr::Ident(id) => visit(id),
        Expr::Tuple(elems, _) => {
            for el in elems {
                visit_type_args_in_expr(el, visit);
            }
        }
        Expr::Index(base, idx, _) => {
            visit_type_args_in_expr(base, visit);
            visit_type_args_in_expr(idx, visit);
        }
        _ => {}
    }
}

fn walk_expr<F: FnMut(&mut Ident)>(expr: &mut Expr, visit: &mut F) {
    match expr {
        Expr::Struct(id, fields, base, _) => {
            visit(id);
            for (_, v) in fields {
                walk_expr(v, visit);
            }
            if let Some(b) = base {
                walk_expr(b, visit);
            }
        }
        Expr::Call(callee, args, _) => {
            walk_expr(callee, visit);
            for a in args {
                walk_expr(a, visit);
            }
        }
        Expr::GenericCall(callee, types, args, _) => {
            walk_expr(callee, visit);
            for t in types {
                walk_type(t, visit);
            }
            for a in args {
                walk_expr(a, visit);
            }
        }
        // `Type.method(...)` / `Type.Variant` -- the base ident is a type.
        Expr::Field(base, _, _) => {
            if let Expr::Ident(id) = base.as_mut() {
                visit(id);
            }
            walk_expr(base, visit);
        }
        Expr::Index(a, b, _) => {
            // Expression-position container type application: `Vec[Record].new()`
            // parses as `Index(Ident("Vec"), Ident("Record"))`, so the bracket
            // argument is a TYPE spelled as an identifier (nested containers
            // recurse through Index/Tuple). Real indexing (`arr[i]`) has a
            // value base and stays untouched.
            let container_base = matches!(
                a.as_ref(),
                Expr::Ident(id) if matches!(
                    id.name.as_str(),
                    "Vec" | "Slice" | "Map" | "Set" | "Option" | "Result"
                )
            );
            if container_base {
                visit_type_args_in_expr(b, visit);
            }
            walk_expr(a, visit);
            walk_expr(b, visit);
        }
        Expr::Paren(e, _) | Expr::Unary(_, e, _) | Expr::Try(e, _) | Expr::AtPre(e, _)
        | Expr::Ref(e, _) | Expr::MutRef(e, _) | Expr::Some(e, _) | Expr::Ok(e, _)
        | Expr::Err(e, _) | Expr::Await(e, _) | Expr::Comptime(e, _) | Expr::ConstBlock(e, _)
        | Expr::PipeClosure(_, e, _) => walk_expr(e, visit),
        Expr::Binary(a, _, b, _) | Expr::Imply(a, b, _) => {
            walk_expr(a, visit);
            walk_expr(b, visit);
        }
        Expr::Is(e, pat, _) => {
            walk_expr(e, visit);
            walk_pattern(pat, visit);
        }
        Expr::As(e, ty, _) => {
            walk_expr(e, visit);
            walk_type(ty, visit);
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
            for e in elems {
                walk_expr(e, visit);
            }
        }
        Expr::BlockExpr(b, _) | Expr::Unsafe(b, _) => walk_block(b, visit),
        Expr::Closure(params, ret, body, _) => {
            for p in params {
                walk_type(&mut p.ty, visit);
            }
            if let Some(ret) = ret {
                walk_type(ret, visit);
            }
            walk_block(body, visit);
        }
        Expr::If(c, t, elifs, els, _) => {
            walk_expr(c, visit);
            walk_block(t, visit);
            for (ec, eb) in elifs {
                walk_expr(ec, visit);
                walk_block(eb, visit);
            }
            if let Some(eb) = els {
                walk_block(eb, visit);
            }
        }
        Expr::Match(e, arms, _) => {
            walk_expr(e, visit);
            for arm in arms {
                walk_pattern(&mut arm.pattern, visit);
                if let Some(g) = &mut arm.guard {
                    walk_expr(g, visit);
                }
                match &mut arm.body {
                    MatchBody::Block(b) => walk_block(b, visit),
                    MatchBody::Expr(e) => walk_expr(e, visit),
                }
            }
        }
        Expr::Ident(_) | Expr::Int(..) | Expr::BigInt(..) | Expr::Float(..) | Expr::Str(..)
        | Expr::Char(..) | Expr::Bool(..) | Expr::None(_) | Expr::Error(..) => {}
    }
}

fn walk_fn<F: FnMut(&mut Ident)>(fd: &mut FnDecl, visit: &mut F) {
    if let Some(recv) = &mut fd.receiver {
        visit(recv);
    }
    for p in &mut fd.params {
        walk_type(&mut p.ty, visit);
    }
    if let Some(rt) = &mut fd.return_type {
        walk_type(rt, visit);
    }
    for clause in &mut fd.contracts {
        match clause {
            ContractClause::Requires(e, _) | ContractClause::Ensures(e, _) => walk_expr(e, visit),
        }
    }
    if let Some(body) = &mut fd.body {
        walk_block(body, visit);
    }
}

fn walk_top<F: FnMut(&mut Ident)>(item: &mut TopDecl, visit: &mut F) {
    match item {
        TopDecl::Type(td) => {
            visit(&mut td.name);
            for f in &mut td.fields {
                walk_type(&mut f.ty, visit);
            }
            for (_, ty, e) in &mut td.derived_fields {
                walk_type(ty, visit);
                walk_expr(e, visit);
            }
            for inv in &mut td.invariants {
                walk_expr(inv, visit);
            }
            if let Some(alias) = &mut td.alias {
                walk_type(alias, visit);
            }
        }
        TopDecl::Enum(ed) => {
            visit(&mut ed.name);
            for v in &mut ed.variants {
                for f in &mut v.fields {
                    walk_type(&mut f.ty, visit);
                }
            }
        }
        TopDecl::Interface(id) => {
            for m in &mut id.members {
                match m {
                    InterfaceMember::Field(f) => walk_type(&mut f.ty, visit),
                    InterfaceMember::FnSignature(fd) => walk_fn(fd, visit),
                }
            }
        }
        TopDecl::Fn(fd) => walk_fn(fd, visit),
        TopDecl::Const(cd) => {
            walk_type(&mut cd.ty, visit);
            walk_expr(&mut cd.value, visit);
        }
        TopDecl::Extern(eb) => {
            for fd in &mut eb.functions {
                walk_fn(fd, visit);
            }
        }
        TopDecl::Impl(id) => {
            visit(&mut id.type_name);
            for a in &mut id.trait_args {
                walk_type(a, visit);
            }
            for m in &mut id.members {
                match m {
                    ImplItem::Fn(fd) => walk_fn(fd, visit),
                    ImplItem::Const(cd) => {
                        walk_type(&mut cd.ty, visit);
                        walk_expr(&mut cd.value, visit);
                    }
                }
            }
        }
        TopDecl::Module(md) => {
            for sub in &mut md.items {
                walk_top(sub, visit);
            }
        }
        TopDecl::Spawn(b, _, _) => walk_block(b, visit),
        TopDecl::Use(_) => {}
    }
}

/// Read-only walk used by `referenced_type_names` (which receives shared
/// references). Mirrors `walk_top`; kept separate only because the AST is not
/// `Clone`-free and the collector owns no mutation.
fn walk_top_ro<F: FnMut(&mut Ident)>(item: &TopDecl, visit: &mut F) {
    // SAFETY-free approach: walk the shared tree by cloning the small idents
    // into a temporary mutable copy only where needed. The collector never
    // mutates, so a shallow clone of the item is sufficient for traversal.
    let mut owned = item.clone();
    walk_top(&mut owned, visit);
}

#[cfg(test)]
mod tests {
    use super::*;
    use xiom_lexer::Lexer;
    use xiom_parser::Parser;

    fn parse(src: &str) -> Vec<TopDecl> {
        let tokens = Lexer::new(src).tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_program().expect("parse").items
    }

    #[test]
    fn collision_set_qualifies_decls_and_references() {
        let items = parse(
            r#"
module benchmark.derive
pub type Metrics = {
  requests: Int;
  errors: Int;
}
pub fn make() -> Metrics {
  return Metrics{ requests: 1, errors: 2 };
}
pub fn read(m: &Metrics) -> Int {
  return m.requests;
}
"#,
        );
        let mut colliding: HashMap<String, BTreeSet<String>> = HashMap::new();
        colliding.insert(
            "Metrics".to_string(),
            ["benchmark.borrow".to_string(), "benchmark.derive".to_string()]
                .into_iter()
                .collect(),
        );
        let plan = build_renames(&items, "benchmark.derive", &colliding, &BTreeSet::new());
        assert!(!plan.is_empty());
        let out = qualify_type_refs(&items, &plan);
        let mut leaves = BTreeSet::new();
        declared_type_leaves(&out, &mut leaves);
        assert!(leaves.contains("benchmark.derive.Metrics"), "{leaves:?}");
        // Type decl name + return annotation + struct literal ident.
        let dump = format!("{out:?}");
        assert!(
            dump.matches("benchmark.derive.Metrics").count() >= 3,
            "{dump}"
        );
        assert!(!dump.contains("Ident { name: \"Metrics\""), "bare reference left: {dump}");
    }

    #[test]
    fn non_colliding_modules_are_untouched() {
        let items = parse(
            r#"
module alpha.metrics
pub type Metrics = { a: Int; }
pub fn make() -> Metrics { return Metrics{ a: 1 }; }
"#,
        );
        let colliding: HashMap<String, BTreeSet<String>> = HashMap::new();
        let plan = build_renames(&items, "alpha.metrics", &colliding, &BTreeSet::new());
        assert!(plan.is_empty());
    }

    #[test]
    fn excluded_leaves_are_not_renamed() {
        let items = parse(
            r#"
module alpha.metrics
pub type Metrics = { a: Int; }
"#,
        );
        let mut colliding: HashMap<String, BTreeSet<String>> = HashMap::new();
        colliding.insert(
            "Metrics".to_string(),
            ["alpha.metrics".to_string(), "beta.metrics".to_string()].into_iter().collect(),
        );
        let mut excluded = BTreeSet::new();
        excluded.insert("Metrics".to_string());
        let plan = build_renames(&items, "alpha.metrics", &colliding, &excluded);
        assert!(plan.is_empty());
    }

    #[test]
    fn qualifies_container_arg_in_expression_position() {
        let items = parse(
            r#"
module alpha.metrics
pub type Metrics = { a: Int; }
pub fn build() -> Int {
  var recs = Vec[Metrics].new();
  return 0;
}
"#,
        );
        let mut colliding: HashMap<String, BTreeSet<String>> = HashMap::new();
        colliding.insert(
            "Metrics".to_string(),
            ["alpha.metrics".to_string(), "beta.metrics".to_string()].into_iter().collect(),
        );
        let plan = build_renames(&items, "alpha.metrics", &colliding, &BTreeSet::new());
        let out = qualify_type_refs(&items, &plan);
        let dump = format!("{out:?}");
        assert!(
            dump.contains("Ident { name: \"alpha.metrics.Metrics\""),
            "container arg not qualified: {dump}"
        );
    }

    #[test]
    fn referenced_names_collects_annotations_and_literals() {
        let items = parse(
            r#"
module gateway
fn use_it(m: &Metrics) -> Metrics {
  var x: Metrics = Metrics{ a: 1 };
  return x;
}
"#,
        );
        let names = referenced_type_names(&items);
        assert!(names.contains("Metrics"), "{names:?}");
    }
}
