use super::{IrEmitter, TypeMeta};
use xiom_ast::*;
use std::collections::HashMap;
use std::collections::HashSet;

impl IrEmitter {
    pub(crate) fn register_type_layout(&mut self, item: &TopDecl) {
        self.register_type_layout_impl(item, "");
    }

    pub(crate) fn register_type_layout_impl(&mut self, item: &TopDecl, prefix: &str) {
        if let TopDecl::Type(td) = item {
            if td.fields.is_empty() && td.alias.is_some() { return; }
            let fields: Vec<String> = td.fields.iter()
                .map(|f| f.name.name.clone())
                .collect();
            let bare_name = td.name.name.clone();
            let type_name = if prefix.is_empty() { bare_name.clone() } else { format!("{}.{}", prefix, bare_name) };
            // Record generic type names so their methods are skipped from direct
            // (un-monomorphised) emission ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â such bodies produce malformed IR.
            if !td.generics.is_empty() {
                self.generic_type_names.insert(bare_name.clone());
                self.generic_type_names.insert(type_name.clone());
            }
            let full_fields: Vec<(String, String)> = td.fields.iter()
                .map(|f| (f.name.name.clone(), Self::type_from_ast_with_args(&f.ty)))
                .collect();
            self.types.entry(type_name.clone()).or_insert(fields);
            // Use or_insert_with so manual pre-registrations (e.g. Map with
            // resolved Vec type names) are not overwritten by the generic
            // type definition (which uses Vec[K] with unresolved generics).
            self.type_meta.entry(type_name).or_insert_with(|| TypeMeta {
                fields: full_fields,
                derives: td.derives.clone(),
                invariants: td.invariants.clone(),
            });
        }
        if let TopDecl::Enum(ed) = item {
            let bare_name = ed.name.name.clone();
            let enum_name = if prefix.is_empty() { bare_name.clone() } else { format!("{}.{}", prefix, bare_name) };
            if self.types.contains_key(&enum_name) && !self.types.get(&enum_name).map(|f| f.is_empty()).unwrap_or(true) {
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
                    }
                    vfields.push(fname);
                    // Keep generic args (Vec[JsonValue]) â€” 5c.30 handle detection.
                    vftypes.push(Self::type_from_ast_with_args(&field.ty));
                }
                variants_info.push((vname.clone(), vfields));
                variants_types.push((vname, vftypes));
            }
            self.types.insert(enum_name.clone(), all_fields);
            self.type_meta.insert(enum_name.clone(), TypeMeta {
                fields: all_meta,
                derives: ed.derives.clone(),
                invariants: Vec::new(),
            });
            self.enum_variants.insert(enum_name.clone(), variants_info);
            self.enum_variant_field_types.insert(enum_name, variants_types);
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
                if self.type_meta.contains_key(&name) { return; }
                let field_names: Vec<String> = (0..elems.len()).map(|i| format!("_{i}")).collect();
                let field_types: Vec<(String, String)> = elems.iter()
                    .enumerate()
                    .map(|(i, t)| (format!("_{i}"), Self::type_from_ast(t)))
                    .collect();
                self.types.insert(name.clone(), field_names);
                self.type_meta.insert(name, TypeMeta {
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
                if self.type_meta.contains_key(&name) { return; }
                let field_names: Vec<String> = (0..elems.len()).map(|i| format!("_{i}")).collect();
                let field_types: Vec<(String, String)> = elems.iter()
                    .enumerate()
                    .map(|(i, e)| (format!("_{i}"), Self::substitute_concrete_name(e, type_map)))
                    .collect();
                self.types.insert(name.clone(), field_names);
                self.type_meta.insert(name, TypeMeta {
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
                // existing behavior ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â and the green test gate ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â is preserved.
                let ty_name = Self::type_from_ast(&cd.ty);
                if let Ok(llvm_ty) = self.llvm_type_for(&ty_name) {
                    let symbol = if let Some(ref m) = self.current_module {
                        format!("{}.{}", m, cd.name.name)
                    } else {
                        cd.name.name.clone()
                    };
                    // Register lookups under BOTH the bare and qualified names so a
                    // reference resolves whether the module is compiled directly or
                    // its decls are injected flattened at top level.
                    self.module_globals.insert(cd.name.name.clone(), (symbol.clone(), llvm_ty.clone()));
                    self.module_globals.insert(symbol.clone(), (symbol.clone(), llvm_ty.clone()));
                    // Dedup the emitted definition by symbol name.
                    if !self.module_global_defs.iter().any(|(s, _, _)| s == &symbol) {
                        let init = Self::global_const_init(&cd.value, &llvm_ty);
                        self.module_global_defs.push((symbol, llvm_ty, init));
                    }
                } else {
                    // Type doesn't lower to a simple global: keep old behavior.
                    self.constants.insert(cd.name.name.clone(), cd.value.clone());
                }
            } else {
                // Record module/global constants so a bare reference can be substituted
                // with its literal value (constants are not emitted as globals). Last
                // definition wins; both bare and module-qualified names are keyed.
                self.constants.insert(cd.name.name.clone(), cd.value.clone());
            }
        }
        if let TopDecl::Fn(fd) = item {
            // Register tuple types used in function signature before resolving LLVM types
            fd.return_type.as_ref().map(|t| self.ensure_tuple_type_registered(t));
            for p in &fd.params {
                self.ensure_tuple_type_registered(&p.ty);
            }
            let mut param_types: Vec<String> = Vec::new();
            // Detect self param: either named "self" OR first param whose type
            // matches the receiver type (ecosystem pattern: `fn T.method(h: &mut T, ...)`).
            let is_first_param_self = fd.receiver.is_some() && fd.params.first().map_or(false, |p| {
                let pt = Self::type_from_ast(&p.ty);
                let ptn = pt.trim_start_matches('*');
                fd.receiver.as_ref().map_or(false, |r| ptn == r.name)
            });
            let has_self_param = fd.params.iter().any(|p| p.name.name == "self")
                || is_first_param_self;
            let has_recv = fd.receiver.is_some() && has_self_param;
            // For `this`-based methods (receiver exists but no explicit `self`
            // param, AND body uses `this`), register the receiver as a pointer
            // type so call-site receiver handling can detect the need for a
            // pointer and coerce instance method calls (v.method()) correctly.
            let is_this_based = fd.receiver.is_some() && !has_self_param
                && fd.body.as_ref().map_or(false, |b| Self::block_uses_this(b));
            // If first param IS the self (type matches receiver), don't add
            // receiver type ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the first param already covers it.
            if has_recv && !is_first_param_self {
                if let Some(recv) = fd.receiver.as_ref() {
                    param_types.push(self.llvm_type_for(&recv.name).unwrap_or_else(|_| "i64".to_string()));
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
            let self_param_name: Option<String> = if is_first_param_self {
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
                .map(|t| self.llvm_type_for(&Self::type_from_ast(t)).unwrap_or_else(|_| "i64".to_string()))
                .unwrap_or_else(|| "void".to_string());
            let key = self.fn_key(fd);
            self.functions.insert(key.clone(), (param_types.clone(), ret_type.clone()));
            // 5c.30: keep the declared XIOM return type WITH generic args so
            // Option/Result payload types survive LLVM erasure.
            if let Some(rt) = fd.return_type.as_ref() {
                self.fn_return_xiom.insert(key.clone(), Self::type_string_full(rt));
            }
            // Detect interface-typed params: store these functions so call sites
            // can monomorphise them for each concrete implementor (BUG-007).
            let has_iface_param = fd.params.iter().any(|p| {
                let name = Self::type_from_ast(&p.ty);
                self.interfaces.contains_key(&name)
            }) || fd.return_type.as_ref().map_or(false, |t| {
                let name = Self::type_from_ast(t);
                self.interfaces.contains_key(&name)
            });
            if has_iface_param {
                // Add to generic_fn_decls as a pseudo-generic so the
                // monomorphisation loop picks it up.
                if !self.generic_fn_decls.iter().any(|(k, _)| k == &key) {
                    self.generic_fn_decls.push((key.clone(), fd.clone()));
                }
            }
            // Register leaf-module key (e.g. "mem.replace", "ptr.replace") so
            // call sites like `mem.replace(...)` / `ptr.replace(...)` resolve
            // to module-disambiguated names.  This prevents monomorphisation
            // naming collisions between same-named generic functions from
            // different modules (BUG-005).
            if fd.receiver.is_none() {
                if let Some(ref module) = self.current_module {
                    if let Some(leaf) = module.rsplit('.').next() {
                        let leaf_key = format!("{}.{}", leaf, key);
                        if leaf_key != key {
                            self.functions.insert(leaf_key.clone(), (param_types.clone(), ret_type.clone()));
                        }
                    }
                }
            }
            if !fd.generics.is_empty() {
                self.generic_fn_decls.push((key.clone(), fd.clone()));
                // Also register with leaf-module key for generic resolution
                if fd.receiver.is_none() {
                    if let Some(ref module) = self.current_module {
                        if let Some(leaf) = module.rsplit('.').next() {
                            let leaf_key = format!("{}.{}", leaf, key);
                            if leaf_key != key {
                                self.generic_fn_decls.push((leaf_key, fd.clone()));
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
                }
            }
            self.interfaces.insert(id.name.name.clone(), methods);
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
                self.functions.insert(fd.name.name.clone(), (param_types, ret_type));
            }
        }
        if let TopDecl::Module(md) = item {
            let saved_module = self.current_module.clone();
            self.current_module = Some(if let Some(ref prev) = saved_module {
                format!("{}.{}", prev, md.name.name)
            } else {
                md.name.name.clone()
            });
            for sub in &md.items {
                self.register_functions(sub);
            }
            self.current_module = saved_module;
        }
    }

    /// Scan all registered interfaces and concrete types to determine which
    /// types implement which interfaces (BUG-007). A type implements an
    /// interface if, for every method in the interface, there is a function
    /// registered as `TypeName.methodName` in self.functions.
    pub(crate) fn scan_interface_impls(&mut self) {
        for (iface_name, methods) in self.interfaces.clone().iter() {
            for type_name in self.types.keys().cloned().collect::<Vec<_>>().iter() {
                // Skip builtin types (Option, Result, Vec, etc.)
                if ["Option", "Result", "Vec", "Slice", "Map", "Set"].contains(&type_name.as_str()) { continue; }
                let mut all_implemented = true;
                for (method_name, _) in methods {
                    let fn_key = format!("{}.{}", type_name, method_name);
                    let leaf_parts: Vec<&str> = type_name.rsplitn(2, '.').collect();
                    let leaf_key = if leaf_parts.len() > 1 {
                        format!("{}.{}", leaf_parts[1], method_name)
                    } else { fn_key.clone() };
                    if !self.functions.contains_key(&fn_key) && !self.functions.contains_key(&leaf_key) {
                        if !self.generic_fn_decls.iter().any(|(k, _)| k == &fn_key || k == &leaf_key) {
                            all_implemented = false;
                            break;
                        }
                    }
                }
                if all_implemented {
                    self.interface_impls.entry(iface_name.clone())
                        .or_insert_with(HashSet::new)
                        .insert(type_name.clone());
                }
            }
        }
    }

    pub(crate) fn fn_key(&self, fd: &FnDecl) -> String {
        if let Some(recv_name) = &fd.receiver {
            // Use module context to resolve receiver type to qualified name
            let recv_type = if let Some(ref module) = self.current_module {
                let qualified = format!("{}.{}", module, recv_name.name);
                if self.type_meta.contains_key(&qualified) { qualified } else { recv_name.name.clone() }
            } else {
                recv_name.name.clone()
            };
            format!("{}.{}", recv_type, fd.name.name)
        } else {
            fd.name.name.clone()
        }
    }

    /// Return the LLVM symbol name for a function, avoiding collisions.
    /// If a bare name already exists in emitted_fns, use module-qualified.
    pub(crate) fn fn_symbol(&self, fd: &FnDecl) -> String {
        let bare = self.fn_key(fd);
        // Keep `main` as bare entry point regardless of module
        if bare == "main" { return bare; }
        // If bare name already emitted (collision from multi-file merge), qualify it
        if self.emitted_fns.contains(&bare) {
            if let Some(ref module) = self.current_module {
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
            if self.functions.contains_key(&leaf_key)
                || self.generic_fn_decls.iter().any(|(k, _)| k == &leaf_key)
            {
                return leaf_key;
            }
            // Try parent-qualified: "benchmark.math.run_all" (current_module parent + module_name)
            if let Some(ref cur_mod) = self.current_module {
                if let Some(parent) = cur_mod.rsplitn(2, '.').last() {
                    let parent_key = format!("{}.{}.{}", parent, module_name, fn_name);
                    if self.functions.contains_key(&parent_key)
                        || self.generic_fn_decls.iter().any(|(k, _)| k == &parent_key)
                    {
                        return parent_key;
                    }
                }
            }
            // Try any key ending with ".module_name.fn_name" as a fallback
            let suffix = format!(".{}.{}", module_name, fn_name);
            for k in self.functions.keys() {
                if k.ends_with(&suffix) {
                    return k.clone();
                }
            }
            // Also search generic function decls for the suffix
            for (k, _) in &self.generic_fn_decls {
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
                // Skip generic functions ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â they will be monomorphised later
                if fd.generics.is_empty() {
                    // Skip methods on generic types (e.g. `BinaryHeap[T].push`). The
                    // generic parameter lives on the RECEIVER type, not in fd.generics,
                    // so it evades the check above; emitting such a body concretely
                    // erases `self` to i64 and produces malformed struct-access IR.
                    // These are monomorphised on demand at call sites instead.
                    let recv_is_generic = fd.receiver.as_ref()
                        .map(|r| self.generic_type_names.contains(&r.name))
                        .unwrap_or(false);
                    if !recv_is_generic && fd.body.is_some() {
                        let fn_name = self.fn_symbol(fd);
                        self.emitted_fns.insert(fn_name);
                        self.compile_fn(fd)?;
                    }
                }
                Ok(())
            }
            TopDecl::Module(md) => {
                let saved_module = self.current_module.clone();
                self.current_module = Some(if let Some(ref prev) = saved_module {
                    format!("{}.{}", prev, md.name.name)
                } else {
                    md.name.name.clone()
                });
                for sub in &md.items {
                    self.compile_top_decl(sub)?;
                }
                self.current_module = saved_module;
                Ok(())
            }
            TopDecl::Interface(_) | TopDecl::Enum(_) | TopDecl::Const(_) | TopDecl::Type(_) | TopDecl::Use(_) | TopDecl::Extern(_) => Ok(()),
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
                && self.already_declared.contains(&bare)
            {
                return Ok(());
            }
        }
        // --strict mode: enforce #[safety_audit] on functions with unsafe blocks
        if self.strict_mode && fd.body.as_ref().map_or(false, |b| {
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
        self.fn_ptr_return_types.clear();
        self.bool_locals.clear();
        self.ptr_locals.clear();
        self.local_vec_elem.clear();
        self.local_opt_payload.clear();
        self.local_boxed_struct.clear();
        self.local_vec_handle.clear();

        let ret_llvm = fd.return_type.as_ref()
            .map(|t| self.llvm_type_for(&Self::type_from_ast(t)).unwrap_or_else(|_| "i64".to_string()))
            .unwrap_or_else(|| "void".to_string());
        self.current_return_type = ret_llvm.clone();
        self.current_param_llvm_types = fd.params.iter()
            .map(|p| self.llvm_type_for(&Self::type_from_ast(&p.ty)).unwrap_or_else(|_| "i64".to_string()))
            .collect();

        // Store ensures clauses for return point checking
        self.current_ensures = Vec::new();
        if self.check_contracts {
            for clause in &fd.contracts {
                if let ContractClause::Ensures(e, _) = clause {
                    self.current_ensures.push(e.clone());
                }
            }
        }

        let name = self.fn_key(fd);
        self.current_fn = Some(name.clone());
        // 5c.30: track receiver type for implicit-self method calls (G-10)
        self.current_receiver = fd.receiver.as_ref().map(|r| r.name.clone());

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
        let is_first_param_self = fd.receiver.is_some() && !has_self_param
            && fd.params.first().map_or(false, |p| {
                let pt = Self::type_from_ast(&p.ty);
                // type_from_ast returns "*T" for &mut T â€” strip the pointer
                // prefix to compare with the bare receiver name.
                let ptn = pt.trim_start_matches('*');
                fd.receiver.as_ref().map_or(false, |r| ptn == r.name)
            });
        // Match registration: implicit self only for explicit-`self` methods
        // and `this`-based methods (body actually references `this`).
        // 5c.30: G-10 implicit-self is a CHECKER-only feature â€” the codegen
        // does NOT inject self arguments. Method bodies already have all
        // receiver fields as locals via the prologue, so read-only access to
        // fields works without param_self. Only `this`-based methods that
        // explicitly reference `this` get param_self (mutation support).
        let is_this_based = fd.receiver.is_some() && !has_self_param && !is_first_param_self
            && fd.body.as_ref().map_or(false, |b| Self::block_uses_this(b));
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

        self.emitln(&format!("define {ret_llvm} @{name}({}) {{", params_str.join(", ")));

        // Recursion depth check
        let entry_block = self.fresh_block("entry");
        self.emitln(&format!("{entry_block}:"));
        let depth_tmp = self.fresh_tmp();
        self.emitln(&format!("  {depth_tmp} = load i64, i64* @xiom_recursion_counter"));
        let new_depth = self.fresh_tmp();
        self.emitln(&format!("  {new_depth} = add i64 {depth_tmp}, 1"));
        let depth_ok = self.fresh_tmp();
        self.emitln(&format!("  {depth_ok} = icmp slt i64 {new_depth}, {}", self.max_recursion_depth));
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
                // Try bare name first, then module-qualified if not found.
                let fields = self.types.get(&recv.name)
                    .or_else(|| {
                        // Try module-qualified name (e.g. "tests.ecosystem.test_net.IpAddr")
                        let suffix = format!(".{}", recv.name);
                        self.types.keys().find(|k| k.ends_with(&suffix))
                            .and_then(|k| self.types.get(k))
                    })
                    .cloned();
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
                let fields = self.types.get(&recv.name)
                    .or_else(|| {
                        let suffix = format!(".{}", recv.name);
                        self.types.keys().find(|k| k.ends_with(&suffix)).and_then(|k| self.types.get(k))
                    })
                    .cloned();
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
            // Record raw-pointer params (`*T`/`&T`/`&mut T` over a pointer) so that
            // `param[i]` indexing treats the i64 value as an address (byte buffer).
            if matches!(&param.ty, Type::Ptr(_)) {
                self.ptr_locals.insert(param.name.name.clone());
            } else {
                self.ptr_locals.remove(&param.name.name);
            }
            // Track function pointer return types for function pointer parameters
            if let Type::Fn(_, ret) = &param.ty {
                let ret_ty_name = Self::type_from_ast(ret);
                let ret_llvm = self.llvm_type_for_fallback(&ret_ty_name);
                self.fn_ptr_return_types.insert(param.name.name.clone(), ret_llvm);
            }
        }

        // Capture self@pre for ensures (method functions with self@pre references)
        if self.check_contracts && !self.current_ensures.is_empty() {
            // Phase 5c @pre snapshot: for every variable referenced in an
            // `ensures` clause with `@pre`, store its entry-point value so the
            // ensures check uses the pre-state value, not the current one.
            let mut pre_vars: HashSet<String> = HashSet::new();
            for expr in &self.current_ensures {
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
        self.result_ptr = None;
        if !self.current_ensures.is_empty() && fd.return_type.is_some() {
            let result_alloca = self.fresh_tmp();
            self.emitln(&format!("  {result_alloca} = alloca {ret_llvm}"));
            self.add_local("result", result_alloca.clone(), &ret_llvm);
            self.result_ptr = Some(result_alloca);
        }

        // Emit requires checks at function entry
        if self.check_contracts {
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
        if fd.return_type.is_none() {
            // Check ensures before implicit void return
            if !self.current_ensures.is_empty() {
                self.compile_ensures_checks();
            }
            // Decrement recursion depth
            let depth_dec = self.fresh_tmp();
            self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
            let new_depth_dec = self.fresh_tmp();
            self.emitln(&format!("  {new_depth_dec} = sub i64 {depth_dec}, 1"));
            self.emitln(&format!("  store i64 {new_depth_dec}, i64* @xiom_recursion_counter"));
            self.emitln("  ret void");
        } else if !self.current_block_terminated() {
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
        }

        self.emitln("}\n");
        self.pop_scope();
        self.current_fn = None;
        self.current_receiver = None;
        self.current_ensures.clear();
        self.result_ptr = None;
        Ok(())
    }

    /// Return the LLVM struct type for an `Option<Inner>` with the given
    /// inner LLVM type.  For scalar payloads uses `%struct.Option`; for
    /// struct payloads creates a concrete type like `%struct.Option__Point`
    /// that stores the struct inline (BUG-006 fix).
    pub(crate) fn get_concrete_option_type(&mut self, inner_ty: &str) -> String {
        if !inner_ty.starts_with('%') {
            return "%struct.Option".to_string();
        }
        let inner_name = inner_ty.trim_start_matches("%struct.");
        let concrete_name = format!("Option__{inner_name}");
        if !self.type_meta.contains_key(&concrete_name) {
            self.types.insert(concrete_name.clone(), vec!["discriminant".to_string(), "value".to_string()]);
            self.type_meta.insert(concrete_name.clone(), TypeMeta {
                fields: vec![("discriminant".to_string(), "i64".to_string()), ("value".to_string(), inner_ty.to_string())],
                derives: vec![],
                invariants: vec![],
            });
            // Defer LLVM type emission until flush_deferred_types()
            let field_llvm_ty = if inner_ty.starts_with('%') { inner_ty.to_string() }
                else { self.llvm_type_for(inner_ty).unwrap_or_else(|_| "i64".to_string()) };
            self.deferred_struct_types.push((
                concrete_name.clone(),
                format!("{{ i64, {field_llvm_ty} }}"),
            ));
        }
        format!("%struct.{concrete_name}")
    }

    /// Return the LLVM struct type for a `Result<Ok, Err>` with the given
    /// concrete inner types.  Same logic as `get_concrete_option_type`
    /// but for 3-field Result structs.
    pub(crate) fn get_concrete_result_type(&mut self, ok_ty: &str, err_ty: &str) -> String {
        let ok_struct = ok_ty.starts_with('%');
        let err_struct = err_ty.starts_with('%');
        if !ok_struct && !err_struct {
            return "%struct.Result".to_string();
        }
        let ok_name = ok_ty.trim_start_matches("%struct.");
        let err_name = err_ty.trim_start_matches("%struct.");
        let concrete_name = format!("Result__{ok_name}__{err_name}");
        if !self.type_meta.contains_key(&concrete_name) {
            let mut fields = vec![
                ("discriminant".to_string(), "i64".to_string()),
                ("value".to_string(), ok_ty.to_string()),
                ("error".to_string(), err_ty.to_string()),
            ];
            let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
            self.types.insert(concrete_name.clone(), field_names);
            self.type_meta.insert(concrete_name.clone(), TypeMeta {
                fields,
                derives: vec![],
                invariants: vec![],
            });
        }
        format!("%struct.{concrete_name}")
    }

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
            Expr::Call(f, args, _) => { Self::collect_atpre_vars(f, vars); for a in args { Self::collect_atpre_vars(a, vars); } }
            Expr::Field(e, _, _) | Expr::Index(e, _, _) => Self::collect_atpre_vars(e, vars),
            Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _) => Self::collect_atpre_vars(e, vars),
            _ => {}
        }
    }

    /// Emit any deferred struct type definitions (concrete Option__Point,
    /// Result__X__Y, etc.) before the next function body.
    pub(crate) fn flush_deferred_types(&mut self) {
        if self.deferred_struct_types.is_empty() { return; }
        for (name, body) in std::mem::take(&mut self.deferred_struct_types) {
            self.emitln(&format!("%struct.{name} = type {body}"));
        }
        if !self.deferred_struct_types.is_empty() {
            self.emitln("");
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
            S::While(c, b, _) => Self::expr_contains_unsafe(c) || Self::block_contains_unsafe(b),
            S::Match(e, arms, _) => Self::expr_contains_unsafe(e) || arms.iter().any(|a| match &a.body {
                xiom_ast::MatchBody::Block(b) => Self::block_contains_unsafe(b),
                xiom_ast::MatchBody::Expr(e) => Self::expr_contains_unsafe(e),
            }),
            _ => false,
        }
    }
}
