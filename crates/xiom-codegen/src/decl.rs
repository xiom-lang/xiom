// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{IrEmitter, TypeMeta};
use crate::context::{TypeContext, SyncRegistry};
use xiom_ast::*;
use std::collections::HashMap;
use std::collections::HashSet;

impl IrEmitter {
    pub(crate) fn register_type_layout(&mut self, item: &TopDecl) {
        self.register_type_layout_impl(item, "");
    }

    pub(crate) fn register_type_layout_impl(&mut self, item: &TopDecl, prefix: &str) {
        if let TopDecl::Type(td) = item {
            let bare_name = td.name.name.clone();
            let type_name = if prefix.is_empty() { bare_name.clone() } else { format!("{}.{}", prefix, bare_name) };
            // M36: Register type aliases so codegen can resolve them to concrete types.
            if td.fields.is_empty() {
                if let Some(ref alias_ty) = td.alias {
                    let resolved = Self::type_from_ast(alias_ty);
                    self.types.type_aliases.insert(type_name.clone(), resolved.clone());
                    // Also register under bare name for unqualified lookup
                    if !prefix.is_empty() {
                        self.types.type_aliases.insert(bare_name.clone(), resolved);
                    }
                    return;
                }
                // Empty struct (no fields, no alias) -- still register as a type
                // so it resolves in LLVM type lookups. Uses a sentinel field.
                self.types.types.or_insert_with(type_name.clone(), || vec!["__xiom_empty".to_string()]);
                self.types.type_meta.or_insert_with(type_name, || TypeMeta {
                    fields: vec![("__xiom_empty".to_string(), "Int".to_string())],
                    derives: vec![],
                    invariants: vec![],
                });
                return;
            }
            // Record generic type names so their methods are skipped from direct
            // (un-monomorphised) emission -- such bodies produce malformed IR.
            if !td.generics.is_empty() {
                self.types.generic_type_names.insert(bare_name.clone());
                self.types.generic_type_names.insert(type_name.clone());
            }
            let fields: Vec<String> = td.fields.iter()
                .map(|f| f.name.name.clone())
                .collect();
            let full_fields: Vec<(String, String)> = td.fields.iter()
                .map(|f| (f.name.name.clone(), Self::type_from_ast_with_args(&f.ty)))
                .collect();
            // BUG 52 (2026-08-18): keep the generic-args field types for
            // GENERIC type decls -- the builtin Map/Set pre-registrations win
            // type_meta with bare "Vec" field types, losing "Vec[K]"/"Vec[V]".
            // mono'd method bodies need them to substitute the concrete args.
            if !td.generics.is_empty() {
                self.types.generic_type_field_types.insert(bare_name.clone(), full_fields.clone());
                if !prefix.is_empty() {
                    self.types.generic_type_field_types.insert(type_name.clone(), full_fields.clone());
                }
                // M65 Part 2: keep the declared parameter ORDER so field
                // types keeping generic args ("Vec[V]") can be substituted
                // from a concrete instantiation ("Map[Str, JsonValue]").
                let param_names: Vec<String> = td.generics.iter()
                    .map(|g| g.name.name.clone())
                    .collect();
                self.types.generic_type_params.insert(bare_name.clone(), param_names.clone());
                if !prefix.is_empty() {
                    self.types.generic_type_params.insert(type_name.clone(), param_names);
                }
            }
            self.types.types.or_insert_with(type_name.clone(), || fields);
            // Use or_insert_with so manual pre-registrations (e.g. Map with
            // resolved Vec type names) are not overwritten by the generic
            // type definition (which uses Vec[K] with unresolved generics).
            self.types.type_meta.or_insert_with(type_name, || TypeMeta {
                fields: full_fields,
                derives: td.derives.clone(),
                invariants: td.invariants.clone(),
            });
            // Register nested tuple types used in struct fields so codegen
            // can resolve them (e.g. Vec[(Str,Str)] needs Tuple__Str__Str).
            for f in &td.fields {
                self.ensure_tuple_type_registered(&f.ty);
            }
        }
        if let TopDecl::Enum(ed) = item {
            let bare_name = ed.name.name.clone();
            let enum_name = if prefix.is_empty() { bare_name.clone() } else { format!("{}.{}", prefix, bare_name) };
            if self.types.types.contains_key(&enum_name) && !self.types.types.get(&enum_name).map(|f| f.is_empty()).unwrap_or(true) {
                return;
            }
            let mut all_fields = vec!["discriminant".to_string()];
            let mut all_meta = vec![("discriminant".to_string(), "Int".to_string())];
            let mut variants_info = Vec::new();
            let mut variants_types = Vec::new();
            for variant in &ed.variants {
                let vname = variant.name.name.clone();
                let mut vfields = Vec::new();
                let mut vftypes = Vec::new();
                for field in &variant.fields {
                    let fname = field.name.name.clone();
                    if !all_fields.contains(&fname) {
                        all_fields.push(fname.clone());
                        all_meta.push((fname.clone(), Self::type_from_ast(&field.ty)));
                    } else {
                        // M19: When a field name collides across variants (e.g.,
                        // `Bool(val)` and `String(val)`), the LLVM struct slot
                        // already exists with the first variant's type. Force the
                        // type_meta entry to Int (i64) so all variant payloads
                        // are stored as bitcast/ptrtoint and correctly decoded
                        // via enum_variant_field_types at runtime.
                        if let Some(existing) = all_meta.iter_mut().find(|(n, _)| n == &fname) {
                            existing.1 = "Int".to_string();
                        }
                    }
                    vfields.push(fname);
                    // Keep generic args (Vec[JsonValue]) -- 5c.30 handle detection.
                    vftypes.push(Self::type_from_ast_with_args(&field.ty));
                }
                variants_info.push((vname.clone(), vfields));
                variants_types.push((vname, vftypes));
            }
            self.types.types.insert(enum_name.clone(), all_fields);
            self.types.type_meta.insert(enum_name.clone(), TypeMeta {
                fields: all_meta,
                derives: ed.derives.clone(),
                invariants: Vec::new(),
            });
            self.types.enum_variants.insert(enum_name.clone(), variants_info);
            self.types.enum_variant_field_types.insert(enum_name.clone(), variants_types);
            // R30: keep declaration order for the checker-consistent bare
            // variant parent pick.
            if !self.types.enum_decl_order.contains(&enum_name) {
                self.types.enum_decl_order.push(enum_name.clone());
            }
        }
        if let TopDecl::Module(md) = item {
            let new_prefix = if prefix.is_empty() { md.name.name.clone() } else { format!("{}.{}", prefix, md.name.name) };
            for sub in &md.items {
                self.register_type_layout_impl(sub, &new_prefix);
            }
        }
    }

    pub(crate) fn ensure_tuple_type_registered(&mut self, ty: &Type) {
        match ty {
            Type::Tuple(elems) => {
                let name = Self::type_from_ast(ty);
                if self.types.type_meta.contains_key(&name) { return; }
                let field_names: Vec<String> = (0..elems.len()).map(|i| format!("_{i}")).collect();
                let field_types: Vec<(String, String)> = elems.iter()
                    .enumerate()
                    .map(|(i, t)| (format!("_{i}"), Self::type_from_ast(t)))
                    .collect();
                self.types.types.insert(name.clone(), field_names);
                self.types.type_meta.insert(name, TypeMeta {
                    fields: field_types,
                    derives: vec![],
                    invariants: vec![],
                });
                for elem in elems {
                    self.ensure_tuple_type_registered(elem);
                }
            }
            Type::Ref(inner) | Type::MutRef(inner) | Type::Option(inner) | Type::Vec(inner) |
            Type::Slice(inner) | Type::Set(inner) | Type::Ptr(inner) => {
                self.ensure_tuple_type_registered(inner);
            }
            Type::Result(ok, err) => {
                self.ensure_tuple_type_registered(ok);
                self.ensure_tuple_type_registered(err);
            }
            Type::Map(k, v) => {
                self.ensure_tuple_type_registered(k);
                self.ensure_tuple_type_registered(v);
            }
            Type::Fn(params, ret) => {
                for p in params { self.ensure_tuple_type_registered(p); }
                self.ensure_tuple_type_registered(ret);
            }
            _ => {}
        }
    }

    pub(crate) fn substitute_concrete_name(ty: &Type, type_map: &HashMap<String, String>) -> String {
        match ty {
            Type::Named(id, _) => type_map.get(&id.name).cloned().unwrap_or_else(|| id.name.clone()),
            Type::Tuple(elems) => {
                let parts: Vec<String> = elems.iter()
                    .map(|e| Self::substitute_concrete_name(e, type_map))
                    .collect();
                format!("Tuple_{}", parts.join("_"))
            }
            _ => Self::type_from_ast(ty),
        }
    }

    /// Rebuild a pointer/ref `Type` with its inner generic name substituted by the
    /// concrete type from `type_map`, preserving the `Ptr`/`MutRef`/`Ref` wrapper so
    /// `type_from_ast` still produces a `*Inner` name. `outer` is the wrapper node
    /// and `inner` its boxed inner type. Only the inner Named leaf is remapped.
    pub(crate) fn substitute_type(outer: &Type, inner: &Type, type_map: &HashMap<String, String>) -> Type {
        let new_inner: Type = match inner {
            Type::Named(id, args) => {
                if let Some(ct) = type_map.get(&id.name) {
                    Type::Named(Ident::new(ct, id.span), args.clone())
                } else {
                    // round-13 (tuple payloads): recurse into the type ARGS
                    // ("Vec[(Int, T)]" -- the Tuple arg's T must substitute).
                    // The old code kept args untouched, so generic tuple
                    // returns ("Option[(Int, T)]" from EnumerateIter.next)
                    // never concretized and the payload/vec-elem tracking
                    // dropped them.
                    let new_args: Vec<Type> = args.iter()
                        .map(|a| Self::substitute_type(a, a, type_map))
                        .collect();
                    Type::Named(id.clone(), new_args)
                }
            }
            // Recursively substitute type params in composite types.
            Type::Array(size, elem) => {
                Type::Array(size.clone(), Box::new(Self::substitute_type(inner, elem, type_map)))
            }
            Type::Tuple(elems) => Type::Tuple(
                elems.iter().map(|e| Self::substitute_type(e, e, type_map)).collect()
            ),
            Type::Option(e) => Type::Option(Box::new(Self::substitute_type(inner, e, type_map))),
            Type::Result(ok, err) => Type::Result(
                Box::new(Self::substitute_type(inner, ok, type_map)),
                Box::new(Self::substitute_type(inner, err, type_map)),
            ),
            Type::Vec(e) => Type::Vec(Box::new(Self::substitute_type(inner, e, type_map))),
            Type::Map(k, v) => Type::Map(
                Box::new(Self::substitute_type(inner, k, type_map)),
                Box::new(Self::substitute_type(inner, v, type_map)),
            ),
            Type::Set(e) => Type::Set(Box::new(Self::substitute_type(inner, e, type_map))),
            // smoke_ptr_offset fix (2026-09-11): substitute the POINTEE with
            // outer = the pointee, not the whole node -- the old
            // `substitute_type(inner, i2, ..)` re-applied the outer Ptr wrap,
            // producing Ptr(Ptr(Int)) ("**Int" -> i64** call site vs the i64*
            // mono def).
            Type::Ptr(i2) | Type::MutRef(i2) | Type::Ref(i2) => Self::substitute_type(i2, i2, type_map),
            _ => inner.clone(),
        };
        match outer {
            Type::Ptr(_) => Type::Ptr(Box::new(new_inner)),
            Type::MutRef(_) => Type::MutRef(Box::new(new_inner)),
            Type::Ref(_) => Type::Ref(Box::new(new_inner)),
            _ => new_inner,
        }
    }

    pub(crate) fn ensure_concrete_tuple_type_registered(&mut self, ty: &Type, type_map: &HashMap<String, String>) {
        match ty {
            Type::Tuple(elems) => {
                let name = Self::substitute_concrete_name(ty, type_map);
                if self.types.type_meta.contains_key(&name) { return; }
                let field_names: Vec<String> = (0..elems.len()).map(|i| format!("_{i}")).collect();
                let field_types: Vec<(String, String)> = elems.iter()
                    .enumerate()
                    .map(|(i, e)| (format!("_{i}"), Self::substitute_concrete_name(e, type_map)))
                    .collect();
                self.types.types.insert(name.clone(), field_names);
                self.types.type_meta.insert(name, TypeMeta {
                    fields: field_types,
                    derives: vec![],
                    invariants: vec![],
                });
                for e in elems {
                    self.ensure_concrete_tuple_type_registered(e, type_map);
                }
            }
            Type::Ref(inner) | Type::MutRef(inner) | Type::Option(inner) | Type::Vec(inner) |
            Type::Slice(inner) | Type::Set(inner) | Type::Ptr(inner) => {
                self.ensure_concrete_tuple_type_registered(inner, type_map);
            }
            Type::Result(ok, err) => {
                self.ensure_concrete_tuple_type_registered(ok, type_map);
                self.ensure_concrete_tuple_type_registered(err, type_map);
            }
            Type::Map(k, v) => {
                self.ensure_concrete_tuple_type_registered(k, type_map);
                self.ensure_concrete_tuple_type_registered(v, type_map);
            }
            Type::Fn(params, ret) => {
                for p in params { self.ensure_concrete_tuple_type_registered(p, type_map); }
                self.ensure_concrete_tuple_type_registered(ret, type_map);
            }
            _ => {}
        }
    }

    /// BUG 3 fix: true when a module-global initializer is a COMPILE-TIME
    /// constant that `global_const_init` can materialize (int/bool/float/char
    /// literals, optionally negated). Runtime expressions (fn calls, struct
    /// constructors, enum variants) return false and are handled via
    /// @llvm.global_ctors startup initializers instead.
    fn expr_is_const_init(value: &Expr) -> bool {
        match value {
            Expr::Int(..) | Expr::Bool(..) | Expr::Float(..) | Expr::Char(..) => true,
            Expr::Unary(UnaryOp::Neg, inner, _) => matches!(inner.as_ref(), Expr::Int(..)),
            // R2 (M58 residual): an all-constant ARRAY literal is a valid
            // compile-time initializer -- global_const_init renders it as an
            // LLVM constant aggregate. Mixed/runtime elements still take the
            // runtime-init path.
            Expr::Array(items, _) => items.iter().all(|e| Self::expr_is_const_init(e)),
            _ => false,
        }
    }

    /// BUG 22 #11 fix: pre-assign the emitted LLVM symbol for EVERY
    /// non-generic fn with a body BEFORE any body compiles, walking the
    /// program in declaration order with fn_symbol's dedup rule (the first
    /// same-key fn emits the bare symbol; later ones module-qualify). The
    /// map keys cover the bare key AND the leaf-qualified/module-qualified
    /// call keys so both definitions and call sites resolve to the SAME
    /// symbol -- a call compiled before its def can no longer emit a
    /// qualified key the def went bare on (zero-param stub -> garbage).
    pub(crate) fn preassign_fn_symbols(&mut self, items: &[TopDecl]) {
        if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() {
            let kinds: Vec<&str> = items.iter().map(|i| match i { TopDecl::Use(_) => "Use", TopDecl::Fn(_) => "Fn", _ => "Other" }).collect();
            eprintln!("[preassign] items: {kinds:?}");
        }
        // BUG 25 #2 fix: resolve the checker-surfaced use-alias paths to
        // registered fn keys (the driver strips UseDecls; the checker records
        // `use X.Y.f as alias` -> "X.Y.f"). Bare calls through the alias then
        // resolve to the real registered key.
        let alias_paths = std::mem::take(&mut self.config.use_alias_paths);
        // R15b: deterministic two-pass -- plain (full-path) entries first,
        // then the "::qualified" leaf forms. Both may target the same alias
        // name (fn aliases record both), and a HashMap iteration let either
        // win at random.
        for (alias, dotted) in alias_paths.iter() {
            if alias.ends_with("::qualified") {
                continue;
            }
            self.mono.use_alias_map.insert(alias.clone(), dotted.clone());
        }
        for (alias, dotted) in alias_paths.iter() {
            if let Some(q) = alias.strip_suffix("::qualified") {
                // leaf-qualified form: "af" -> "math.abs_float" (stdlib-stripped)
                self.mono.use_alias_map.insert(q.to_string(), dotted.clone());
            }
        }
        let mut seen_bare: std::collections::HashSet<String> = std::collections::HashSet::new();
        // R15b: keys defined by MORE THAN ONE module must qualify EVERY
        // definition's symbol (including the first). The old dedup gave the
        // first module the bare symbol; a same-named call through a module
        // alias then resolved to the CALLER's own qualified symbol
        // (`canon.encode` inside beta.base32 emitted `call @beta.base32.encode`
        // -- infinite self-recursion/0xC000001D).
        fn count_module_keys(
            em: &IrEmitter,
            items: &[TopDecl],
            module: &Option<String>,
            per_key: &mut std::collections::HashMap<String, std::collections::HashSet<String>>,
        ) {
            for item in items {
                match item {
                    TopDecl::Fn(fd) => {
                        let is_generic = !fd.generics.is_empty()
                            || fd.receiver.as_ref().map(|r| em.types.generic_type_names.contains(&r.name)).unwrap_or(false);
                        let is_empty_main = fd.name.name == "main"
                            && fd.body.as_ref().map_or(false, |b| b.stmts.is_empty());
                        if !is_generic && fd.body.is_some() && !is_empty_main {
                            per_key
                                .entry(em.fn_key(fd))
                                .or_default()
                                .insert(module.clone().unwrap_or_default());
                        }
                    }
                    TopDecl::Module(md) => {
                        let new_module = Some(if let Some(prev) = module.as_ref() {
                            format!("{prev}.{}", md.name.name)
                        } else {
                            md.name.name.clone()
                        });
                        count_module_keys(em, &md.items, &new_module, per_key);
                    }
                    _ => {}
                }
            }
        }

        fn walk(em: &IrEmitter, items: &[TopDecl], module: &Option<String>, seen_bare: &mut std::collections::HashSet<String>, map: &mut std::collections::HashMap<String, String>, per_key: &std::collections::HashMap<String, std::collections::HashSet<String>>) {
            for item in items {
                match item {
                    TopDecl::Fn(fd) => {
                        let is_generic = !fd.generics.is_empty()
                            || fd.receiver.as_ref().map(|r| em.types.generic_type_names.contains(&r.name)).unwrap_or(false);
                        // BUG 29 (m21_async_spawn_007): skip EMPTY-BODY `main`
                        // fns here too -- compile_top_decl already skips emitting
                        // them when a non-empty main exists, but the preassign
                        // walk claimed the bare "main" symbol for the empty
                        // `async fn main() {}` placeholder, so the REAL main
                        // got module-qualified (@m21_async_spawn_007.main) and
                        // the link failed with "undefined symbol: main".
                        let is_empty_main = fd.name.name == "main"
                            && fd.body.as_ref().map_or(false, |b| b.stmts.is_empty());
                        if !is_generic && fd.body.is_some() && !is_empty_main {
                            let key = em.fn_key(fd);
                            // R15b: qualify EVERY definition of a cross-module
                            // key, not just the later ones.
                            let cross_module = module.is_some()
                                && per_key.get(&key).map_or(false, |mods| mods.len() > 1);
                            let sym = if seen_bare.contains(&key) || cross_module {
                                module.as_ref().map(|m| format!("{m}.{key}")).unwrap_or_else(|| key.clone())
                            } else {
                                key.clone()
                            };
                            seen_bare.insert(key.clone());
                            // Keep the first mapping for the ambiguous bare slot
                            // (deterministic; qualified slots below are exact).
                            map.entry(key.clone()).or_insert_with(|| sym.clone());
                            if let Some(m) = module {
                                map.insert(format!("{m}.{key}"), sym.clone());
                            }
                        }
                    }
                    TopDecl::Module(md) => {
                        let new_module = Some(if let Some(prev) = module.as_ref() {
                            format!("{prev}.{}", md.name.name)
                        } else {
                            md.name.name.clone()
                        });
                        walk(em, &md.items, &new_module, seen_bare, map, per_key);
                    }
                    _ => {}
                }
            }
        }
        let mut map = std::mem::take(&mut self.mono.fn_symbol_map);
        let current_module = self.local.current_module.clone();
        let mut per_key: std::collections::HashMap<String, std::collections::HashSet<String>> =
            std::collections::HashMap::new();
        count_module_keys(self, items, &current_module, &mut per_key);
        walk(self, items, &current_module, &mut seen_bare, &mut map, &per_key);
        self.mono.fn_symbol_map = map;
    }

    pub(crate) fn register_functions(&mut self, item: &TopDecl) {
        if let TopDecl::Const(cd) = item {
            if cd.is_mut {
                // Mutable module-level `var`: emit as a REAL LLVM global and route
                // reads/writes to load/store (see compile_expr / Stmt::Assign).
                // Only do this when the declared type resolves to a concrete LLVM
                // type; otherwise (e.g. `Map[K,V]`, whose LLVM lowering isn't a
                // simple global slot) fall back to constant substitution so the
                // existing behavior -- and the green test gate -- is preserved.
                // BUG 29: an UNANNOTATED `var g = FnBox{...}` parses with
                // Type::Named("_"); type_from_ast("_") degrades to i64, emitting
                // `@g = internal global i64 0` -- the struct field store was
                // dropped and reads hit a bare @f stub (module-scope fn storage
                // silently read-only). Infer the type from the initializer
                // expression (struct literal -> its type name) when the declared
                // type is elided.
                let declared_ty = Self::type_from_ast(&cd.ty);
                let ty_name = if declared_ty == "_" || declared_ty == "()" || declared_ty.is_empty() {
                    self.struct_type_from_expr(&cd.value).unwrap_or(declared_ty)
                } else {
                    declared_ty
                };
                if let Ok(llvm_ty) = self.llvm_type_for(&ty_name) {
                    let symbol = if let Some(ref m) = self.local.current_module {
                        format!("{}.{}", m, cd.name.name)
                    } else {
                        cd.name.name.clone()
                    };
                    // Register lookups under BOTH the bare and qualified names so a
                    // reference resolves whether the module is compiled directly or
                    // its decls are injected flattened at top level.
                    self.local.module_globals.insert(cd.name.name.clone(), (symbol.clone(), llvm_ty.clone()));
                    self.local.module_globals.insert(symbol.clone(), (symbol.clone(), llvm_ty.clone()));
                    // BUG 29 (Map.keys on module globals): record the XIOM type
                    // with args ("Map[Str, Bool]") so generic METHOD calls on the
                    // global (`_coverage.keys()`) can infer concrete type args
                    // instead of defaulting to Int (wrong monomorphisation
                    // Map.keys_Int_Int + undefined @Map.keys fn-value).
                    if let Some(xiom_ty) = Self::type_from_ast_with_args_opt(&cd.ty) {
                        self.local.global_xiom_types.insert(cd.name.name.clone(), xiom_ty.clone());
                        self.local.global_xiom_types.insert(symbol.clone(), xiom_ty);
                    }
                    // Dedup the emitted definition by symbol name.
                    if !self.local.module_global_defs.iter().any(|(s, _, _)| s == &symbol) {
                        let init = Self::global_const_init(&cd.value, &llvm_ty);
                        self.local.module_global_defs.push((symbol.clone(), llvm_ty.clone(), init));
                        // BUG 3 fix: a RUNTIME initializer (fn call etc.) cannot
                        // become a compile-time constant -- the global is emitted
                        // zero-initialized and a @llvm.global_ctors entry runs the
                        // initializer expression at startup.
                        if !Self::expr_is_const_init(&cd.value) {
                            self.local.global_runtime_inits.push((symbol.clone(), llvm_ty.clone(), cd.value.clone()));
                        }
                    }
                    // 5e.5c: track for hot reload state migration
                    if self.config.hot_reload {
                        let byte_sz = Self::llvm_type_byte_size(&llvm_ty, &self.types.type_meta);
                        if !self.config.xiom_hot_globals.iter().any(|(s, _, _)| s == &symbol) {
                            self.config.xiom_hot_globals.push((symbol.clone(), llvm_ty.clone(), byte_sz));
                        }
                    }
                } else {
                    // Type doesn't lower to a simple global: keep old behavior.
                    self.local.constants.insert(cd.name.name.clone(), cd.value.clone());
                }
            } else {
                // Record module/global constants so a bare reference can be substituted
                // with its literal value (constants are not emitted as globals). Last
                // definition wins; both bare and module-qualified names are keyed.
                // M33/CTFE: evaluate the init expression at compile time and store
                // the literal result instead of the raw expression tree.
                let evaluated = self.evaluate_const_init(&cd.value);
                self.local.constants.insert(cd.name.name.clone(), evaluated);
            }
        }
        if let TopDecl::Fn(fd) = item {
            // Register tuple types used in function signature before resolving LLVM types
            fd.return_type.as_ref().map(|t| self.ensure_tuple_type_registered(t));
            for p in &fd.params {
                self.ensure_tuple_type_registered(&p.ty);
            }
            // 5c.33: Register anonymous struct types so field access works
            fd.return_type.as_ref().map(|t| self.register_anon_struct_from_ast(t));
            for p in &fd.params {
                self.register_anon_struct_from_ast(&p.ty);
            }
            let mut param_types: Vec<String> = Vec::new();
            // Detect self param: either named "self" OR receiver-style first
            // param. G-20 fix: receiver-style `fn T.method(h: &T, ...)` applies
            // ONLY to by-REFERENCE first params (&T / &mut T / *T). A by-VALUE
            // first param of the receiver type (math lerp/dot pattern
            // `fn V2.lerp(other: V2, t)`) is a REAL argument, never the receiver
            // -- the old type-only heuristic hijacked it and shifted every arg
            // (silent-swap miscompile class).
            let is_first_param_self = fd.receiver.is_some() && fd.params.first().map_or(false, |p| {
                let is_ref = matches!(&p.ty, Type::Ref(_) | Type::MutRef(_) | Type::Ptr(_));
                if !is_ref { return false; }
                let pt = Self::type_from_ast(&p.ty);
                let ptn = pt.trim_start_matches('*');
                fd.receiver.as_ref().map_or(false, |r| ptn == r.name)
            });
            let has_self_param = fd.params.iter().any(|p| p.name.name == "self")
                || is_first_param_self;
            let has_recv = fd.receiver.is_some() && has_self_param;
            // For `this`-based methods (receiver exists but no explicit `self`
            // param, AND body references receiver STATE -- `this` or bare
            // fields), register the receiver as a pointer type so call-site
            // receiver handling can detect the need for a pointer and coerce
            // instance method calls (v.method()) correctly. (G-20: bare-field
            // bodies included so `fn Counter.inc() { return val + 1; }` gets
            // a real receiver slot instead of reading garbage.)
            let is_this_based = fd.receiver.is_some() && !has_self_param
                && self.body_uses_receiver_state(fd);
            // 5c.32: by-value self methods where body uses `self` but it's not
            // in fd.params (parser stores receiver separately). Detect and add
            // the struct type to the signature so call-site dispatch matches.
            let is_by_value_self = fd.receiver.is_some() && !has_self_param
                && !self.body_uses_receiver_state(fd)
                && fd.body.as_ref().map_or(false, |b| IrEmitter::block_uses_self_ident(b));
            if has_recv && !is_first_param_self {
                if let Some(recv) = fd.receiver.as_ref() {
                    let base = self.llvm_type_for(&recv.name).unwrap_or_else(|_| "i64".to_string());
                    // BUG 31: mutating `self` methods (body assigns self fields,
                    // e.g. Formatter.write_int's `self.buf = ...`) must register
                    // the POINTER ABI -- mirror compile_fn's self_llvm_ty decision.
                    // BUG 38b (iter family): BARE receiver-field assignments
                    // (`start = start + 1` in Range.next) get the same pointer
                    // ABI (by-value copies lost the mutation -> infinite loops).
                    let is_mut = fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self)
                        || self.block_mutates_self(fd)
                        || self.block_mutates_receiver_state(fd);
                    if is_mut && base.starts_with('%') {
                        param_types.push(format!("{base}*"));
                    } else {
                        param_types.push(base);
                    }
                }
            }
            if is_this_based {
                if let Some(recv) = fd.receiver.as_ref() {
                    let recv_ty = self.llvm_type_for(&recv.name).unwrap_or_else(|_| "i64".to_string());
                    if recv_ty.starts_with('%') && !recv_ty.ends_with('*') {
                        param_types.push(format!("{recv_ty}*"));
                    } else {
                        param_types.push(recv_ty);
                    }
                }
            }
            if is_by_value_self {
                if let Some(recv) = fd.receiver.as_ref() {
                    let base = self.llvm_type_for(&recv.name).unwrap_or_else(|_| "i64".to_string());
                    // 5c.32: use pointer type so call-site passes alloca address
                    // and callee can write through it (by-value mutating self).
                    if base.starts_with('%') {
                        param_types.push(format!("{base}*"));
                    } else {
                        param_types.push(base);
                    }
                }
            }
            let _self_param_name: Option<String> = if is_first_param_self {
                fd.params.first().map(|p| p.name.name.clone())
            } else if has_recv {
                Some("self".to_string())
            } else { None };
            let explicit_params: Vec<String> = fd.params.iter()
                .filter(|p| !(has_recv && !is_first_param_self && p.name.name == "self"))
                .map(|p| self.param_llvm_type(&p.ty))
                .collect();
            param_types.extend(explicit_params);
            let ret_type = fd.return_type.as_ref()
                .map(|t| {
                    // B-001: Use concrete type for Result/Option with struct payloads
                    let concrete = self.concrete_type_for(t);
                    self.llvm_type_for(&concrete).unwrap_or_else(|_| "i64".to_string())
                })
                .unwrap_or_else(|| "void".to_string());
            let key = self.fn_key(fd);
            self.types.functions.insert(key.clone(), (param_types.clone(), ret_type.clone()));
            // R52: track visibility for bare-call ranking (imported pub >
            // pub > private) without making private helpers unreachable.
            if fd.is_pub {
                self.types.pub_fns.insert(key.clone(), true);
            }
            // R15b: same-leaf same-name modules declared in the USER program
            // share the bare key. Register the MODULE-QUALIFIED key alongside
            // it so (a) aliased delegation (`use alpha.base32 as canon;`
            // then `canon.encode(...)`) binds the TARGET module's signature,
            // not the caller's own, and (b) intra-module calls resolve their
            // own module's definition. Symbols are preassigned qualified for
            // colliding keys (preassign_fn_symbols). FREE fns only: methods
            // already carry a receiver-qualified key ("Person.greet") and an
            // extra module-qualified alias made suffix searches ambiguous.
            if fd.receiver.is_none() {
                if let Some(ref module) = self.local.current_module {
                    let qualified = format!("{module}.{key}");
                    if !self.types.functions.contains_key(&qualified) {
                        self.types.functions.insert(qualified, (param_types.clone(), ret_type.clone()));
                    }
                }
            }
            // B-007: record fn-typed (closure) param positions + return types
            // for the call sites -- the direct path must wrap raw fn-REFERENCE
            // args into closure envs (the erased signature can't tell them
            // apart; the ret type drives the forwarding thunk).
            let fn_param_indices: Vec<(usize, String)> = fd.params.iter().enumerate()
                .filter(|(_, p)| matches!(&p.ty, Type::Fn(..)))
                .map(|(i, p)| (i, match &p.ty { Type::Fn(_, ret) => Self::type_from_ast(ret), _ => "Int".to_string() }))
                .collect();
            if !fn_param_indices.is_empty() {
                self.mono.fn_typed_params.insert(key.clone(), fn_param_indices);
            }
            // Injected stdlib free fns arrive leaf-qualified ("array.contains")
            // because the driver merge drops TopDecl::Module wrappers. Register a
            // bare-leaf alias ("contains" -> "array.contains") so unqualified
            // internal stdlib calls (e.g. env.args_os -> args()) resolve to the
            // qualified key and the emitted symbol matches the definition.
            // Keep-first: a user-defined bare fn (registered earlier) wins.
            // R52 (packages relay): visibility is tracked in `pub_fns` and
            // ranked at RESOLUTION time (imported-module exports first, then
            // pub, then private) -- private helpers must stay reachable from
            // their own module (`normalize_duration` inside Duration.add) but
            // must not hijack an unqualified call from an unrelated program.
            if fd.receiver.is_none() {
                if let Some((_, bare)) = key.rsplit_once('.') {
                    if !bare.is_empty() && bare != key.as_str() {
                        if !self.types.functions.contains_key(&bare.to_string()) {
                            self.mono.bare_fn_aliases.entry(bare.to_string())
                                .or_insert_with(|| key.clone());
                        }
                    }
                }
            }
            // Track by-value self methods (not &self) for store_back.
            // A by-value self method has a self param that is NOT &self/&mut self/*self.
            if fd.receiver.is_some() {
                let is_by_value_self = fd.params.iter().any(|p| {
                    p.name.name == "self" && !p.is_ref_self && !p.is_mut_self
                        && !matches!(&p.ty, Type::Ref(_) | Type::MutRef(_) | Type::Ptr(_))
                });
                if is_by_value_self {
                    self.types.by_value_self_methods.insert(key.clone());
                }
            }
            // 5c.30: keep the declared XIOM return type WITH generic args so
            // Option/Result payload types survive LLVM erasure.
            if let Some(rt) = fd.return_type.as_ref() {
                self.types.fn_return_xiom.insert(key.clone(), Self::type_string_full(rt));
                // R15b: keep the module-qualified slot too for FREE fns (see above).
                if fd.receiver.is_none() {
                    if let Some(ref module) = self.local.current_module {
                        let qualified = format!("{module}.{key}");
                        if !self.types.fn_return_xiom.contains_key(&qualified) {
                            self.types.fn_return_xiom.insert(qualified, Self::type_string_full(rt));
                        }
                    }
                }
            }
            // Detect interface-typed params: store these functions so call sites
            // can monomorphise them for each concrete implementor (BUG-007).
            let has_iface_param = fd.params.iter().any(|p| {
                let name = Self::type_from_ast(&p.ty);
                self.types.interfaces.contains_key(&name)
            }) || fd.return_type.as_ref().map_or(false, |t| {
                let name = Self::type_from_ast(t);
                self.types.interfaces.contains_key(&name)
            });
            if has_iface_param {
                // Add to generic_fn_decls as a pseudo-generic so the
                // monomorphisation loop picks it up.
                if !self.mono.generic_fn_decls.iter().any(|(k, _)| k == &key) {
                    self.mono.generic_fn_decls.push((key.clone(), fd.clone()));
                }
            }
            // Register leaf-module key (e.g. "mem.replace", "ptr.replace") so
            // call sites like `mem.replace(...)` / `ptr.replace(...)` resolve
            // to module-disambiguated names.  This prevents monomorphisation
            // naming collisions between same-named generic functions from
            // different modules (BUG-005).
            if fd.receiver.is_none() {
                if let Some(ref module) = self.local.current_module {
                    if let Some(leaf) = module.rsplit('.').next() {
                        let leaf_key = format!("{}.{}", leaf, key);
                        if leaf_key != key && !self.types.functions.contains_key(&leaf_key) {
                            // R15b: FIRST registrant keeps the leaf key -- two
                            // same-leaf modules both inserting made the leaf
                            // alias clobber (last wins), and a full-path call
                            // resolving through it bound the wrong module.
                            self.types.functions.insert(leaf_key.clone(), (param_types.clone(), ret_type.clone()));
                        }
                    }
                }
            }
            if !fd.generics.is_empty() {
                // Bare-key collision guard: two modules can define the same-named
                // generic fn (e.g. array.contains vs core.contains). The bare key
                // must stay owned by the FIRST registrant so module-qualified calls
                // (array.contains -- leaf key) resolve unambiguously; a second bare
                // entry would make the fallback suffix-search pick a random one.
                if !self.mono.generic_fn_decls.iter().any(|(k, _)| k == &key) {
                    self.mono.generic_fn_decls.push((key.clone(), fd.clone()));
                }
                // Also register with leaf-module key for generic resolution
                if fd.receiver.is_none() {
                    if let Some(ref module) = self.local.current_module {
                        if let Some(leaf) = module.rsplit('.').next() {
                            let leaf_key = format!("{}.{}", leaf, key);
                            if leaf_key != key {
                                // Leaf key is unambiguous per module -- always add.
                                self.mono.generic_fn_decls.push((leaf_key, fd.clone()));
                            }
                        }
                    }
                }
            }
            // v0.54 Phase B: Register function body for CTFE evaluation.
            // Skip methods (self-receiver) and generic functions.
            if fd.receiver.is_none() && fd.generics.is_empty() {
                if let Some(ref body) = fd.body {
                    let params: Vec<String> = fd.params.iter()
                        .map(|p| p.name.name.clone())
                        .collect();
                    self.ctfe.borrow_mut().register_function(&fd.name.name, params, &body.stmts);
                }
            }
        }
        if let TopDecl::Interface(id) = item {
            let mut methods = Vec::new();
            for member in &id.members {
                if let InterfaceMember::FnSignature(fd) = member {
                    let param_type_names: Vec<String> = fd.params.iter()
                        .map(|p| Self::type_from_ast(&p.ty))
                        .collect();
                    methods.push((fd.name.name.clone(), param_type_names));
                    // M19: Store default method bodies for fallback emission.
                    if fd.body.is_some() {
                        let key = format!("{}.{}", id.name.name, fd.name.name);
                        self.types.interface_defaults.insert(key, fd.clone());
                    }
                }
            }
            self.types.interfaces.insert(id.name.name.clone(), methods);
        }
        if let TopDecl::Extern(eb) = item {
            // Register extern "C" functions so calls to them use correct LLVM types.
            // These functions have no receiver and are not module-qualified (C linkage).
            for fd in &eb.functions {
                let param_types: Vec<String> = fd.params.iter()
                    .map(|p| self.extern_type_to_llvm(&p.ty))
                    .collect();
                let ret_type = fd.return_type.as_ref()
                    .map(|t| self.extern_type_to_llvm(t))
                    .unwrap_or_else(|| "void".to_string());
                self.types.functions.insert(fd.name.name.clone(), (param_types, ret_type));
                if let Some(rt) = &fd.return_type {
                    // R16: record the XIOM return type of externs too. Raw
                    // pointers (malloc -> *UInt8) must resolve so an
                    // unannotated `var buf = malloc(n)` is typed as a POINTER:
                    // otherwise `buf + len` compiled as Str concatenation
                    // (i8* + Int) and passed a heap string as the dest address
                    // (memcpy at `buf + len_a` corrupted the buffer).
                    self.types.fn_return_xiom.insert(fd.name.name.clone(), Self::type_string_full(rt));
                }
            }
        }
        if let TopDecl::Module(md) = item {
            let saved_module = self.local.current_module.clone();
            self.local.current_module = Some(if let Some(ref prev) = saved_module {
                format!("{}.{}", prev, md.name.name)
            } else {
                md.name.name.clone()
            });
            for sub in &md.items {
                self.register_functions(sub);
            }
            self.local.current_module = saved_module;
        }
    }

    /// Scan all registered interfaces and concrete types to determine which
    /// types implement which interfaces (BUG-007). A type implements an
    /// interface if, for every method in the interface, there is a function
    /// registered as `TypeName.methodName` in self.types.functions.
    pub(crate) fn scan_interface_impls(&mut self) {
        for (iface_name, methods) in self.types.interfaces.entries() {
            for type_name in self.types.types.keys() {
                // Skip builtin types (Option, Result, Vec, etc.)
                if ["Option", "Result", "Vec", "Slice", "Map", "Set"].contains(&type_name.as_str()) { continue; }
                let mut all_implemented = true;
                for (method_name, _) in &methods {
                    let fn_key = format!("{}.{}", type_name, method_name);
                    let leaf_parts: Vec<&str> = type_name.rsplitn(2, '.').collect();
                    let leaf_key = if leaf_parts.len() > 1 {
                        format!("{}.{}", leaf_parts[1], method_name)
                    } else { fn_key.clone() };
                    if !self.types.functions.contains_key(&fn_key) && !self.types.functions.contains_key(&leaf_key) {
                        if !self.mono.generic_fn_decls.iter().any(|(k, _)| k == &fn_key || k == &leaf_key) {
                            all_implemented = false;
                            break;
                        }
                    }
                }
                if all_implemented {
                    self.types.interface_impls.or_insert_with(iface_name.clone(), HashSet::new)
                        .insert(type_name.clone());
                }
            }
        }
    }

    /// Like type_from_ast_with_args but returns None for elided/unknown
    /// annotations (`Type::Named("_")`) so callers can skip inference.
    fn type_from_ast_with_args_opt(ty: &Type) -> Option<String> {
        match ty {
            Type::Named(id, _) if id.name == "_" || id.name.is_empty() => None,
            _ => Some(Self::type_from_ast_with_args(ty)),
        }
    }

    pub(crate) fn fn_key(&self, fd: &FnDecl) -> String {
        if let Some(recv_name) = &fd.receiver {
            // Use module context to resolve receiver type to qualified name
            let recv_type = if let Some(ref module) = self.local.current_module {
                let qualified = format!("{}.{}", module, recv_name.name);
                if self.types.type_meta.contains_key(&qualified) { qualified } else { recv_name.name.clone() }
            } else {
                recv_name.name.clone()
            };
            // Strip receiver prefix from the name to avoid doubling
            // (e.g. "Company" + "Company.greet" -> "Company.Company.greet").
            let bare_method = fd.name.name.rsplit('.').next().unwrap_or(&fd.name.name);
            format!("{}.{}", recv_type, bare_method)
        } else {
            fd.name.name.clone()
        }
    }

    /// Return the LLVM symbol name for a function, avoiding collisions.
    /// If a bare name already exists in emitted_fns, use module-qualified.
    pub(crate) fn fn_symbol(&self, fd: &FnDecl) -> String {
        let bare = self.fn_key(fd);
        // BUG 22 #11: prefer the PRE-ASSIGNED symbol (assigned once for all
        // fns before body compilation, in program order) -- definitions and
        // call sites then always agree, even when a call compiles before its
        // def (fn_symbol's lazy emitted_fns dedup was order-dependent).
        // R15: prefer the MODULE-QUALIFIED preassigned slot. Two modules with
        // the same LEAF and the same fn name share the bare map slot, and the
        // later collision overwrites it -- resolving both definitions to the
        // SAME symbol ("invalid redefinition"). The qualified slot is unique
        // per module path. (Duplicate emission of one fn via the driver's
        // injected flattened copies is prevented by the recursive dedup in
        // crates/xiom/src/lib.rs, so this cannot collide with a second path.)
        if let Some(ref module) = self.local.current_module {
            if let Some(sym) = self.mono.fn_symbol_map.get(&format!("{module}.{bare}")) {
                return sym.clone();
            }
        }
        if let Some(sym) = self.mono.fn_symbol_map.get(&bare) {
            return sym.clone();
        }
        // Keep `main` as bare entry point. If a second `main` is encountered
        // (e.g. module-level `async fn main()` shadowing the real entry point),
        // qualify the duplicate with its module name.
        if bare == "main" {
            if self.mono.emitted_fns.contains(&bare) {
                if let Some(ref module) = self.local.current_module {
                    return format!("{}.{}", module, bare);
                }
            }
            return bare;
        }
        // If bare name already emitted (collision from multi-file merge), qualify it
        if self.mono.emitted_fns.contains(&bare) {
            if let Some(ref module) = self.local.current_module {
                return format!("{}.{}", module, bare);
            }
        }
        bare
    }

    /// Resolve a module-qualified function call like `math.run_all()`.
    /// Looks up `math.run_all`, then `*.math.run_all` in registered functions.
    pub(crate) fn resolve_module_call(&self, receiver: &Expr, fn_name: &str) -> String {
        // Resolve the module name from the receiver. `xiom.rsa` parses as a
        // dotted Ident; `xiom.rsa.encrypt` style receivers arrive as nested
        // Field exprs rooted at an Ident. Walk to the root ident and join the
        // field names so "xiom.rsa" and "rsa" both resolve.
        let mut segments: Vec<String> = Vec::new();
        let mut cur = receiver;
        loop {
            match cur {
                Expr::Ident(id) => { segments.insert(0, id.name.clone()); break; }
                Expr::Field(base, fname, _) => {
                    segments.insert(0, fname.name.clone());
                    cur = base;
                }
                _ => break,
            }
        }
        if segments.is_empty() {
            return fn_name.to_string();
        }
        // R15b: a single-segment receiver may be a `use X.Y as alias;` ALIAS.
        // Expand it through the codegen alias map (populated from the
        // checker's use_alias_paths) so the call binds the TARGET module's
        // definition -- without this, `canon.encode(...)` inside a same-leaf
        // module fell to the in-caller-module fallback and emitted a
        // self-recursive call.
        if segments.len() == 1 {
            // R22: the checker records EVERY module binding (alias or plain
            // item import) in module_receiver_paths. Prefer it: a plain
            // `use xiom.convert.percent;` + `percent.percent_encode(...)`
            // must bind the convert module even when xiom.encoding.percent is
            // also in the graph (same leaf).
            let target = self.config.module_receiver_paths.get(&segments[0])
                .or_else(|| self.mono.use_alias_map.get(&segments[0]));
            if let Some(target) = target {
                let expanded: Vec<String> = target.split('.').map(|s| s.to_string()).collect();
                if expanded.len() >= 2 {
                    segments = expanded;
                }
            }
        }
        let dotted = segments.join(".");
        // R15b: for a multi-segment receiver, prefer the FULL dotted key when
        // it is registered -- user-program modules register their fns under
        // "<module>.<fn>", and the leaf key may belong to another same-leaf
        // module. Catalog injected names are xiom-stripped leaf-qualified, so
        // the leaf fallback below still handles them.
        if segments.len() >= 2 {
            let full_key = format!("{}.{}", dotted, fn_name);
            if self.types.functions.contains_key(&full_key)
                || self.mono.generic_fn_decls.iter().any(|(k, _)| k == &full_key)
            {
                return full_key;
            }
        }
        // R15/R22: injected catalog names are xiom-STRIPPED full paths
        // ("iter.map.iter_map", "convert.percent.percent_encode"), while
        // fully-qualified source uses the "xiom." prefix
        // (`xiom.iter.map.iter_map`). The stripped dotted form must win over
        // the single LEAF form: `use xiom.convert.percent;` +
        // `percent.percent_encode(...)` used to bind the sibling
        // "percent.percent_encode" (xiom.encoding.percent) registered first.
        if segments.len() > 1 {
            let stripped_dotted = segments.iter().skip(1).cloned().collect::<Vec<_>>().join(".");
            let stripped_key = format!("{stripped_dotted}.{fn_name}");
            if self.types.functions.contains_key(&stripped_key)
                || self.mono.generic_fn_decls.iter().any(|(k, _)| k == &stripped_key)
            {
                return stripped_key;
            }
        }
        // The LEAF module segment is what injected decls register under
        // ("rsa.rsa_encrypt"), so try it for dotted receivers like "xiom.rsa"
        // -- the full "xiom.rsa.rsa_encrypt" key is never registered and
        // falling to the bare name lets the keep-first alias hand the call to
        // the WRONG module's same-named fn (e.g. crypto's rsa_encrypt vs
        // rsa's rsa_encrypt).
        if let Some(leaf) = segments.last() {
            let leaf_key = format!("{}.{}", leaf, fn_name);
            if self.types.functions.contains_key(&leaf_key)
                || self.mono.generic_fn_decls.iter().any(|(k, _)| k == &leaf_key)
            {
                return leaf_key;
            }
        }
        // Try the full dotted path: "xiom.rsa.encrypt"
        let full_key = format!("{}.{}", dotted, fn_name);
        if self.types.functions.contains_key(&full_key)
            || self.mono.generic_fn_decls.iter().any(|(k, _)| k == &full_key)
        {
            return full_key;
        }
        // Try parent-qualified: "benchmark.math.run_all" (current_module parent + module_name)
        if let Some(ref cur_mod) = self.local.current_module {
            if let Some(parent) = cur_mod.rsplitn(2, '.').last() {
                let parent_key = format!("{}.{}.{}", parent, dotted, fn_name);
                if self.types.functions.contains_key(&parent_key)
                    || self.mono.generic_fn_decls.iter().any(|(k, _)| k == &parent_key)
                {
                    return parent_key;
                }
            }
        }
        // Try any key ending with ".module_name.fn_name" as a fallback.
        // R20: collect ALL candidates, drop the CURRENT function (a shim
        // delegating to a same-named canonical fn must never bind itself --
        // infinite recursion -> stack overflow) and pick deterministically:
        // longest key first (most module-qualified), then name. The previous
        // first-HashMap-hit scan was process-order dependent and bound the
        // shim itself or a zero-arg fallback stub on some runs.
        let suffix = format!(".{}.{}", dotted, fn_name);
        let mut candidates: Vec<String> = self.types.functions.keys().into_iter()
            .filter(|k| k.ends_with(&suffix))
            .filter(|k| Some(k.as_str()) != self.fctx.current_fn.as_deref())
            .collect();
        for (k, _) in &self.mono.generic_fn_decls {
            if k.ends_with(&suffix) && Some(k.as_str()) != self.fctx.current_fn.as_deref() {
                candidates.push(k.clone());
            }
        }
        if !candidates.is_empty() {
            candidates.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
            return candidates.into_iter().next().unwrap();
        }
        fn_name.to_string()
    }

    pub(crate) fn compile_top_decl(&mut self, item: &TopDecl) -> Result<(), String> {
        match item {
            TopDecl::Fn(fd) => {
                // Skip generic functions -- they will be monomorphised later
                if fd.generics.is_empty() {
                    // Skip methods on generic types (e.g. `BinaryHeap[T].push`). The
                    // generic parameter lives on the RECEIVER type, not in fd.generics,
                    // so it evades the check above; emitting such a body concretely
                    // erases `self` to i64 and produces malformed struct-access IR.
                    // These are monomorphised on demand at call sites instead.
                    let recv_is_generic = fd.receiver.as_ref()
                        .map(|r| self.types.generic_type_names.contains(&r.name))
                        .unwrap_or(false);
                    if !recv_is_generic && fd.body.is_some() {
                        // M21: Skip empty-body `main` functions (e.g. `async fn main() {  }`)
                        // ONLY when a non-empty main exists -- a placeholder would shadow
                        // the real entry point. A standalone `fn main() { }` IS emitted
                        // (JIT/shared-lib paths require a callable @main entry point).
                        let is_empty_main = fd.name.name == "main"
                            && fd.body.as_ref().map_or(false, |b| b.stmts.is_empty());
                        if is_empty_main && self.has_non_empty_main {
                            return Ok(());
                        }
                        let fn_name = self.fn_symbol(fd);
                        // R49 (playground C17 residue): the preassigned symbol
                        // map returns the SAME name for a duplicated definition
                        // (L6-31's lesson solution declares `new_student`
                        // twice); compiling both bodies produced clang
                        // "invalid redefinition of function". First wins.
                        if !self.mono.emitted_fns.insert(fn_name.clone()) {
                            return Ok(());
                        }
                        // 5e.5a: track pub functions for hot reload thunk dispatch
                        if self.config.hot_reload && fd.is_pub {
                            self.config.pub_functions.insert(fn_name.clone());
                        }
                        self.compile_fn(fd)?;
                    }
                }
                Ok(())
            }
            TopDecl::Module(md) => {
                let saved_module = self.local.current_module.clone();
                self.local.current_module = Some(if let Some(ref prev) = saved_module {
                    format!("{}.{}", prev, md.name.name)
                } else {
                    md.name.name.clone()
                });
                for sub in &md.items {
                    self.compile_top_decl(sub)?;
                }
                self.local.current_module = saved_module;
                Ok(())
            }
            TopDecl::Spawn(_block, _, _move) => {
                // M21: Module-level spawn blocks are compiled inline at
                // program init. For now, skip -- spawn is a no-op runtime.
                Ok(())
            }
            TopDecl::Interface(_) | TopDecl::Enum(_) | TopDecl::Const(_) | TopDecl::Type(_) | TopDecl::Use(_) | TopDecl::Extern(_) | TopDecl::Impl(_) => Ok(()),
        }
    }

    pub(crate) fn compile_fn(&mut self, fd: &FnDecl) -> Result<(), String> {
        // Skip emitting a body for a function whose bare name collides with a C
        // symbol we already `declare` (libc/libm like `free`/`sqrt`/`floor`, or any
        // `extern "C"` fn). Such stdlib "wrappers" (e.g. `pub fn sqrt(x) { sqrt(x) }`)
        // both redeclare and infinitely self-recurse; the `declare` + direct calls
        // to the C function are what's actually used. The `xiom_*` runtime family is
        // intentionally defined by the selfhost compiler, so it is exempt.
        {
            let bare = self.fn_key(fd);
            if bare != "main"
                && fd.receiver.is_none()
                && !bare.starts_with("xiom_")
                && self.mono.already_declared.contains(&bare)
            {
                return Ok(());
            }
        }
        // --strict mode: enforce #[safety_audit] on functions with unsafe blocks
        if self.config.strict_mode && fd.body.as_ref().map_or(false, |b| {
            Self::block_contains_unsafe(b)
        }) {
            let has_audit = fd.attributes.iter().any(|a| a.name.name == "safety_audit");
            if !has_audit {
                let fn_name = self.fn_key(fd);
                eprintln!("  warning: --strict: function '{}' contains unsafe block(s) without #[safety_audit] attribute", fn_name);
                eprintln!("    --> add #[safety_audit(justification: \"...\")] to document the safety invariant");
            }
        }
        self.push_scope();
        self.block_counter = 0;
        self.tmp_counter = 0;
        // D2.1 (Phase 6): `#[unsafe_no_retry]` on the enclosing fn disables the
        // once-only transient-fault retry for its unsafe blocks (deterministic
        // faults shouldn't be retried).
        self.fctx.unsafe_allow_retry = !fd.attributes.iter().any(|a| a.name.name == "unsafe_no_retry");
        // D2.1 (Phase 7): `#[unsafe_direct]` marks the fn's unsafe blocks as
        // trusted (no trampoline/arena/guard page). Restricted to stdlib/trusted
        // or user code with --enable-unsafe-direct.
        let has_direct = fd.attributes.iter().any(|a| a.name.name == "unsafe_direct");
        let is_stdlib = self.config.source_file.contains("stdlib")
            || self.config.source_file.contains("selfhost");
        let direct_allowed = has_direct && (is_stdlib || self.config.enable_unsafe_direct);
        self.fctx.unsafe_direct = direct_allowed;
        if has_direct && !direct_allowed {
            eprintln!("  warning: `#[unsafe_direct]` is restricted to stdlib/trusted code; add --enable-unsafe-direct to allow user code. Block will be confined.");
        }
        self.types.fn_ptr_return_types = SyncRegistry::default();
        self.local.bool_locals.clear();
        self.local.ptr_locals.clear();
        self.local.local_vec_elem.clear();
        // AUDIT BUG 57 FOLLOW-UP (stdlib r17 finding): the chained-index
        // element-type map is keyed by AST Debug form (same SOURCE spans
        // across monomorphisations of a generic fn) -- leaving entries in
        // across function bodies leaked the FIRST instantiation's concrete
        // types into later ones ("%tmpN defined with type i64 but expected
        // %struct.Vec"; geom_vec/mat compile errors). Scope it per body.
        self.indexed_elem_types.clear();
        self.local.local_opt_payload.clear();
        self.local.local_opt_payload_xiom.clear();
        self.local.loop_depth = 0;
        self.local.hoisted_allocas.clear();
        self.local.local_boxed_struct.clear();
        self.local.local_vec_handle.clear();
        self.local.signed_locals.clear();
        self.local.local_xiom_types.clear();
        self.local.reg_signed.clear();
        self.local.ref_locals.clear();
        // round-15 (array.first off-by-one): array-binding metadata must not
        // leak between functions -- a caller's `var arr = [...]` left "arr" in
        // array_locals/local_array_elem, so a LATER fn's &[N]T param body
        // misrouted arr[0] through the array-BUFFER path (index+1, probe_arr8).
        self.local.array_locals.clear();
        self.local.local_array_elem.clear();
        self.local.local_array_elem_xiom.clear();
        self.local.local_array_sizes.clear();
        self.local.array_value_regs.clear();
        // BUG 47 (2026-08-18): param_locals/ref_params were NEVER cleared
        // between functions. A `&T` param named "b" in an earlier fn (e.g.
        // `cmp_int(a: &Int, b: &Int)`) left a stale entry, so a LATER fn's
        // plain value param named "b" was misidentified as a ref-param --
        // `x.compare(&b)` compiled the VALUE and dereferenced address 7
        // (inttoptr + load) -> AV. Same for param_locals (by-value &T deref
        // logic and the eq/compare fast-path both consult these sets).
        self.local.param_locals.clear();
        self.local.ref_params.clear();
        // P0-2: Clear deferred cleanup stack at function start
        self.clear_deferred_cleanups();

        let ret_llvm = fd.return_type.as_ref()
            .map(|t| {
                // B-001: Use concrete type for Result/Option with struct payloads
                let concrete = self.concrete_type_for(t);
                self.llvm_type_for(&concrete).unwrap_or_else(|_| "i64".to_string())
            })
            .unwrap_or_else(|| "void".to_string());
        // M16: The entry point must return i64 (not void) so the process
        // exit code is well-defined. A void main produces garbage in RAX.
        let ret_llvm = if fd.name.name == "main" && ret_llvm == "void" {
            "i64".to_string()
        } else {
            ret_llvm
        };
        self.fctx.current_return_type = ret_llvm.clone();
        // P2-4: Reset Never-return flag (may persist from previous function).
        // Set early so that Return statements inside the body emit unreachable.
        self.fctx.is_never_return = fd.return_type.as_ref().map_or(false, |t| {
            let type_str = Self::type_from_ast(t);
            type_str == "!"
        });
        self.fctx.current_param_llvm_types = fd.params.iter()
            .map(|p| self.param_llvm_type(&p.ty))
            .collect();

        // Store ensures clauses for return point checking
        self.fctx.current_ensures = Vec::new();
        if self.config.check_contracts {
            for clause in &fd.contracts {
                if let ContractClause::Ensures(e, _) = clause {
                    self.fctx.current_ensures.push(e.clone());
                }
            }
        }

        let name = self.fn_key(fd);
        // BUG 29: the emitted LLVM symbol MUST come from the pre-assigned
        // fn_symbol map (assigned once in preassign_fn_symbols, program
        // order) -- NOT fn_key. For a method inside `module X { ... }`,
        // fn_key resolves the receiver to the module-qualified type
        // (X.Person.greet) while call sites and the preassign map agree on
        // the bare symbol (Person.greet). Emitting the qualified key while
        // calls use the bare symbol made the call land on the zero-arg
        // auto-stub (ret null) with the struct argument -> ABI mismatch ->
        // 0xC0000005 (m19_default_* cluster, all 125 tests).
        let emit_symbol = self.fn_symbol(fd);
        self.fctx.current_fn = Some(name.clone());
        // 5c.30: track receiver type for implicit-self method calls (G-10)
        self.fctx.current_receiver = fd.receiver.as_ref().map(|r| r.name.clone());

        // For methods, prepend the self struct parameter. A receiver-qualified fn
        // with NO `self` param is a static constructor (e.g. `Layout.new(size)`):
        // it keeps its qualified name but takes no receiver argument.
        let has_self_param = fd.params.iter().any(|p| p.name.name == "self");
        // 5c.29: ecosystem pattern `fn T.method(h: &T, ...)` -- the first
        // explicit param IS the receiver. Signature registration and all call
        // sites never include an implicit self argument for these, so the
        // definition must not emit `%param_self` either (the extra leading
        // param shifted every argument and made the body read uninitialized
        // registers -- HTTP/SQLITE ACCESS_VIOLATION).
        // G-20 fix: receiver-style requires a by-REFERENCE first param
        // (&T/&mut T/*T). By-value same-type params are real arguments.
        let is_first_param_self = fd.receiver.is_some() && !has_self_param
            && fd.params.first().map_or(false, |p| {
                let is_ref = matches!(&p.ty, Type::Ref(_) | Type::MutRef(_) | Type::Ptr(_));
                if !is_ref { return false; }
                let pt = Self::type_from_ast(&p.ty);
                // type_from_ast returns "*T" for &mut T -- strip the pointer
                // prefix to compare with the bare receiver name.
                let ptn = pt.trim_start_matches('*');
                fd.receiver.as_ref().map_or(false, |r| ptn == r.name)
            });
        // Match registration: implicit self only for explicit-`self` methods
        // and `this`-based methods (body references receiver state).
        // G-20: `this`-based now includes BARE-FIELD bodies (e.g.
        // `fn Counter.inc() -> Int { return val + 1; }`) -- the %param_self
        // slot is emitted and the prologue GEP-binds every field, so bare
        // reads are correct instead of garbage. Must match registration.
        let is_this_based = fd.receiver.is_some() && !has_self_param && !is_first_param_self
            && self.body_uses_receiver_state(fd);
        // 5c.32: by-value self methods -- `fn Type.method(params) { self.field = ... }`
        // The parser stores the receiver but does NOT add `self` to fd.params.
        // Detect self usage in the body so the LLVM signature gets the struct param.
        let body_uses_self = fd.receiver.is_some() && !has_self_param && !is_first_param_self
            && !self.body_uses_receiver_state(fd)
            && fd.body.as_ref().map_or(false, |b| Self::block_uses_self_ident(b));
        let self_llvm_ty = if has_self_param {
            fd.receiver.as_ref().map(|r| {
                let base = self.llvm_type_for(&r.name).unwrap_or_else(|_| "i64".to_string());
                // BUG 31: a `self` param whose body ASSIGNS to self fields
                // (Formatter.write_int's `self.buf = ...`) must pass BY
                // POINTER even when not declared `mut` -- the caller's copy
                // would never see the mutation (finish() returned "").
                // BUG 38b (iter family): BARE receiver-field assignments
                // (`start = start + 1` in Range.next) need the same pointer
                // ABI -- a by-value copy silently dropped the mutation and
                // iterators looped forever on the first element.
                let is_mut = fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self)
                    || self.block_mutates_self(fd)
                    || self.block_mutates_receiver_state(fd);
                if is_mut && base.starts_with('%') { format!("{base}*") } else { base }
            })
        } else if is_this_based {
            // `this`-based methods: allocate a pointer-typed self slot so
            // the body can access receiver fields through `this`/`self`.
            fd.receiver.as_ref().map(|r| {
                let base = self.llvm_type_for(&r.name).unwrap_or_else(|_| "i64".to_string());
                if base.starts_with('%') { format!("{base}*") } else { base }
            })
        } else if body_uses_self {
            // 5c.32: by-value self method -- body uses `self` variable.
            // Pass the struct by POINTER so mutations propagate to the
            // caller's storage. The callee loads from the pointer into
            // its alloca and stores back through the pointer on return.
            fd.receiver.as_ref().map(|r| {
                let base = self.llvm_type_for(&r.name).unwrap_or_else(|_| "i64".to_string());
                if base.starts_with('%') { format!("{base}*") } else { base }
            })
        } else {
            None
        };
        let self_offset: usize = if self_llvm_ty.is_some() { 1 } else { 0 };

        let mut params_str: Vec<String> = Vec::new();
        if let Some(ref st) = self_llvm_ty {
            params_str.push(format!("{st} %param_self"));
        }
        let explicit_params: Vec<String> = fd.params.iter()
            .filter(|p| !(self_offset == 1 && p.name.name == "self"))
            .enumerate()
            .map(|(i, p)| {
                // param_llvm_type: tuple params get module-qualified concrete
                // names; struct `&T` params pass the ADDRESS (%struct.X*).
                let llvm_ty = self.param_llvm_type(&p.ty);
                format!("{llvm_ty} %param{}", i + self_offset)
            })
            .collect();
        params_str.extend(explicit_params);

        // Flush deferred concrete struct types (Option__Point etc.) BEFORE
        // the function header so they appear at LLVM top level.
        self.flush_deferred_types();

        // M20-A1: Flush any pre-body struct definitions (e.g. closure env structs)
        // before the function definition so they're visible inside the body.
        for pre_def in std::mem::take(&mut self.local.deferred_pre_body_defs) {
            self.output.push_str(&pre_def);
        }
        // R1: Emit DWARF subprogram metadata BEFORE the define line
        // (LLVM requires metadata nodes to be defined before they are referenced)
        let dbg_attach = if self.config.debug_symbols {
            let di_node = self.local.di_node_counter;
            self.local.di_node_counter += 1;
            let line = fd.name.span.line.max(1);
            // Stage 5: scope subsequent instructions in this body to the
            // function's DISubprogram (cleared at the epilogue).
            self.local.current_di_subprogram = Some(di_node);
            self.local.current_debug_loc = Some((line, fd.name.span.col.max(1)));
            // Emit the DISubprogram metadata inline, right before the define
            self.emitln(&format!("!{} = distinct !DISubprogram(name: \"{emit_symbol}\", linkageName: \"{emit_symbol}\", scope: !4, file: !4, line: {line}, type: !5, spFlags: DISPFlagDefinition, unit: !0)", di_node));
            format!(" !dbg !{}", di_node)
        } else {
            String::new()
        };
        // P0-4: Mark SMALL functions alwaysinline so clang/LLVM can eliminate
        // call overhead for small hot functions (e.g., read_u16_be called 17M
        // times). Large functions must NOT be alwaysinline: always-inlining
        // every function into a hot caller (e.g. the extended bigint smoke
        // inlines the whole library into main) makes LLVM's -O2 pass explode --
        // observed: clang hung >300s on a 735KB module, finishing in ~3s once
        // the attribute was removed. Medium functions get `inlinehint` so the
        // optimizer decides; large ones get no attribute.
        let body_size = fd.body.as_ref().map_or(0, |b| Self::approx_block_cost(b));
        let inline_attr = if body_size <= 10 {
            " alwaysinline"
        } else if body_size <= 48 {
            " inlinehint"
        } else {
            ""
        };
        // The program entry point `main` receives argc/argv from the OS so the
        // runtime's xiom_set_args can populate xiom_argc/xiom_argv (env.args()).
        // Native-only: wasm has no argc/argv and no xiom_set_args runtime link.
        let is_native = self.config.target_triple.contains("pc-windows")
            || self.config.target_triple.contains("unknown-linux")
            || self.config.target_triple.contains("apple-darwin");
        let is_entry_main = is_native && name == "main" && fd.params.is_empty() && fd.receiver.is_none();
        let main_sig = if is_entry_main {
            "i32 %argc, i8** %argv".to_string()
        } else {
            params_str.join(", ")
        };
        // LLVM define-line grammar: function ATTRIBUTES come before metadata
        // attachments, so `!dbg !N` must follow `alwaysinline` -- the old
        // `){dbg}{attrs} {` order made clang reject honest -g builds
        // ("expected '{' in function body" at the define line).
        self.emitln(&format!("define {ret_llvm} @{emit_symbol}({}){inline_attr}{} {{", main_sig, dbg_attach));

        // Recursion depth check
        let entry_block = self.fresh_block("entry");
        self.emitln(&format!("{entry_block}:"));
        if is_entry_main {
            // Seed the runtime arg table so env.args()/io.args() work.
            self.emitln("  call void @xiom_set_args(i32 %argc, i8** %argv)");
        }
        let depth_tmp = self.fresh_tmp();
        self.emitln(&format!("  {depth_tmp} = load i64, i64* @xiom_recursion_counter"));
        let new_depth = self.fresh_tmp();
        self.emitln(&format!("  {new_depth} = add i64 {depth_tmp}, 1"));
        let depth_ok = self.fresh_tmp();
        self.emitln(&format!("  {depth_ok} = icmp slt i64 {new_depth}, {}", self.config.max_recursion_depth));
        let trap_block = self.fresh_block("depth_trap");
        let ok_block = self.fresh_block("depth_ok");
        self.emitln(&format!("  br i1 {depth_ok}, label %{ok_block}, label %{trap_block}"));
        self.emitln(&format!("\n{trap_block}:"));
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln(&format!("\n{ok_block}:"));
        self.emitln(&format!("  store i64 {new_depth}, i64* @xiom_recursion_counter"));

        // Allocate parameters as locals
        // For methods, first allocate the self struct
        if let (Some(recv), Some(st)) = (fd.receiver.as_ref(), self_llvm_ty.as_ref()) {
            let self_alloca = self.fresh_tmp();
            // BUG 31: only STRUCT pointers (`%struct.X*`) take the pointer-
            // receiver branch. Str's i8* is the VALUE -- treating it as a
            // struct pointer registered self as the POINTEE ("i8"), so the
            // body `self` read the first BYTE of the string
            // (load i8 -> zext -> inttoptr -> strcmp(NULL) -> AV in
            // Str.to_str passthroughs).
            let is_ptr_receiver = st.ends_with('*')
                && st.trim_end_matches('*').starts_with("%struct.");
            self.emitln(&format!("  {self_alloca} = alloca {st}"));
            self.emitln(&format!("  store {st} %param_self, {st}* {self_alloca}"));
            if is_ptr_receiver {
                // Load the struct pointer from the alloca, then register
                // the loaded pointer as the base for field access.
                let loaded_ptr = self.fresh_tmp();
                let struct_ty = st.trim_end_matches('*');
                self.emitln(&format!("  {loaded_ptr} = load {st}, {st}* {self_alloca}"));
                self.add_local("self", loaded_ptr.clone(), struct_ty);
                // Add struct fields via GEP on the loaded pointer.
                // 5e.3: try types first, then type_meta (catalog-loaded structs).
                let types_fields = self.types.types.get(&recv.name)
                    .or_else(|| {
                        let suffix = format!(".{}", recv.name);
                        self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix))
                            .and_then(|k|self.types.types.get(&k))
                    })
                    ;
                let fields: Option<Vec<String>> = types_fields.or_else(|| {
                    self.types.type_meta.get(&recv.name).map(|m| {
                        m.fields.iter().map(|(n, _)| n.clone()).collect()
                    })
                });
                if let Some(fields) = fields {
                    for (idx, field_name) in fields.iter().enumerate() {
                        let field_llvm_ty = self.field_llvm_type(&recv.name, idx);
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {loaded_ptr}, i32 0, i32 {idx}"));
                        self.add_local(field_name, gep, &field_llvm_ty);
                    }
                }
            } else {
                self.add_local("self", self_alloca.clone(), st);
                // Also add struct fields as locals for direct access
                // 5e.3: types first, then type_meta for catalog-loaded structs.
                let types_fields = self.types.types.get(&recv.name)
                    .or_else(|| {
                        let suffix = format!(".{}", recv.name);
                        self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix)).and_then(|k|self.types.types.get(&k))
                    })
                    ;
                let fields: Option<Vec<String>> = types_fields.or_else(|| {
                    self.types.type_meta.get(&recv.name).map(|m| {
                        m.fields.iter().map(|(n, _)| n.clone()).collect()
                    })
                });
                if let Some(fields) = fields {
                    let alloca_ref = self_alloca;
                    for (idx, field_name) in fields.iter().enumerate() {
                        let field_llvm_ty = self.field_llvm_type(&recv.name, idx);
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {st}, {st}* {alloca_ref}, i32 0, i32 {idx}"));
                        self.add_local(field_name, gep, &field_llvm_ty);
                    }
                }
            }
        }
        // Then allocate explicit parameters. Number them by their position in the
        // EMITTED signature (which skips the duplicate `self` in fd.params), using
        // a counter that only advances for emitted params --  keeping %paramN indices
        // in lock-step with the signature above.
        let mut emitted_param_idx = self_offset;
        for param in fd.params.iter() {
            // A `self`-receiver method records `self` in BOTH fd.receiver and
            // fd.params (the parser does this). The receiver block above already
            // bound the `self` local to the receiver struct; skip the duplicate
            // here (it is also filtered from the signature), so `self` refers to
            // the real struct receiver and `match self` works.
            if self_offset == 1 && param.name.name == "self" {
                continue;
            }
            let llvm_ty = self.param_llvm_type(&param.ty);
            let alloca = self.fresh_tmp();
            let param_idx = emitted_param_idx;
            emitted_param_idx += 1;
            // D1: i128/fp128 params need 16-byte aligned allocas (x86-64).
            self.emitln(&format!("  {alloca} = alloca {llvm_ty}{}", self.alloca_align(&llvm_ty)));
            self.emitln(&format!("  store {llvm_ty} %param{param_idx}, {llvm_ty}* {alloca}{}", self.store_align(&llvm_ty)));
            self.add_local(&param.name.name, alloca, &llvm_ty);
            self.local.param_locals.insert(param.name.name.clone());
            // Track plain `&T` ref params (address carried as i64) so deref and
            // eq/compare can load through them. &mut T / *T are real pointers.
            // STRUCT-typed &T params are excluded: they are real `%struct.X*`
            // pointers now (param_llvm_type), so the i64-address deref path
            // must not fire for them.
            if let Type::Ref(inner) = &param.ty {
                let inner_llvm = self.llvm_type_for(&Self::type_from_ast(inner)).unwrap_or_else(|_| "i64".to_string());
                if !inner_llvm.starts_with("%struct.") {
                    self.local.ref_params.insert(param.name.name.clone());
                }
            }
            // 5c.39: Track Vec element type for function parameters so
            // downstream local bindings (var x = param) can inherit it.
            if let Some(elem) = Self::vec_elem_from_type_annotation(&param.ty) {
                self.local.local_vec_elem.insert(param.name.name.clone(), elem);
            }
            // M17: Track parameter signedness for narrow-int widening.
            let xiom_ty_name = Self::type_from_ast(&param.ty);
            // round-8 (rw1): KEEP a top-level reference in the tracked XIOM
            // name ("&Str" / "&mut Int") -- type_from_ast erases the ref, but a
            // reference-typed value holds the T SLOT ADDRESS at the ABI and
            // value uses (strcmp/icmp) must auto-deref it; the & is the only
            // discriminator from a plain T. auto_deref_ref consumes this.
            let xiom_ty_name = Self::ref_preserving_name(&param.ty).unwrap_or(xiom_ty_name.clone());
            self.local.local_xiom_types.insert(param.name.name.clone(), xiom_ty_name.clone());
            // LET-array P2 (docs/LET_ARRAY_DECISION.md): a `&[N]T` /
            // `&mut [N]T` param is an ELEMENT pointer (param_llvm_type now
            // matches the mono catalog ABI), so record the element type and
            // const N for the body's element reads/writes and const-generic
            // inference (`array.len(a)` inside the callee -> N=3). Mirrors
            // the mono Ref-Array binding in compile_generic_monomorphisations.
            if let Type::Ref(inner) | Type::MutRef(inner) = &param.ty {
                if let Type::Array(size_expr, elem) = inner.as_ref() {
                    let elem_xiom = Self::type_from_ast(elem);
                    let elem_llvm = self.llvm_type_for(&elem_xiom).unwrap_or_else(|_| "i8".to_string());
                    if !elem_llvm.is_empty()
                        && !elem_llvm.starts_with('[')
                        && !elem_llvm.starts_with('%')
                    {
                        self.local.local_array_elem.insert(param.name.name.clone(), elem_llvm);
                        self.local.local_xiom_types.insert(param.name.name.clone(), elem_xiom);
                    }
                    if let Expr::Int(n, _) = size_expr.as_ref() {
                        self.local.local_array_sizes.insert(param.name.name.clone(), *n as i64);
                    }
                }
            }
            // B-007: fn-typed PARAMS hold a closure ENV pointer (field 0 =
            // the fn ptr) -- calling `f(x)` inside the body must go through
            // the M20-A1 closure path (load the fn ptr from the env struct),
            // NOT inttoptr the env pointer as a code pointer (0xC0000005).
            if let Type::Fn(_, ret) = &param.ty {
                self.local.closure_locals.insert(param.name.name.clone());
                self.local.fn_local_returns.insert(param.name.name.clone(), Self::type_from_ast(ret));
            }
            if Self::is_signed_xiom_type(&xiom_ty_name) {
                self.local.signed_locals.insert(param.name.name.clone());
            } else {
                self.local.signed_locals.remove(&param.name.name);
            }
            // gzip-DECOMPRESS fix (2026-08-19): track Option/Result PARAM
            // payload types (mirrors the let/var tracking in stmt.rs) so
            // payload-FIELD access on a param (`r.value` for a
            // Result[Vec[UInt8], Str] param) resolves the payload type --
            // type_from_ast renders the bare "Result", losing the args.
            // or_insert keeps the FIRST (value) type: option_type_param's
            // "Result" arm returns the ERROR type for a direct Type::Result.
            if let Some(opt_inner) = Self::option_type_param(&param.ty, "Option") {
                self.local.local_opt_payload.entry(param.name.name.clone()).or_insert(opt_inner);
            }
            if let Some(res_val) = Self::option_type_param(&param.ty, "Result") {
                self.local.local_opt_payload.entry(param.name.name.clone()).or_insert(res_val);
            }
            if let Some(err_val) = Self::result_err_type_param(&param.ty) {
                self.local.local_err_payload.insert(param.name.name.clone(), err_val);
            }
            // Record raw-pointer params (`*T`/`&T`/`&mut T` over a pointer) so that
            // `param[i]` indexing treats the i64 value as an address (byte buffer).
            if matches!(&param.ty, Type::Ptr(_)) {
                self.local.ptr_locals.insert(param.name.name.clone());
            } else {
                self.local.ptr_locals.remove(&param.name.name);
            }
            // Track function pointer return types for function pointer parameters
            if let Type::Fn(_, ret) = &param.ty {
                let ret_ty_name = Self::type_from_ast(ret);
                let ret_llvm = self.llvm_type_for_fallback(&ret_ty_name);
                self.types.fn_ptr_return_types.insert(param.name.name.clone(), ret_llvm);
            }
        }

        // Capture self@pre for ensures (method functions with self@pre references)
        if self.config.check_contracts && !self.fctx.current_ensures.is_empty() {
            // Phase 5c @pre snapshot: for every variable referenced in an
            // `ensures` clause with `@pre`, store its entry-point value so the
            // ensures check uses the pre-state value, not the current one.
            let mut pre_vars: HashSet<String> = HashSet::new();
            for expr in &self.fctx.current_ensures {
                Self::collect_atpre_vars(expr, &mut pre_vars);
            }
            // R25: SORT the snapshot worklist -- HashSet iteration assigned the
            // per-field pre-slots in a different order per run, shifting the
            // emitted alloca numbering (Gauge.adjust: `%tmp9/%tmp11/%tmp13`
            // permuted across runs; same semantics, non-identical IR).
            let mut pre_vars_sorted: Vec<String> = pre_vars.into_iter().collect();
            pre_vars_sorted.sort();
            self.fctx.pre_snapshot_vars = pre_vars_sorted.clone();
            for var_name in &pre_vars_sorted {
                if let Some((ptr, llvm_ty)) = self.lookup_local(var_name).cloned() {
                    // For `&mut T` parameters (llvm_ty ends with `*`), the local
                    // holds a pointer. We must snapshot the POINTED-TO VALUE, not
                    // the pointer itself. Alloca the inner struct type, dereference
                    // the pointer, and store the struct copy.
                    if llvm_ty.ends_with('*') {
                        let inner_ty = llvm_ty.trim_end_matches('*');
                        let pre_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {pre_alloca} = alloca {inner_ty}"));
                        let loaded_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {loaded_ptr} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                        let loaded_val = self.fresh_tmp();
                        self.emitln(&format!("  {loaded_val} = load {inner_ty}, {inner_ty}* {loaded_ptr}"));
                        self.emitln(&format!("  store {inner_ty} {loaded_val}, {inner_ty}* {pre_alloca}"));
                        self.snapshot_deep_copy_vec_fields(var_name, &pre_alloca, inner_ty);
                        let pre_name = format!("__{}_pre", var_name);
                        self.add_local(&pre_name, pre_alloca, inner_ty);
                    } else {
                        let pre_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {pre_alloca} = alloca {llvm_ty}"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                        self.emitln(&format!("  store {llvm_ty} {loaded}, {llvm_ty}* {pre_alloca}"));
                        self.snapshot_deep_copy_vec_fields(var_name, &pre_alloca, &llvm_ty);
                        let pre_name = format!("__{}_pre", var_name);
                        self.add_local(&pre_name, pre_alloca, &llvm_ty);
                    }
                }
            }
            // Also snapshot self receiver (backward compat)
            if let Some(recv) = fd.receiver.as_ref() {
                if let Some((ptr, llvm_ty)) = self.lookup_local(&recv.name).cloned() {
                    if !pre_vars_sorted.contains(&recv.name) {
                        let pre_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {pre_alloca} = alloca {llvm_ty}"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                        self.emitln(&format!("  store {llvm_ty} {loaded}, {llvm_ty}* {pre_alloca}"));
                        self.add_local("__self_pre", pre_alloca, &llvm_ty);
                    }
                }
            }
        }

        // Create result alloca for ensures if function returns a value
        self.fctx.result_ptr = None;
        self.fctx.result_llvm_ty = None;
        if !self.fctx.current_ensures.is_empty() && fd.return_type.is_some() {
            let result_alloca = self.fresh_tmp();
            self.emitln(&format!("  {result_alloca} = alloca {ret_llvm}"));
            self.add_local("result", result_alloca.clone(), &ret_llvm);
            self.fctx.result_llvm_ty = Some(ret_llvm.clone());
            // BUG 29 (contract Some-payload ensures): track the RETURN value's
            // XIOM type so `result is Some => result.len() > 0` can dispatch
            // the Some-bound payload as Str (not fall through to Map.len).
            // BUG 30: MUST use type_string_full (like fn_return_xiom) --
            // type_from_ast_with_args drops Option/Result payload args, so
            // `Result[Vec[UInt8], Str]` became "Result" and the payload rebind
            // recorded NO type -> `result.len()` went down the Str path and
            // xiom_str_len'd a boxed Vec handle (AV, smoke_utf8).
            if let Some(rt) = &fd.return_type {
                self.local.local_xiom_types.insert("result".to_string(), Self::type_string_full(rt));
            }
            self.fctx.result_ptr = Some(result_alloca);
        }

        // Emit requires checks at function entry
        if self.config.check_contracts {
            for clause in &fd.contracts {
                if let ContractClause::Requires(expr, _span) = clause {
                    self.compile_contract_check(expr, "requires");
                }
            }
        }

        // Compile body
        if let Some(body) = fd.body.as_ref() {
            self.compile_block(body, fd.return_type.is_some())?;
        }
        
        // Implicit return
        if ret_llvm == "void" {
            // Check ensures before implicit void return
            if !self.fctx.current_ensures.is_empty() {
                self.compile_ensures_checks();
            }
            if self.fctx.is_never_return {
                // P2-4: Never-returning functions must not emit `ret`.
                self.emitln("  unreachable");
            } else {
                // P0-2: Emit deferred cleanups before return
                self.compile_deferred_cleanups()?;
                // Decrement recursion depth
                let depth_dec = self.fresh_tmp();
                self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
                let new_depth_dec = self.fresh_tmp();
                self.emitln(&format!("  {new_depth_dec} = sub i64 {depth_dec}, 1"));
                self.emitln(&format!("  store i64 {new_depth_dec}, i64* @xiom_recursion_counter"));
                self.emitln("  ret void");
            }
        } else if !self.current_block_terminated() {
            // P2-4: Never-returning functions -- emit unreachable instead of ret
            if self.fctx.is_never_return {
                self.emitln("  unreachable");
            } else {
            // A4 fix: the function declares a return type but control reached the
            // end of the body without a terminator --  the body ends in a loop, an
            // `if` without `else`, or a trailing statement, so no tail `ret` was
            // emitted. Append a safe fallback return so the trailing block is
            // terminated and the module is valid LLVM IR. (Functions that already
            // end in a tail expression / explicit return report `terminated`, so
            // their IR is unchanged and no double terminator is produced.)
            let depth_dec = self.fresh_tmp();
            self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
            let new_depth_dec = self.fresh_tmp();
            self.emitln(&format!("  {new_depth_dec} = sub i64 {depth_dec}, 1"));
            self.emitln(&format!("  store i64 {new_depth_dec}, i64* @xiom_recursion_counter"));
            let zero = Self::default_const_for(&ret_llvm);
            self.emitln(&format!("  ret {ret_llvm} {zero}"));
            } // close is_never else block
        }

        self.emitln("}\n");

        // 5e.5a: emit hot reload thunk for pub functions
        if self.config.hot_reload && fd.is_pub {
            // The thunk hash must match the REGISTERED symbol (pub_functions
            // uses fn_symbol, the preassigned map) -- not fn_key, which is
            // module-qualified for methods inside `module X { ... }`.
            let thunk_name = format!("xiom_hot_thunk_{}", emit_symbol);
            let hash = Self::djb2_hash(&emit_symbol);

            // Build the thunk signature and forwarding param list.
            // Uses simple p0, p1, ... naming to avoid matching complexities.
            let mut thunk_param_decls: Vec<String> = Vec::new();
            let mut thunk_param_names: Vec<String> = Vec::new();
            let mut thunk_param_types: Vec<String> = Vec::new();
            let mut idx: usize = 0;

            if let Some(ref st) = self_llvm_ty {
                let pname = format!("%p{idx}");
                thunk_param_decls.push(format!("{st} {pname}"));
                thunk_param_names.push(pname);
                thunk_param_types.push(st.clone());
                idx += 1;
            }
            for (i, p) in fd.params.iter().enumerate() {
                if self_offset == 1 && p.name.name == "self" { continue; }
                let llvm_ty = if i < self.fctx.current_param_llvm_types.len() {
                    self.fctx.current_param_llvm_types[i].clone()
                } else {
                    "i64".to_string()
                };
                let pname = format!("%p{idx}");
                thunk_param_decls.push(format!("{llvm_ty} {pname}"));
                thunk_param_names.push(pname);
                thunk_param_types.push(llvm_ty);
                idx += 1;
            }

            let params_join = thunk_param_decls.join(", ");
            let param_names_join = thunk_param_names.join(", ");
            let param_types_join = thunk_param_types.join(", ");

            // Handle void return specially
            let is_void = ret_llvm == "void";
            let fn_ptr_ty = if is_void {
                format!("void ({param_types_join})*")
            } else {
                format!("{ret_llvm} ({param_types_join})*")
            };

            // Emit the thunk: lazy self-registration on first call via xiom_hot_get_ptr
            self.emitln(&format!("define {ret_llvm} @{thunk_name}({params_join}) {{"));
            self.emitln("entry:");
            let ptr_i64 = self.fresh_tmp();
            self.emitln(&format!("  {ptr_i64} = call i64 @xiom_hot_get_ptr(i64 {hash})"));
            let is_null = self.fresh_tmp();
            self.emitln(&format!("  {is_null} = icmp eq i64 {ptr_i64}, 0"));
            let reg_block = self.fresh_block("hot_reg");
            let call_block = self.fresh_block("hot_call");
            self.emitln(&format!("  br i1 {is_null}, label %{reg_block}, label %{call_block}"));
            self.emitln(&format!("\n{reg_block}:"));
            self.emitln(&format!("  call void @xiom_hot_set_ptr(i64 {hash}, i64 ptrtoint ({fn_ptr_ty} @{name} to i64))"));
            self.emitln(&format!("  br label %{call_block}"));
            self.emitln(&format!("\n{call_block}:"));
            let ptr_phi = self.fresh_tmp();
            self.emitln(&format!("  {ptr_phi} = phi i64 [ {ptr_i64}, %entry ], [ ptrtoint ({fn_ptr_ty} @{name} to i64), %{reg_block} ]"));
            let fp = self.fresh_tmp();
            self.emitln(&format!("  {fp} = inttoptr i64 {ptr_phi} to {fn_ptr_ty}"));
            if is_void {
                self.emitln(&format!("  call void {fp}({param_names_join})"));
                self.emitln("  ret void");
            } else {
                let result = self.fresh_tmp();
                self.emitln(&format!("  {result} = call {ret_llvm} {fp}({param_names_join})"));
                self.emitln(&format!("  ret {ret_llvm} {result}"));
            }
            self.emitln("}\n");
        }

        self.pop_scope();
        self.fctx.current_fn = None;
        self.fctx.current_receiver = None;
        self.fctx.current_ensures.clear();
        self.fctx.result_ptr = None;
        // Stage 5: leave the function's DWARF scope (deferred closures etc.
        // must not attach locations to the enclosing subprogram).
        self.local.current_di_subprogram = None;
        self.local.current_debug_loc = None;
        // BUG 22 #6: splice loop-body-hoisted allocas into the entry block.
        self.finish_hoisted_allocas();
        // M20-A1: Emit any deferred closure function definitions
        self.flush_deferred_closures();
        Ok(())
    }

    /// R49 (stdlib relay p_pre_call_capture): a snapshot COPY of a struct
    /// shares its Vec fields' DATA POINTERS, so `@pre` element reads saw the
    /// live (post-mutation) buffer -- `total(b) == total(b)@pre + 1` violated.
    /// Clone each inline `%struct.Vec` field's buffer into fresh memory and
    /// point the snapshot header at it. Shallow per face: nested container
    /// elements inside the cloned buffer are still aliased (documented limit).
    fn snapshot_deep_copy_vec_fields(&mut self, var_name: &str, pre_alloca: &str, struct_llvm_ty: &str) {
        let raw_name = match self.local.local_xiom_types.get(var_name) {
            Some(t) => t.clone(),
            None => return,
        };
        let struct_name = raw_name
            .trim_start_matches('&')
            .trim_start_matches("mut ")
            .trim()
            .to_string();
        let struct_name = struct_name.split('[').next().unwrap_or(&struct_name).to_string();
        let key = match self.types.type_meta.get(&struct_name) {
            Some(_) => struct_name.clone(),
            None => {
                let suffix = format!(".{struct_name}");
                match self.types.type_meta.keys().into_iter().find(|k| k.ends_with(&suffix)) {
                    Some(k) => k.to_string(),
                    None => return,
                }
            }
        };
        let vec_fields: Vec<(usize, String)> = match self.types.type_meta.get(&key) {
            Some(m) => m
                .fields
                .iter()
                .enumerate()
                .filter(|(_, (_, t))| t.starts_with("Vec["))
                .map(|(i, (_, t))| (i, t.clone()))
                .collect(),
            None => return,
        };
        for (fi, _fty) in vec_fields {
            let field_llvm = self.field_llvm_type(&key, fi);
            if field_llvm != "%struct.Vec" { continue; }
            let fgep = self.fresh_tmp();
            self.emitln(&format!("  {fgep} = getelementptr {struct_llvm_ty}, {struct_llvm_ty}* {pre_alloca}, i32 0, i32 {fi}"));
            let dgep = self.fresh_tmp();
            self.emitln(&format!("  {dgep} = getelementptr %struct.Vec, %struct.Vec* {fgep}, i32 0, i32 0"));
            let lgep = self.fresh_tmp();
            self.emitln(&format!("  {lgep} = getelementptr %struct.Vec, %struct.Vec* {fgep}, i32 0, i32 1"));
            let egep = self.fresh_tmp();
            self.emitln(&format!("  {egep} = getelementptr %struct.Vec, %struct.Vec* {fgep}, i32 0, i32 3"));
            let len = self.fresh_tmp();
            self.emitln(&format!("  {len} = load i64, i64* {lgep}"));
            let esz = self.fresh_tmp();
            self.emitln(&format!("  {esz} = load i64, i64* {egep}"));
            let bytes = self.fresh_tmp();
            self.emitln(&format!("  {bytes} = mul i64 {len}, {esz}"));
            let nz = self.fresh_tmp();
            let do_lbl = self.fresh_block("preclone_do");
            let done_lbl = self.fresh_block("preclone_done");
            self.emitln(&format!("  {nz} = icmp sgt i64 {bytes}, 0"));
            self.emitln(&format!("  br i1 {nz}, label %{do_lbl}, label %{done_lbl}"));
            self.emitln(&format!("\n{do_lbl}:"));
            let src = self.fresh_tmp();
            self.emitln(&format!("  {src} = load i8*, i8** {dgep}"));
            let buf = self.fresh_tmp();
            self.emitln(&format!("  {buf} = call i8* @malloc(i64 {bytes})"));
            let ok_lbl = self.fresh_block("preclone_ok");
            let fail_lbl = self.fresh_block("preclone_fail");
            let chk = self.fresh_tmp();
            self.emitln(&format!("  {chk} = icmp eq i8* {buf}, null"));
            self.emitln(&format!("  br i1 {chk}, label %{fail_lbl}, label %{ok_lbl}"));
            self.emitln(&format!("\n{fail_lbl}:"));
            self.emitln("  call void @llvm.trap()");
            self.emitln("  unreachable");
            self.emitln(&format!("\n{ok_lbl}:"));
            self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {buf}, i8* {src}, i64 {bytes}, i1 false)"));
            self.emitln(&format!("  store i8* {buf}, i8** {dgep}"));
            self.emitln(&format!("  br label %{done_lbl}"));
            self.emitln(&format!("\n{done_lbl}:"));
        }
    }

    /// Return the LLVM struct type for an `Option<Inner>` with the given
    /// Collect all variable names referenced through `@pre` in an expression.
    pub(crate) fn collect_atpre_vars(expr: &Expr, vars: &mut HashSet<String>) {
        match expr {
            Expr::AtPre(inner, _) => {
                if let Expr::Ident(id) = inner.as_ref() {
                    vars.insert(id.name.clone());
                } else {
                    // R49 (stdlib relay p_pre_call_capture): `expr@pre` on a
                    // CALL/FIELD wraps the whole expression; every VARIABLE
                    // inside it is evaluated in the pre-state, so collect them
                    // all. The old recursion only handled variables DIRECTLY
                    // under AtPre (Ident) -- `total(b)@pre` collected NOTHING,
                    // no snapshot was emitted, and the ensures read post-state.
                    Self::collect_pre_idents(inner, vars);
                    Self::collect_atpre_vars(inner, vars);
                }
            }
            Expr::Binary(l, _, r, _) => { Self::collect_atpre_vars(l, vars); Self::collect_atpre_vars(r, vars); }
            // R51 (stdlib p_pre_capture_callee): contract clauses are usually
            // IMPLICATIONS (`result is Some => total(b) == total(b)@pre - 1`).
            // Without an Imply arm the walk never reached the @pre, no entry
            // snapshot was emitted, and the ensures compared the LIVE pointer
            // with itself (2 == 2 - 1 violations in list/queue/rbtree/fenwick
            // and every other implication-wrapped size clause).
            Expr::Imply(l, r, _) => { Self::collect_atpre_vars(l, vars); Self::collect_atpre_vars(r, vars); }
            Expr::Is(e, _, _) => Self::collect_atpre_vars(e, vars),
            Expr::Unary(_, e, _) => Self::collect_atpre_vars(e, vars),
            Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => { Self::collect_atpre_vars(f, vars); for a in args { Self::collect_atpre_vars(a, vars); } }
            Expr::Field(e, _, _) | Expr::Index(e, _, _) => Self::collect_atpre_vars(e, vars),
            Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _) => Self::collect_atpre_vars(e, vars),
            _ => {}
        }
    }

    /// R49: collect every identifier name appearing in an expression. Used for
    /// pre-state snapshots under `expr@pre` (a call/field chain evaluates ALL
    /// of its variables at entry). Unknown variants are skipped conservatively.
    pub(crate) fn collect_pre_idents(expr: &Expr, vars: &mut HashSet<String>) {
        match expr {
            Expr::Ident(id) => { vars.insert(id.name.clone()); }
            Expr::Binary(l, _, r, _) => { Self::collect_pre_idents(l, vars); Self::collect_pre_idents(r, vars); }
            // R51: implications and `is` tests wrap most contract clauses --
            // descend so `... @pre ...` inside them gets a snapshot.
            Expr::Imply(l, r, _) => { Self::collect_pre_idents(l, vars); Self::collect_pre_idents(r, vars); }
            Expr::Is(e, _, _) => Self::collect_pre_idents(e, vars),
            Expr::Unary(_, e, _) | Expr::Paren(e, _) | Expr::AtPre(e, _) | Expr::Try(e, _) => Self::collect_pre_idents(e, vars),
            Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => {
                Self::collect_pre_idents(f, vars);
                for a in args { Self::collect_pre_idents(a, vars); }
            }
            Expr::Field(e, _, _) | Expr::Index(e, _, _) => Self::collect_pre_idents(e, vars),
            Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _) => Self::collect_pre_idents(e, vars),
            Expr::Struct(_, fields, _, _) => {
                for (_, fe) in fields { Self::collect_pre_idents(fe, vars); }
            }
            Expr::Tuple(items, _) | Expr::Array(items, _) => {
                for i in items { Self::collect_pre_idents(i, vars); }
            }
            Expr::As(e, _, _) => Self::collect_pre_idents(e, vars),
            _ => {}
        }
    }

    /// Emit any deferred struct type definitions (concrete Option__Point,
    /// Result__X__Y, etc.) before the next function body.
    pub(crate) fn flush_deferred_types(&mut self) {
        if self.local.deferred_struct_types.is_empty() { return; }
        for (name, body) in std::mem::take(&mut self.local.deferred_struct_types) {
            self.emitln(&format!("%struct.{name} = type {body}"));
        }
        if !self.local.deferred_struct_types.is_empty() {
            self.emitln("");
        }
    }

    // ========================================================================
    // 5c.36: Pre-register expression-level tuple types
    // ========================================================================

    /// Approximate cost of a function body for the inline-attribute policy:
    /// 1 per statement/expression, recursing into nested blocks (if/while/
    /// for/match/spawn). Used to avoid `alwaysinline` on large functions,
    /// which makes LLVM's -O2 hang when a hot caller inlines the whole
    /// library (see the `inline_attr` policy at the define-line emission).
    pub(crate) fn approx_block_cost(block: &Block) -> usize {
        Self::block_stmt_cost(block)
    }

    fn block_stmt_cost(block: &Block) -> usize {
        block.stmts.iter().map(Self::stmt_cost).sum()
    }

    fn stmt_cost(stmt: &StmtOrExpr) -> usize {
        match stmt {
            StmtOrExpr::Expr(_) => 1,
            StmtOrExpr::Stmt(s) => 1 + match s {
                Stmt::If(_, t, elifs, e, _) => {
                    Self::block_stmt_cost(t)
                        + elifs.iter().map(|(_, b)| Self::block_stmt_cost(b)).sum::<usize>()
                        + e.as_ref().map_or(0, Self::block_stmt_cost)
                }
                Stmt::While(_, b, _, _, _) => Self::block_stmt_cost(b),
                Stmt::For(_, _, b, _, _) => Self::block_stmt_cost(b),
                Stmt::Match(_, arms, _) => arms.iter().map(|arm| match &arm.body {
                    MatchBody::Block(b) => Self::block_stmt_cost(b),
                    MatchBody::Expr(_) => 1,
                }).sum(),
                Stmt::Spawn(b, _, _) => Self::block_stmt_cost(b),
                _ => 0,
            },
        }
    }

    /// the corresponding `Tuple__Type1__Type2` struct types so their LLVM
    /// definitions are emitted at module level before any function bodies.
    /// M65 R7: collect every TYPE ANNOTATION reachable in a declaration tree
    /// (params, returns, local let/var annotations, As casts, nested blocks).
    /// The emitter then runs concrete_type_for over them so concrete container
    /// definitions land in the type-decl emission block.
    fn collect_annotation_types(item: &TopDecl, out: &mut Vec<Type>) {
        match item {
            TopDecl::Fn(fd) => {
                for p in &fd.params { out.push(p.ty.clone()); }
                if let Some(rt) = &fd.return_type { out.push(rt.clone()); }
                if let Some(ref body) = fd.body { Self::collect_block_annotations(body, out); }
            }
            TopDecl::Module(md) => {
                for item in &md.items { Self::collect_annotation_types(item, out); }
            }
            TopDecl::Type(td) => {
                for f in &td.fields { out.push(f.ty.clone()); }
            }
            _ => {}
        }
    }

    fn collect_block_annotations(block: &Block, out: &mut Vec<Type>) {
        for stmt in &block.stmts {
            match stmt {
                StmtOrExpr::Stmt(s) => Self::collect_stmt_annotations(s, out),
                StmtOrExpr::Expr(e) => Self::collect_expr_annotations(e, out),
            }
        }
    }

    fn collect_stmt_annotations(stmt: &Stmt, out: &mut Vec<Type>) {
        match stmt {
                Stmt::Let(_, ty, init, _) | Stmt::Var(_, ty, init, _) => {
                    if let Some(t) = ty { out.push((**t).clone()); }
                    Self::collect_expr_annotations(init, out);
                }
            Stmt::Assign(l, r, _) => { Self::collect_expr_annotations(l, out); Self::collect_expr_annotations(r, out); }
            Stmt::If(c, b, elifs, eb, _) => {
                Self::collect_expr_annotations(c, out);
                Self::collect_block_annotations(b, out);
                for (c2, b2) in elifs { Self::collect_expr_annotations(c2, out); Self::collect_block_annotations(b2, out); }
                if let Some(b2) = eb { Self::collect_block_annotations(b2, out); }
            }
            Stmt::While(c, b, _, _, _) => { Self::collect_expr_annotations(c, out); Self::collect_block_annotations(b, out); }
            Stmt::For(_, it, b, _, _) => { Self::collect_expr_annotations(it, out); Self::collect_block_annotations(b, out); }
            Stmt::Match(scr, arms, _) => {
                Self::collect_expr_annotations(scr, out);
                for arm in arms {
                    match &arm.body {
                        MatchBody::Block(b) => Self::collect_block_annotations(b, out),
                        MatchBody::Expr(e) => Self::collect_expr_annotations(e, out),
                    }
                }
            }
            Stmt::Return(Some(e), _) | Stmt::Expr(e, _) => Self::collect_expr_annotations(e, out),
            _ => {}
        }
    }

    /// M65 regex-family fix: extract the bracketed type argument of a
    /// constructor callee -- `Vec[Option[X]].new` parses as
    /// Field(Index(Ident(Vec), Option[X]), new); also accept Index(Field(..)).
    fn call_bracket_type_arg(f: &Expr) -> Option<&Expr> {
        match f {
            Expr::Field(obj, m, _) if matches!(m.name.as_str(), "new" | "with_capacity" | "fill") => {
                match obj.as_ref() {
                    Expr::Index(_, idx, _) => Some(idx.as_ref()),
                    _ => None,
                }
            }
            Expr::Index(base, idx, _) => match base.as_ref() {
                Expr::Field(_, m, _) if matches!(m.name.as_str(), "new" | "with_capacity" | "fill") => Some(idx.as_ref()),
                _ => None,
            },
            _ => None,
        }
    }

    fn collect_expr_annotations(expr: &Expr, out: &mut Vec<Type>) {
        match expr {
            Expr::As(_, ty, _) => { out.push(ty.clone()); }
            Expr::Tuple(items, _) | Expr::Array(items, _) => { for i in items { Self::collect_expr_annotations(i, out); } }
            Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => {
                // M65 regex-family fix: bracketed CONSTRUCTOR type args
                // (`Vec[Option[VecPayload]].new()`) must pre-register their
                // concrete Option/Result element types -- otherwise the type
                // is first created while compiling the body and clang rejects
                // the alloca ("Cannot allocate unsized type").
                if let Some(ta) = Self::call_bracket_type_arg(f) {
                    let rendered = Self::type_arg_to_name(ta);
                    if rendered.contains('[') {
                        out.push(Self::synth_type_named(&rendered));
                    }
                }
                Self::collect_expr_annotations(f, out);
                for a in args { Self::collect_expr_annotations(a, out); }
            }
            Expr::Binary(a, _, b, _) => { Self::collect_expr_annotations(a, out); Self::collect_expr_annotations(b, out); }
            Expr::Unary(_, e, _) | Expr::Paren(e, _) | Expr::Ref(e, _) | Expr::MutRef(e, _)
            | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _) | Expr::Try(e, _) => Self::collect_expr_annotations(e, out),
            Expr::Field(o, _, _) => Self::collect_expr_annotations(o, out),
            Expr::Index(a, i, _) => { Self::collect_expr_annotations(a, out); Self::collect_expr_annotations(i, out); }
            Expr::If(c, b, elifs, eb, _) => {
                Self::collect_expr_annotations(c, out);
                Self::collect_block_annotations(b, out);
                for (c2, b2) in elifs { Self::collect_expr_annotations(c2, out); Self::collect_block_annotations(b2, out); }
                if let Some(b2) = eb { Self::collect_block_annotations(b2, out); }
            }
            Expr::Match(scr, arms, _) => {
                Self::collect_expr_annotations(scr, out);
                for arm in arms {
                    match &arm.body {
                        MatchBody::Block(b) => Self::collect_block_annotations(b, out),
                        MatchBody::Expr(e) => Self::collect_expr_annotations(e, out),
                    }
                }
            }
            Expr::Struct(_, elems, _, _) => { for (_, e) in elems { Self::collect_expr_annotations(e, out); } }
            _ => {}
        }
    }

    pub(crate) fn register_expr_tuple_types(&mut self, item: &TopDecl) {
        // M65 R7: pre-register concrete container types referenced by BODY
        // ANNOTATIONS before the type-decl emission, so their definitions are
        // available when clang parses the function bodies (a trailing
        // definition is too late -- alloca/GEP demand SIZED types at parse
        // time). `var r: Result[Payload, Int]` must become
        // %struct.Result__Payload__Int in the decl block.
        let mut ann_types: Vec<Type> = Vec::new();
        Self::collect_annotation_types(item, &mut ann_types);
        for t in &ann_types {
            let _ = self.concrete_type_for(t);
        }
        match item {
            TopDecl::Fn(fd) => {
                if let Some(ref body) = fd.body {
                    let mut vars: HashMap<String, String> = HashMap::new();
                    // BUG 23 #7 fix: seed the tuple-scan variable map from the
                    // fn's PARAM types -- `fn mk_bb(b1: Bool, b2: Bool) -> (Bool, Bool)`
                    // previously registered Tuple__Int__Int (params defaulted to
                    // "Int" in the scan), mis-typing every cross-module Bool tuple.
                    for p in &fd.params {
                        vars.insert(p.name.name.clone(), Self::type_from_ast(&p.ty));
                    }
                    Self::scan_block_for_tuples(&mut self.types, body, &mut vars);
                }
            }
            TopDecl::Module(md) => {
                for item in &md.items {
                    self.register_expr_tuple_types(item);
                }
            }
            _ => {}
        }
    }

    fn scan_block_for_tuples(types: &mut TypeContext, block: &Block, vars: &mut HashMap<String, String>) {
        for stmt in &block.stmts {
            match stmt {
                StmtOrExpr::Stmt(s) => Self::scan_stmt_for_tuples(types, s, vars),
                StmtOrExpr::Expr(e) => Self::scan_expr_for_tuples(types, e, vars),
            }
        }
    }

    fn scan_stmt_for_tuples(types: &mut TypeContext, stmt: &Stmt, vars: &mut HashMap<String, String>) {
        match stmt {
            Stmt::Let(name, ty, init, _) | Stmt::Var(name, ty, init, _) => {
                let bound = if let Some(t) = ty {
                    Self::type_from_ast_static(t)
                } else {
                    Self::infer_var_type(init, vars)
                };
                vars.insert(name.name.clone(), bound);
                Self::scan_expr_for_tuples(types, init, vars);
            }
            Stmt::Assign(lhs, rhs, _) => {
                // Track re-assignment types for tuple inference consistency
                if let Expr::Ident(id) = lhs {
                    if let Some(rt) = Self::infer_simple_expr_type(rhs, vars) {
                        vars.insert(id.name.clone(), rt);
                    }
                }
                Self::scan_expr_for_tuples(types, lhs, vars);
                Self::scan_expr_for_tuples(types, rhs, vars);
            }
            Stmt::If(cond, then_b, elifs, else_b, _) => {
                Self::scan_expr_for_tuples(types, cond, vars);
                Self::scan_block_for_tuples(types, then_b, vars);
                for (c, b) in elifs { Self::scan_expr_for_tuples(types, c, vars); Self::scan_block_for_tuples(types, b, vars); }
                if let Some(b) = else_b { Self::scan_block_for_tuples(types, b, vars); }
            }
            Stmt::While(cond, body, _, _, _) => { Self::scan_expr_for_tuples(types, cond, vars); Self::scan_block_for_tuples(types, body, vars); }
            Stmt::For(_, iter, body, _, _) => { Self::scan_expr_for_tuples(types, iter, vars); Self::scan_block_for_tuples(types, body, vars); }
            Stmt::Match(scrut, arms, _) => {
                Self::scan_expr_for_tuples(types, scrut, vars);
                for arm in arms {
                    match &arm.body { MatchBody::Block(b) => Self::scan_block_for_tuples(types, b, vars), MatchBody::Expr(e) => Self::scan_expr_for_tuples(types, e, vars) }
                }
            }
            Stmt::Return(Some(e), _) | Stmt::Expr(e, _) => Self::scan_expr_for_tuples(types, e, vars),
            _ => {}
        }
    }

    fn scan_expr_for_tuples(types: &mut TypeContext, expr: &Expr, vars: &mut HashMap<String, String>) {
        // Register tuple types from expression-level tuples
        if let Expr::Tuple(items, _) = expr {
            if items.len() > 1 {
                let elem_types: Vec<String> = items.iter()
                    .map(|i| Self::infer_expr_type_name(i, vars))
                    .collect();
                let name = format!("Tuple__{}", elem_types.join("__"));
                if !types.type_meta.contains_key(&name) {
                    let field_names: Vec<String> = (0..elem_types.len()).map(|i| format!("_{i}")).collect();
                    let field_meta: Vec<(String, String)> = elem_types.iter().enumerate()
                        .map(|(i, tn)| (format!("_{i}"), tn.clone()))
                        .collect();
                    types.types.insert(name.clone(), field_names);
                    types.type_meta.or_insert_with(name.clone(), || TypeMeta {
                        fields: field_meta,
                        derives: vec![],
                        invariants: vec![],
                    });
                }
            }
        }
        match expr {
            Expr::Tuple(items, _) => { for item in items { Self::scan_expr_for_tuples(types, item, vars); } }
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => { Self::scan_expr_for_tuples(types, func, vars); for a in args { Self::scan_expr_for_tuples(types, a, vars); } }
            Expr::Binary(a, _, b, _) => { Self::scan_expr_for_tuples(types, a, vars); Self::scan_expr_for_tuples(types, b, vars); }
            Expr::Unary(_, e, _) | Expr::Paren(e, _) | Expr::Ref(e, _) | Expr::MutRef(e, _) | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _) | Expr::As(e, _, _) | Expr::Try(e, _) => Self::scan_expr_for_tuples(types, e, vars),
            Expr::Field(obj, _, _) => Self::scan_expr_for_tuples(types, obj, vars),
            Expr::Index(arr, idx, _) => { Self::scan_expr_for_tuples(types, arr, vars); Self::scan_expr_for_tuples(types, idx, vars); }
            Expr::If(cond, then_b, elifs, else_b, _) => {
                Self::scan_expr_for_tuples(types, cond, vars); Self::scan_block_for_tuples(types, then_b, vars);
                for (c, b) in elifs { Self::scan_expr_for_tuples(types, c, vars); Self::scan_block_for_tuples(types, b, vars); }
                if let Some(b) = else_b { Self::scan_block_for_tuples(types, b, vars); }
            }
            Expr::Match(scrut, arms, _) => {
                Self::scan_expr_for_tuples(types, scrut, vars);
                for arm in arms { match &arm.body { MatchBody::Block(b) => Self::scan_block_for_tuples(types, b, vars), MatchBody::Expr(e) => Self::scan_expr_for_tuples(types, e, vars) } }
            }
            Expr::Array(elems, _) => { for e in elems { Self::scan_expr_for_tuples(types, e, vars); } }
            Expr::Struct(_, elems, _, _) => { for (_, e) in elems { Self::scan_expr_for_tuples(types, e, vars); } }
            _ => {}
        }
    }

    /// Infer a variable's type from its binding for tuple pre-registration.
    /// Matches codegen's practical inference: Str/Int/Float/Bool/Char literals,
    /// ident inheritance, otherwise Int.
    fn infer_var_type(expr: &Expr, vars: &HashMap<String, String>) -> String {
        Self::infer_simple_expr_type(expr, vars).unwrap_or_else(|| "Int".to_string())
    }

    fn infer_simple_expr_type(expr: &Expr, vars: &HashMap<String, String>) -> Option<String> {
        match expr {
            Expr::Int(..) | Expr::Bool(..) => Some("Int".to_string()),
            Expr::Float(..) => Some("Float64".to_string()),
            Expr::Str(..) => Some("Str".to_string()),
            Expr::Char(..) => Some("Char".to_string()),
            Expr::Ident(id) => vars.get(&id.name).cloned().or_else(|| {
                if id.name.len() == 1 && id.name.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                    Some(id.name.clone())
                } else {
                    None
                }
            }),
            Expr::Some(..) | Expr::None(_) => Some("Option".to_string()),
            Expr::Ok(..) | Expr::Err(..) => Some("Result".to_string()),
            _ => None,
        }
    }

    fn type_from_ast_static(ty: &Type) -> String {
        match ty {
            Type::Named(id, _) => id.name.clone(),
            Type::Ref(inner) => Self::type_from_ast_static(inner),
            Type::MutRef(inner) => format!("*{}", Self::type_from_ast_static(inner)),
            Type::Ptr(inner) => format!("*{}", Self::type_from_ast_static(inner)),
            Type::Option(_) => "Option".to_string(),
            Type::Result(_, _) => "Result".to_string(),
            Type::Vec(_) => "Vec".to_string(),
            Type::Tuple(types) => {
                let parts: Vec<String> = types.iter().map(Self::type_from_ast_static).collect();
                format!("Tuple__{}", parts.join("__"))
            }
            _ => "Int".to_string(),
        }
    }

    /// round-8 (rw1): the declared name of `ty` KEEPING a top-level reference
    /// ("&Str", "&mut Int") -- type_from_ast erases the ref, but reference-typed
    /// values hold the T SLOT ADDRESS at the ABI and value uses must auto-deref;
    /// the & is the only discriminator from a plain T.
    pub(crate) fn ref_preserving_name(ty: &Type) -> Option<String> {
        match ty {
            Type::Ref(inner) => Some(format!("&{}", Self::type_from_ast(inner))),
            Type::MutRef(inner) => Some(format!("&mut {}", Self::type_from_ast(inner))),
            _ => None,
        }
    }

    /// R49 (playground C17 residue): bracket-preserving rendering of a type
    /// ANNOTATION for local type tracking. `type_from_ast` truncates a named
    /// generic to its base ("PriorityQueue[Task]" -> "PriorityQueue"), which
    /// starved the generic-method receiver inference: `pq.pop()` on an
    /// annotated `var pq: PriorityQueue[Task]` fell through to the receiver
    /// default and mono'd `PriorityQueue.pop_Int` (C001 "Int does not
    /// implement Priority", L6-28). Container forms keep their existing
    /// bracket spelling (the same shape ctor-bound locals already store);
    /// everything else falls through to type_from_ast unchanged.
    pub(crate) fn type_annotation_name(ty: &Type) -> String {
        match ty {
            Type::Named(id, args) if !args.is_empty() => format!(
                "{}[{}]",
                id.name,
                args.iter().map(Self::type_annotation_name).collect::<Vec<_>>().join(", ")
            ),
            // R49: container annotations keep their args too -- the generic
            // receiver inference parses the LOCAL's recorded type
            // (`m.get(&k)` on `let m: Map[Int, Vec[Str]]` must mono
            // Map.get[Int, Vec[Str]], not get_Int_Int; L5-40).
            Type::Vec(inner) => format!("Vec[{}]", Self::type_annotation_name(inner)),
            Type::Map(k, v) => format!(
                "Map[{}, {}]",
                Self::type_annotation_name(k),
                Self::type_annotation_name(v)
            ),
            Type::Set(inner) => format!("Set[{}]", Self::type_annotation_name(inner)),
            Type::Option(inner) => format!("Option[{}]", Self::type_annotation_name(inner)),
            Type::Result(ok, err) => format!(
                "Result[{}, {}]",
                Self::type_annotation_name(ok),
                Self::type_annotation_name(err)
            ),
            Type::Slice(inner) => format!("Slice[{}]", Self::type_annotation_name(inner)),
            Type::Array(size_expr, elem) => {
                let elem_name = Self::type_annotation_name(elem);
                match size_expr.as_ref() {
                    Expr::Int(n, _) => format!("[{n} x {elem_name}]"),
                    Expr::Ident(id) => format!("[{} x {elem_name}]", id.name),
                    _ => elem_name,
                }
            }
            other => Self::type_from_ast(other),
        }
    }

    /// Infer a type name from an expression for pre-registration purposes.
    /// Falls back to "Int" for unknown types.
    fn infer_expr_type_name(expr: &Expr, vars: &HashMap<String, String>) -> String {
        match expr {
            Expr::Int(..) | Expr::Bool(..) => "Int".to_string(),
            Expr::Str(..) => "Str".to_string(),
            Expr::Char(..) => "Char".to_string(),
            Expr::Ident(id) => {
                // Generic params (single uppercase) stay as-is
                if id.name.len() == 1 && id.name.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                    id.name.clone()
                } else {
                    vars.get(&id.name).cloned().unwrap_or_else(|| "Int".to_string())
                }
            }
            Expr::Some(..) | Expr::None(_) => "Option".to_string(),
            Expr::Ok(..) | Expr::Err(..) => "Result".to_string(),
            _ => "Int".to_string(),
        }
    }

    pub(crate) fn stmt_or_expr_contains_unsafe(item: &xiom_ast::StmtOrExpr) -> bool {
        match item {
            xiom_ast::StmtOrExpr::Expr(e) => Self::expr_contains_unsafe(e),
            xiom_ast::StmtOrExpr::Stmt(s) => Self::stmt_contains_unsafe(s),
        }
    }

    pub(crate) fn expr_contains_unsafe(expr: &Expr) -> bool {
        matches!(expr, Expr::Unsafe(..))
    }

    pub(crate) fn stmt_contains_unsafe(stmt: &Stmt) -> bool {
        use xiom_ast::Stmt as S;
        match stmt {
            S::Let(_, _, e, _) | S::Var(_, _, e, _) | S::Return(Some(e), _)
            | S::Expr(e, _) | S::Assign(_, e, _) => Self::expr_contains_unsafe(e),
            S::If(c, t, _, _, _) => Self::expr_contains_unsafe(c) || Self::block_contains_unsafe(t),
            S::While(c, b, _, _, _) => Self::expr_contains_unsafe(c) || Self::block_contains_unsafe(b),
            S::Match(e, arms, _) => Self::expr_contains_unsafe(e) || arms.iter().any(|a| match &a.body {
                xiom_ast::MatchBody::Block(b) => Self::block_contains_unsafe(b),
                xiom_ast::MatchBody::Expr(e) => Self::expr_contains_unsafe(e),
            }),
            _ => false,
        }
    }
}
