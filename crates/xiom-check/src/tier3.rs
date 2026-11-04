// XIOM -- Stage 6 tier 3 opt-in lints (`--lint-strict`)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Usage-analysis lints (W015 unused local, W010 unused parameter, W011
// unused import). Kept out of the type-checker hot path because a full
// lexical usage walk is only valuable opt-in; `check_program` calls this
// module only when `Checker::lint_strict` is set.
//
// Contract:
// - user-program scope only (the driver never enables the pass for catalog
//   bodies or corpus-loading checkers);
// - warning-only, and every finding is emitted through `warn_coded_at` so
//   the driver prints `warning[WNNN]` and JSON carries the code;
// - conservative by construction: an unresolved/ambiguous shape is treated
//   as a USE (silence), never as an unused binding.
//
// W009 is already the reserved-system-symbol lint (REL-20261028-0052), so
// the unused-local lint ships as W015; the rest keep the planned W010-W014
// numbering.

use std::collections::HashSet;

use xiom_ast::{
    Block, Expr, FnDecl, MatchBody, Pattern, Program, Span, Stmt, StmtOrExpr, TopDecl, Type,
    UseDecl,
};

use crate::Checker;

pub(crate) fn lint_user_program(checker: &mut Checker, program: &Program) {
    lint_unused_imports(checker, &program.items);
    let mut walker = UsageWalker::new(checker);
    walk_items(&mut walker, &program.items);
}

// ============================================================================
// W010 / W015 -- unused parameters and local bindings
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum BindKind {
    Local,
    LoopVar,
    Param,
    /// Introduced only so it shadows an outer binding; never warned.
    Pattern,
}

struct Binding {
    name: String,
    span: Span,
    used: bool,
    warn: bool,
    kind: BindKind,
}

struct UsageWalker<'a> {
    checker: &'a mut Checker,
    scopes: Vec<Vec<Binding>>,
    closure_depth: usize,
}

impl<'a> UsageWalker<'a> {
    fn new(checker: &'a mut Checker) -> Self {
        Self {
            checker,
            scopes: Vec::new(),
            closure_depth: 0,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        let scope = self.scopes.pop().unwrap_or_default();
        for binding in scope {
            if !binding.warn || binding.used {
                continue;
            }
            match binding.kind {
                BindKind::Param => self.checker.warn_coded_at(
                    "W010",
                    format!("unused function parameter '{}'", binding.name),
                    binding.span,
                ),
                BindKind::Local => self.checker.warn_coded_at(
                    "W015",
                    format!("unused local binding '{}'", binding.name),
                    binding.span,
                ),
                BindKind::LoopVar => self.checker.warn_coded_at(
                    "W015",
                    format!("unused loop variable '{}'", binding.name),
                    binding.span,
                ),
                BindKind::Pattern => {}
            }
        }
    }

    /// Declare a binding in the innermost scope. `_` is a wildcard (not a
    /// binding); `_name` is intentionally unused and never warned; bindings
    /// declared inside a closure are tracked for shadowing but not warned
    /// (capture semantics stay conservative).
    fn declare(&mut self, name: &str, span: Span, kind: BindKind, allow_warn: bool) {
        if name == "_" {
            return;
        }
        let warn = allow_warn
            && !name.starts_with('_')
            && self.closure_depth == 0
            && kind != BindKind::Pattern;
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(Binding {
                name: name.to_string(),
                span,
                used: false,
                warn,
                kind,
            });
        }
    }

    /// Resolve a name through the lexical scope stack and mark the nearest
    /// binding used. A use before a same-scope declaration resolves outward,
    /// exactly like the language's lexical scoping.
    fn use_name(&mut self, name: &str) {
        if matches!(name, "_" | "self" | "this" | "Self") {
            return;
        }
        for scope in self.scopes.iter_mut().rev() {
            for binding in scope.iter_mut().rev() {
                if binding.name == name {
                    binding.used = true;
                    return;
                }
            }
        }
    }

    fn walk_fn(&mut self, fd: &FnDecl, lint_params: bool) {
        self.push_scope();
        for param in &fd.params {
            self.declare(&param.name.name, param.name.span, BindKind::Param, lint_params);
        }
        for clause in &fd.contracts {
            match clause {
                xiom_ast::ContractClause::Requires(e, _)
                | xiom_ast::ContractClause::Ensures(e, _) => self.walk_expr(e),
            }
        }
        if let Some(body) = &fd.body {
            self.walk_block(body);
        }
        self.pop_scope();
    }

    fn walk_block(&mut self, block: &Block) {
        self.push_scope();
        for item in &block.stmts {
            match item {
                StmtOrExpr::Stmt(stmt) => self.walk_stmt(stmt),
                StmtOrExpr::Expr(expr) => self.walk_expr(expr),
            }
        }
        self.pop_scope();
    }

    fn walk_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, _, value, _) | Stmt::Var(name, _, value, _) => {
                self.walk_expr(value);
                self.declare(&name.name, name.span, BindKind::Local, true);
            }
            Stmt::Assign(place, value, _) => {
                self.walk_expr(place);
                self.walk_expr(value);
            }
            Stmt::Return(Some(expr), _) => self.walk_expr(expr),
            Stmt::Return(None, _) => {}
            Stmt::Expr(expr, _) => self.walk_expr(expr),
            Stmt::If(cond, then_block, elifs, else_block, _) => {
                self.walk_expr(cond);
                self.walk_block(then_block);
                for (elif_cond, elif_block) in elifs {
                    self.walk_expr(elif_cond);
                    self.walk_block(elif_block);
                }
                if let Some(block) = else_block {
                    self.walk_block(block);
                }
            }
            Stmt::Match(scrutinee, arms, _) => {
                self.walk_expr(scrutinee);
                for arm in arms {
                    self.push_scope();
                    self.declare_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.walk_expr(guard);
                    }
                    match &arm.body {
                        MatchBody::Block(block) => self.walk_block(block),
                        MatchBody::Expr(expr) => self.walk_expr(expr),
                    }
                    self.pop_scope();
                }
            }
            Stmt::While(cond, body, _, _, _) => {
                self.walk_expr(cond);
                self.walk_block(body);
            }
            Stmt::For(var, iter, body, _, _) => {
                self.walk_expr(iter);
                self.push_scope();
                self.declare(&var.name, var.span, BindKind::LoopVar, true);
                self.walk_block(body);
                self.pop_scope();
            }
            Stmt::Spawn(block, _, _) => self.walk_block(block),
            Stmt::Destructure(names, value, _, _) => {
                self.walk_expr(value);
                for name in names {
                    self.declare(&name.name, name.span, BindKind::Local, true);
                }
            }
            Stmt::Break(_, _) | Stmt::Continue(_, _) => {}
            Stmt::Asm(asm) => {
                for (_, expr) in &asm.inputs {
                    self.walk_expr(expr);
                }
                // Outputs WRITE the variable; counting them as uses avoids a
                // false "unused" report for asm-bound locals.
                for (_, ident) in &asm.outputs {
                    self.use_name(&ident.name);
                }
            }
            Stmt::Defer(block, _) => self.walk_block(block),
            Stmt::Assert(cond, msg, _) => {
                self.walk_expr(cond);
                if let Some(msg) = msg {
                    self.walk_expr(msg);
                }
            }
            Stmt::Debugger(_) => {}
        }
    }

    fn walk_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(ident) => self.use_name(&ident.name),
            Expr::Int(..) | Expr::BigInt(..) | Expr::Float(..) | Expr::Str(..)
            | Expr::Char(..) | Expr::Bool(..) | Expr::None(..) | Expr::Error(..) => {}
            Expr::Paren(inner, _)
            | Expr::Unary(_, inner, _)
            | Expr::Try(inner, _)
            | Expr::AtPre(inner, _)
            | Expr::Ref(inner, _)
            | Expr::MutRef(inner, _)
            | Expr::Some(inner, _)
            | Expr::Ok(inner, _)
            | Expr::Err(inner, _)
            | Expr::Await(inner, _)
            | Expr::Comptime(inner, _)
            | Expr::As(inner, _, _)
            | Expr::ConstBlock(inner, _) => self.walk_expr(inner),
            Expr::Binary(left, _, right, _) | Expr::Imply(left, right, _) => {
                self.walk_expr(left);
                self.walk_expr(right);
            }
            Expr::Is(inner, _, _) => self.walk_expr(inner),
            Expr::Field(obj, _, _) => self.walk_expr(obj),
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
                self.walk_expr(func);
                for arg in args {
                    self.walk_expr(arg);
                }
            }
            Expr::Index(base, index, _) => {
                self.walk_expr(base);
                self.walk_expr(index);
            }
            Expr::Struct(_, fields, base, _) => {
                for (_, value) in fields {
                    self.walk_expr(value);
                }
                if let Some(base) = base {
                    self.walk_expr(base);
                }
            }
            Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
                for elem in elems {
                    self.walk_expr(elem);
                }
            }
            Expr::BlockExpr(block, _) | Expr::Unsafe(block, _) => self.walk_block(block),
            Expr::Closure(params, _, body, _) => {
                self.closure_depth += 1;
                self.push_scope();
                for param in params {
                    self.declare(&param.name.name, param.name.span, BindKind::Param, false);
                }
                self.walk_block(body);
                self.pop_scope();
                self.closure_depth -= 1;
            }
            Expr::PipeClosure(params, body, _) => {
                self.closure_depth += 1;
                self.push_scope();
                for param in params {
                    self.declare(&param.name, param.span, BindKind::Param, false);
                }
                self.walk_expr(body);
                self.pop_scope();
                self.closure_depth -= 1;
            }
            Expr::If(cond, then_block, elifs, else_block, _) => {
                self.walk_expr(cond);
                self.walk_block(then_block);
                for (elif_cond, elif_block) in elifs {
                    self.walk_expr(elif_cond);
                    self.walk_block(elif_block);
                }
                if let Some(block) = else_block {
                    self.walk_block(block);
                }
            }
            Expr::Match(scrutinee, arms, _) => {
                self.walk_expr(scrutinee);
                for arm in arms {
                    self.push_scope();
                    self.declare_pattern(&arm.pattern);
                    if let Some(guard) = &arm.guard {
                        self.walk_expr(guard);
                    }
                    match &arm.body {
                        MatchBody::Block(block) => self.walk_block(block),
                        MatchBody::Expr(expr) => self.walk_expr(expr),
                    }
                    self.pop_scope();
                }
            }
        }
    }

    /// Pattern bindings shadow outer names. They are not warned (a match arm
    /// often binds a value for structural completeness), but registering them
    /// keeps the usage analysis lexical and sound.
    fn declare_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard(_) | Pattern::Lit(_) | Pattern::None(_) => {}
            Pattern::Ident(ident) => {
                self.declare(&ident.name, ident.span, BindKind::Pattern, false)
            }
            Pattern::Variant(_, fields, _) => {
                for field in fields {
                    self.declare(&field.name, field.span, BindKind::Pattern, false);
                }
            }
            Pattern::Struct(_, fields, _) => {
                for (_, field) in fields {
                    self.declare_pattern(field);
                }
            }
            Pattern::Tuple(patterns, _) | Pattern::Or(patterns, _) => {
                for inner in patterns {
                    self.declare_pattern(inner);
                }
            }
            Pattern::Some(inner, _) | Pattern::Ok(inner, _) | Pattern::Err(inner, _) => {
                self.declare_pattern(inner)
            }
        }
    }
}

fn walk_items(walker: &mut UsageWalker<'_>, items: &[TopDecl]) {
    for item in items {
        match item {
            TopDecl::Fn(fd) => {
                // W010 is conservative: skip methods (receiver/trait/impl) and
                // any body-less signature. W015 still covers method locals.
                walker.walk_fn(fd, fd.receiver.is_none() && fd.body.is_some());
            }
            TopDecl::Module(module) => walk_items(walker, &module.items),
            TopDecl::Impl(decl) => {
                for member in &decl.members {
                    if let xiom_ast::ImplItem::Fn(fd) = member {
                        walker.walk_fn(fd, false);
                    }
                }
            }
            TopDecl::Spawn(block, _, _) => {
                walker.push_scope();
                walker.walk_block(block);
                walker.pop_scope();
            }
            TopDecl::Use(_)
            | TopDecl::Type(_)
            | TopDecl::Enum(_)
            | TopDecl::Interface(_)
            | TopDecl::Const(_)
            | TopDecl::Extern(_) => {}
        }
    }
}

// ============================================================================
// W011 -- unused import
// ============================================================================

fn lint_unused_imports(checker: &mut Checker, items: &[TopDecl]) {
    let mut refs: HashSet<String> = HashSet::new();
    collect_item_refs(items, &mut refs);

    for item in items {
        if let TopDecl::Use(use_decl) = item {
            check_use(checker, use_decl, &refs);
        }
    }
    for item in items {
        if let TopDecl::Module(module) = item {
            lint_unused_imports(checker, &module.items);
        }
    }
}

fn check_use(checker: &mut Checker, use_decl: &UseDecl, refs: &HashSet<String>) {
    // Re-exports and glob imports are API surface, not local references.
    if use_decl.is_pub || use_decl.glob {
        return;
    }
    let leaf = use_decl
        .alias
        .as_ref()
        .map(|alias| alias.name.as_str())
        .or_else(|| use_decl.path.last().map(|ident| ident.name.as_str()));
    let Some(leaf) = leaf else { return };
    if leaf.is_empty() || leaf == "_" || leaf == "self" {
        return;
    }
    if !refs.contains(leaf) {
        let span = use_decl
            .alias
            .as_ref()
            .map(|alias| alias.span)
            .unwrap_or(use_decl.span);
        checker.warn_coded_at(
            "W011",
            format!("unused import '{leaf}' (the imported name is never referenced)"),
            span,
        );
    }
}

fn collect_item_refs(items: &[TopDecl], refs: &mut HashSet<String>) {
    for item in items {
        match item {
            // Deliberately exclude the use itself: its path/alias is not a
            // reference to the imported binding.
            TopDecl::Use(_) => {}
            TopDecl::Fn(fd) => collect_fn_refs(fd, refs),
            TopDecl::Module(module) => collect_item_refs(&module.items, refs),
            TopDecl::Type(td) => {
                for generic in &td.generics {
                    collect_generic_refs(generic, refs);
                }
                for field in &td.fields {
                    collect_type_refs(&field.ty, refs);
                }
                for (_, ty, expr) in &td.derived_fields {
                    collect_type_refs(ty, refs);
                    collect_expr_refs(expr, refs);
                }
                for invariant in &td.invariants {
                    collect_expr_refs(invariant, refs);
                }
                if let Some(alias) = &td.alias {
                    collect_type_refs(alias, refs);
                }
            }
            TopDecl::Enum(ed) => {
                for generic in &ed.generics {
                    collect_generic_refs(generic, refs);
                }
                for variant in &ed.variants {
                    for field in &variant.fields {
                        collect_type_refs(&field.ty, refs);
                    }
                }
            }
            TopDecl::Interface(id) => {
                for generic in &id.generics {
                    collect_generic_refs(generic, refs);
                }
                if let Some(parent) = &id.parent {
                    refs.insert(parent.name.clone());
                }
                for member in &id.members {
                    match member {
                        xiom_ast::InterfaceMember::Field(field) => {
                            collect_type_refs(&field.ty, refs)
                        }
                        xiom_ast::InterfaceMember::FnSignature(fd) => collect_fn_refs(fd, refs),
                    }
                }
            }
            TopDecl::Const(cd) => {
                collect_type_refs(&cd.ty, refs);
                collect_expr_refs(&cd.value, refs);
            }
            TopDecl::Extern(block) => {
                for fd in &block.functions {
                    collect_fn_refs(fd, refs);
                }
            }
            TopDecl::Impl(decl) => {
                refs.insert(decl.trait_name.name.clone());
                refs.insert(decl.type_name.name.clone());
                for ty in &decl.trait_args {
                    collect_type_refs(ty, refs);
                }
                for member in &decl.members {
                    match member {
                        xiom_ast::ImplItem::Fn(fd) => collect_fn_refs(fd, refs),
                        xiom_ast::ImplItem::Const(cd) => {
                            collect_type_refs(&cd.ty, refs);
                            collect_expr_refs(&cd.value, refs);
                        }
                    }
                }
            }
            TopDecl::Spawn(block, _, _) => collect_block_refs(block, refs),
        }
    }
}

fn collect_generic_refs(generic: &xiom_ast::GenericParam, refs: &mut HashSet<String>) {
    for bound in &generic.bounds {
        refs.insert(bound.name.clone());
    }
    if let Some(ty) = &generic.const_ty {
        collect_type_refs(ty, refs);
    }
}

fn collect_fn_refs(fd: &FnDecl, refs: &mut HashSet<String>) {
    // A method receiver (`fn Counter.bump`) is a reference to the receiver
    // TYPE, so an import used only as a receiver must not read as unused.
    if let Some(receiver) = &fd.receiver {
        refs.insert(receiver.name.clone());
    }
    for generic in &fd.generics {
        collect_generic_refs(generic, refs);
    }
    for param in &fd.params {
        collect_type_refs(&param.ty, refs);
    }
    if let Some(ret) = &fd.return_type {
        collect_type_refs(ret, refs);
    }
    for clause in &fd.contracts {
        match clause {
            xiom_ast::ContractClause::Requires(e, _)
            | xiom_ast::ContractClause::Ensures(e, _) => collect_expr_refs(e, refs),
        }
    }
    if let Some(body) = &fd.body {
        collect_block_refs(body, refs);
    }
}

fn collect_block_refs(block: &Block, refs: &mut HashSet<String>) {
    for item in &block.stmts {
        match item {
            StmtOrExpr::Stmt(stmt) => collect_stmt_refs(stmt, refs),
            StmtOrExpr::Expr(expr) => collect_expr_refs(expr, refs),
        }
    }
}

fn collect_stmt_refs(stmt: &Stmt, refs: &mut HashSet<String>) {
    match stmt {
        Stmt::Let(_, ty, value, _) | Stmt::Var(_, ty, value, _) => {
            if let Some(ty) = ty {
                collect_type_refs(ty, refs);
            }
            collect_expr_refs(value, refs);
        }
        Stmt::Assign(place, value, _) => {
            collect_expr_refs(place, refs);
            collect_expr_refs(value, refs);
        }
        Stmt::Return(Some(expr), _) => collect_expr_refs(expr, refs),
        Stmt::Return(None, _) => {}
        Stmt::Expr(expr, _) => collect_expr_refs(expr, refs),
        Stmt::If(cond, then_block, elifs, else_block, _) => {
            collect_expr_refs(cond, refs);
            collect_block_refs(then_block, refs);
            for (elif_cond, elif_block) in elifs {
                collect_expr_refs(elif_cond, refs);
                collect_block_refs(elif_block, refs);
            }
            if let Some(block) = else_block {
                collect_block_refs(block, refs);
            }
        }
        Stmt::Match(scrutinee, arms, _) => {
            collect_expr_refs(scrutinee, refs);
            for arm in arms {
                collect_pattern_refs(&arm.pattern, refs);
                if let Some(guard) = &arm.guard {
                    collect_expr_refs(guard, refs);
                }
                match &arm.body {
                    MatchBody::Block(block) => collect_block_refs(block, refs),
                    MatchBody::Expr(expr) => collect_expr_refs(expr, refs),
                }
            }
        }
        Stmt::While(cond, body, invariant, _, _) => {
            collect_expr_refs(cond, refs);
            collect_block_refs(body, refs);
            if let Some(invariant) = invariant {
                collect_expr_refs(invariant, refs);
            }
        }
        Stmt::For(_, iter, body, _, _) => {
            collect_expr_refs(iter, refs);
            collect_block_refs(body, refs);
        }
        Stmt::Spawn(block, _, _) => collect_block_refs(block, refs),
        Stmt::Destructure(_, value, _, _) => collect_expr_refs(value, refs),
        Stmt::Break(_, _) | Stmt::Continue(_, _) => {}
        Stmt::Asm(asm) => {
            for (_, expr) in &asm.inputs {
                collect_expr_refs(expr, refs);
            }
            for (_, ident) in &asm.outputs {
                refs.insert(ident.name.clone());
            }
        }
        Stmt::Defer(block, _) => collect_block_refs(block, refs),
        Stmt::Assert(cond, msg, _) => {
            collect_expr_refs(cond, refs);
            if let Some(msg) = msg {
                collect_expr_refs(msg, refs);
            }
        }
        Stmt::Debugger(_) => {}
    }
}

fn collect_expr_refs(expr: &Expr, refs: &mut HashSet<String>) {
    match expr {
        Expr::Ident(ident) => {
            refs.insert(ident.name.clone());
        }
        Expr::Int(..) | Expr::BigInt(..) | Expr::Float(..) | Expr::Str(..)
        | Expr::Char(..) | Expr::Bool(..) | Expr::None(..) | Expr::Error(..) => {}
        Expr::Paren(inner, _)
        | Expr::Unary(_, inner, _)
        | Expr::Try(inner, _)
        | Expr::AtPre(inner, _)
        | Expr::Ref(inner, _)
        | Expr::MutRef(inner, _)
        | Expr::Some(inner, _)
        | Expr::Ok(inner, _)
        | Expr::Err(inner, _)
        | Expr::Await(inner, _)
        | Expr::Comptime(inner, _)
        | Expr::As(inner, _, _)
        | Expr::ConstBlock(inner, _) => collect_expr_refs(inner, refs),
        Expr::Binary(left, _, right, _) | Expr::Imply(left, right, _) => {
            collect_expr_refs(left, refs);
            collect_expr_refs(right, refs);
        }
        Expr::Is(inner, pattern, _) => {
            collect_expr_refs(inner, refs);
            collect_pattern_refs(pattern, refs);
        }
        Expr::Field(obj, field, _) => {
            collect_expr_refs(obj, refs);
            // Conservative: a same-named field/method counts as a reference
            // too (never turns a used import into a false positive).
            refs.insert(field.name.clone());
        }
        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
            collect_expr_refs(func, refs);
            for arg in args {
                collect_expr_refs(arg, refs);
            }
        }
        Expr::Index(base, index, _) => {
            collect_expr_refs(base, refs);
            collect_expr_refs(index, refs);
        }
        Expr::Struct(name, fields, base, _) => {
            refs.insert(name.name.clone());
            for (field, value) in fields {
                refs.insert(field.name.clone());
                collect_expr_refs(value, refs);
            }
            if let Some(base) = base {
                collect_expr_refs(base, refs);
            }
        }
        Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
            for elem in elems {
                collect_expr_refs(elem, refs);
            }
        }
        Expr::BlockExpr(block, _) | Expr::Unsafe(block, _) => collect_block_refs(block, refs),
        Expr::Closure(params, ret, body, _) => {
            for param in params {
                collect_type_refs(&param.ty, refs);
            }
            if let Some(ret) = ret {
                collect_type_refs(ret, refs);
            }
            collect_block_refs(body, refs);
        }
        Expr::PipeClosure(params, body, _) => {
            for param in params {
                refs.insert(param.name.clone());
            }
            collect_expr_refs(body, refs);
        }
        Expr::If(cond, then_block, elifs, else_block, _) => {
            collect_expr_refs(cond, refs);
            collect_block_refs(then_block, refs);
            for (elif_cond, elif_block) in elifs {
                collect_expr_refs(elif_cond, refs);
                collect_block_refs(elif_block, refs);
            }
            if let Some(block) = else_block {
                collect_block_refs(block, refs);
            }
        }
        Expr::Match(scrutinee, arms, _) => {
            collect_expr_refs(scrutinee, refs);
            for arm in arms {
                collect_pattern_refs(&arm.pattern, refs);
                if let Some(guard) = &arm.guard {
                    collect_expr_refs(guard, refs);
                }
                match &arm.body {
                    MatchBody::Block(block) => collect_block_refs(block, refs),
                    MatchBody::Expr(expr) => collect_expr_refs(expr, refs),
                }
            }
        }
    }
}

fn collect_pattern_refs(pattern: &Pattern, refs: &mut HashSet<String>) {
    match pattern {
        Pattern::Wildcard(_) | Pattern::Lit(_) | Pattern::None(_) => {}
        Pattern::Ident(ident) => {
            refs.insert(ident.name.clone());
        }
        Pattern::Variant(name, fields, _) => {
            refs.insert(name.name.clone());
            for field in fields {
                refs.insert(field.name.clone());
            }
        }
        Pattern::Struct(name, fields, _) => {
            refs.insert(name.name.clone());
            for (field, inner) in fields {
                refs.insert(field.name.clone());
                collect_pattern_refs(inner, refs);
            }
        }
        Pattern::Tuple(patterns, _) | Pattern::Or(patterns, _) => {
            for inner in patterns {
                collect_pattern_refs(inner, refs);
            }
        }
        Pattern::Some(inner, _) | Pattern::Ok(inner, _) | Pattern::Err(inner, _) => {
            collect_pattern_refs(inner, refs)
        }
    }
}

fn collect_type_refs(ty: &Type, refs: &mut HashSet<String>) {
    match ty {
        Type::Named(name, args) => {
            // Qualified type spellings (`pkg.T`) reference their leaf too.
            for segment in name.name.split('.') {
                refs.insert(segment.to_string());
            }
            for arg in args {
                collect_type_refs(arg, refs);
            }
        }
        Type::Ref(inner)
        | Type::MutRef(inner)
        | Type::Option(inner)
        | Type::Vec(inner)
        | Type::Slice(inner)
        | Type::Set(inner)
        | Type::Ptr(inner) => collect_type_refs(inner, refs),
        Type::Result(ok, err) => {
            collect_type_refs(ok, refs);
            collect_type_refs(err, refs);
        }
        Type::Map(key, value) => {
            collect_type_refs(key, refs);
            collect_type_refs(value, refs);
        }
        Type::Tuple(elems) => {
            for elem in elems {
                collect_type_refs(elem, refs);
            }
        }
        Type::Array(len, elem) => {
            collect_expr_refs(len, refs);
            collect_type_refs(elem, refs);
        }
        Type::Fn(params, ret) => {
            for param in params {
                collect_type_refs(param, refs);
            }
            collect_type_refs(ret, refs);
        }
        Type::ImplTrait(traits) => {
            for tr in traits {
                refs.insert(tr.name.clone());
            }
        }
        Type::AnonStruct(fields) => {
            for field in fields {
                refs.insert(field.name.name.clone());
                collect_type_refs(&field.ty, refs);
            }
        }
        Type::Never => {}
    }
}
