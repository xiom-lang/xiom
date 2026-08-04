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
                // Empty struct (no fields, no alias) — still register as a type
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
            self.types.types.or_insert_with(type_name.clone(), || fields);
            // Use or_insert_with so manual pre-registrations (e.g. Map with
            // resolved Vec type names) are not overwritten by the generic
            // type definition (which uses Vec[K] with unresolved generics).
            self.types.type_meta.or_insert_with(type_name, || TypeMeta {
                fields: full_fields,
                derives: td.derives.clone(),
                invariants: td.invariants.clone(),
            });
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
                    // Keep generic args (Vec[JsonValue]) — 5c.30 handle detection.
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
            self.types.enum_variant_field_types.insert(enum_name, variants_types);
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
                    inner.clone()
                }
            }
            // Recursively substitute type params in composite types.
            Type::Array(size, elem) => {
                Type::Array(size.clone(), Box::new(Self::substitute_type(inner, elem, type_map)))
            }
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
            Type::Ptr(i2) | Type::MutRef(i2) | Type::Ref(i2) => Self::substitute_type(inner, i2, type_map),
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

    pub(crate) fn register_functions(&mut self, item: &TopDecl) {
        if let TopDecl::Const(cd) = item {
            if cd.is_mut {
                // Mutable module-level `var`: emit as a REAL LLVM global and route
                // reads/writes to load/store (see compile_expr / Stmt::Assign).
                // Only do this when the declared type resolves to a concrete LLVM
                // type; otherwise (e.g. `Map[K,V]`, whose LLVM lowering isn't a
                // simple global slot) fall back to constant substitution so the
                // existing behavior -- and the green test gate -- is preserved.
                let ty_name = Self::type_from_ast(&cd.ty);
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
                    // Dedup the emitted definition by symbol name.
                    if !self.local.module_global_defs.iter().any(|(s, _, _)| s == &symbol) {
                        let init = Self::global_const_init(&cd.value, &llvm_ty);
                        self.local.module_global_defs.push((symbol.clone(), llvm_ty.clone(), init));
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
            // — the old type-only heuristic hijacked it and shifted every arg
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
            // param, AND body references receiver STATE — `this` or bare
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
                    let is_mut = fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self);
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
                .map(|p| self.llvm_type_for(&Self::type_from_ast(&p.ty)).unwrap_or_else(|_| "i64".to_string()))
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
                        if leaf_key != key {
                            self.types.functions.insert(leaf_key.clone(), (param_types.clone(), ret_type.clone()));
                        }
                    }
                }
            }
            if !fd.generics.is_empty() {
                self.mono.generic_fn_decls.push((key.clone(), fd.clone()));
                // Also register with leaf-module key for generic resolution
                if fd.receiver.is_none() {
                    if let Some(ref module) = self.local.current_module {
                        if let Some(leaf) = module.rsplit('.').next() {
                            let leaf_key = format!("{}.{}", leaf, key);
                            if leaf_key != key {
                                self.mono.generic_fn_decls.push((leaf_key, fd.clone()));
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
                    }
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
        if let Expr::Ident(id) = receiver {
            let module_name = &id.name;
            // Try leaf-qualified: "math.run_all"
            let leaf_key = format!("{}.{}", module_name, fn_name);
            if self.types.functions.contains_key(&leaf_key)
                || self.mono.generic_fn_decls.iter().any(|(k, _)| k == &leaf_key)
            {
                return leaf_key;
            }
            // Try parent-qualified: "benchmark.math.run_all" (current_module parent + module_name)
            if let Some(ref cur_mod) = self.local.current_module {
                if let Some(parent) = cur_mod.rsplitn(2, '.').last() {
                    let parent_key = format!("{}.{}.{}", parent, module_name, fn_name);
                    if self.types.functions.contains_key(&parent_key)
                        || self.mono.generic_fn_decls.iter().any(|(k, _)| k == &parent_key)
                    {
                        return parent_key;
                    }
                }
            }
            // Try any key ending with ".module_name.fn_name" as a fallback
            let suffix = format!(".{}.{}", module_name, fn_name);
            for k in self.types.functions.keys() {
                if k.ends_with(&suffix) {
                    return k.clone();
                }
            }
            // Also search generic function decls for the suffix
            for (k, _) in &self.mono.generic_fn_decls {
                if k.ends_with(&suffix) {
                    return k.clone();
                }
            }
        }
        fn_name.to_string()
    }

    pub(crate) fn compile_top_decl(&mut self, item: &TopDecl) -> Result<(), String> {
        match item {
            TopDecl::Fn(fd) => {
                // Skip generic functions — they will be monomorphised later
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
                        // that would shadow the real entry point in a brace-less module.
                        let is_empty_main = fd.name.name == "main"
                            && fd.body.as_ref().map_or(false, |b| b.stmts.is_empty());
                        if is_empty_main { return Ok(()); }
                        let fn_name = self.fn_symbol(fd);
                        self.mono.emitted_fns.insert(fn_name.clone());
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
                // program init. For now, skip — spawn is a no-op runtime.
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
        self.types.fn_ptr_return_types = SyncRegistry::default();
        self.local.bool_locals.clear();
        self.local.ptr_locals.clear();
        self.local.local_vec_elem.clear();
        self.local.local_opt_payload.clear();
        self.local.local_boxed_struct.clear();
        self.local.local_vec_handle.clear();
        self.local.signed_locals.clear();
        self.local.local_xiom_types.clear();
        self.local.reg_signed.clear();
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
            .map(|p| self.llvm_type_for(&Self::type_from_ast(&p.ty)).unwrap_or_else(|_| "i64".to_string()))
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
        self.fctx.current_fn = Some(name.clone());
        // 5c.30: track receiver type for implicit-self method calls (G-10)
        self.fctx.current_receiver = fd.receiver.as_ref().map(|r| r.name.clone());

        // For methods, prepend the self struct parameter. A receiver-qualified fn
        // with NO `self` param is a static constructor (e.g. `Layout.new(size)`):
        // it keeps its qualified name but takes no receiver argument.
        let has_self_param = fd.params.iter().any(|p| p.name.name == "self");
        // 5c.29: ecosystem pattern `fn T.method(h: &T, ...)` â€” the first
        // explicit param IS the receiver. Signature registration and all call
        // sites never include an implicit self argument for these, so the
        // definition must not emit `%param_self` either (the extra leading
        // param shifted every argument and made the body read uninitialized
        // registers â€” HTTP/SQLITE ACCESS_VIOLATION).
        // G-20 fix: receiver-style requires a by-REFERENCE first param
        // (&T/&mut T/*T). By-value same-type params are real arguments.
        let is_first_param_self = fd.receiver.is_some() && !has_self_param
            && fd.params.first().map_or(false, |p| {
                let is_ref = matches!(&p.ty, Type::Ref(_) | Type::MutRef(_) | Type::Ptr(_));
                if !is_ref { return false; }
                let pt = Self::type_from_ast(&p.ty);
                // type_from_ast returns "*T" for &mut T â€” strip the pointer
                // prefix to compare with the bare receiver name.
                let ptn = pt.trim_start_matches('*');
                fd.receiver.as_ref().map_or(false, |r| ptn == r.name)
            });
        // Match registration: implicit self only for explicit-`self` methods
        // and `this`-based methods (body references receiver state).
        // G-20: `this`-based now includes BARE-FIELD bodies (e.g.
        // `fn Counter.inc() -> Int { return val + 1; }`) — the %param_self
        // slot is emitted and the prologue GEP-binds every field, so bare
        // reads are correct instead of garbage. Must match registration.
        let is_this_based = fd.receiver.is_some() && !has_self_param && !is_first_param_self
            && self.body_uses_receiver_state(fd);
        // 5c.32: by-value self methods — `fn Type.method(params) { self.field = ... }`
        // The parser stores the receiver but does NOT add `self` to fd.params.
        // Detect self usage in the body so the LLVM signature gets the struct param.
        let body_uses_self = fd.receiver.is_some() && !has_self_param && !is_first_param_self
            && !self.body_uses_receiver_state(fd)
            && fd.body.as_ref().map_or(false, |b| Self::block_uses_self_ident(b));
        let self_llvm_ty = if has_self_param {
            fd.receiver.as_ref().map(|r| {
                let base = self.llvm_type_for(&r.name).unwrap_or_else(|_| "i64".to_string());
                let is_mut = fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self);
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
            // 5c.32: by-value self method — body uses `self` variable.
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
                let llvm_ty = self.llvm_type_for(&Self::type_from_ast(&p.ty)).unwrap_or_else(|_| "i64".to_string());
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
            // Emit the DISubprogram metadata inline, right before the define
            self.emitln(&format!("!{} = distinct !DISubprogram(name: \"{name}\", linkageName: \"{name}\", scope: !4, file: !4, line: {line}, type: !{{}}, spFlags: DISPFlagDefinition, unit: !0)", di_node));
            format!(" !dbg !{}", di_node)
        } else {
            String::new()
        };
        self.emitln(&format!("define {ret_llvm} @{name}({}){}{{", params_str.join(", "), dbg_attach));

        // Recursion depth check
        let entry_block = self.fresh_block("entry");
        self.emitln(&format!("{entry_block}:"));
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
            let is_ptr_receiver = st.ends_with('*');
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
        // a counter that only advances for emitted params ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â keeping %paramN indices
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
            let llvm_ty = self.llvm_type_for_fallback(&Self::type_from_ast(&param.ty));
            let alloca = self.fresh_tmp();
            let param_idx = emitted_param_idx;
            emitted_param_idx += 1;
            self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
            self.emitln(&format!("  store {llvm_ty} %param{param_idx}, {llvm_ty}* {alloca}"));
            self.add_local(&param.name.name, alloca, &llvm_ty);
            // 5c.39: Track Vec element type for function parameters so
            // downstream local bindings (var x = param) can inherit it.
            if let Some(elem) = Self::vec_elem_from_type_annotation(&param.ty) {
                self.local.local_vec_elem.insert(param.name.name.clone(), elem);
            }
            // M17: Track parameter signedness for narrow-int widening.
            let xiom_ty_name = Self::type_from_ast(&param.ty);
            self.local.local_xiom_types.insert(param.name.name.clone(), xiom_ty_name.clone());
            if Self::is_signed_xiom_type(&xiom_ty_name) {
                self.local.signed_locals.insert(param.name.name.clone());
            } else {
                self.local.signed_locals.remove(&param.name.name);
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
            for var_name in &pre_vars {
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
                        let pre_name = format!("__{}_pre", var_name);
                        self.add_local(&pre_name, pre_alloca, inner_ty);
                    } else {
                        let pre_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {pre_alloca} = alloca {llvm_ty}"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                        self.emitln(&format!("  store {llvm_ty} {loaded}, {llvm_ty}* {pre_alloca}"));
                        let pre_name = format!("__{}_pre", var_name);
                        self.add_local(&pre_name, pre_alloca, &llvm_ty);
                    }
                }
            }
            // Also snapshot self receiver (backward compat)
            if let Some(recv) = fd.receiver.as_ref() {
                if let Some((ptr, llvm_ty)) = self.lookup_local(&recv.name).cloned() {
                    if !pre_vars.contains(&recv.name) {
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
        if !self.fctx.current_ensures.is_empty() && fd.return_type.is_some() {
            let result_alloca = self.fresh_tmp();
            self.emitln(&format!("  {result_alloca} = alloca {ret_llvm}"));
            self.add_local("result", result_alloca.clone(), &ret_llvm);
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
            // P2-4: Never-returning functions — emit unreachable instead of ret
            if self.fctx.is_never_return {
                self.emitln("  unreachable");
            } else {
            // A4 fix: the function declares a return type but control reached the
            // end of the body without a terminator ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the body ends in a loop, an
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
            let thunk_name = format!("xiom_hot_thunk_{}", name);
            let hash = Self::djb2_hash(&name);

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
        // M20-A1: Emit any deferred closure function definitions
        self.flush_deferred_closures();
        Ok(())
    }

    /// Return the LLVM struct type for an `Option<Inner>` with the given
    /// Collect all variable names referenced through `@pre` in an expression.
    pub(crate) fn collect_atpre_vars(expr: &Expr, vars: &mut HashSet<String>) {
        match expr {
            Expr::AtPre(inner, _) => {
                if let Expr::Ident(id) = inner.as_ref() {
                    vars.insert(id.name.clone());
                } else {
                    Self::collect_atpre_vars(inner, vars);
                }
            }
            Expr::Binary(l, _, r, _) => { Self::collect_atpre_vars(l, vars); Self::collect_atpre_vars(r, vars); }
            Expr::Unary(_, e, _) => Self::collect_atpre_vars(e, vars),
            Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => { Self::collect_atpre_vars(f, vars); for a in args { Self::collect_atpre_vars(a, vars); } }
            Expr::Field(e, _, _) | Expr::Index(e, _, _) => Self::collect_atpre_vars(e, vars),
            Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _) => Self::collect_atpre_vars(e, vars),
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

    /// Scan function bodies for `Expr::Tuple` with 2+ elements and register
    /// the corresponding `Tuple__Type1__Type2` struct types so their LLVM
    /// definitions are emitted at module level before any function bodies.
    pub(crate) fn register_expr_tuple_types(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Fn(fd) => {
                if let Some(ref body) = fd.body {
                    Self::scan_block_for_tuples(&mut self.types, body);
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

    fn scan_block_for_tuples(types: &mut TypeContext, block: &Block) {
        for stmt in &block.stmts {
            match stmt {
                StmtOrExpr::Stmt(s) => Self::scan_stmt_for_tuples(types, s),
                StmtOrExpr::Expr(e) => Self::scan_expr_for_tuples(types, e),
            }
        }
    }

    fn scan_stmt_for_tuples(types: &mut TypeContext, stmt: &Stmt) {
        match stmt {
            Stmt::Let(_, _, init, _) | Stmt::Var(_, _, init, _) => Self::scan_expr_for_tuples(types, init),
            Stmt::Assign(lhs, rhs, _) => { Self::scan_expr_for_tuples(types, lhs); Self::scan_expr_for_tuples(types, rhs); }
            Stmt::If(cond, then_b, elifs, else_b, _) => {
                Self::scan_expr_for_tuples(types, cond);
                Self::scan_block_for_tuples(types, then_b);
                for (c, b) in elifs { Self::scan_expr_for_tuples(types, c); Self::scan_block_for_tuples(types, b); }
                if let Some(b) = else_b { Self::scan_block_for_tuples(types, b); }
            }
            Stmt::While(cond, body, _, _, _) => { Self::scan_expr_for_tuples(types, cond); Self::scan_block_for_tuples(types, body); }
            Stmt::For(_, iter, body, _, _) => { Self::scan_expr_for_tuples(types, iter); Self::scan_block_for_tuples(types, body); }
            Stmt::Match(scrut, arms, _) => {
                Self::scan_expr_for_tuples(types, scrut);
                for arm in arms {
                    match &arm.body { MatchBody::Block(b) => Self::scan_block_for_tuples(types, b), MatchBody::Expr(e) => Self::scan_expr_for_tuples(types, e) }
                }
            }
            Stmt::Return(Some(e), _) | Stmt::Expr(e, _) => Self::scan_expr_for_tuples(types, e),
            _ => {}
        }
    }

    fn scan_expr_for_tuples(types: &mut TypeContext, expr: &Expr) {
        // Register tuple types from expression-level tuples
        if let Expr::Tuple(items, _) = expr {
            if items.len() > 1 {
                let elem_types: Vec<String> = items.iter()
                    .map(|i| Self::infer_expr_type_name(i))
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
            Expr::Tuple(items, _) => { for item in items { Self::scan_expr_for_tuples(types, item); } }
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => { Self::scan_expr_for_tuples(types, func); for a in args { Self::scan_expr_for_tuples(types, a); } }
            Expr::Binary(a, _, b, _) => { Self::scan_expr_for_tuples(types, a); Self::scan_expr_for_tuples(types, b); }
            Expr::Unary(_, e, _) | Expr::Paren(e, _) | Expr::Ref(e, _) | Expr::MutRef(e, _) | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _) | Expr::As(e, _, _) | Expr::Try(e, _) => Self::scan_expr_for_tuples(types, e),
            Expr::Field(obj, _, _) => Self::scan_expr_for_tuples(types, obj),
            Expr::Index(arr, idx, _) => { Self::scan_expr_for_tuples(types, arr); Self::scan_expr_for_tuples(types, idx); }
            Expr::If(cond, then_b, elifs, else_b, _) => {
                Self::scan_expr_for_tuples(types, cond); Self::scan_block_for_tuples(types, then_b);
                for (c, b) in elifs { Self::scan_expr_for_tuples(types, c); Self::scan_block_for_tuples(types, b); }
                if let Some(b) = else_b { Self::scan_block_for_tuples(types, b); }
            }
            Expr::Match(scrut, arms, _) => {
                Self::scan_expr_for_tuples(types, scrut);
                for arm in arms { match &arm.body { MatchBody::Block(b) => Self::scan_block_for_tuples(types, b), MatchBody::Expr(e) => Self::scan_expr_for_tuples(types, e) } }
            }
            Expr::Array(elems, _) => { for e in elems { Self::scan_expr_for_tuples(types, e); } }
            Expr::Struct(_, elems, _, _) => { for (_, e) in elems { Self::scan_expr_for_tuples(types, e); } }
            _ => {}
        }
    }

    /// Infer a type name from an expression for pre-registration purposes.
    /// Falls back to "Int" for unknown types.
    fn infer_expr_type_name(expr: &Expr) -> String {
        match expr {
            Expr::Int(..) | Expr::Bool(..) => "Int".to_string(),
            Expr::Str(..) => "Str".to_string(),
            Expr::Char(..) => "Char".to_string(),
            Expr::Ident(id) => {
                // Generic params (single uppercase) stay as-is
                if id.name.len() == 1 && id.name.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                    id.name.clone()
                } else {
                    "Int".to_string()
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
