// XIOM -- Lowering Passes
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Semantic LOWERING passes. Home of `expand_impl_blocks`, which used to
//! live inside xiom-ast (a LAYERING VIOLATION -- audited: the AST crate
//! performed semantic transformation, injected Span(0,0) dummy spans, and
//! cloned the whole program per call). The AST crate now holds ONLY
//! syntax; lowering lives here as an explicit pipeline stage between
//! parse and check.

use xiom_ast::*;
/// M20: Expand impl blocks into freestanding functions.
/// `impl Trait for Type { fn m() { body } }` becomes `fn Type.m() { body }`.
/// M22: Recurse into modules so impl blocks inside `module { ... }` are expanded.
pub fn expand_impl_blocks(program: &Program) -> Program {
    // Helper: rewrite bare method calls (e.g. `value()`) to `self.value()`
    // in interface default bodies. This ensures the expanded inherent method
    // uses proper self.method() syntax for calls to other interface methods.
    fn rewrite_bare_calls(block: &mut Block, iface_methods: &std::collections::HashSet<String>, span: Span) {
        for se in &mut block.stmts {
            match se {
                StmtOrExpr::Stmt(s) => rewrite_stmt(s, iface_methods, span),
                StmtOrExpr::Expr(e) => rewrite_expr(e, iface_methods, span),
            }
        }
    }
    fn rewrite_stmt(stmt: &mut Stmt, iface_methods: &std::collections::HashSet<String>, span: Span) {
        match stmt {
            Stmt::Expr(e, _) | Stmt::Return(Some(e), _) => rewrite_expr(e, iface_methods, span),
            Stmt::Var(_, _, e, _) | Stmt::Let(_, _, e, _) => rewrite_expr(e, iface_methods, span),
            Stmt::Assign(_, e, _) => rewrite_expr(e, iface_methods, span),
            Stmt::While(cond, body, _, _, _) => { rewrite_expr(cond, iface_methods, span); rewrite_bare_calls(body, iface_methods, span); }
            Stmt::If(cond, tb, elifs, eb, _) => {
                rewrite_expr(cond, iface_methods, span);
                rewrite_bare_calls(tb, iface_methods, span);
                for (ec, eb2) in elifs { rewrite_expr(ec, iface_methods, span); rewrite_bare_calls(eb2, iface_methods, span); }
                if let Some(eb3) = eb { rewrite_bare_calls(eb3, iface_methods, span); }
            }
            Stmt::Match(sc, arms, _) => {
                rewrite_expr(sc, iface_methods, span);
                for arm in arms {
                    if let Some(ref mut g) = arm.guard { rewrite_expr(g, iface_methods, span); }
                    match &mut arm.body {
                        MatchBody::Block(b) => rewrite_bare_calls(b, iface_methods, span),
                        MatchBody::Expr(e) => rewrite_expr(e, iface_methods, span),
                    }
                }
            }
            Stmt::For(_, iter, body, _, _) => { rewrite_expr(iter, iface_methods, span); rewrite_bare_calls(body, iface_methods, span); }
            Stmt::Spawn(body, _, _move) => rewrite_bare_calls(body, iface_methods, span),
            _ => {}
        }
    }
    fn rewrite_expr(expr: &mut Expr, iface_methods: &std::collections::HashSet<String>, span: Span) {
        match expr {
            Expr::Call(func, args, _) => {
                // Rewrite bare `method()` to `self.method()`
                if let Expr::Ident(id) = func.as_ref() {
                    if iface_methods.contains(&id.name) && id.name != "self" {
                        *func = Box::new(Expr::Field(
                            Box::new(Expr::Ident(Ident { name: "self".to_string(), span })),
                            Ident { name: id.name.clone(), span }, span,
                        ));
                    }
                }
                rewrite_expr(func, iface_methods, span);
                for arg in args { rewrite_expr(arg, iface_methods, span); }
            }
            Expr::Binary(a, _, b, _) | Expr::Imply(a, b, _) => {
                rewrite_expr(a, iface_methods, span); rewrite_expr(b, iface_methods, span);
            }
            Expr::Unary(_, e, _) | Expr::Ref(e, _) | Expr::MutRef(e, _)
            | Expr::Paren(e, _) | Expr::Try(e, _) | Expr::AtPre(e, _)
            | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _)
            | Expr::As(e, _, _) => rewrite_expr(e, iface_methods, span),
            Expr::Field(obj, _, _) => rewrite_expr(obj, iface_methods, span),
            Expr::Index(arr, idx, _) => { rewrite_expr(arr, iface_methods, span); rewrite_expr(idx, iface_methods, span); }
            Expr::Struct(_, fields, base, _) => {
                for (_, v) in fields { rewrite_expr(v, iface_methods, span); }
                if let Some(b) = base { rewrite_expr(b, iface_methods, span); }
            }
            Expr::If(c, t, elifs, els, _) => {
                rewrite_expr(c, iface_methods, span);
                rewrite_bare_calls(t, iface_methods, span);
                for (ec, eb) in elifs { rewrite_expr(ec, iface_methods, span); rewrite_bare_calls(eb, iface_methods, span); }
                if let Some(eb) = els { rewrite_bare_calls(eb, iface_methods, span); }
            }
            Expr::Match(sc, arms, _) => {
                rewrite_expr(sc, iface_methods, span);
                for arm in arms {
                    if let Some(ref mut g) = arm.guard { rewrite_expr(g, iface_methods, span); }
                    match &mut arm.body {
                        MatchBody::Block(b) => rewrite_bare_calls(b, iface_methods, span),
                        MatchBody::Expr(e) => rewrite_expr(e, iface_methods, span),
                    }
                }
            }
            Expr::Is(e, _, _) => rewrite_expr(e, iface_methods, span),
            Expr::Array(items, _) => for e in items { rewrite_expr(e, iface_methods, span); },
            Expr::Closure(_, _, _body, _) => {
                // Don't recurse into closures -- they have their own scope
            }
            _ => {}
        }
    }
    // Helper: collect interface defaults recursively (including inside modules).
    fn collect_interface_defaults(items: &[TopDecl]) -> std::collections::HashMap<String, Vec<(String, FnDecl)>> {
        let mut defaults: std::collections::HashMap<String, Vec<(String, FnDecl)>> = std::collections::HashMap::new();
        for item in items {
            match item {
                TopDecl::Interface(id) => {
                    let mut methods = Vec::new();
                    for member in &id.members {
                        if let InterfaceMember::FnSignature(fd) = member {
                            if fd.body.is_some() {
                                methods.push((fd.name.name.clone(), fd.clone()));
                            }
                        }
                    }
                    if !methods.is_empty() {
                        defaults.insert(id.name.name.clone(), methods);
                    }
                }
                TopDecl::Module(md) => {
                    let nested = collect_interface_defaults(&md.items);
                    for (k, v) in nested {
                        if !defaults.contains_key(&k) { defaults.insert(k, v); }
                    }
                }
                _ => {}
            }
        }
        defaults
    }

    // Helper: collect interface REQUIRED methods (those WITHOUT body).
    fn collect_interface_required(items: &[TopDecl]) -> std::collections::HashMap<String, Vec<String>> {
        let mut required: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for item in items {
            match item {
                TopDecl::Interface(id) => {
                    let methods: Vec<String> = id.members.iter()
                        .filter_map(|m| if let InterfaceMember::FnSignature(fd) = m {
                            if fd.body.is_none() { Some(fd.name.name.clone()) } else { None }
                        } else { None })
                        .collect();
                    if !methods.is_empty() {
                        required.insert(id.name.name.clone(), methods);
                    }
                }
                TopDecl::Module(md) => {
                    let nested = collect_interface_required(&md.items);
                    for (k, v) in nested {
                        if !required.contains_key(&k) { required.insert(k, v); }
                    }
                }
                _ => {}
            }
        }
        required
    }

    // Helper: collect inherent methods per type from fn items with receivers.
    // `fn TypeName.method(...)` has receiver TypeName and name `TypeName.method`
    // or just `method`. Extract the bare method name (after any dot).
    fn collect_inherent_methods(items: &[TopDecl]) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
        let mut methods: std::collections::HashMap<String, std::collections::HashSet<String>> = std::collections::HashMap::new();
        for item in items {
            match item {
                TopDecl::Fn(fd) => {
                    if let Some(ref receiver) = fd.receiver {
                        let type_name = receiver.name.clone();
                        let method_name = fd.name.name.rsplit('.').next().unwrap_or(&fd.name.name).to_string();
                        methods.entry(type_name).or_default().insert(method_name);
                    }
                }
                TopDecl::Module(md) => {
                    let nested = collect_inherent_methods(&md.items);
                    for (k, v) in nested {
                        methods.entry(k).or_default().extend(v);
                    }
                }
                _ => {}
            }
        }
        methods
    }

    // Collect ALL declared type names (TypeDecl + EnumDecl), recursing into
    // modules, so the auto-detect pass knows about types with zero inherent
    // methods (e.g. `type Cat = {}` that should still receive interface defaults).
    fn collect_declared_types(items: &[TopDecl]) -> Vec<String> {
        let mut names = Vec::new();
        for item in items {
            match item {
                TopDecl::Type(td) => { names.push(td.name.name.clone()); }
                TopDecl::Enum(ed) => { names.push(ed.name.name.clone()); }
                TopDecl::Module(md) => {
                    names.extend(collect_declared_types(&md.items));
                }
                _ => {}
            }
        }
        names
    }

    let interface_defaults = collect_interface_defaults(&program.items);
    let interface_required = collect_interface_required(&program.items);
    let mut inherent_methods = collect_inherent_methods(&program.items);
    // Merge bare type declarations (types with NO inherent methods) into
    // the map so auto-detect considers them for interface defaults.
    for type_name in collect_declared_types(&program.items) {
        inherent_methods.entry(type_name).or_default();
    }

    // Helper: expand impl blocks in a list of items, recursing into modules.
    // Also auto-detects types that satisfy interfaces through inherent methods.
    fn expand_items(
        items: &[TopDecl],
        interface_defaults: &std::collections::HashMap<String, Vec<(String, FnDecl)>>,
        interface_required: &std::collections::HashMap<String, Vec<String>>,
        inherent_methods: &std::collections::HashMap<String, std::collections::HashSet<String>>,
    ) -> Vec<TopDecl> {
        let mut out = Vec::new();
        let mut seen_impls: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
        for item in items {
            match item {
                TopDecl::Impl(impl_decl) => {
                    // D1: `impl Num[Int]` (no `for Type`) -- the implementing
                    // type is the first trait arg. `impl Trait for Type` uses
                    // type_name directly.
                    let type_name = if impl_decl.type_name.name != "_" {
                        impl_decl.type_name.name.clone()
                    } else if let Some(first_arg) = impl_decl.trait_args.first() {
                        type_name_from_ast(first_arg)
                    } else {
                        impl_decl.type_name.name.clone()
                    };
                    // D1: `impl Trait for Type` methods get a receiver
                    // (`fn Type.method(self, ...)`); `impl Trait[Args]`
                    // methods are STATIC (no self -- the arg IS the impl
                    // type). Only set the receiver for the `for` form.
                    let has_for_type = impl_decl.type_name.name != "_";
                    let iface_name = impl_decl.trait_name.name.clone();
                    seen_impls.insert((type_name.clone(), iface_name.clone()));
                    let mut provided_methods: std::collections::HashSet<String> = std::collections::HashSet::new();
                    for member in &impl_decl.members {
                        if let ImplItem::Fn(fn_decl) = member {
                            let mut new_fn = fn_decl.clone();
                            new_fn.name = Ident { name: format!("{}.{}", type_name, fn_decl.name.name), span: fn_decl.name.span };
                            if has_for_type {
                                new_fn.receiver = Some(Ident { name: type_name.clone(), span: impl_decl.type_name.span });
                            }
                            provided_methods.insert(fn_decl.name.name.clone());
                            out.push(TopDecl::Fn(new_fn));
                        }
                    }
                    // M19: Fill in default methods from the interface that weren't provided
                    if let Some(defaults) = interface_defaults.get(&iface_name) {
                        for (method_name, default_fd) in defaults {
                            if provided_methods.contains(method_name) { continue; }
                            let mut new_fn = default_fd.clone();
                            new_fn.name = Ident { name: format!("{}.{}", type_name, method_name), span: impl_decl.span };
                            if has_for_type {
                                new_fn.receiver = Some(Ident { name: type_name.clone(), span: impl_decl.type_name.span });
                            }
                            out.push(TopDecl::Fn(new_fn));
                        }
                    }
                }
                TopDecl::Module(md) => {
                    // M22: Recurse into module to expand nested impl blocks
                    let expanded_items = expand_items(&md.items, interface_defaults, interface_required, inherent_methods);
                    out.push(TopDecl::Module(ModuleDecl {
                        name: md.name.clone(), path: md.path.clone(),
                        items: expanded_items, is_file_level: md.is_file_level,
                        source_file: md.source_file.clone(), span: md.span,
                    }));
                }
                other => {
                    out.push(other.clone());
                }
            }
        }
        // Auto-detect: for each interface, find types with inherent methods
        // matching all required methods, and expand default methods.
        // Collect already-emitted function names (recursing into modules)
        // to prevent duplicates when expand_impl_blocks is called multiple times.
        fn collect_fn_names(items: &[TopDecl]) -> std::collections::HashSet<String> {
            let mut names = std::collections::HashSet::new();
            for item in items {
                match item {
                    TopDecl::Fn(fd) => { names.insert(fd.name.name.clone()); }
                    TopDecl::Module(md) => { names.extend(collect_fn_names(&md.items)); }
                    _ => {}
                }
            }
            names
        }
        let already_emitted = collect_fn_names(&out);
        // Auto-detect: iterate over all interfaces that have defaults.
        // Interfaces with only default methods (no required) still need
        // expansion for every type (e.g. `interface Greeter { fn greet() -> Str { return "hello"; } }`).
        for (iface_name, _defaults) in interface_defaults.iter() {
            let required_methods = interface_required.get(iface_name).cloned().unwrap_or_default();
            for (type_name, type_methods) in inherent_methods {
                // Skip if explicit impl already exists
                if seen_impls.contains(&(type_name.clone(), iface_name.clone())) { continue; }
                // Check if type has ALL required methods as inherent methods
                let all_required_present = required_methods.iter()
                    .all(|rm| type_methods.contains(rm));
                if all_required_present {
                    // Auto-expand default methods for this type+interface pair
                    if let Some(defaults) = interface_defaults.get(iface_name) {
                        for (method_name, default_fd) in defaults {
                            // Skip if type already has this method
                            if type_methods.contains(method_name) { continue; }
                            let fn_name = format!("{}.{}", type_name, method_name);
                            // Skip if already emitted (e.g. from previous expand_impl_blocks call)
                            if already_emitted.contains(&fn_name) { continue; }
                            let ds = default_fd.name.span;
                            let mut new_fn = default_fd.clone();
                            // Use the type-qualified name (matching explicit impl expansion).
                            // fn_key strips the receiver prefix to avoid doubling.
                            new_fn.name = Ident { name: format!("{}.{}", type_name, method_name), span: ds };
                            new_fn.receiver = Some(Ident { name: type_name.clone(), span: ds });
                            // Rewrite bare method calls to self.method() in the default body
                            // Build set of all interface method names for rewriting
                            let mut all_iface_methods: std::collections::HashSet<String> = std::collections::HashSet::new();
                            if let Some(req) = interface_required.get(iface_name) {
                                for m in req { all_iface_methods.insert(m.clone()); }
                            }
                            if let Some(defs) = interface_defaults.get(iface_name) {
                                for (m, _) in defs { all_iface_methods.insert(m.clone()); }
                            }
                            if let Some(ref mut body) = new_fn.body {
                                rewrite_bare_calls(body, &all_iface_methods, ds);
                            }
                            out.push(TopDecl::Fn(new_fn));
                        }
                    }
                }
            }
        }
        out
    }

    let items = expand_items(&program.items, &interface_defaults, &interface_required, &inherent_methods);
    Program { items, source_files: program.source_files.clone(), root_dir: program.root_dir.clone(), span: program.span }
}
