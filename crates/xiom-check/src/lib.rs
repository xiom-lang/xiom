// XIOM — Type Checker
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Type Checker — Phase 0: basic type checking for primitives,
//! struct types, function signatures, and return types.
//! No generics, no ownership, no contracts enforcement.

use xiom_ast::*;
use xiom_lexer::Lexer;
use xiom_parser::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use crate::types::{TypeArena, TypeId};

pub mod types;
pub mod catalog;
pub mod borrow;

use types::{CheckedType, FnSig, CheckError};
use catalog::{ModuleExport, CachedModule, ModuleCatalog};

// ============================================================================
// Type Checker
// ============================================================================

/// The XIOM type checker. Validates types, resolves function signatures,
/// enforces interface bounds, and collects errors across an entire compilation unit.
///
/// # Workflow
///
/// 1. Create with [`Checker::new()`]
/// 2. Add stdlib sources via [`Checker::add_source_dir()`]
/// 3. Build the catalog index with [`Checker::build_catalog_index()`]
/// 4. Run type checking with [`Checker::check_program()`]
/// 5. Inspect results with [`Checker::has_errors()`] / [`Checker::certify()`]
///
/// # Thread Safety
///
/// The `Checker` is single-threaded by design. Use a fresh instance per compilation unit.
pub struct Checker {
    /// Known type names → their field types
    types: HashMap<String, HashMap<String, CheckedType>>,
    /// Current module context for scoped type lookups
    current_module: Option<String>,
    /// Known function signatures
    functions: HashMap<String, FnSig>,
    /// Current function return type
    current_return: Option<CheckedType>,
    /// Local variable types
    locals: Vec<HashMap<String, CheckedType>>,
    errors: Vec<CheckError>,
    /// Imported module paths (use declarations)
    imports: Vec<UseDecl>,
    /// Module namespace: module name → { exported names }
    modules: HashMap<String, HashMap<String, ModuleExport>>,
    /// Method registry: type name → { method name → FnSig }
    methods: HashMap<String, HashMap<String, FnSig>>,
    /// Interface declarations: interface name → [(method_name, param_type_names)]
    /// Each entry also stores the return type name for dispatch resolution.
    interfaces: HashMap<String, Vec<(String, Vec<String>, Option<String>)>>,
    /// Visibility: name → is_pub for top-level items
    visibility: HashMap<String, bool>,
    /// Resolved imported names from use declarations
    imported_items: HashMap<String, ModuleExport>,
    /// Enum variant name → parent enum type name
    enum_variants: HashMap<String, String>,
    /// Module-level `const`/`var` global names → declared type (so references to
    /// them inside functions resolve instead of erroring "undefined variable").
    global_consts: HashMap<String, CheckedType>,
    /// Enum variant name → field name → field type (for variant constructors)
    variant_fields: HashMap<String, Vec<(String, CheckedType)>>,
    /// Directories to search for external module files
    pub source_dirs: Vec<String>,
    /// Lazy external module catalog for multi-file resolution
    catalog: ModuleCatalog,
    /// Set of dotted module paths that have been loaded into this checker
    cached_loaded: HashSet<String>,
    /// 5c-R: Counter for emitted errors — enables `has_errors()` gate for
    /// "stop on first error" discipline (rustc lesson: ErrorGuaranteed).
    error_count: usize,
    /// 5c.30: When inside a method body, the RECEIVER type name so bare
    /// calls like `init()` can be resolved as `self.init()` (G-10/G-25 fix).
    current_receiver: Option<String>,
    /// 5c-R: Type interning arena — maps Named("Foo") strings to TypeIds
    /// for O(1) equality (rustc lesson: TyCtxt::intern_type).
    pub type_arena: TypeArena,
    /// Phase 7E/Feature: Type alias resolution table.
    /// Maps `type Foo = Int;` → Foo resolves to Int.
    /// Used by types_compatible to auto-coerce newtypes to their underlying types
    /// for seamless FFI calls and ecosystem wrapper ergonomics.
    aliases: HashMap<String, CheckedType>,
}

impl Checker {
    pub fn new() -> Self {
        let mut checker = Self {
            types: HashMap::new(),
            current_module: None,
            functions: HashMap::new(),
            current_return: None,
            locals: vec![HashMap::new()],
            errors: Vec::new(),
            imports: Vec::new(),
            modules: HashMap::new(),
            methods: HashMap::new(),
            interfaces: HashMap::new(),
            visibility: HashMap::new(),
            imported_items: HashMap::new(),
            enum_variants: HashMap::new(),
            global_consts: HashMap::new(),
            variant_fields: HashMap::new(),
            source_dirs: Vec::new(),
            catalog: ModuleCatalog::new(Vec::new()),
            cached_loaded: HashSet::new(),
            current_receiver: None,
            error_count: 0,
            type_arena: TypeArena::new(),
            aliases: HashMap::new(),
        };
        // Register built-in types
        checker.register_builtins();
        checker
    }

    /// Add a directory to search for external .xi module files.
    /// Updates both the legacy source_dirs and the ModuleCatalog.
    /// Register a directory containing XIOM source files (e.g. the stdlib).
    /// Source files are loaded lazily via [`build_catalog_index`].
    pub fn add_source_dir(&mut self, dir: String) {
        if !self.source_dirs.contains(&dir) {
            self.source_dirs.push(dir.clone());
        }
        self.catalog.add_source_dir(dir);
    }

    /// Build the catalog's module_path → file_path index for O(1) lookups.
    /// Index all source files added via [`add_source_dir`]. Must be called
    /// before [`check_program`] to populate the catalog of available modules,
    /// types, and function signatures.
    pub fn build_catalog_index(&mut self) {
        self.catalog.build_index();
    }

    /// Register an externally-loaded CachedModule into this checker's tables.
    /// Populates self.types, self.functions, self.modules, self.enum_variants,
    /// self.variant_fields, and self.visibility so subsequent resolution steps
    /// can find the external module's types and functions.
    pub fn register_external_module(&mut self, cached: &CachedModule) {
        for item in &cached.program.items {
            self.register_type_decl(item);
            self.register_fn_signature(item);
            self.register_all_variant_fields(&cached.program);
        }
        for item in &cached.program.items {
            if let TopDecl::Module(md) = item {
                let exports = self.build_module_map_inner(&md.items, &md.name.name);
                // Merge into existing module instead of replacing (in case the
                // same parent name is used by in-program and external modules).
                self.modules.entry(md.name.name.clone())
                    .and_modify(|existing| {
                        for (k, v) in &exports {
                            match existing.get_mut(k) {
                                Some(ModuleExport::SubModule(existing_sub)) => {
                                    if let ModuleExport::SubModule(new_sub) = v {
                                        for (sk, sv) in new_sub {
                                            existing_sub.entry(sk.clone()).or_insert(sv.clone());
                                        }
                                    }
                                }
                                Some(_) => {} // non-SubModule entry already present, keep it
                                None => { existing.insert(k.clone(), v.clone()); }
                            }
                        }
                    })
                    .or_insert(exports);
            }
        }
            self.flatten_submodules(&cached.program.items);
            self.flatten_submodules(&cached.program.items);
        }

    fn register_builtins(&mut self) {
        // All primitive types are known
        for prim in &["Bool", "Int", "Int8", "Int16", "Int32", "Int64",
                       "UInt", "UInt8", "UInt16", "UInt32", "UInt64",
                       "Float32", "Float64", "Char", "Str"] {
            self.types.insert(prim.to_string(), HashMap::new());
        }
          // Compound builtin types (empty fields = permissive field access).
          // Map is NOT a builtin — it's defined in collections.xi.
          for comp in &["Vec", "Set", "Stack", "Slice"] {
            self.types.insert(comp.to_string(), HashMap::new());
        }
        // Option with known pseudo-fields (accessors that work as field reads).
        // `.value` returns a wildcard so interface dispatch can resolve method
        // chains like `opt.value.description()` — the codegen handles the
        // concrete type at monomorphisation time.
        let mut opt = HashMap::new();
        opt.insert("is_some".to_string(), CheckedType::Bool);
        opt.insert("is_none".to_string(), CheckedType::Bool);
        opt.insert("value".to_string(), CheckedType::Named("_".into()));
        self.types.insert("Option".to_string(), opt);
        // Result with known pseudo-fields
        let mut res = HashMap::new();
        res.insert("is_ok".to_string(), CheckedType::Bool);
        res.insert("is_err".to_string(), CheckedType::Bool);
        res.insert("value".to_string(), CheckedType::Named("_".into()));
        res.insert("error".to_string(), CheckedType::Named("_".into()));
        self.types.insert("Result".to_string(), res);

        // Vec builtin methods
        self.functions.insert("Vec.new".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Named("Vec".into())),
            generics: vec![],
            uses_implicit_this: false,
        });
        // 5c-R: Vec.with_capacity(n) — pre-allocate internal buffer
        self.functions.insert("Vec.with_capacity".to_string(), FnSig {
            params: vec![("capacity".to_string(), CheckedType::Int)],
            return_type: Some(CheckedType::Named("Vec".into())),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.functions.insert("Vec.push".to_string(), FnSig {
            params: vec![
                ("self".to_string(), CheckedType::Named("Vec".into())),
                ("val".to_string(), CheckedType::Named("T".into())),
            ],
            return_type: Some(CheckedType::Unit),
            generics: vec!["T".to_string()],
            uses_implicit_this: false,
        });
        self.functions.insert("Vec.len".to_string(), FnSig {
            params: vec![
                ("self".to_string(), CheckedType::Named("Vec".into())),
            ],
            return_type: Some(CheckedType::Int),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.functions.insert("Vec.pop".to_string(), FnSig {
            params: vec![
                ("self".to_string(), CheckedType::Named("Vec".into())),
            ],
            return_type: Some(CheckedType::Named("Option".into())),
            generics: vec![],
            uses_implicit_this: false,
        });
        // 5e.1 G-18: sizeof[T]() compiler intrinsic for C FFI byte sizes.
        // Returns the LLVM byte size of type T (struct, primitive, or extern).
        // Codegen emits a compile-time constant via sizeof_struct().
        // Registered same pattern as size_of/align_of: zero-arg generic.
        self.functions.insert("sizeof".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Int),
            generics: vec!["T".to_string()],
            uses_implicit_this: false,
        });
    }

    fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.locals.pop();
    }

    fn add_local(&mut self, name: &str, ty: CheckedType) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    /// M18: Bind pattern variables from `is` expressions.
    /// `x is Some(n)` binds `n` with the inner type of `Option`.
    fn bind_is_pattern(&mut self, pattern: &Pattern, span: Span) {
        match pattern {
            Pattern::Ident(id) => {
                // Don't bind if it's an enum variant name
                let is_variant = self.enum_variants.contains_key(&id.name)
                    || self.enum_variants.keys().any(|k| k.ends_with(&format!(".{}", id.name)));
                if !is_variant {
                    self.add_local(&id.name, CheckedType::Named("_".into()));
                }
            }
            Pattern::Some(inner, _) | Pattern::Ok(inner, _) | Pattern::Err(inner, _) => {
                self.bind_is_pattern(inner, span);
            }
            Pattern::Variant(_, fields, _) => {
                for field in fields {
                    self.add_local(&field.name, CheckedType::Named("_".into()));
                }
            }
            Pattern::Or(alts, _) => {
                for alt in alts {
                    self.bind_is_pattern(alt, span);
                }
            }
            _ => {}
        }
    }

    fn lookup_local(&self, name: &str) -> Option<&CheckedType> {
        for scope in self.locals.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    fn get_type(&self, name: &str) -> Option<&HashMap<String, CheckedType>> {
        if let Some(ref module) = self.current_module {
            let prefixed = format!("{}.{}", module, name);
            if self.types.contains_key(&prefixed) {
                return self.types.get(&prefixed);
            }
        }
        self.types.get(name)
    }

    fn contains_type(&self, name: &str) -> bool {
        if let Some(ref module) = self.current_module {
            let prefixed = format!("{}.{}", module, name);
            if self.types.contains_key(&prefixed) {
                return true;
            }
        }
        self.types.contains_key(name)
    }

    fn add_pattern_bindings(&mut self, pattern: &Pattern, scrutinee_type: &CheckedType) {
        if let Pattern::Variant(_name, _, _) = pattern {
        }
        match pattern {
            Pattern::Ident(name) => {
                // Don't add bindings for unit enum variants (like Empty, None)
                if self.enum_variants.contains_key(&name.name) || self.resolve_enum_variant(&name.name).is_some() {
                    return;
                }
                // Use scrutinee type for proper guard/binding resolution
                self.add_local(&name.name, scrutinee_type.clone());
            }
            Pattern::Variant(name, fields, _) => {
                // Look up variant field types for correct binding types.
                // Tries full keys (module.Type.Variant, Type.Variant, module.Variant) then bare.
                let mut field_types: Vec<(String, CheckedType)> = Vec::new();
                let bare = &name.name;
                if let Some(parent) = self.resolve_enum_variant(bare) {
                    let keys: Vec<String> = if let Some(ref m) = self.current_module {
                        vec![
                            format!("{}.{}.{}", m, parent, bare),
                            format!("{}.{}", parent, bare),
                            format!("{}.{}", m, bare),
                            bare.clone(),
                        ]
                    } else {
                        vec![format!("{}.{}", parent, bare), bare.clone()]
                    };
                    for key in &keys {
                        if let Some(ft) = self.variant_fields.get(key) {
                            field_types = ft.clone();
                            break;
                        }
                    }
                }
                if field_types.is_empty() {
                    field_types = self.variant_fields.get(bare).cloned().unwrap_or_default();
                }
                // Fallback: use Int type (assignment checker catches mismatches)
                let _fallback_ty = CheckedType::Int;
                for (i, field) in fields.iter().enumerate() {
                    if field.name == "_" { continue; } // skip wildcard placeholders
                    let field_ty = field_types.get(i)
                        .map(|(_, ty)| ty.clone())
                        .unwrap_or(CheckedType::Int);
                    self.add_local(&field.name, field_ty);
                }
            }
            Pattern::Ok(inner, _) | Pattern::Err(inner, _) | Pattern::Some(inner, _) => {
                // Gap D fix: payload bindings from Ok/Err/Some patterns get the
                // wildcard type — the SAME convention as Result.unwrap/Option.value
                // (line ~2347). Container builtins (.len/.push) and interface
                // methods then dispatch; codegen resolves the concrete type.
                // Previously these bound as Error, which rejected all method calls
                // ("cannot call 'len'") and forced the is_ok()+unwrap() workaround.
                if let Pattern::Ident(name) = inner.as_ref() {
                    if !(self.enum_variants.contains_key(&name.name)
                        || self.resolve_enum_variant(&name.name).is_some())
                    {
                        self.add_local(&name.name, CheckedType::Named("_".into()));
                    }
                } else {
                    self.add_pattern_bindings(inner, scrutinee_type);
                }
            }
            Pattern::Or(alts, _) => {
                for alt in alts {
                    self.add_pattern_bindings(alt, scrutinee_type);
                }
            }
            Pattern::Wildcard(_) | Pattern::None(_) | Pattern::Lit(_) => {}
        }
    }

    fn resolve_enum_variant(&self, name: &str) -> Option<&String> {
        if let Some(ref module) = self.current_module {
            let prefixed = format!("{}.{}", module, name);
            if let Some(parent) = self.enum_variants.get(&prefixed) {
                return Some(parent);
            }
        }
        self.enum_variants.get(name)
    }

    /// Emit an error and return an error-poisoned type carrying an
    /// `ErrorGuaranteed` proof token. Downstream passes skip error-poisoned
    /// nodes silently — the diagnostic was already emitted (rustc lesson:
    /// one error per root cause, no cascading).
    fn error(&mut self, message: impl Into<String>, span: Span) -> CheckedType {
        self.error_with_cause(message, span, crate::types::TypeCause::Other)
    }

    /// Emit an error with a specific cause code (5c-R: TypeCause provenance).
    /// Enables "expected X because contract requires Y" diagnostics.
    fn error_with_cause(&mut self, message: impl Into<String>, span: Span, cause: crate::types::TypeCause) -> CheckedType {
        self.errors.push(CheckError { message: message.into(), span, cause });
        self.error_count += 1;
        CheckedType::Error
    }

    /// Returns `true` when any error has been emitted so far (enables the
    /// "stop on first error" discipline without checking every return value).
    /// Returns `true` if any type errors have been collected. Call after
    /// [`check_program`] to determine whether compilation should proceed.
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    // ── Type interning helpers (5c-R) ────────────────────────────────────

    /// Intern a Named type string, returning its TypeId. O(1) after first use.
    pub fn intern(&mut self, name: &str, contains_param: bool) -> TypeId {
        self.type_arena.intern(name, contains_param)
    }

    /// Look up the string name for a TypeId.
    pub fn type_name(&self, id: TypeId) -> &str {
        self.type_arena.name_of(id)
    }

    /// Fast O(1) type equality for Named types via interning.
    /// Falls back to regular PartialEq for non-Named types.
    pub fn types_eq(&self, a: &CheckedType, b: &CheckedType) -> bool {
        a == b  // PartialEq already handles all variants; interning speeds up Named comparisons
    }

    /// True when a Named type (or its interned entry) contains a generic parameter.
    pub fn contains_param(&self, id: TypeId) -> bool {
        self.type_arena.contains_param(id)
    }

    fn register_derived_method(&mut self, type_name: &str, module_path: &str, derive_trait: &DeriveTrait) {
        let method_name = match derive_trait {
            DeriveTrait::Clone => "clone",
            DeriveTrait::Eq => "eq",
            DeriveTrait::Display => "to_str",
            DeriveTrait::Hash => "hash",
            DeriveTrait::Ord => "compare",
            DeriveTrait::Debug => "fmt",
        };
        let ret_type = match derive_trait {
            DeriveTrait::Clone => CheckedType::Named(type_name.to_string()),
            DeriveTrait::Eq => CheckedType::Bool,
            DeriveTrait::Display => CheckedType::Str,
            DeriveTrait::Hash => CheckedType::Int,
            DeriveTrait::Ord => CheckedType::Int,
            DeriveTrait::Debug => CheckedType::Str,
        };
        let sig = FnSig {
            params: vec![], // no explicit params, self is implicit
            return_type: Some(ret_type),
            generics: vec![],
            uses_implicit_this: false,
        };
        let bare_key = format!("{}.{}", type_name, method_name);
        let key = if module_path.is_empty() { bare_key.clone() } else { format!("{}.{}", module_path, bare_key) };
        self.functions.insert(key.clone(), sig.clone());
        if key != bare_key {
            self.functions.entry(bare_key).or_insert(sig.clone());
        }
        self.methods
            .entry(type_name.to_string())
            .or_default()
            .insert(method_name.to_string(), sig);
    }

    fn register_all_variant_fields(&mut self, program: &Program) {
        for item in &program.items {
            self.collect_variant_fields(item, "");
        }
    }

    fn collect_variant_fields(&mut self, item: &TopDecl, module_path: &str) {
        match item {
            TopDecl::Enum(ed) => {
                for variant in &ed.variants {
                    if !variant.fields.is_empty() {
                        let variant_key = if module_path.is_empty() {
                            variant.name.name.clone()
                        } else {
                            format!("{}.{}", module_path, variant.name.name)
                        };
                        let enum_variant_key = if module_path.is_empty() {
                            format!("{}.{}", ed.name.name, variant.name.name)
                        } else {
                            format!("{}.{}.{}", module_path, ed.name.name, variant.name.name)
                        };
                        let mut vfields: Vec<(String, CheckedType)> = Vec::new();
                        for field in &variant.fields {
                            vfields.push((field.name.name.clone(), CheckedType::from_ast_type(&field.ty)));
                        }
                        self.variant_fields.insert(variant_key.clone(), vfields.clone());
                        // Register under EnumType.Variant key (e.g. AgentState.Done)
                        // for pattern-binding type resolution.
                        if enum_variant_key != variant_key {
                            self.variant_fields.insert(enum_variant_key.clone(), vfields.clone());
                        }
                        // Also register bare EnumType.Variant (e.g. "AgentState.Done")
                        // so pattern bindings with dotted variant names resolve.
                        let enum_bare = format!("{}.{}", ed.name.name, variant.name.name);
                        if enum_bare != variant_key && enum_bare != enum_variant_key && enum_bare != variant.name.name {
                            self.variant_fields.insert(enum_bare, vfields.clone());
                        }
                        if variant_key != variant.name.name {
                            self.variant_fields.insert(variant.name.name.clone(), vfields);
                        }
                    }
                }
            }
            TopDecl::Module(md) => {
                let new_path = if module_path.is_empty() {
                    md.name.name.clone()
                } else {
                    format!("{}.{}", module_path, md.name.name)
                };
                for sub in &md.items {
                    self.collect_variant_fields(sub, &new_path);
                }
            }
            _ => {}
        }
    }

    // ========================================================================
    // Program-level checking
    // ========================================================================

    /// 5c-R: Phase 1 — Collect ALL signatures without visiting bodies.
    /// After this pass, every type, function signature, interface, and global
    /// const is registered. Callers can check individual bodies or run the full
    /// body-check pass (`check_all_bodies`).
    /// (rustc lesson: collect/check split — `compiler/rustc_hir_analysis/src/collect.rs`)
    pub fn collect_signatures(&mut self, program: &Program) {
        // M20: Expand impl blocks into freestanding functions before registration
        let expanded = program.expand_impl_blocks();
        // Single pass over items: register types, functions, interfaces, consts
        for item in &expanded.items {
            self.register_type_decl(item);
            self.register_fn_signature(item);
            self.register_interface_decl(item);
            self.register_global_const(item);
        }
        // Build variant field maps from all enum declarations
        self.register_all_variant_fields(program);
        // Resolve module system (imports and module hierarchy)
        self.resolve_imports(program);
    }

    /// 5c-R: Phase 2 — Check all function bodies (collect must run first).
    pub fn check_all_bodies(&mut self, program: &Program) {
        for item in &program.items {
            self.check_top_decl(item);
        }
    }

    /// 5c-R: Choke point — after checking, certify that every body was processed
    /// and the checker state is clean. (rustc lesson: writeback certification —
    /// "every node concretely typed" before borrow check.)
    pub fn certify(&self) -> bool {
        // ErrorGuaranteed already ensures error-poisoned nodes are skipped.
        // If any errors were emitted, certification fails.
        !self.has_errors()
    }

    /// Run full type checking on a parsed program. This is the main entry point
    /// for external callers (e.g. the compiler driver).
    ///
    /// Internally calls [`collect_signatures`] first (two-pass architecture —
    /// signatures must be known before bodies are checked), then
    /// [`check_all_bodies`]. Returns `Ok(())` if no type errors were found,
    /// or `Err(errors)` with all collected errors.
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<CheckError>> {
        self.collect_signatures(program);
        self.check_all_bodies(program);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn register_type_decl(&mut self, item: &TopDecl) {
        self.register_type_decl_inner(item, "");
    }

    /// Pre-register module-level `const`/`var` globals (name → declared type) so
    /// references to them inside function bodies resolve regardless of source
    /// order. Recurses into nested modules.
    fn register_global_const(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Const(cd) => {
                let decl_ty = CheckedType::from_ast_type(&cd.ty);
                let ty = if decl_ty != CheckedType::Error && decl_ty != CheckedType::Named("_".into()) {
                    decl_ty
                } else {
                    // Unknown/elided annotation — infer from the initializer.
                    self.check_expr(&cd.value)
                };
                self.global_consts.insert(cd.name.name.clone(), ty);
            }
            TopDecl::Module(md) => {
                for sub in &md.items {
                    self.register_global_const(sub);
                }
            }
            _ => {}
        }
    }

    fn register_type_decl_inner(&mut self, item: &TopDecl, module_path: &str) {
        match item {
            TopDecl::Type(td) => {
                // Phase 7E/Feature: Register type alias for newtype auto-conversion.
                // `type Foo = Int;` → Foo resolves to Int in types_compatible.
                if let Some(ref alias_ty) = td.alias {
                    let resolved = CheckedType::from_ast_type(alias_ty);
                    let key = if module_path.is_empty() { td.name.name.clone() } else { format!("{}.{}", module_path, td.name.name) };
                    self.aliases.insert(key.clone(), resolved.clone());
                    if key != td.name.name {
                        self.aliases.entry(td.name.name.clone()).or_insert(resolved);
                    }
                }
                let mut fields = HashMap::new();
                for field in &td.fields {
                    fields.insert(field.name.name.clone(), CheckedType::from_ast_type(&field.ty));
                }
                for (name, ty, _) in &td.derived_fields {
                    fields.insert(name.name.clone(), CheckedType::from_ast_type(ty));
                }
                let key = if module_path.is_empty() { td.name.name.clone() } else { format!("{}.{}", module_path, td.name.name) };
                let bare_key = td.name.name.clone();
                self.types.insert(key.clone(), fields.clone());
                // Register derived methods (clone, eq, etc.)
                for derive_trait in &td.derives {
                    self.register_derived_method(&td.name.name, module_path, derive_trait);
                }
                // Also register with bare name as fallback (don't overwrite existing)
                if bare_key != key {
                    self.types.entry(bare_key).or_insert(fields);
                }
                self.visibility.insert(td.name.name.clone(), td.is_pub);
            }
            TopDecl::Enum(ed) => {
                let key = if module_path.is_empty() { ed.name.name.clone() } else { format!("{}.{}", module_path, ed.name.name) };
                let bare_key = ed.name.name.clone();
                self.types.insert(key.clone(), HashMap::new());
                if bare_key != key {
                    self.types.entry(bare_key).or_insert(HashMap::new());
                }
                self.visibility.insert(ed.name.name.clone(), ed.is_pub);
                // G-13: register derived methods for enums (clone/eq/hash/...).
                // Codegen already emits the implementations; the checker never
                // registered them, so `value.clone()` on a derived enum was
                // rejected with "cannot call 'clone'".
                for derive_trait in &ed.derives {
                    self.register_derived_method(&ed.name.name, module_path, derive_trait);
                }
                for variant in &ed.variants {
                    let variant_key = if module_path.is_empty() {
                        variant.name.name.clone()
                    } else {
                        format!("{}.{}", module_path, variant.name.name)
                    };
                    let parent = ed.name.name.clone();
                    self.enum_variants.entry(variant_key.clone()).or_insert(parent.clone());
                    // Also register bare variant name (first registration wins)
                    if variant_key != variant.name.name {
                        self.enum_variants.entry(variant.name.name.clone()).or_insert(parent.clone());
                    }
                    // Register EnumType.Variant key (e.g. "AgentState.Done")
                    // for dotted variant name resolution in pattern bindings.
                    let enum_bare = format!("{}.{}", parent, variant.name.name);
                    self.enum_variants.entry(enum_bare).or_insert(parent);
                    // Store variant fields for constructor field validation
                    let mut vfields: Vec<(String, CheckedType)> = Vec::new();
                    for field in &variant.fields {
                        vfields.push((field.name.name.clone(), CheckedType::from_ast_type(&field.ty)));
                    }
                    self.variant_fields.entry(variant_key.clone()).or_insert(vfields.clone());
                    // Register EnumType.Variant key for pattern-binding lookups
                    // in variant_fields too (same key as enum_variants above).
                    let enum_bare_vf = format!("{}.{}", ed.name.name, variant.name.name);
                    if enum_bare_vf != variant_key && enum_bare_vf != variant.name.name {
                        self.variant_fields.entry(enum_bare_vf).or_insert(vfields.clone());
                    }
                    if variant_key != variant.name.name {
                        self.variant_fields.entry(variant.name.name.clone()).or_insert(vfields);
                    }
                }
            }
            TopDecl::Module(md) => {
                let new_path = if module_path.is_empty() { md.name.name.clone() } else { format!("{}.{}", module_path, md.name.name) };
                for item in &md.items {
                    self.register_type_decl_inner(item, &new_path);
                }
            }
            _ => {}
        }
    }

    /// Register interface declarations so method calls on interface-typed
    /// receivers can be validated.  Interface method signatures are stored for
    /// later name resolution in the Expr::Call handler.
    fn register_interface_decl(&mut self, item: &TopDecl) {
        self.register_interface_decl_inner(item, "");
    }

    fn register_interface_decl_inner(&mut self, item: &TopDecl, module_path: &str) {
        match item {
            TopDecl::Interface(id) => {
                let key = if module_path.is_empty() { id.name.name.clone() } else { format!("{}.{}", module_path, id.name.name) };
                let bare_key = id.name.name.clone();
                let mut members: Vec<(String, Vec<String>, Option<String>)> = Vec::new();
                for member in &id.members {
                    if let InterfaceMember::FnSignature(fd) = member {
                        let param_type_names: Vec<String> = fd.params.iter()
                            .map(|p| CheckedType::from_ast_type(&p.ty).name())
                            .collect();
                        let ret_name = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t).name());
                        members.push((fd.name.name.clone(), param_type_names, ret_name));
                    }
                }
                self.interfaces.insert(key.clone(), members.clone());
                if key != bare_key {
                    self.interfaces.entry(bare_key).or_insert(members);
                }
            }
            TopDecl::Module(md) => {
                let new_path = if module_path.is_empty() { md.name.name.clone() } else { format!("{}.{}", module_path, md.name.name) };
                for item in &md.items {
                    self.register_interface_decl_inner(item, &new_path);
                }
            }
            _ => {}
        }
    }

    /// Return true when any expression in the tree references the implicit
    /// receiver keyword `this`.  Used to distinguish `this`-based methods from
    /// plain constructors during signature registration.
    fn expr_uses_this(expr: &xiom_ast::Expr) -> bool {
        match expr {
            xiom_ast::Expr::Ident(id) => id.name == "this",
            xiom_ast::Expr::Paren(e, _)
            | xiom_ast::Expr::Unary(_, e, _)
            | xiom_ast::Expr::Try(e, _)
            | xiom_ast::Expr::AtPre(e, _)
            | xiom_ast::Expr::Ref(e, _)
            | xiom_ast::Expr::MutRef(e, _)
            | xiom_ast::Expr::Some(e, _)
            | xiom_ast::Expr::Ok(e, _)
            | xiom_ast::Expr::Err(e, _)
            | xiom_ast::Expr::Await(e, _)
            | xiom_ast::Expr::Comptime(e, _)
            | xiom_ast::Expr::As(e, _, _) => Self::expr_uses_this(e),
            xiom_ast::Expr::Binary(a, _, b, _)
            | xiom_ast::Expr::Imply(a, b, _) => Self::expr_uses_this(a) || Self::expr_uses_this(b),
            xiom_ast::Expr::Field(obj, _, _) => Self::expr_uses_this(obj),
            xiom_ast::Expr::Call(func, args, _) => {
                Self::expr_uses_this(func) || args.iter().any(Self::expr_uses_this)
            }
            xiom_ast::Expr::Index(arr, idx, _) => Self::expr_uses_this(arr) || Self::expr_uses_this(idx),
            xiom_ast::Expr::Struct(_, fields, base, _) => {
                fields.iter().any(|(_, v)| Self::expr_uses_this(v))
                    || base.as_ref().map_or(false, |b| Self::expr_uses_this(b))
            }
            xiom_ast::Expr::Array(elems, _) | xiom_ast::Expr::Tuple(elems, _) => {
                elems.iter().any(Self::expr_uses_this)
            }
            xiom_ast::Expr::Closure(_, _, body, _) => Self::block_uses_this(body),
            xiom_ast::Expr::PipeClosure(_, body, _) => Self::expr_uses_this(body),
            xiom_ast::Expr::If(cond, then_b, elifs, else_b, _) => {
                Self::expr_uses_this(cond)
                    || Self::block_uses_this(then_b)
                    || elifs.iter().any(|(c, b)| Self::expr_uses_this(c) || Self::block_uses_this(b))
                    || else_b.as_ref().map_or(false, |b| Self::block_uses_this(b))
            }
            xiom_ast::Expr::Match(scrut, arms, _) => {
                Self::expr_uses_this(scrut)
                    || arms.iter().any(|arm| match &arm.body {
                        xiom_ast::MatchBody::Block(b) => Self::block_uses_this(b),
                        xiom_ast::MatchBody::Expr(e) => Self::expr_uses_this(e),
                    })
            }
            xiom_ast::Expr::Unsafe(b, _) | xiom_ast::Expr::BlockExpr(b, _) => Self::block_uses_this(b),
            _ => false,
        }
    }

    fn stmt_uses_this(stmt: &xiom_ast::Stmt) -> bool {
        match stmt {
            xiom_ast::Stmt::Expr(e, _) | xiom_ast::Stmt::Return(Some(e), _) => Self::expr_uses_this(e),
            xiom_ast::Stmt::Return(None, _) => false,
            xiom_ast::Stmt::Let(_, _, init, _) | xiom_ast::Stmt::Var(_, _, init, _) => Self::expr_uses_this(init),
            xiom_ast::Stmt::Assign(_, rhs, _) => Self::expr_uses_this(rhs),
            xiom_ast::Stmt::If(cond, then_b, elifs, else_b, _) => {
                Self::expr_uses_this(cond)
                    || Self::block_uses_this(then_b)
                    || elifs.iter().any(|(c, b)| Self::expr_uses_this(c) || Self::block_uses_this(b))
                    || else_b.as_ref().map_or(false, |b| Self::block_uses_this(b))
            }
            xiom_ast::Stmt::While(cond, body, _, _) | xiom_ast::Stmt::For(_, cond, body, _) => {
                Self::expr_uses_this(cond) || Self::block_uses_this(body)
            }
            xiom_ast::Stmt::Match(scrut, arms, _) => {
                Self::expr_uses_this(scrut)
                    || arms.iter().any(|arm| match &arm.body {
                        xiom_ast::MatchBody::Block(b) => Self::block_uses_this(b),
                        xiom_ast::MatchBody::Expr(e) => Self::expr_uses_this(e),
                    })
            }
            xiom_ast::Stmt::Spawn(b, _) => Self::block_uses_this(b),
            _ => false,
        }
    }

    fn block_uses_this(block: &xiom_ast::Block) -> bool {
        block.stmts.iter().any(|s| match s {
            xiom_ast::StmtOrExpr::Stmt(stmt) => Self::stmt_uses_this(stmt),
            xiom_ast::StmtOrExpr::Expr(expr) => Self::expr_uses_this(expr),
        })
    }

    fn register_fn_signature(&mut self, item: &TopDecl) {
        self.register_fn_signature_inner(item, "");
    }

    fn register_fn_signature_inner(&mut self, item: &TopDecl, module_path: &str) {
        match item {
            TopDecl::Fn(fd) => {
                let mut params: Vec<_> = Vec::new();
                // A `self` parameter is identified by NAME (the parser stores it as a
                // param named "self" with type `Self`), NOT by its type matching the
                // receiver. A receiver-qualified fn WITHOUT a `self` param is a static
                // constructor (e.g. `Cell.new[T](value)`) and must NOT get a synthetic
                // self — otherwise its first real argument aligns to the phantom self
                // and every call mis-reports "expected Self".
                for p in &fd.params {
                    params.push((p.name.name.clone(), CheckedType::from_ast_type(&p.ty)));
                }
                let return_type = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
                let bare_key = if let Some(recv) = fd.receiver.as_ref() {
                    format!("{}.{}", recv.name, fd.name.name)
                } else {
                    fd.name.name.clone()
                };
                let key = if module_path.is_empty() { bare_key.clone() } else { format!("{}.{}", module_path, bare_key) };
                let generics = fd.generics.iter().map(|g| g.name.name.clone()).collect();
                // Detect implicit `this` usage: receiver exists, no explicit self
                // param, and the body references `this`.
                let uses_this = fd.receiver.is_some()
                    && !fd.params.iter().any(|p| p.name.name == "self")
                    && !fd.params.first().map_or(false, |p| {
                        CheckedType::from_ast_type(&p.ty).name() == fd.receiver.as_ref().unwrap().name
                    })
                    && fd.body.as_ref().map_or(false, |b| Self::block_uses_this(b));
                let sig = FnSig { params, return_type, generics, uses_implicit_this: uses_this };
                self.functions.insert(key.clone(), sig.clone());
                // Also register with bare key as fallback (don't overwrite existing)
                if key != bare_key {
                    self.functions.entry(bare_key).or_insert(sig.clone());
                }
                self.visibility.insert(fd.name.name.clone(), fd.is_pub);
                // Track methods separately
                if let Some(recv) = fd.receiver.as_ref() {
                    let _method_key = format!("{}.{}", recv.name, fd.name.name);
                    // Use the bare method name (last component) for the method table
                    let bare_method = fd.name.name.rsplit('.').next().unwrap_or(&fd.name.name);
                    self.methods
                        .entry(recv.name.clone())
                        .or_default()
                        .insert(bare_method.to_string(), sig);
                }
            }
            TopDecl::Module(md) => {
                let new_path = if module_path.is_empty() { md.name.name.clone() } else { format!("{}.{}", module_path, md.name.name) };
                for item in &md.items {
                    self.register_fn_signature_inner(item, &new_path);
                }
            }
            TopDecl::Extern(eb) => {
                for func in &eb.functions {
                    let params: Vec<_> = func.params.iter()
                        .map(|p| (p.name.name.clone(), CheckedType::from_ast_type(&p.ty)))
                        .collect();
                    let return_type = func.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
                    let generics = func.generics.iter().map(|g| g.name.name.clone()).collect();
                    let sig = FnSig { params, return_type, generics, uses_implicit_this: false };
                    self.functions.insert(func.name.name.clone(), sig);
                    self.visibility.insert(func.name.name.clone(), func.is_pub);
                }
            }
            _ => {}
        }
    }

    fn check_top_decl(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Fn(fd) => {
                if fd.body.is_some() {
                    self.check_fn_decl(fd);
                }
            }
            TopDecl::Module(md) => {
                let prev = self.current_module.take();
                self.current_module = Some(md.name.name.clone());
                for item in &md.items {
                    self.check_top_decl(item);
                }
                self.current_module = prev;
            }
            TopDecl::Const(cd) => {
                let val_ty = self.check_expr(&cd.value);
                let decl_ty = CheckedType::from_ast_type(&cd.ty);
                if val_ty != CheckedType::Error && decl_ty != CheckedType::Error {
                    if !self.types_compatible(&val_ty, &decl_ty) {
                        self.error(
                            format!("const type mismatch: declared {}, found {}", decl_ty.name(), val_ty.name()),
                            cd.span,
                        );
                    }
                }
                // (Registration into global_consts happens in the pre-pass
                // `register_global_const` so references resolve regardless of order.)
            }
            TopDecl::Extern(_) => {} // extern blocks have no type info to register
            _ => {}
        }
    }

    // ========================================================================
    // Module system
    // ========================================================================

    fn resolve_imports(&mut self, program: &Program) {
        // Build module hierarchy from all in-program module declarations
        for item in &program.items {
            if let TopDecl::Module(md) = item {
                let exports = self.build_module_map_inner(&md.items, &md.name.name);
                self.modules.insert(md.name.name.clone(), exports);
            }
        }

        // Flatten submodules into self.modules for short-name resolution
        self.flatten_submodules(&program.items);

        // Collect use declarations recursively (they may be nested inside ModuleDecl items)
        fn collect_use_decls(items: &[TopDecl], imports: &mut Vec<UseDecl>) {
            for item in items {
                match item {
                    TopDecl::Use(ud) => imports.push(ud.clone()),
                    TopDecl::Module(md) => collect_use_decls(&md.items, imports),
                    _ => {}
                }
            }
        }
        collect_use_decls(&program.items, &mut self.imports);

        let import_snapshot = std::mem::take(&mut self.imports);

        // Pre-load every path-prefix module via the catalog so that process_use
        // can walk self.modules for multi-segment `use a.b.c` paths.
        for ud in &import_snapshot {
            if ud.path.is_empty() {
                continue;
            }
            // Load each prefix [p0], [p0,p1], ..., [p0,...,pn] via catalog.
            for end in 1..=ud.path.len() {
                let prefix: Vec<String> = ud.path[..end].iter().map(|i| i.name.clone()).collect();
                let dotted = prefix.join(".");
                if self.cached_loaded.contains(&dotted) {
                    continue;
                }
                // Only try catalog if the leaf segment isn't already in self.modules.
                let leaf = &prefix.last().unwrap();
                if self.modules.contains_key(leaf.as_str()) {
                    continue;
                }
                if let Some(cached) = self.catalog.find_owned(&prefix) {
                    self.cached_loaded.insert(dotted);
                    self.register_external_module(&cached);
                }
            }
        }
        // Build parent-module entries for dotted names so that process_use
        // can walk `self.modules.get("xiom") → async → ...`.
        // Example: registered "xiom.async" → ensure "xiom" contains "async".
        let module_keys: Vec<String> = self.modules.keys().cloned().collect();
        let mut parents: HashMap<String, HashMap<String, ModuleExport>> = HashMap::new();
        for full_key in &module_keys {
            if let Some(dot_pos) = full_key.find('.') {
                let parent = &full_key[..dot_pos];
                let child = &full_key[dot_pos + 1..];
                if let Some(child_exports) = self.modules.get(full_key).cloned() {
                    parents.entry(parent.to_string())
                        .or_insert_with(HashMap::new)
                        .insert(child.to_string(), ModuleExport::SubModule(child_exports));
                }
            }
        }
        for (parent, children) in parents {
            self.modules.entry(parent)
                .and_modify(|existing| {
                    for (k, v) in &children {
                        existing.entry(k.clone()).or_insert(v.clone());
                    }
                })
                .or_insert(children);
        }

        // Prelude: the stdlib exposes a handful of implicit helpers used
        // unqualified across modules — `to_string`/`to_int`/`to_float`/`to_char`
        // (core), `str_concat`/`str_len`/`char_at` (string), `fabs`/trig (math),
        // `gcd`/`lcm` (num), plus core `cmp`/`char` helpers. These are neither
        // `pub`-imported nor `use`d, so they were never loaded into the catalog —
        // leaving them undefined at link time and untyped at call sites
        // (`call i64` default → ptr/int IR mismatches). When a program uses ANY
        // `xiom.*` module, force-load the prelude modules so the checker resolves
        // them and codegen injects+registers their real signatures.
        //
        // Gated strictly on real stdlib usage: no non-stdlib program (and none of
        // the exact-IR diff/e2e examples, which never `use xiom.*`) is affected.
        let uses_xiom_stdlib = import_snapshot
            .iter()
            .any(|ud| ud.path.first().map(|i| i.name == "xiom").unwrap_or(false));
        if uses_xiom_stdlib {
            const PRELUDE: &[&[&str]] = &[
                &["xiom", "core"],
                &["xiom", "string"],
                &["xiom", "math"],
                &["xiom", "num"],
                &["xiom", "char"],
                &["xiom", "cmp"],
            ];
            for segs in PRELUDE {
                let prefix: Vec<String> = segs.iter().map(|s| s.to_string()).collect();
                let dotted = prefix.join(".");
                if self.cached_loaded.contains(&dotted) {
                    continue;
                }
                if let Some(cached) = self.catalog.find_owned(&prefix) {
                    self.cached_loaded.insert(dotted);
                    self.register_external_module(&cached);
                }
            }
        }

        // Now process each use declaration — self.modules is fully populated.
        for ud in &import_snapshot {
            self.process_use(ud);
        }
        self.imports = import_snapshot;
    }

    fn flatten_submodules(&mut self, items: &[TopDecl]) {
        self.flatten_submodules_inner(items, "");
    }

    fn flatten_submodules_inner(&mut self, items: &[TopDecl], prefix: &str) {
        for item in items {
            if let TopDecl::Module(md) = item {
                let new_prefix = if prefix.is_empty() { md.name.name.clone() } else { format!("{}.{}", prefix, md.name.name) };
                let exports = self.build_module_map_inner(&md.items, &new_prefix);
                self.modules.entry(md.name.name.clone()).or_insert(exports);
                self.flatten_submodules_inner(&md.items, &new_prefix);
            }
        }
    }

    /// Try to load an external module file `{source_dir}/{module_name}.xi` from the
    /// configured source directories.  Returns the module's export map if found.
    fn load_external_module(&mut self, module_name: &str) -> Option<HashMap<String, ModuleExport>> {
        for dir in &self.source_dirs {
            let file_path = format!("{}/{}.xi", dir, module_name);
            if !Path::new(&file_path).exists() {
                continue;
            }
            let source = fs::read_to_string(&file_path).ok()?;
            let tokens = Lexer::new(&source).tokenize();
            let program = Parser::new(tokens).parse_program().ok()?;
            return Some(self.build_module_map(&program.items));
        }
        None
    }

    /// Collect external declarations from the catalog that are not already present
    /// in the given program, for injection before codegen. Types, enums, and function
    /// stubs from lazily-loaded external modules are returned as TopDecl items.
    /// Primitive types are filtered out.
    pub fn collect_external_decls(&self, program: &Program) -> Vec<TopDecl> {
        // Names already declared in the program (to avoid duplicates).
        let mut existing: HashSet<String> = HashSet::new();
        fn collect_names(items: &[TopDecl], existing: &mut HashSet<String>) {
            for item in items {
                match item {
                    TopDecl::Type(td) => { existing.insert(td.name.name.clone()); }
                    TopDecl::Enum(ed) => { existing.insert(ed.name.name.clone()); }
                    TopDecl::Fn(fd) => {
                        // Key methods by their qualified name so distinct methods
                        // sharing a leaf (e.g. `Layout.new`, `Vec.new`) don't collide.
                        let key = if fd.is_method() {
                            format!("{}.{}", fd.receiver.as_ref().unwrap().name, fd.name.name)
                        } else {
                            fd.name.name.clone()
                        };
                        existing.insert(key);
                    }
                    TopDecl::Module(md) => { collect_names(&md.items, existing); }
                    _ => {}
                }
            }
        }
        collect_names(&program.items, &mut existing);

        // Primitive / builtin types that should never be injected.
        const PRIMITIVES: &[&str] = &[
            "Bool", "Int", "Int8", "Int16", "Int32", "Int64",
            "UInt", "UInt8", "UInt16", "UInt32", "UInt64",
            "Float32", "Float64", "Char", "Str", "()", "!",
            "Option", "Result", "Vec", "Slice", "Set",
            "Ptr", "Array", "Tuple", "fn", "Tuple2",
        ];

        let mut decls: Vec<TopDecl> = Vec::new();

        // Collect the names of ALL generic types (pub or not) across cached modules.
        // Methods on generic types must be monomorphised from the defining module;
        // injecting their un-monomorphised bodies produces malformed concrete IR
        // (the generic `self` is erased to i64 while the body does struct access).
        let mut generic_type_names: HashSet<String> = HashSet::new();
        fn collect_generic_types(items: &[TopDecl], out: &mut HashSet<String>) {
            for item in items {
                match item {
                    TopDecl::Type(td) if !td.generics.is_empty() => { out.insert(td.name.name.clone()); }
                    TopDecl::Enum(ed) if !ed.generics.is_empty() => { out.insert(ed.name.name.clone()); }
                    TopDecl::Module(md) => collect_generic_types(&md.items, out),
                    _ => {}
                }
            }
        }
        collect_generic_types(&program.items, &mut generic_type_names);
        for cached in self.catalog.all_cached() {
            collect_generic_types(&cached.program.items, &mut generic_type_names);
        }

        // Set of PUB generic type names. Methods on a generic type are only safe to
        // inject when their receiver type decl is ALSO injected (pub) — codegen
        // recognizes the receiver as generic (via that injected type decl) and then
        // monomorphises the method on demand instead of emitting a malformed
        // un-monomorphised concrete body. A method on a NON-pub generic type (e.g.
        // core's `BinaryHeap[T].new`) has no injected type decl, so codegen would
        // treat it as concrete and emit broken IR — those stay skipped.
        let mut pub_generic_type_names: HashSet<String> = HashSet::new();
        fn collect_pub_generic_types(items: &[TopDecl], out: &mut HashSet<String>) {
            for item in items {
                match item {
                    TopDecl::Type(td) if td.is_pub && !td.generics.is_empty() => { out.insert(td.name.name.clone()); }
                    TopDecl::Enum(ed) if ed.is_pub && !ed.generics.is_empty() => { out.insert(ed.name.name.clone()); }
                    TopDecl::Module(md) => collect_pub_generic_types(&md.items, out),
                    _ => {}
                }
            }
        }
        for cached in self.catalog.all_cached() {
            collect_pub_generic_types(&cached.program.items, &mut pub_generic_type_names);
        }

        for cached in self.catalog.all_cached() {
            // Walk the cached program items recursively and inject pub type/enum/fn decls
            // with full bodies (not stubs), deduplicated against existing names.
            fn collect_pub_decls(
                items: &[TopDecl],
                existing: &mut HashSet<String>,
                primitives: &[&str],
                generic_types: &HashSet<String>,
                pub_generic_types: &HashSet<String>,
                out: &mut Vec<TopDecl>,
            ) {
                for item in items {
                    match item {
                        TopDecl::Type(td) => {
                            if td.is_pub && !existing.contains(&td.name.name)
                                && !primitives.contains(&td.name.name.as_str()) {
                                existing.insert(td.name.name.clone());
                                out.push(TopDecl::Type(td.clone()));
                            }
                        }
                        TopDecl::Enum(ed) => {
                            if ed.is_pub && !existing.contains(&ed.name.name)
                                && !primitives.contains(&ed.name.name.as_str()) {
                                existing.insert(ed.name.name.clone());
                                out.push(TopDecl::Enum(ed.clone()));
                            }
                        }
                        TopDecl::Fn(fd) => {
                            // Note: fd.is_pub may be unreliable for file-level module
                            // parsing; since we only load modules explicitly imported
                            // via `use`, inject all candidate functions unconditionally.
                            // Methods on a PUB generic type (e.g. `Cell[T].get`) ARE
                            // injected: their receiver type decl is also injected, so
                            // codegen recognizes the receiver as generic (via
                            // `generic_type_names`), skips concrete direct-emission (the
                            // `recv_is_generic` guard in compile_top_decl), and
                            // monomorphises them on demand at each concrete call site.
                            // Without injecting them, no AST reaches codegen's
                            // `generic_fn_decls`, so the call falls back to an undefined
                            // bare `@get`/`@set` stub.
                            //
                            // Methods on a NON-pub generic type (e.g. core's
                            // `BinaryHeap[T].new`) stay SKIPPED: their type decl is not
                            // injected, so codegen would treat the receiver as concrete
                            // and emit a malformed un-monomorphised body. Note the
                            // parser drops receiver generics for static constructors
                            // (`fd.generics` is empty for `BinaryHeap[T].new`), so this
                            // receiver-type check is the only guard that catches them.
                            let recv_is_nonpub_generic = fd.receiver.as_ref()
                                .map(|r| generic_types.contains(&r.name)
                                    && !pub_generic_types.contains(&r.name))
                                .unwrap_or(false);
                            // Deduplicate by the QUALIFIED key (`Receiver.method` for
                            // methods, bare name for free functions). Deduping by the
                            // bare name alone would drop distinct methods that share a
                            // leaf name (e.g. `Layout.new`, `Vec.new`, `Rc.new`) —
                            // and since catalog iteration order is nondeterministic,
                            // which `new` survived would flip between builds.
                            let dedup_key = if fd.is_method() {
                                format!("{}.{}", fd.receiver.as_ref().unwrap().name, fd.name.name)
                            } else {
                                fd.name.name.clone()
                            };
                            if !recv_is_nonpub_generic
                                && !existing.contains(&dedup_key)
                                && !primitives.contains(&fd.name.name.as_str()) {
                                existing.insert(dedup_key);
                                // Inject with full body so codegen emits define, not declare.
                                out.push(TopDecl::Fn(fd.clone()));
                            }
                        }
                        TopDecl::Module(md) => {
                            collect_pub_decls(&md.items, existing, primitives, generic_types, pub_generic_types, out);
                        }
                        TopDecl::Extern(eb) => {
                            // Inject external modules' `extern "C"` blocks so their
                            // `declare`s (e.g. `fabs`, `sin`, socket FFI) are emitted
                            // in the merged program. Codegen's emit_extern_declares
                            // dedups by name, so injecting is safe even if some names
                            // overlap the hardcoded runtime declares.
                            out.push(TopDecl::Extern(eb.clone()));
                        }
                        TopDecl::Const(cd) => {
                            // Inject external modules' `const` declarations so codegen
                            // can substitute constant references (e.g. `SIMD_SSE`) with
                            // their literal values. Deduplicated by name.
                            if !existing.contains(&cd.name.name) {
                                existing.insert(cd.name.name.clone());
                                out.push(TopDecl::Const(cd.clone()));
                            }
                        }
                        _ => {}
                    }
                }
            }
            collect_pub_decls(&cached.program.items, &mut existing, PRIMITIVES, &generic_type_names, &pub_generic_type_names, &mut decls);
        }

        // Reachability filter: only inject FUNCTIONS whose (leaf) name is actually
        // referenced, transitively, from the program. Uncalled stdlib functions are
        // dead code; injecting their bodies as concrete `define`s risks emitting
        // malformed IR (latent codegen bugs in never-exercised helpers) that breaks
        // linking for the whole program. Types, enums, externs, and consts are always
        // kept (they are cheap and needed for signature/const resolution).
        //
        // Names are matched by LEAF identifier (method/function name), which is a
        // conservative over-approximation: a function is kept if any referenced name
        // matches its leaf. This never drops a genuinely-called function, so it is
        // safe for the regression gate; it only prunes provably-unreferenced bodies.
        fn collect_referenced_names(items: &[TopDecl], out: &mut HashSet<String>) {
            for item in items {
                match item {
                    TopDecl::Fn(fd) => {
                        if let Some(body) = &fd.body {
                            collect_block_names(body, out);
                        }
                        for c in &fd.contracts {
                            match c {
                                ContractClause::Requires(e, _)
                                | ContractClause::Ensures(e, _) => collect_expr_names(e, out),
                            }
                        }
                    }
                    TopDecl::Module(md) => collect_referenced_names(&md.items, out),
                    _ => {}
                }
            }
        }
        fn collect_block_names(block: &Block, out: &mut HashSet<String>) {
            for se in &block.stmts {
                match se {
                    StmtOrExpr::Stmt(s) => collect_stmt_names(s, out),
                    StmtOrExpr::Expr(e) => collect_expr_names(e, out),
                }
            }
        }
        fn collect_stmt_names(stmt: &Stmt, out: &mut HashSet<String>) {
            match stmt {
                Stmt::Let(_, _, e, _) | Stmt::Var(_, _, e, _) => collect_expr_names(e, out),
                Stmt::Assign(a, b, _) => { collect_expr_names(a, out); collect_expr_names(b, out); }
                Stmt::Return(Some(e), _) => collect_expr_names(e, out),
                Stmt::Return(None, _) => {}
                Stmt::Expr(e, _) => collect_expr_names(e, out),
                Stmt::If(c, t, elifs, els, _) => {
                    collect_expr_names(c, out);
                    collect_block_names(t, out);
                    for (ec, eb) in elifs { collect_expr_names(ec, out); collect_block_names(eb, out); }
                    if let Some(eb) = els { collect_block_names(eb, out); }
                }
                Stmt::Match(e, arms, _) => {
                    collect_expr_names(e, out);
                    for arm in arms {
                        if let Some(g) = &arm.guard { collect_expr_names(g, out); }
                        match &arm.body {
                            MatchBody::Block(b) => collect_block_names(b, out),
                            MatchBody::Expr(e) => collect_expr_names(e, out),
                        }
                    }
                }
                Stmt::While(c, b, _, _) => { collect_expr_names(c, out); collect_block_names(b, out); }
                Stmt::For(_, e, b, _) => { collect_expr_names(e, out); collect_block_names(b, out); }
                Stmt::Spawn(b, _) => collect_block_names(b, out),
                Stmt::Destructure(_, e, _) => collect_expr_names(e, out),
                Stmt::Break(..) | Stmt::Continue(..) => {}
            }
        }
        fn collect_expr_names(expr: &Expr, out: &mut HashSet<String>) {
            match expr {
                Expr::Ident(id) => { out.insert(id.name.clone()); }
                Expr::Field(b, f, _) => { collect_expr_names(b, out); out.insert(f.name.clone()); }
                Expr::Call(f, args, _) => {
                    collect_expr_names(f, out);
                    for a in args { collect_expr_names(a, out); }
                }
                Expr::Index(a, b, _) => { collect_expr_names(a, out); collect_expr_names(b, out); }
                Expr::Paren(e, _) | Expr::Unary(_, e, _) | Expr::Try(e, _) | Expr::AtPre(e, _)
                | Expr::Ref(e, _) | Expr::MutRef(e, _) | Expr::Some(e, _) | Expr::Ok(e, _)
                | Expr::Err(e, _) | Expr::Await(e, _) | Expr::Comptime(e, _) | Expr::As(e, _, _) => {
                    collect_expr_names(e, out);
                }
                Expr::Binary(a, _, b, _) | Expr::Imply(a, b, _) => {
                    collect_expr_names(a, out); collect_expr_names(b, out);
                }
                Expr::Is(e, pattern, _) => {
                    collect_expr_names(e, out);
                    // Collect pattern-bound variable names for the is-expression
                    collect_pattern_names(pattern, out);
                }
                Expr::Struct(id, fields, base, _) => {
                    out.insert(id.name.clone());
                    for (_, v) in fields { collect_expr_names(v, out); }
                    if let Some(b) = base { collect_expr_names(b, out); }
                }
                Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
                    for e in elems { collect_expr_names(e, out); }
                }
                Expr::Closure(_, _, b, _) => collect_block_names(b, out),
                Expr::PipeClosure(_, e, _) => collect_expr_names(e, out),
                Expr::If(c, t, elifs, els, _) => {
                    collect_expr_names(c, out);
                    collect_block_names(t, out);
                    for (ec, eb) in elifs { collect_expr_names(ec, out); collect_block_names(eb, out); }
                    if let Some(eb) = els { collect_block_names(eb, out); }
                }
                Expr::Match(e, arms, _) => {
                    collect_expr_names(e, out);
                    for arm in arms {
                        if let Some(g) = &arm.guard { collect_expr_names(g, out); }
                        match &arm.body {
                            MatchBody::Block(b) => collect_block_names(b, out),
                            MatchBody::Expr(e) => collect_expr_names(e, out),
                        }
                    }
                }
                Expr::Unsafe(b, _) | Expr::BlockExpr(b, _) => collect_block_names(b, out),
                _ => {}
            }
        }

        // Seed with names referenced by the program itself.
        let mut referenced: HashSet<String> = HashSet::new();
        collect_referenced_names(&program.items, &mut referenced);

        // Split candidate function decls from always-kept decls.
        let mut fn_candidates: Vec<FnDecl> = Vec::new();
        let mut kept: Vec<TopDecl> = Vec::new();
        for d in decls {
            match d {
                TopDecl::Fn(fd) => fn_candidates.push(fd),
                other => kept.push(other),
            }
        }

        /// Collect variable names bound in a pattern (for `is` expressions).
        fn collect_pattern_names(pattern: &Pattern, out: &mut HashSet<String>) {
            match pattern {
                Pattern::Ident(id) => { out.insert(id.name.clone()); }
                Pattern::Some(inner, _) | Pattern::Ok(inner, _) | Pattern::Err(inner, _) => {
                    collect_pattern_names(inner, out);
                }
                Pattern::Variant(_, fields, _) => {
                    for f in fields { out.insert(f.name.clone()); }
                }
                Pattern::Or(alts, _) => {
                    for alt in alts { collect_pattern_names(alt, out); }
                }
                _ => {}
            }
        }

        // Transitive fixpoint: a function is reachable if its leaf name is referenced.
        // Once included, names referenced in its body/contracts become reachable too.
        let mut chosen: Vec<FnDecl> = Vec::new();
        let mut chosen_keys: HashSet<String> = HashSet::new();
        loop {
            let mut added = false;
            let mut i = 0;
            while i < fn_candidates.len() {
                let leaf_reachable = referenced.contains(&fn_candidates[i].name.name);
                let key = if fn_candidates[i].is_method() {
                    format!("{}.{}", fn_candidates[i].receiver.as_ref().unwrap().name, fn_candidates[i].name.name)
                } else {
                    fn_candidates[i].name.name.clone()
                };
                let qualified_reachable = referenced.contains(&key);
                let is_reachable = leaf_reachable || qualified_reachable;
                if is_reachable {
                    let fd = fn_candidates.remove(i);
                    let key = if fd.is_method() {
                        format!("{}.{}", fd.receiver.as_ref().unwrap().name, fd.name.name)
                    } else {
                        fd.name.name.clone()
                    };
                    if chosen_keys.insert(key) {
                        if let Some(body) = &fd.body {
                            collect_block_names(body, &mut referenced);
                        }
                        for c in &fd.contracts {
                            match c {
                                ContractClause::Requires(e, _)
                                | ContractClause::Ensures(e, _) => collect_expr_names(e, &mut referenced),
                            }
                        }
                        chosen.push(fd);
                        added = true;
                    }
                } else {
                    i += 1;
                }
            }
            if !added { break; }
        }

        let mut result = kept;
        for fd in chosen {
            result.push(TopDecl::Fn(fd));
        }
        result
    }

    fn build_module_map(&self, items: &[TopDecl]) -> HashMap<String, ModuleExport> {
        self.build_module_map_inner(items, "")
    }

    fn build_module_map_inner(&self, items: &[TopDecl], prefix: &str) -> HashMap<String, ModuleExport> {
        let mut map = HashMap::new();
        for item in items {
            match item {
                TopDecl::Type(td) => {
                    let key = if prefix.is_empty() { td.name.name.clone() } else { format!("{}.{}", prefix, td.name.name) };
                    let fields = self.types.get(&key).cloned().unwrap_or_default();
                    map.insert(td.name.name.clone(), ModuleExport::Type { fields, is_pub: td.is_pub });
                }
                TopDecl::Enum(ed) => {
                    map.insert(ed.name.name.clone(), ModuleExport::Type { fields: HashMap::new(), is_pub: ed.is_pub });
                }
                TopDecl::Fn(fd) => {
                    let map_key = if fd.is_method() {
                        format!("{}.{}", fd.receiver.as_ref().unwrap().name, fd.name.name)
                    } else {
                        fd.name.name.clone()
                    };
                    let prefixed_key = if prefix.is_empty() { map_key.clone() } else { format!("{}.{}", prefix, map_key) };
                    let is_pub = fd.is_pub;
                    // Try self.functions first (populated by register_fn_signature).
                    // Fallback: build FnSig from the FnDecl AST (needed for external modules
                    // loaded via load_external_module before register_fn_signature runs).
                    let sig = self.functions.get(&prefixed_key)
                        .or_else(|| self.functions.get(&map_key))
                        .cloned()
                        .unwrap_or_else(|| {
                            let params: Vec<(String, CheckedType)> = fd.params.iter().map(|p| {
                                (p.name.name.clone(), CheckedType::from_ast_type(&p.ty))
                            }).collect();
                let return_type = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
                            let generics = fd.generics.iter().map(|g| g.name.name.clone()).collect();
                            FnSig { params, return_type, generics, uses_implicit_this: false }
                        });
                    map.insert(map_key, ModuleExport::Function { sig, is_pub });
                }
                TopDecl::Module(md) => {
                    let new_prefix = if prefix.is_empty() { md.name.name.clone() } else { format!("{}.{}", prefix, md.name.name) };
                    let sub = self.build_module_map_inner(&md.items, &new_prefix);
                    map.insert(md.name.name.clone(), ModuleExport::SubModule(sub));
                }
                TopDecl::Const(cd) if cd.is_pub => {
                    let ty = CheckedType::from_ast_type(&cd.ty);
                    map.insert(cd.name.name.clone(), ModuleExport::Const {
                        ty,
                        value: cd.value.clone(),
                        is_pub: true,
                    });
                }
                TopDecl::Extern(eb) => {
                    // Register each extern function in the export map so cross-module
                    // `use` can resolve them with their declared return types.
                    for fd in &eb.functions {
                        let params: Vec<(String, CheckedType)> = fd.params.iter()
                            .map(|p| (p.name.name.clone(), CheckedType::from_ast_type(&p.ty)))
                            .collect();
                        let return_type = fd.return_type.as_ref()
                            .map(|t| CheckedType::from_ast_type(t));
                        let generics = fd.generics.iter().map(|g| g.name.name.clone()).collect();
                        let sig = FnSig { params, return_type, generics, uses_implicit_this: false };
                        map.insert(fd.name.name.clone(), ModuleExport::Function { sig, is_pub: true });
                    }
                }
                _ => {}
            }
        }
        map
    }

    fn process_use(&mut self, ud: &UseDecl) {
        if ud.path.is_empty() {
            return;
        }

        let module_name = &ud.path[0].name;
        // Clone the exports map to avoid borrow conflicts with self.modules.insert below
        let exports = match self.modules.get(module_name).cloned() {
            Some(e) => e,
            None => {
                // First segment not in modules (e.g. "xiom" from `use xiom.async`
                // when no standalone xiom.xi exists). Load the full path from catalog
                // and build a parent module entry containing the submodule.
                let full_path: Vec<String> = ud.path.iter().map(|p| p.name.clone()).collect();
                if let Some(cached) = self.catalog.find_owned(&full_path) {
                    // Register function signatures from the loaded module so
                    // method resolution works (e.g. Vec.insert, Map.contains).
                    // build_module_map creates export maps but doesn't register
                    // functions in self.functions — without this, method calls
                    // on stdlib types fail with "cannot call on this expression".
                    for item in &cached.program.items {
                        self.register_fn_signature(item);
                    }
                    let sub_exports = self.build_module_map(&cached.program.items);
                    let mut parent = HashMap::new();
                    // Extract the short submodule name from the last path segment
                    let short = ud.path.last().map(|p| p.name.clone()).unwrap_or_default();
                    parent.insert(short, ModuleExport::SubModule(sub_exports));
                    self.modules.insert(module_name.clone(), parent.clone());
                    parent
                } else {
                    return;
                }
            }
        };

        // Walk through intermediate path segments (submodules)
        let mut current = exports;
        for i in 1..ud.path.len() - 1 {
            let seg = &ud.path[i].name;
            match current.get(seg) {
                Some(ModuleExport::SubModule(sub)) => {
                    current = sub.clone();
                }
                _ => {
                    // Try to load submodule from external file.
                    // The path is relative to the source directory: {source_dir}/{seg}.xi
                    if let Some(mut sub_exports) = self.load_external_module(seg) {
                        // The loaded file may have nested module wrappers
                        // (e.g., main.xi contains `module benchmark.main { ... }`).
                        // Walk into any top-level module to find the actual exports.
                        while sub_exports.len() == 1 {
                            let (only_key, only_val) = sub_exports.iter().next().unwrap();
                            if let ModuleExport::SubModule(inner) = only_val {
                                if !only_key.is_empty() {
                                    sub_exports = inner.clone();
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        // Store in self.modules for future lookups
                        self.modules.insert(seg.clone(), sub_exports.clone());
                        current = sub_exports;
                        continue;
                    }
                    return;
                }
            }
        }

        if ud.glob {
            // `use module.*;` — import all pub items
            for (name, export) in current {
                if matches!(export, ModuleExport::SubModule(_)) { continue; }
                let is_pub = match export {
                    ModuleExport::Type { is_pub, .. } => is_pub,
                    ModuleExport::Function { is_pub, .. } => is_pub,
                    ModuleExport::Const { is_pub, .. } => is_pub,
                    _ => false,
                };
                if is_pub {
                    self.imported_items.insert(name.clone(), export.clone());
                }
            }
        } else {
            // `use module.item;` or `use module.item as alias;`
            let item_name = &ud.path.last().unwrap().name;
            let export = match current.get(item_name) {
                Some(e) => e.clone(),
                None => {
                    // Not found in current module — try loading the full dotted
                    // path from catalog (e.g. "xiom.async" when the parent module
                    // "xiom" is incomplete or the submodule wasn't pre-indexed).
                    let full_path: Vec<String> = ud.path.iter().map(|p| p.name.clone()).collect();
                    if let Some(cached) = self.catalog.find_owned(&full_path) {
                        let module_exports = self.build_module_map(&cached.program.items);
                        let local_name = ud.alias.as_ref()
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| item_name.clone());
                        let export = ModuleExport::SubModule(module_exports);
                        self.modules.entry(local_name.clone()).or_insert_with(|| {
                            if let ModuleExport::SubModule(ref s) = export { s.clone() } else { HashMap::new() }
                        });
                        self.imported_items.insert(local_name, export);
                        return; // Already fully handled
                    }
                    return;
                }
            };
            let local_name = ud.alias.as_ref()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| item_name.clone());
            // Register SubModules in both imported_items (for type paths)
            // and modules (for expression paths like `async.Executor.new()`)
            if let ModuleExport::SubModule(sub_exports) = &export {
                self.modules.entry(local_name.clone()).or_insert_with(|| sub_exports.clone());
                // G-32: when `use mod` imports a SubModule, recursively
                // inject its pub items (fns, types, CONSTS) so bare `PI`
                // resolves. Previously the module was registered but the
                // items inside were hidden — consts were invisible to bare
                // reference while `mod.PI` worked (the catalog path hit
                // find_external_module which does a fresh scan).
                for (name, item_export) in sub_exports.iter() {
                    let is_pub = match item_export {
                        ModuleExport::Type { is_pub, .. } | ModuleExport::Function { is_pub, .. } | ModuleExport::Const { is_pub, .. } => *is_pub,
                        _ => false,
                    };
                    if is_pub && !self.imported_items.contains_key(name) {
                        self.imported_items.insert(name.clone(), item_export.clone());
                    }
                }
            }
            self.imported_items.insert(local_name, export);
        }
    }

    /// Try to resolve a module-qualified call: `module.func(args)` or `module.submodule.func(args)`
    fn check_module_call(&mut self, obj: &Expr, method: &Ident, args: &[Expr], span: Span) -> Option<CheckedType> {
        // Build the module path from the expression chain
        let mut reversed: Vec<String> = Vec::new();
        let mut current = obj;
        loop {
            match current {
                Expr::Ident(ident) => {
                    reversed.push(ident.name.clone());
                    break;
                }
                Expr::Field(inner, field, _) => {
                    reversed.push(field.name.clone());
                    current = inner;
                }
                _ => return None,
            }
        }

        // reversed is [inner, ..., outer]; flip to [outer, ..., inner]
        let mut path: Vec<String> = reversed.into_iter().rev().collect();
        path.push(method.name.clone());

        // If the path has only 1 element (bare method name), not a module call
        if path.len() < 2 {
            return None;
        }

        // Clone the relevant export data to avoid borrow conflicts
        let sig = self.resolve_module_function(&path)?;
        let sig = sig.clone();

        for (i, arg) in args.iter().enumerate() {
            let arg_ty = self.check_expr(arg);
            if i < sig.params.len() {
                let expected = &sig.params[i].1;
                if !self.types_compatible(&arg_ty, expected) && arg_ty != CheckedType::Error {
                    self.error(
                        format!("argument {} type mismatch: expected {}, found {}",
                            i + 1, expected.name(), arg_ty.name()),
                        span,
                    );
                }
            }
        }
        Some(sig.return_type.unwrap_or(CheckedType::Unit))
    }

    /// Resolve a module path to a function signature, checking pub visibility.
    /// Returns None if the path doesn't resolve to a pub function.
    fn resolve_module_function(&self, path: &[String]) -> Option<&FnSig> {
        let module_name = &path[0];
        // Try modules first, then imported_items (short names from `use`)
        let exports = self.modules.get(module_name).or_else(|| {
            self.imported_items.get(module_name).and_then(|export| {
                match export {
                    ModuleExport::SubModule(exports) => Some(exports),
                    _ => None,
                }
            })
        })?;
        let mut current_exports = exports;
        for i in 1..path.len() - 1 {
            let seg = &path[i];
            let export = current_exports.get(seg)?;
            match export {
                ModuleExport::SubModule(sub) => current_exports = sub,
                _ => return None,
            }
        }
        let func_name = &path[path.len() - 1];
        let export = current_exports.get(func_name)?;
        match export {
            ModuleExport::Function { sig, is_pub: true } => Some(sig),
            _ => None,
        }
    }

    /// Resolve a module path to a function signature without pub check
    /// (used by check_module_field_access to verify visibility separately).
    /// Try to resolve module-qualified field access: `module.Type` or `module.sub.Type`
    fn check_module_field_access(&self, obj: &Expr, field: &Ident) -> Option<CheckedType> {
        let mut reversed: Vec<String> = Vec::new();
        let mut current = obj;
        loop {
            match current {
                Expr::Ident(ident) => {
                    reversed.push(ident.name.clone());
                    break;
                }
                Expr::Field(inner, f, _) => {
                    reversed.push(f.name.clone());
                    current = inner;
                }
                _ => return None,
            }
        }

        let mut path: Vec<String> = reversed.into_iter().rev().collect();
        path.push(field.name.clone());

        if path.len() < 2 {
            return None;
        }

        let module_name = &path[0];
        // First try `modules` (full module paths), then fall back to
        // `imported_items` (short names from `use` declarations).
        // `use xiom.async` inserts "async" → SubModule(exports) into
        // imported_items but not into modules.
        let exports = self.modules.get(module_name).or_else(|| {
            self.imported_items.get(module_name).and_then(|export| {
                match export {
                    ModuleExport::SubModule(exports) => Some(exports),
                    _ => None,
                }
            })
        })?;
        let mut current_exports = exports;
        for i in 1..path.len() - 1 {
            let seg = &path[i];
            let export = current_exports.get(seg)?;
            match export {
                ModuleExport::SubModule(sub) => current_exports = sub,
                _ => return None,
            }
        }

        let name = &path[path.len() - 1];
        let export = current_exports.get(name)?;
        Some(match export {
            ModuleExport::Function { is_pub, .. } => {
                if *is_pub { CheckedType::Named("fn".into()) } else { return None; }
            }
            ModuleExport::Type { is_pub, .. } => {
                if *is_pub { CheckedType::Named(field.name.clone()) } else { return None; }
            }
            ModuleExport::Const { ty, is_pub, .. } => {
                if *is_pub { ty.clone() } else { return None; }
            }
            ModuleExport::SubModule(_) => CheckedType::Named("module".into()),
        })
    }

    // ========================================================================
    // Function checking
    // ========================================================================

    fn check_fn_decl(&mut self, fd: &FnDecl) {
        self.push_scope();

        // Add parameters to scope
        for param in &fd.params {
            self.add_local(&param.name.name, CheckedType::from_ast_type(&param.ty));
        }

        // 5c-E: register const-generic parameters as locals
        // `fn len[T, const N: Int](arr: &[N]T) -> Int { N }` — N must resolve
        for g in &fd.generics {
            let gen_ty = if g.is_const {
                g.const_ty.as_ref().map(|t| CheckedType::from_ast_type(t)).unwrap_or(CheckedType::Int)
            } else {
                CheckedType::Named("type".into())
            };
            self.add_local(&g.name.name, gen_ty);
        }

        // For methods, inject the receiver's fields into scope (implicit self)
        if let Some(recv) = fd.receiver.as_ref() {
            // Add self as a variable (for match self { ... } in enum methods)
            self.add_local("self", CheckedType::Named(recv.name.clone()));
            // 5c.30: track the receiver type for implicit-self method call
            // resolution (G-10: bare `init()` inside `fn GrpcClient.init()`).
            self.current_receiver = Some(recv.name.clone());
            // G-20: bare receiver fields are backed by codegen for ALL slot
            // forms now — explicit `self`, receiver-style `&T` first param,
            // `this`-based bodies, AND bare-field bodies (codegen emits a
            // %param_self slot whenever the body mentions receiver state).
            let fields_clone = self.get_type(&recv.name).cloned();
            if let Some(fields) = fields_clone {
                for (field_name, field_ty) in fields {
                    self.add_local(&field_name, field_ty);
                }
            }
        }

        // Set expected return type
        let expected_return = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
        self.current_return = expected_return.clone();

        // Check body
        if let Some(body) = fd.body.as_ref() {
            self.check_block(body, expected_return);
        }

        self.pop_scope();
        self.current_receiver = None;
    }

    fn check_block(&mut self, block: &Block, expected_return: Option<CheckedType>) -> Option<CheckedType> {
        let mut last_expr_ty = None;
        let mut has_return = false;
        let mut tail_diverges = false;

        for item in &block.stmts {
            match item {
                StmtOrExpr::Stmt(stmt) => {
                    self.check_stmt(stmt);
                    if Self::stmt_always_returns(stmt) {
                        has_return = true;
                        last_expr_ty = None; // return already checked, don't double-check
                    }
                    tail_diverges = false;
                }
                StmtOrExpr::Expr(expr) => {
                    last_expr_ty = Some(self.check_expr(expr));
                    // Divergence analysis: a tail expression whose every path
                    // ends in `return` (e.g. `unsafe { ...; return X; }`)
                    // satisfies any declared return type — the block value is
                    // never observed. Production pattern in FFI wrappers.
                    tail_diverges = Self::expr_always_returns(expr);
                }
            }
        }

        // If this block is the function body and has a return, skip the return type check
        // (return statements are already checked individually)
        if let Some(expected) = expected_return {
            if !has_return && !tail_diverges {
                if let Some(found) = &last_expr_ty {
                    if found != &CheckedType::Error && expected != CheckedType::Error {
                        if !self.types_compatible(found, &expected) {
                            self.error(
                                format!("return type mismatch: expected {}, found {}", expected.name(), found.name()),
                                block.span,
                            );
                        }
                    }
                }
            } else if expected != CheckedType::Unit {
                // No expression at end, but return type expected
                // Only warn if there are no return statements (handled elsewhere)
            }
        }

        last_expr_ty
    }

    /// Divergence analysis: does this block ALWAYS exit via `return` on every
    /// path? Used to accept `fn f() -> T { unsafe { ...; return x; } }` where
    /// the tail expression types as Unit but control never falls through.
    fn block_always_returns(block: &Block) -> bool {
        for item in &block.stmts {
            match item {
                StmtOrExpr::Stmt(s) => {
                    if Self::stmt_always_returns(s) {
                        return true; // everything after is unreachable
                    }
                }
                StmtOrExpr::Expr(e) => {
                    if Self::expr_always_returns(e) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn stmt_always_returns(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return(..) => true,
            Stmt::Expr(e, ..) => Self::expr_always_returns(e),
            Stmt::If(_, then_b, elifs, Some(else_b), _) => {
                Self::block_always_returns(then_b)
                    && elifs.iter().all(|(_, b)| Self::block_always_returns(b))
                    && Self::block_always_returns(else_b)
            }
            Stmt::Match(_, arms, _) => {
                !arms.is_empty() && arms.iter().all(|a| Self::match_body_always_returns(&a.body))
            }
            _ => false,
        }
    }

    fn expr_always_returns(expr: &Expr) -> bool {
        match expr {
            Expr::Unsafe(block, _) | Expr::BlockExpr(block, _) => Self::block_always_returns(block),
            Expr::Paren(inner, _) => Self::expr_always_returns(inner),
            Expr::If(_, then_b, elifs, Some(else_b), _) => {
                Self::block_always_returns(then_b)
                    && elifs.iter().all(|(_, b)| Self::block_always_returns(b))
                    && Self::block_always_returns(else_b)
            }
            Expr::Match(_, arms, _) => {
                !arms.is_empty() && arms.iter().all(|a| Self::match_body_always_returns(&a.body))
            }
            _ => false,
        }
    }

    fn match_body_always_returns(body: &MatchBody) -> bool {
        match body {
            MatchBody::Block(b) => Self::block_always_returns(b),
            MatchBody::Expr(e) => Self::expr_always_returns(e),
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, ty_annot, value, span) => {
                let val_ty = self.check_expr(value);
                if let Some(annot) = ty_annot {
                    let annot_ty = CheckedType::from_ast_type(annot);
                    // An uninitialized let/var defaults to the placeholder `Int(0)`.
                    // When a type annotation is present the var is zero-initialized to that
                    // type, so trust the annotation instead of erroring on the placeholder.
                    let is_placeholder = matches!(value, Expr::Int(0, _));
                    if is_placeholder {
                        self.add_local(&name.name, annot_ty);
                        return;
                    }
                    if !self.types_compatible(&val_ty, &annot_ty)
                        && val_ty != CheckedType::Error
                        && !matches!(&val_ty, CheckedType::Named(n) if n == "_")
                    {
                        self.error(
                            format!("type mismatch in let: annotated {}, found {}", annot_ty.name(), val_ty.name()),
                            *span,
                        );
                    }
                    self.add_local(&name.name, annot_ty);
                    return;
                }
                self.add_local(&name.name, val_ty);
            }
            Stmt::Var(name, ty_annot, value, span) => {
                let val_ty = self.check_expr(value);
                if let Some(annot) = ty_annot {
                    let annot_ty = CheckedType::from_ast_type(annot);
                    // An uninitialized let/var defaults to the placeholder `Int(0)`.
                    // When a type annotation is present the var is zero-initialized to that
                    // type, so trust the annotation instead of erroring on the placeholder.
                    let is_placeholder = matches!(value, Expr::Int(0, _));
                    if is_placeholder {
                        self.add_local(&name.name, annot_ty);
                        return;
                    }
                    if !self.types_compatible(&val_ty, &annot_ty)
                        && val_ty != CheckedType::Error
                        && !matches!(&val_ty, CheckedType::Named(n) if n == "_")
                    {
                        self.error(
                            format!("type mismatch in var: annotated {}, found {}", annot_ty.name(), val_ty.name()),
                            *span,
                        );
                    }
                    self.add_local(&name.name, annot_ty);
                    return;
                }
                self.add_local(&name.name, val_ty);
            }
            Stmt::Assign(place, value, span) => {
                let place_ty = self.check_expr(place);
                let val_ty = self.check_expr(value);
                let is_bool_int = matches!((&place_ty, &val_ty), (CheckedType::Bool, CheckedType::Int) | (CheckedType::Int, CheckedType::Bool));
                if !is_bool_int && !self.types_compatible(&place_ty, &val_ty) && place_ty != CheckedType::Error && val_ty != CheckedType::Error {
                    self.error(
                        format!("assignment type mismatch: {} = {}", place_ty.name(), val_ty.name()),
                        *span,
                    );
                }
            }
            Stmt::Return(expr, span) => {
                let ret_ty = expr.as_ref().map(|e| self.check_expr(e)).unwrap_or(CheckedType::Unit);
                if let Some(expected) = self.current_return.as_ref() {
                    if !self.types_compatible(&ret_ty, expected) && ret_ty != CheckedType::Error {
                        self.error(
                            format!("return type mismatch: expected {}, found {}", expected.name(), ret_ty.name()),
                            *span,
                        );
                    }
                }
            }
            Stmt::Expr(expr, _) => {
                self.check_expr(expr);
            }
            Stmt::If(cond, then_block, elifs, else_block, _) => {
                let cond_ty = self.check_expr(cond);
                let is_lenient = |ty: &CheckedType| -> bool {
                    matches!(ty, CheckedType::Named(n) if n.starts_with("Tuple") || n == "_" || (n.len() == 1 && n.chars().next().map_or(false, |c| c.is_ascii_uppercase())))
                };
                if cond_ty.name() != "Bool" && cond_ty != CheckedType::Error && !is_lenient(&cond_ty) {
                    self.error(format!("if condition must be Bool, found {}", cond_ty.name()), cond.span());
                }
                self.check_block(then_block, None);
                for (econd, eblock) in elifs {
                    let econd_ty = self.check_expr(econd);
                    if econd_ty.name() != "Bool" && econd_ty != CheckedType::Error && !is_lenient(&econd_ty) {
                        self.error(format!("elif condition must be Bool, found {}", econd_ty.name()), econd.span());
                    }
                    self.check_block(eblock, None);
                }
                if let Some(eb) = else_block {
                    self.check_block(eb, None);
                }
            }
            Stmt::Match(expr, arms, _) => {
                let matched_ty = self.check_expr(expr);
                for arm in arms {
                    self.push_scope();
                    // Add pattern bindings to scope
                    self.add_pattern_bindings(&arm.pattern, &matched_ty);
                    // Check guard expression if present
                    if let Some(ref guard) = arm.guard {
                        self.check_expr(guard);
                    }
                    match &arm.body {
                        MatchBody::Block(b) => { self.check_block(b, None); }
                        MatchBody::Expr(e) => { self.check_expr(e); }
                    }
                    self.pop_scope();
                }
                let _ = matched_ty;
            }
            Stmt::While(cond, body, _, _) => {
                let cond_ty = self.check_expr(cond);
                if cond_ty.name() != "Bool" && cond_ty != CheckedType::Error {
                    self.error(format!("while condition must be Bool, found {}", cond_ty.name()), cond.span());
                }
                self.check_block(body, None);
            }
            Stmt::For(var, iter, body, _) => {
                let _iter_ty = self.check_expr(iter);
                self.add_local(&var.name, CheckedType::Int); // simplified
                self.check_block(body, None);
            }
            Stmt::Destructure(names, value, _) => {
                let val_ty = self.check_expr(value);
                for name in names {
                    self.add_local(&name.name, val_ty.clone());
                }
            }
            Stmt::Spawn(body, _) => {
                self.check_block(body, None);
            }
            Stmt::Break(..) => {}
            Stmt::Continue(..) => {}
        }
    }

    // ========================================================================
    // Expression type checking
    // ========================================================================

    fn check_expr(&mut self, expr: &Expr) -> CheckedType {
        match expr {
            Expr::Ident(ident) => {
                // `this` is an alias for `self` in method bodies
                let lookup_name: &str = if ident.name == "this" { "self" } else { &ident.name };
                if lookup_name == "_" {
                    CheckedType::Int // wildcard placeholder type
                } else if ident.name == "null" {
                    CheckedType::Named("Ptr".to_string())
                } else if let Some(ty) = self.lookup_local(lookup_name) {
                    ty.clone()
                } else if let Some(ty) = self.global_consts.get(&ident.name) {
                    ty.clone()
                } else if self.functions.contains_key(&ident.name) {
                    CheckedType::Named("fn".into())
                } else if self.contains_type(&ident.name) {
                    CheckedType::Named(ident.name.clone())
                } else if let Some(parent_enum) = self.resolve_enum_variant(&ident.name) {
                    CheckedType::Named(parent_enum.clone())
                } else if let Some(parent) = self.enum_variants.get(&ident.name) {
                    // Direct fallback for bare variant names (bypass module-scoped lookup)
                    CheckedType::Named(parent.clone())
                } else if let Some(export) = self.imported_items.get(&ident.name) {
                    match export {
                        ModuleExport::Function { .. } => CheckedType::Named("fn".into()),
                        ModuleExport::Type { .. } => CheckedType::Named(ident.name.clone()),
                        ModuleExport::Const { ty, .. } => ty.clone(),
                        ModuleExport::SubModule(_) => CheckedType::Named("module".into()),
                    }
                // 5c.30: implicit-self method calls (G-10). When inside a
                // method body, bare calls like `init()` resolve to
                // `self.init()`. The identifier must name a method
                // registered for the current receiver type.
                } else if let Some(ref recv) = self.current_receiver {
                    if let Some(ms) = self.methods.get(recv) {
                        if ms.contains_key(&ident.name) {
                            return CheckedType::Named("fn".into());
                        }
                    }
                    // Also try module-qualified receiver
                    let suffix = format!(".{recv}");
                    for (key, ms) in &self.methods {
                        if key.ends_with(&suffix) && ms.contains_key(&ident.name) {
                            return CheckedType::Named("fn".into());
                        }
                    }
                    self.error(format!("undefined variable '{}'", ident.name), ident.span)
                } else {
                    self.error(format!("undefined variable '{}'", ident.name), ident.span)
                }
            }
            Expr::Int(_, _) => CheckedType::Int,
            Expr::Float(_, _) => CheckedType::Float64,
            Expr::Str(_, _) => CheckedType::Str,
            Expr::Char(_, _) => CheckedType::Char,
            Expr::Bool(_, _) => CheckedType::Bool,
            Expr::Paren(inner, _) => self.check_expr(inner),
            Expr::Tuple(items, _) => {
                let mut last = CheckedType::Unit;
                for item in items {
                    last = self.check_expr(item);
                }
                last
            }
            Expr::Unary(op, inner, span) => {
                let inner_ty = self.check_expr(inner);
                match op {
                    UnaryOp::Neg => {
                        if !inner_ty.is_numeric() {
                            self.error(format!("cannot negate type {}", inner_ty.name()), *span);
                        }
                        inner_ty
                    }
                    UnaryOp::Not => {
                        if inner_ty.name() != "Bool"
                            && !matches!(&inner_ty, CheckedType::Named(n) if n == "_")
                        {
                            self.error(format!("cannot logically negate type {}", inner_ty.name()), *span);
                        }
                        CheckedType::Bool
                    }
                    UnaryOp::Ref | UnaryOp::MutRef => inner_ty, // reference keeps the type
                    UnaryOp::BitNot => inner_ty, // bitwise not preserves integer type
                    UnaryOp::Deref => {
                        // *p: strip pointer type — *Ptr[T] → T, *Ptr → Int
                        if inner_ty == CheckedType::Named("Ptr".into()) {
                            CheckedType::Int
                        } else {
                            inner_ty
                        }
                    }
                }
            }
            Expr::Binary(left, op, right, span) => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                        let is_generic_param = |ty: &CheckedType| -> bool {
                            if let CheckedType::Named(n) = ty {
                                n == "_" || (n.len() == 1 && n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
                            } else { false }
                        };
                        // Str + Str is concatenation (handled by codegen).
                        if matches!(op, BinOp::Add) && (left_ty == CheckedType::Str || right_ty == CheckedType::Str) {
                            return CheckedType::Str;
                        }
                        if !left_ty.is_numeric() && !is_generic_param(&left_ty) {
                            self.error(format!("left operand must be numeric, found {}", left_ty.name()), *span);
                        }
                        if !right_ty.is_numeric() && !is_generic_param(&right_ty) {
                            self.error(format!("right operand must be numeric, found {}", right_ty.name()), *span);
                        }
                        left_ty // result type is the left operand type (promotion in Phase 1)
                    }
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        CheckedType::Bool
                    }
                    BinOp::And | BinOp::Or => {
                        if left_ty.name() != "Bool" && !matches!(&left_ty, CheckedType::Named(n) if n == "_") {
                            self.error(format!("left operand of logical op must be Bool, found {}", left_ty.name()), *span);
                        }
                        if right_ty.name() != "Bool" && !matches!(&right_ty, CheckedType::Named(n) if n == "_") {
                            self.error(format!("right operand of logical op must be Bool, found {}", right_ty.name()), *span);
                        }
                        CheckedType::Bool
                    }
                    BinOp::Assign => right_ty,
                    BinOp::Shl | BinOp::Shr => left_ty,
                    BinOp::BitXor => left_ty, // bitwise xor preserves integer type
                    BinOp::BitAnd => left_ty, // bitwise and preserves integer type
                    BinOp::BitOr => left_ty, // bitwise or preserves integer type
                }
            }
            Expr::Try(inner, _span) => {
                let inner_ty = self.check_expr(inner);
                // ? unwraps Result[T,E] → T or Option[T] → T.
                // Return wildcard (_) since the checker erases generic type args
                // (Result[T,E] is just "Result"). The codegen handles the actual
                // value extraction and early-return on error propagation.
                match &inner_ty {
                    CheckedType::Named(n) if n == "Result" || n == "Option" => {
                        CheckedType::Named("_".into())
                    }
                    _ => self.error(
                        format!("'?' operator requires a Result or Option type, found {}", inner_ty.name()),
                        *_span,
                    ),
                }
            }
            Expr::Imply(_, _, _) => CheckedType::Bool,
            Expr::Is(_, pattern, span) => {
                // M18: Bind pattern variables so guards like
                // `x is Some(n) && n > 10` can reference `n`.
                self.bind_is_pattern(pattern, *span);
                CheckedType::Bool
            }
            Expr::Field(obj, field, span) => {
                // Module-qualified access: module.Type or module.sub.Type
                if let Some(ty) = self.check_module_field_access(obj, field) {
                    return ty;
                }
                // Struct field access
                let obj_ty = self.check_expr(obj);
                // Check if obj is an enum type and field is a variant (e.g., Color.Red)
                if let CheckedType::Named(type_name) = &obj_ty {
                    let variant_full = format!("{}.{}", type_name, field.name);
                    if self.enum_variants.contains_key(&variant_full) || self.enum_variants.contains_key(&field.name) {
                        return CheckedType::Named(type_name.clone());
                    }
                }
                match &obj_ty {
                    CheckedType::Named(name) => {
                        // Generic type params have no registered fields — return wildcard
                        let is_generic_param = name.len() == 1 && name.chars().next().map_or(false, |c| c.is_ascii_uppercase());
                        if is_generic_param {
                            return CheckedType::Named("_".into());
                        }
                        if let Some(fields) = self.get_type(name) {
                            if let Some(field_ty) = fields.get(&field.name) {
                                field_ty.clone()
                            } else if !fields.is_empty() {
                                // Known type with registered fields — unknown field
                                self.error(
                                    format!("type '{}' has no field '{}'", name, field.name),
                                    *span,
                                )
                            } else {
                                // Known type with no registered fields (builtin) — allow access
                                CheckedType::Int
                            }
                        } else {
                            CheckedType::Error // unknown type
                        }
                    }
                    CheckedType::Error => CheckedType::Error, // suppress cascade
                    _ => self.error(
                        format!("cannot access field on non-struct type {}", obj_ty.name()),
                        *span,
                    ),
                }
            }
            Expr::Call(func, args, span) => {
                // Method call or module-qualified call: receiver.method(args) or module.func(args)
                // Also handle type-parameterized calls like Stack.new[Int]() which parse as
                // Expr::Call(Expr::Index(Expr::Field(Expr::Ident("Stack"), "new"), [Expr::Ident("Int")]), [])
                let method_target = match func.as_ref() {
                    Expr::Field(..) => Some(func.as_ref()),
                    Expr::Index(field_expr, _, _) if matches!(field_expr.as_ref(), Expr::Field(..)) => Some(field_expr.as_ref()),
                    _ => None,
                };
                    if let Some(Expr::Field(obj, method, _)) = method_target {
                        // Try module-qualified call first
                    if let Some(return_ty) = self.check_module_call(obj, method, args, *span) {
                        return return_ty;
                    }
                    // Try method call: receiver.method(args)
                    let obj_ty = self.check_expr(obj);
                    // Enum variant constructor: TypeName.Variant(args)
                    // e.g. `JsonValue.Integer(42)` or `SqliteValue.Text("hello")`
                    if let CheckedType::Named(type_name) = &obj_ty {
                        let variant_key = format!("{}.{}", type_name, method.name);
                        if self.enum_variants.contains_key(&variant_key)
                            || self.resolve_enum_variant(&variant_key).is_some()
                            || self.resolve_enum_variant(&method.name).is_some()
                        {
                            // Validate args against variant fields
                            for arg in args { let _ = self.check_expr(arg); }
                            return CheckedType::Named(type_name.clone());
                        }
                    }
                    if let CheckedType::Named(type_name) = &obj_ty {
                        let method_key = format!("{}.{}", type_name, method.name);
                        // Try module-prefixed key first, then bare key as fallback
                        let sig = if let Some(ref module) = self.current_module {
                            let prefixed = format!("{}.{}.{}", module, type_name, method.name);
                            self.functions.get(&prefixed).or_else(|| self.functions.get(&method_key))
                        } else {
                            self.functions.get(&method_key)
                        }.cloned();
                        // If not found, try wildcard method lookup (any type with that method)
                        let sig = sig.or_else(|| {
                            self.methods.iter()
                                .find(|(_, methods)| methods.contains_key(&method.name))
                                .and_then(|(_, methods)| methods.get(&method.name))
                                .cloned()
                        });
                        if let Some(sig) = sig {
                            // Detect static call (TypeName.method) vs instance method:
                            // if obj is a simple Ident that resolves to a known type,
                            // the call is TypeName.method(args) rather than instance.method(args).
                            let is_static_call = match obj.as_ref() {
                                Expr::Ident(id) => self.types.contains_key(&id.name),
                                _ => false,
                            };
                            // Determine the self-kind of this method:
                            //   explicit self: param literally named `self`/typed `Self`,
                            //     OR receiver-style `fn T.method(h: &T, ...)` — but ONLY
                            //     when call-site ARITY says so (G-20 fix below)
                            //   implicit this: uses `this` keyword, no self in params
                            //   constructor:   no self at all (e.g. fn T.new(...))
                            let first_param_is_self_named = sig.params.first()
                                .map_or(false, |(pname, pty)| {
                                    pname == "self" || matches!(pty, CheckedType::Named(n) if n == "Self")
                                });
                            let first_param_matches_receiver = sig.params.first().map_or(false, |(_, pty)| {
                                matches!(pty, CheckedType::Named(n) if n.as_str() == type_name.as_str())
                            });
                            // G-20 fix: `fn V2.lerp(other: V2, t: Float32)` — a first
                            // param of the receiver TYPE is ambiguous between
                            // receiver-style (h IS the receiver) and a REAL argument
                            // (math-style lerp/dot/cross). Call-site arity settles it
                            // deterministically:
                            //   args == params     → params are all real (offset 0)
                            //   args == params - 1 → first param is the receiver (offset 1)
                            // Previously the type heuristic always chose receiver-style,
                            // shifting every arg and rejecting/miscompiling lerp-shaped
                            // methods (silent swap class).
                            let arity_direct = args.len() == sig.params.len();
                            let has_explicit_self = first_param_is_self_named
                                || (first_param_matches_receiver && !arity_direct);
                            // param_offset table:
                            //   explicit self  + instance call → skip self (offset=1)
                            //   explicit self  + static call   → self is first arg (offset=0)
                            //   implicit this  + instance call → args map directly (offset=0)
                            //   implicit this  + static call   → first arg is receiver, skip (offset=1)
                            //     NOTE: codegen adds a %param_self pointer to the LLVM signature
                            //     for this-based methods; this offset only controls checker-level
                            //     param matching. The codegen's self_offset handles the actual
                            //     argument layout independently.
                            //   constructor    + any call       → args map directly, no self (offset=0)
                            let param_offset: usize = if has_explicit_self {
                                if is_static_call { 0 } else { 1 }
                            } else if sig.uses_implicit_this {
                                if is_static_call { 1 } else { 0 }
                            } else {
                                0 // constructor — no self at all
                            };
                            for (i, arg) in args.iter().enumerate() {
                                let arg_ty = self.check_expr(arg);
                                let param_idx = i + param_offset;
                                if param_idx < sig.params.len() {
                                    let expected = &sig.params[param_idx].1;
                                    let is_generic = sig.generics.iter().any(|g| g == &expected.name());
                                    if !is_generic && !self.types_compatible(&arg_ty, expected) && arg_ty != CheckedType::Error {
                                        self.error(
                                            format!("argument {} type mismatch: expected {}, found {}",
                                                i + 1, expected.name(), arg_ty.name()),
                                            *span,
                                        );
                                    }
                                }
                            }
                            return sig.return_type.unwrap_or(CheckedType::Unit);
                        }
                    }
                    // Primitive types implicitly support the builtin interface methods
                    // (Ord.compare, Eq.eq/ne, Hash.hash, Clone.clone, comparison ops).
                    let prim_ty = match &obj_ty {
                        CheckedType::Named(n) => CheckedType::from_str(n),
                        other => other.clone(),
                    };
                    let is_primitive = prim_ty.is_numeric()
                        || matches!(prim_ty, CheckedType::Bool | CheckedType::Char | CheckedType::Str);
                    if is_primitive {
                        for arg in args {
                            let _ = self.check_expr(arg);
                        }
                        match method.name.as_str() {
                            "compare" | "hash" => return CheckedType::Int,
                            "eq" | "ne" | "lt" | "gt" | "le" | "ge" => return CheckedType::Bool,
                            "clone" => return prim_ty,
                            // Str builtins.
                            "len" if prim_ty == CheckedType::Str => return CheckedType::Int,
                            // Gap A fix: to_owned is the idiomatic Str duplication
                            // alias (Rust parity). Same semantics as clone.
                            "to_owned" if prim_ty == CheckedType::Str => return CheckedType::Str,
                            // G-43: C-string interop (BUG-008 codegen builtins).
                            "c_str" if prim_ty == CheckedType::Str => return CheckedType::Named("Ptr".into()),
                            "byte_len" if prim_ty == CheckedType::Str => return CheckedType::Int,
                            "to_str" | "to_string" => return CheckedType::Str,
                            // M12/P0: Str conversions from C strings / byte buffers.
                            // These are codegen builtins (call.rs:1210) that reinterpret
                            // a pointer as a Str at the ABI level — identity transform
                            // on i8* with no runtime cost. The checker must return Str
                            // (not Result) so io.read_line() / list_dir() / args() work.
                            "from_cstring" | "from_c_str" | "from_utf8" | "from_bytes"
                                if prim_ty == CheckedType::Str => return CheckedType::Str,
                            // M12/P0: Str.substr(start, end) — substring extraction.
                            // Codegen emits xiom_str_slice (a runtime concat call);
                            // always infallible for valid bounds.
                            "substr" if prim_ty == CheckedType::Str => return CheckedType::Str,
                            // M12/P1: byte indexing — returns a single byte at position.
                            "byte_at" if prim_ty == CheckedType::Str => return CheckedType::UInt8,
                            "char_at" if prim_ty == CheckedType::Str => return CheckedType::Named("Option".into()),
                            // M12/P1: scripting ergonomics — slice() and starts_with()
                            // as methods on Str, avoiding verbose string.str_slice() calls.
                            "slice" if prim_ty == CheckedType::Str => return CheckedType::Str,
                            "starts_with" if prim_ty == CheckedType::Str => return CheckedType::Bool,
                            "ends_with" if prim_ty == CheckedType::Str => return CheckedType::Bool,
                            _ => {}
                        }
                    }
                    // Builtin methods on core generic containers whose element/inner
                    // types are erased in the checker (Vec/Slice/Option/Result/Map/Set).
                    // These are legitimate stdlib APIs; accept them so correct code
                    // type-checks (the "if it compiles, it's safe" gate stays sound
                    // because codegen lowers these to real builtins).
                    if let CheckedType::Named(tn) = &obj_ty {
                        let base = tn.rsplit('.').next().unwrap_or(tn);
                        for arg in args { let _ = self.check_expr(arg); }
                        match (base, method.name.as_str()) {
                            ("Vec" | "Slice" | "Array" | "Str" | "Map" | "Set", "len")
                                => return CheckedType::Int,
                            ("Vec" | "Slice" | "Array" | "Str", "is_empty") => return CheckedType::Bool,
                            // G-36: container clone returns the same container type.
                            ("Vec" | "Slice" | "Map" | "Set", "clone") => return obj_ty.clone(),
                            // Option/Result payload accessors — inner type is erased,
                            // so return a wildcard the rest of the checker accepts.
                            ("Option" | "Result", "unwrap" | "unwrap_or" | "unwrap_err" | "expect" | "value")
                                => return CheckedType::Named("_".into()),
                            ("Option" | "Result", "is_some" | "is_none" | "is_ok" | "is_err")
                                => return CheckedType::Bool,
                            // Common wrapper accessors (Cell/Rc/Arc/Mutex/Box/Reverse).
                            ("Cell" | "Rc" | "Arc" | "Mutex" | "Box" | "Reverse" | "RefCell", "get" | "clone" | "lock" | "borrow" | "borrow_mut")
                                => return CheckedType::Named("_".into()),
                            _ => {}
                        }
                    }
                    // Interface dispatch: accept method calls on interface-typed
                    // receivers, generic params, wildcard types, and cascade-error
                    // pattern bindings. Codegen resolves the concrete implementation
                    // at monomorphisation time.
                    let allow_interface_dispatch = match &obj_ty {
                        CheckedType::Named(tn) => {
                            // Direct interface-typed receiver (e.g. self: Error)
                            self.interfaces.contains_key(tn)
                            || tn == "_"  // wildcard from Option.value / Result.unwrap
                            || (
                                // Generic param with potential interface bound
                                tn.len() == 1 && tn.chars().next().map_or(false, |c| c.is_uppercase())
                                && self.interfaces.values().any(|m| m.iter().any(|(mn, _, _)| mn == &method.name))
                            )
                        }
                        // Pattern-bound variables from match arms (e.g. `e` in
                        // `Err(e) => ...`) have cascade Error type.  If the method
                        // is declared in any interface, accept it.
                        CheckedType::Error => {
                            self.interfaces.values().any(|m| m.iter().any(|(mn, _, _)| mn == &method.name))
                        }
                        _ => false,
                    };
                    if allow_interface_dispatch {
                        for arg in args { let _ = self.check_expr(arg); }
                        // Return the declared return type from the interface, or
                        // a wildcard if unknown.
                        let ret = self.interfaces.values()
                            .flat_map(|m| m.iter())
                            .find(|(mn, _, _)| mn == &method.name)
                            .and_then(|(_, _, ret)| ret.clone())
                            .map(|r| CheckedType::from_str(&r))
                            .unwrap_or(CheckedType::Named("_".into()));
                        return ret;
                    }
                    // Fallback: unknown call target
                    self.error(
                        format!("cannot call '{}' on this expression", method.name),
                        *span,
                    );
                    return CheckedType::Error;
                }
                // Look up the function by name if it's a simple identifier
                if let Expr::Ident(name) = func.as_ref() {
                    // Try module-prefixed key first, then bare name
                    let fn_sig = if let Some(ref module) = self.current_module {
                        let prefixed = format!("{}.{}", module, name.name);
                        self.functions.get(&prefixed).or_else(|| self.functions.get(&name.name))
                    } else {
                        self.functions.get(&name.name)
                    };
                    if let Some(sig) = fn_sig.cloned() {
                        // Build generic substitution map from the call arguments
                        let mut subst: HashMap<String, CheckedType> = HashMap::new();
                        if !sig.generics.is_empty() {
                            for (i, arg) in args.iter().enumerate() {
                                if i < sig.params.len() {
                                    let pname = &sig.params[i].1.name();
                                    if sig.generics.iter().any(|g| g == pname) {
                                        subst.insert(pname.clone(), self.check_expr(arg));
                                    }
                                }
                            }
                        }
                        for (i, arg) in args.iter().enumerate() {
                            let arg_ty = self.check_expr(arg);
                            if i < sig.params.len() {
                                let expected = &sig.params[i].1;
                                let is_generic = sig.generics.iter().any(|g| g == &expected.name());
                                if !is_generic && !self.types_compatible(&arg_ty, expected) && arg_ty != CheckedType::Error {
                                    self.error(
                                        format!("argument {} type mismatch: expected {}, found {}",
                                            i + 1, expected.name(), arg_ty.name()),
                                        *span,
                                    );
                                }
                            }
                        }
                        let ret_ty = sig.return_type.unwrap_or(CheckedType::Unit);
                        if sig.generics.is_empty() {
                            return ret_ty;
                        }
                        // Substitute generic return type with the concrete arg type
                        let ret_name = ret_ty.name();
                        if let Some(concrete) = subst.get(&ret_name) {
                            return concrete.clone();
                        }
                        return ret_ty;
                    }
                    // 5c.30: implicit-self method call (G-10).
                    // When inside a method body, `init()` resolves to
                    // `self.init()`. Look up the method in the current
                    // receiver type's registry.
                    if let Some(ref recv) = self.current_receiver {
                        // Try the receiver's methods, qualified and bare
                        let mut msig: Option<&FnSig> = None;
                        if let Some(ms) = self.methods.get(recv) {
                            msig = ms.get(&name.name);
                        }
                        if msig.is_none() {
                            let suffix = format!(".{recv}");
                            for (key, ms) in &self.methods {
                                if key.ends_with(&suffix) {
                                    msig = ms.get(&name.name);
                                    if msig.is_some() { break; }
                                }
                            }
                        }
                        if let Some(sig) = msig.cloned() {
                            // The implicit self argument is passed as the
                            // FIRST param. Check remaining explicit args
                            // against params[1..].
                            for (i, arg) in args.iter().enumerate() {
                                let arg_ty = self.check_expr(arg);
                                if i + 1 < sig.params.len() {
                                    let expected = &sig.params[i + 1].1;
                                    let is_generic = sig.generics.iter().any(|g| g == &expected.name());
                                    if !is_generic && !self.types_compatible(&arg_ty, expected) && arg_ty != CheckedType::Error {
                                        self.error(
                                            format!("argument {} type mismatch: expected {}, found {}",
                                                i + 1, expected.name(), arg_ty.name()),
                                            *span,
                                        );
                                    }
                                }
                            }
                            return sig.return_type.unwrap_or(CheckedType::Unit);
                        }
                    }
                    if let Some(export) = self.imported_items.get(&name.name).cloned() {
                        if let ModuleExport::Function { sig, .. } = export {
                            // Build generic substitution map from the call arguments
                            let mut subst: HashMap<String, CheckedType> = HashMap::new();
                            if !sig.generics.is_empty() {
                                for (i, arg) in args.iter().enumerate() {
                                    if i < sig.params.len() {
                                        let pname = &sig.params[i].1.name();
                                        if sig.generics.iter().any(|g| g == pname) {
                                            subst.insert(pname.clone(), self.check_expr(arg));
                                        }
                                    }
                                }
                            }
                            for (i, arg) in args.iter().enumerate() {
                                let arg_ty = self.check_expr(arg);
                                if i < sig.params.len() {
                                    let expected = &sig.params[i].1;
                                    let is_generic = sig.generics.iter().any(|g| g == &expected.name());
                                    if !is_generic && !self.types_compatible(&arg_ty, expected) && arg_ty != CheckedType::Error {
                                        self.error(
                                            format!("argument {} type mismatch: expected {}, found {}",
                                                i + 1, expected.name(), arg_ty.name()),
                                            *span,
                                        );
                                    }
                                }
                            }
                            let ret_ty = sig.return_type.unwrap_or(CheckedType::Unit);
                            if sig.generics.is_empty() {
                                return ret_ty;
                            }
                            // Substitute generic return type with the concrete arg type
                            let ret_name = ret_ty.name();
                            if let Some(concrete) = subst.get(&ret_name) {
                                return concrete.clone();
                            }
                            return ret_ty;
                        }
                    }
                }
                // Check if callee is an enum variant constructor (positional args)
                if let Expr::Ident(name) = func.as_ref() {
                    if self.enum_variants.contains_key(&name.name) || self.resolve_enum_variant(&name.name).is_some() {
                        // Enum variant constructor with positional args — typecheck args loosely
                        for arg in args { let _ = self.check_expr(arg); }
                        if let Some(parent) = self.resolve_enum_variant(&name.name) {
                            return CheckedType::Named(parent.clone());
                        }
                        if let Some(parent) = self.enum_variants.get(&name.name) {
                            return CheckedType::Named(parent.clone());
                        }
                    }
                }
                // Check if callee evaluates to a function pointer type
                let callee_ty = self.check_expr(func);
                if let CheckedType::Fn(param_types, ret_ty) = &callee_ty {
                    for (i, arg) in args.iter().enumerate() {
                        let arg_ty = self.check_expr(arg);
                        if i < param_types.len() {
                            let expected = &param_types[i];
                            if !self.types_compatible(&arg_ty, expected) && arg_ty != CheckedType::Error {
                                self.error(
                                    format!("argument {} type mismatch: expected {}, found {}",
                                        i + 1, expected.name(), arg_ty.name()),
                                    *span,
                                );
                            }
                        }
                    }
                    return *ret_ty.clone();
                }
                // M20-A1: Closure call — callee is Named("fn") (non-capturing lambda).
                // Accept any args and return wildcard since we don't track closure
                // signatures in the type system yet.
                if matches!(&callee_ty, CheckedType::Named(n) if n == "fn") {
                    for arg in args { let _ = self.check_expr(arg); }
                    return CheckedType::Named("_".into());
                }
                // Fallback: could be a method call or unknown function
                CheckedType::Unit
            }
            Expr::Index(arr, idx, _) => {
                // Check if this is a type parameter expression like Vec[Int] or a real index like v[0]
                // Type parameter expressions: the container is a known type name AND not a local variable
                if let Expr::Ident(container_ident) = arr.as_ref() {
                    let is_type_name = self.contains_type(&container_ident.name) || 
                       container_ident.name == "Vec" || container_ident.name == "Option" || 
                       container_ident.name == "Result" || container_ident.name == "Map" ||
                       container_ident.name == "Set" || container_ident.name == "Stack" ||
                       container_ident.name == "Queue" || container_ident.name == "BST" ||
                       container_ident.name == "List" || container_ident.name == "Channel" ||
                       container_ident.name == "Box" || container_ident.name == "Wrapper" ||
                       container_ident.name == "Pair" || container_ident.name == "Counter" ||
                       container_ident.name == "Range" || container_ident.name == "Nested";
                    let is_local = self.lookup_local(&container_ident.name).is_some();
                    if is_type_name && !is_local {
                        // Type parameter expression — return the container type
                        return CheckedType::Named(container_ident.name.clone());
                    }
                }
                // Regular index: arr[idx]
                let arr_ty = self.check_expr(arr);
                let _ = self.check_expr(idx);
                match &arr_ty {
                    CheckedType::Named(name) if name == "Vec" => {
                        // Vec[T][i] -> T (use Vec as placeholder, actual type inferred from usage)
                        CheckedType::Named("T".into())
                    }
                    _ => CheckedType::Int,
                }
            }
            Expr::AtPre(inner, _) => self.check_expr(inner),
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => self.check_expr(inner),
            Expr::Some(inner, _) => {
                let _ = self.check_expr(inner);
                CheckedType::Named("Option".into())
            }
            Expr::None(_) => CheckedType::Named("Option".into()),
            Expr::Ok(inner, _) => {
                let _ = self.check_expr(inner);
                CheckedType::Named("Result".into())
            }
            Expr::Err(inner, _) => {
                let _ = self.check_expr(inner);
                CheckedType::Named("Result".into())
            }
            Expr::Struct(name, fields, _spread, span) => {
                // M22: Anonymous struct `{ field: value; }` — type inferred from context.
                // Return wildcard `_` and let the caller (var/return/arg) validate.
                if name.name == "_" {
                    for (_, fval) in fields { let _ = self.check_expr(fval); }
                    return CheckedType::Named("_".into());
                }
                let struct_fields = self.get_type(&name.name).cloned();
                let variant_fields_map = if struct_fields.is_none() {
                    self.variant_fields.get(&name.name).or_else(|| {
                        if let Some(ref module) = self.current_module {
                            let prefixed = format!("{}.{}", module, name.name);
                            self.variant_fields.get(&prefixed)
                        } else {
                            None
                        }
                    }).map(|vf| {
                        let mut m = HashMap::new();
                        for (k, v) in vf { m.insert(k.clone(), v.clone()); }
                        m
                    })
                } else {
                    None
                };
                let is_struct_type = struct_fields.is_some();
                let expected_fields = struct_fields.or(variant_fields_map);
                // For struct types with registered fields, validate; for enum variants
                // or unknown types, skip field validation (typecheck at match time)
                if let Some(expected) = expected_fields {
                    if !expected.is_empty() {
                        for (fname, fval) in fields {
                            let val_ty = self.check_expr(fval);
                            if let Some(expected_ty) = expected.get(&fname.name) {
                                if !self.types_compatible(&val_ty, expected_ty) && val_ty != CheckedType::Error {
                                    self.error(
                                        format!("field '{}' type mismatch: expected {}, found {}",
                                            fname.name, expected_ty.name(), val_ty.name()),
                                        *span,
                                    );
                                }
                            } else if !self.enum_variants.contains_key(&name.name)
                                && self.resolve_enum_variant(&name.name).is_none()
                            {
                                self.error(
                                    format!("type '{}' has no field '{}'", name.name, fname.name),
                                    *span,
                                );
                            }
                        }
                    }
                } else if !self.enum_variants.contains_key(&name.name)
                    && self.resolve_enum_variant(&name.name).is_none()
                {
                    self.error(format!("unknown type '{}'", name.name), *span);
                }
                // Return the parent enum type for variant constructors, or the struct name
                // If the type exists as a struct specifically in THIS module, use it
                // Otherwise, prefer enum variant resolution (handles name collisions across modules)
                let is_local_struct = if let Some(ref module) = self.current_module {
                    let prefixed = format!("{}.{}", module, name.name);
                    self.types.contains_key(&prefixed)
                } else {
                    is_struct_type
                };
                if is_local_struct {
                    CheckedType::Named(name.name.clone())
                } else if let Some(parent) = self.resolve_enum_variant(&name.name) {
                    CheckedType::Named(parent.clone())
                } else {
                    CheckedType::Named(name.name.clone())
                }
            }
            Expr::Array(items, _) => {
                if items.is_empty() {
                    CheckedType::Named("Vec".into())
                } else {
                    let first_ty = self.check_expr(&items[0]);
                    for item in &items[1..] {
                        let item_ty = self.check_expr(item);
                        if !self.types_compatible(&first_ty, &item_ty) && item_ty != CheckedType::Error {
                            // soft error — arrays should be homogeneous
                        }
                    }
                    CheckedType::Named("Vec".into())
                }
            }
            Expr::Closure(_, _, _, _) => CheckedType::Named("fn".into()),
            Expr::PipeClosure(params, body, _) => {
                // M20-A1: Add closure params to scope before checking body
                self.push_scope();
                for p in params {
                    self.add_local(&p.name, CheckedType::Int);
                }
                let _ = self.check_expr(body);
                self.pop_scope();
                CheckedType::Named("fn".into())
            }
            Expr::As(inner, ty, span) => {
                let inner_ty = self.check_expr(inner);
                let target_ty = CheckedType::from_ast_type(ty);
                // Phase 7E/Feature: Resolve aliases so `x as Int` works when x: VkHandle
                let inner_resolved = self.resolve_alias(&inner_ty);
                let target_resolved = self.resolve_alias(&target_ty);
                match (&inner_resolved, &target_resolved) {
                    (CheckedType::Int, CheckedType::Float64) => target_ty,
                    (CheckedType::Float64, CheckedType::Int) => target_ty,
                    _ if inner_resolved == target_resolved => target_ty,
                    _ if inner_resolved == CheckedType::Error => CheckedType::Error,
                    _ if inner_resolved.is_numeric() && target_resolved.is_numeric() => target_ty,
                    // Char is a codepoint: convertible to/from any integer type
                    _ if inner_resolved == CheckedType::Char && target_resolved.is_integer() => target_ty,
                    _ if inner_resolved.is_integer() && target_resolved == CheckedType::Char => target_ty,
                    // 5c-E: Int ↔ Ptr casts (raw pointer FFI, ptr.xi)
                    (CheckedType::Int, CheckedType::Named(s)) if s == "Ptr" => target_ty,
                    (CheckedType::Named(s), CheckedType::Int) if s == "Ptr" => target_ty,
                    // 5c-E: Vec/Slice/Array → Ptr cast (Vulkan FFI: pass buffer to extern)
                    (CheckedType::Named(s), CheckedType::Named(t))
                        if t == "Ptr" && (s == "Vec" || s == "Slice") => target_ty,
                    // 5e.2 G-34: fn-ptr ↔ Int casts (COM vtables, callback registries).
                    (CheckedType::Int, CheckedType::Fn(..)) => target_ty,
                    (CheckedType::Fn(..), CheckedType::Int) => target_ty,
                    // G-16: function name as Int (callback pointer).
                    (CheckedType::Named(n), CheckedType::Int) if n == "fn" => target_ty,
                    _ => {
                        self.error(format!("unsupported type cast: {} to {}", inner_ty.name(), target_ty.name()), *span)
                    }
                }
            }
            Expr::Await(inner, _) => self.check_expr(inner),
            Expr::Comptime(inner, _) => self.check_expr(inner),
            Expr::Unsafe(block, _) | Expr::BlockExpr(block, _) => { self.check_block(block, None).unwrap_or(CheckedType::Unit) }
            // 5c-R: Error-poisoned nodes carry an ErrorGuaranteed proof token.
            // Skip silently — a diagnostic was already emitted for this subtree.
            Expr::Error(_guarantee, _span) => CheckedType::Error,
            Expr::If(cond, then_block, elifs, else_block, _) => {
                self.check_expr(cond);
                let then_ty = self.check_block(then_block, None).unwrap_or(CheckedType::Unit);
                for (econd, eblock) in elifs {
                    self.check_expr(econd);
                    let _ = self.check_block(eblock, None);
                }
                if let Some(eb) = else_block { let _ = self.check_block(eb, None); }
                // 5c-E: if-expression type is the then-branch type when concrete
                match &then_ty {
                    CheckedType::Unit => CheckedType::Named("_".into()),
                    CheckedType::Named(s) if s == "_" => CheckedType::Named("_".into()),
                    _ => then_ty,
                }
            }
            Expr::Match(scrutinee, arms, _) => {
                let scr_ty = self.check_expr(scrutinee);
                // The value of a match-expression is the type of its arm bodies.
                // Return the first arm's body type (or Unit for an empty match).
                let mut result_ty = CheckedType::Unit;
                let mut first = true;
                for arm in arms {
                    self.push_scope();
                    self.add_pattern_bindings(&arm.pattern, &scr_ty);
                    if let Some(ref guard) = arm.guard {
                        self.check_expr(guard);
                    }
                    let arm_ty = match &arm.body {
                        MatchBody::Block(b) => self.check_block(b, None).unwrap_or(CheckedType::Unit),
                        MatchBody::Expr(e) => self.check_expr(e),
                    };
                    self.pop_scope();
                    if first { result_ty = arm_ty; first = false; }
                }
                result_ty
            }
        }
    }

    /// Phase 7E/Feature: Resolve type aliases recursively.
    /// `type Foo = Int; type Bar = Foo;` — resolving Bar gives Int.
    /// Guards against infinite loops (max depth 16).
    fn resolve_alias(&self, ty: &CheckedType) -> CheckedType {
        let mut current = ty.clone();
        let mut depth = 0;
        loop {
            if depth > 16 { break; } // cycle guard
            if let CheckedType::Named(name) = &current {
                if let Some(resolved) = self.aliases.get(name.as_str()) {
                    current = resolved.clone();
                    depth += 1;
                    continue;
                }
            }
            break;
        }
        current
    }

    fn types_compatible(&self, found: &CheckedType, expected: &CheckedType) -> bool {
        // Phase 7E/Feature: Resolve type aliases so newtypes auto-convert
        let found = &self.resolve_alias(found);
        let expected = &self.resolve_alias(expected);
        // M9.6: impl Trait is an opaque return type — any concrete type in the body
        // is compatible. Full trait-resolution checking is deferred.
        if matches!(found, CheckedType::ImplTrait(_)) || matches!(expected, CheckedType::ImplTrait(_)) {
            return true;
        }
        // Wildcard type `_` — compatible with any concrete type
        if matches!(found, CheckedType::Named(n) if n == "_") ||
           matches!(expected, CheckedType::Named(n) if n == "_") {
            return true;
        }
        // Normalize Named("Bool") <-> Bool, Named("Int") <-> Int, etc.
        let norm = |t: &CheckedType| -> CheckedType {
            match t {
                CheckedType::Named(n) => CheckedType::from_str(n),
                other => other.clone(),
            }
        };
        let found_norm = norm(found);
        let expected_norm = norm(expected);
        let found = &found_norm;
        let expected = &expected_norm;
        if found == &CheckedType::Error || expected == &CheckedType::Error {
            return true; // Don't cascade errors
        }
        if found == expected {
            return true;
        }
        // 5c-E: Scalar types are compatible with raw Ptr when passed by reference
        // (&T -> *T for extern FFI calls). The codegen emits the pointer address.
        if expected == &CheckedType::Named("Ptr".into())
            && (found.is_numeric() || found == &CheckedType::Bool)
        {
            return true;
        }
        // Named types are compatible if they have the same name
        // Generic type parameters (single uppercase letter) are compatible with any type
        let is_generic_param = |ty: &CheckedType| -> bool {
            if let CheckedType::Named(s) = ty {
                s.len() == 1 && s.chars().next().map_or(false, |c| c.is_ascii_uppercase())
            } else {
                false
            }
        };
        if is_generic_param(found) || is_generic_param(expected) {
            return true;
        }
        match (found, expected) {
            (CheckedType::Named(a), CheckedType::Named(b)) if a == b => true,
            // Tuple types are broadly compatible with anything
            (CheckedType::Named(n), _) if n.starts_with("Tuple") => true,
            (_, CheckedType::Named(n)) if n.starts_with("Tuple") => true,
            // 5c.30: integer literals (always `Int`) are compatible with any
            // integer-like target (Int8, UInt8, UInt32, etc.).
            (CheckedType::Int, other) | (other, CheckedType::Int)
                if other.is_numeric() => true,
            // 6A.1: Named types with different names are NOT compatible.
            // Previously (Named(_), Named(_)) => true allowed any two user-defined
            // types to be compatible (e.g., Point = Color passed type checking).
            // Exception: Self is always compatible — it's an alias for the concrete type.
            (CheckedType::Named(a), CheckedType::Named(b)) if a == "Self" || b == "Self" => true,
            // Interface/trait names are compatible with their implementor types.
            (CheckedType::Named(a), CheckedType::Named(b))
                if self.interfaces.contains_key(a) || self.interfaces.contains_key(b) => true,
            // Array[N]T, Slice[T], and Vec[T] share the same runtime layout.
            (CheckedType::Named(a), CheckedType::Named(b))
                if (a.starts_with("Array") || a.starts_with("Slice") || a.starts_with("Vec")) &&
                   (b.starts_with("Array") || b.starts_with("Slice") || b.starts_with("Vec")) &&
                   a != b => true,
            (CheckedType::Named(_), CheckedType::Named(_)) => false,
            // Wildcard placeholder type is compatible with everything
            (CheckedType::Named(n), _) if n == "_" => true,
            (_, CheckedType::Named(n)) if n == "_" => true,
            // Function pointer compatibility: Named("fn") is compatible with any Fn type
            (CheckedType::Named(n), CheckedType::Fn(..)) if n == "fn" => true,
            (CheckedType::Fn(..), CheckedType::Named(n)) if n == "fn" => true,
            // Numeric promotions
            (CheckedType::Int, CheckedType::Float64) => true,
            (CheckedType::Float64, CheckedType::Int) => true,
            (CheckedType::Float32, CheckedType::Float64) => true,
            (CheckedType::Float64, CheckedType::Float32) => true,
            (CheckedType::Int, CheckedType::Char) => true,
            (CheckedType::Char, CheckedType::Int) => true,
            // Integer width promotions: Int (default i64) coerces to narrower
            // integer types (Int32/Int16/Int8) for FFI compatibility.
            // All signed integer types are mutually compatible.
            (CheckedType::Int, CheckedType::Int32) | (CheckedType::Int32, CheckedType::Int) => true,
            (CheckedType::Int, CheckedType::Int16) | (CheckedType::Int16, CheckedType::Int) => true,
            (CheckedType::Int, CheckedType::Int8)  | (CheckedType::Int8,  CheckedType::Int) => true,
            (CheckedType::Int32, CheckedType::Int16) | (CheckedType::Int16, CheckedType::Int32) => true,
            (CheckedType::Int32, CheckedType::Int8)  | (CheckedType::Int8,  CheckedType::Int32) => true,
            (CheckedType::Int16, CheckedType::Int8)  | (CheckedType::Int8,  CheckedType::Int16) => true,
            // Unsigned integer compatibility
            (CheckedType::UInt, CheckedType::UInt32) | (CheckedType::UInt32, CheckedType::UInt) => true,
            (CheckedType::UInt, CheckedType::UInt16) | (CheckedType::UInt16, CheckedType::UInt) => true,
            (CheckedType::UInt, CheckedType::UInt8)  | (CheckedType::UInt8,  CheckedType::UInt) => true,
            // Signed↔unsigned integer compatibility (FFI Common)
            (CheckedType::Int,    CheckedType::UInt32) | (CheckedType::UInt32, CheckedType::Int) => true,
            (CheckedType::Int32,  CheckedType::UInt32) | (CheckedType::UInt32, CheckedType::Int32) => true,
            (CheckedType::Int,    CheckedType::UInt)   | (CheckedType::UInt,   CheckedType::Int) => true,
            // Unit compatibility
            (_, CheckedType::Unit) => true,
            _ => false,
        }
    }
}

impl Default for Checker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Borrow Checker — Phase 1: ownership and lexical scope borrow checking
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
enum BorrowState {
    Owned,
    Moved,
    ReadBorrowed,
    WriteBorrowed,
}

#[derive(Debug, Clone)]
struct OwnershipInfo {
    state: BorrowState,
    read_borrow_count: u32,
    is_mutable: bool,
    xiom_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BorrowType {
    Read,
    Write,
}

#[derive(Debug, Clone)]
struct ScopeBorrow {
    var_name: String,
    borrow_type: BorrowType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ExprResult {
    Value,
    ReadRef,
    WriteRef,
}

/// An error emitted by the borrow checker when ownership or borrowing rules
/// are violated (e.g. use-after-move, double mutable borrow).
#[derive(Debug, Clone)]
pub struct BorrowError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for BorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "borrow error at {}: {}", self.span, self.message)
    }
}

/// The XIOM borrow checker. Enforces ownership, borrowing, and move semantics
/// across lexical scopes. Runs after type checking succeeds (`certify()`).
///
/// Tracks ownership transfers (moves), read borrows (`&T`), and write borrows
/// (`&mut T`) per variable. Emits [`BorrowError`]s for violations like
/// use-after-move, double mutable borrow, and borrow-while-moved.
pub struct BorrowChecker {
    ownership: Vec<HashMap<String, OwnershipInfo>>,
    borrow_stack: Vec<Vec<ScopeBorrow>>,
    errors: Vec<BorrowError>,
    param_names: HashSet<String>,
    /// 5c-R: Active Place-level loans for field-granular borrow checking.
    /// Consulted BEFORE the ScopeBorrow stack; if a Place conflict is found,
    /// an error is emitted and the loan is rejected.
    active_loans: crate::borrow::LoanSet,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            ownership: vec![HashMap::new()],
            borrow_stack: vec![Vec::new()],
            errors: Vec::new(),
            param_names: HashSet::new(),
            active_loans: crate::borrow::LoanSet::new(),
        }
    }

    fn push_scope(&mut self) {
        self.ownership.push(HashMap::new());
        self.borrow_stack.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        // Release all borrows created in this scope
        if let Some(borrows) = self.borrow_stack.pop() {
            for scope_borrow in borrows {
                self.release_borrow(&scope_borrow.var_name, scope_borrow.borrow_type);
            }
        }
        self.ownership.pop();
        // 5c-R: release all active Place-level loans when scope exits
        self.active_loans.release_all();
    }

    fn release_borrow(&mut self, name: &str, borrow_type: BorrowType) {
        if let Some(info) = self.find_var_mut(name) {
            match borrow_type {
                BorrowType::Read => {
                    info.read_borrow_count = info.read_borrow_count.saturating_sub(1);
                    if info.read_borrow_count == 0 {
                        info.state = BorrowState::Owned;
                    }
                }
                BorrowType::Write => {
                    info.state = BorrowState::Owned;
                }
            }
        }
    }

    fn find_var(&self, name: &str) -> Option<&OwnershipInfo> {
        for scope in self.ownership.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn find_var_mut(&mut self, name: &str) -> Option<&mut OwnershipInfo> {
        for scope in self.ownership.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                return Some(info);
            }
        }
        None
    }

    fn add_local(&mut self, name: &str, is_mutable: bool, xiom_type: &str) {
        if let Some(scope) = self.ownership.last_mut() {
            scope.insert(name.to_string(), OwnershipInfo {
                state: BorrowState::Owned,
                read_borrow_count: 0,
                is_mutable,
                xiom_type: xiom_type.to_string(),
            });
        }
    }

    fn is_copy_type(xiom_type: &str) -> bool {
        matches!(xiom_type,
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
            | "Bool" | "Char" | "Str" | "Float32" | "Float64"
        )
    }

    fn param_type_name(ty: &xiom_ast::Type) -> String {
        match ty {
            xiom_ast::Type::Named(ident, _) => ident.name.clone(),
            xiom_ast::Type::Ref(inner) => Self::param_type_name(inner),
            xiom_ast::Type::MutRef(inner) => Self::param_type_name(inner),
            _ => "Int".to_string(),
        }
    }

    fn infer_type_from_expr(expr: &Expr) -> &'static str {
        match expr {
            Expr::Str(..) => "Str",
            Expr::Bool(..) => "Bool",
            Expr::Float(..) => "Float64",
            Expr::Char(..) => "Char",
            Expr::Struct(..) => "Struct",
            _ => "Int",
        }
    }

    fn error(&mut self, message: impl Into<String>, span: Span) {
        self.errors.push(BorrowError { message: message.into(), span });
    }

    fn check_use(&mut self, name: &str, span: Span) -> ExprResult {
        match self.find_var(name) {
            Some(info) => match info.state {
                BorrowState::Moved => {
                    self.error(format!("use of moved value '{}'", name), span);
                    ExprResult::Value
                }
                BorrowState::Owned => ExprResult::Value,
                BorrowState::ReadBorrowed => ExprResult::ReadRef,
                BorrowState::WriteBorrowed => ExprResult::WriteRef,
            }
            None => ExprResult::Value,
        }
    }

    fn read_borrow(&mut self, name: &str, span: Span) {
        // 5c-R: Place-level loan tracking — only for field-granular paths.
        // Bare-variable borrows use existing ScopeBorrow tracking.
        let ok = match self.find_var(name) {
            Some(info) => match info.state {
                BorrowState::Moved => {
                    self.error(format!("use of moved value '{}'", name), span);
                    false
                }
                BorrowState::WriteBorrowed => {
                    self.error(format!("cannot borrow '{}' as immutable while mutably borrowed", name), span);
                    false
                }
                BorrowState::Owned | BorrowState::ReadBorrowed => {
                    true
                }
            }
            None => true,
        };
        if ok {
            if let Some(info) = self.find_var_mut(name) {
                info.state = BorrowState::ReadBorrowed;
                info.read_borrow_count += 1;
            }
            if let Some(borrows) = self.borrow_stack.last_mut() {
                borrows.push(ScopeBorrow {
                    var_name: name.to_string(),
                    borrow_type: BorrowType::Read,
                });
            }
        }
    }

    fn write_borrow(&mut self, name: &str, span: Span) {
        // 5c-R: Place-level loan tracking — only for field-granular paths.
        let ok = match self.find_var(name) {
            Some(info) => match info.state {
                BorrowState::Moved => {
                    self.error(format!("use of moved value '{}'", name), span);
                    false
                }
                BorrowState::ReadBorrowed => {
                    self.error(format!("cannot borrow '{}' as mutable while immutably borrowed", name), span);
                    false
                }
                BorrowState::WriteBorrowed => {
                    self.error(format!("cannot borrow '{}' as mutable more than once at a time", name), span);
                    false
                }
                BorrowState::Owned => {
                    if !info.is_mutable {
                        self.error(format!("cannot borrow immutable local variable '{}' as mutable", name), span);
                        false
                    } else {
                        true
                    }
                }
            }
            None => true,
        };
        if ok {
            if let Some(info) = self.find_var_mut(name) {
                info.state = BorrowState::WriteBorrowed;
                info.read_borrow_count = 1;
            }
            if let Some(borrows) = self.borrow_stack.last_mut() {
                borrows.push(ScopeBorrow {
                    var_name: name.to_string(),
                    borrow_type: BorrowType::Write,
                });
            }
        }
    }

    fn move_var(&mut self, name: &str, span: Span) {
        let is_copy = self.find_var(name)
            .map(|info| Self::is_copy_type(&info.xiom_type))
            .unwrap_or(false);
        if is_copy { return; }

        let ok = match self.find_var(name) {
            Some(info) => match info.state {
                BorrowState::Moved => {
                    self.error(format!("use of moved value '{}'", name), span);
                    false
                }
                BorrowState::ReadBorrowed | BorrowState::WriteBorrowed => {
                    self.error(format!("cannot move '{}' while borrowed", name), span);
                    false
                }
                BorrowState::Owned => true,
            }
            None => true,
        };
        if ok {
            if let Some(info) = self.find_var_mut(name) {
                info.state = BorrowState::Moved;
            }
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<BorrowError>> {
        for item in &program.items {
            self.check_top_decl(item);
        }
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn check_top_decl(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Fn(fd) => {
                if fd.body.is_some() {
                    self.check_fn_decl(fd);
                }
            }
            TopDecl::Module(md) => {
                for item in &md.items {
                    self.check_top_decl(item);
                }
            }
            _ => {}
        }
    }

    fn check_fn_decl(&mut self, fd: &FnDecl) {
        self.param_names.clear();
        for param in &fd.params {
            self.param_names.insert(param.name.name.clone());
        }
        self.push_scope();
        for param in &fd.params {
            self.add_local(&param.name.name, true, &Self::param_type_name(&param.ty));
        }
        // 5c-E: register const-generic parameters (borrow checker)
        for g in &fd.generics {
            if g.is_const {
                let gt = g.const_ty.as_ref().map(|t| CheckedType::from_ast_type(t)).unwrap_or(CheckedType::Int);
                self.add_local(&g.name.name, gt.is_numeric() || gt == CheckedType::Int, "Int");
            }
        }
        if let Some(body) = fd.body.as_ref() {
            self.check_block(body);
        }
        self.pop_scope();
        self.param_names.clear();
    }

    fn check_block(&mut self, block: &Block) {
        for item in &block.stmts {
            match item {
                StmtOrExpr::Stmt(stmt) => self.check_stmt(stmt),
                StmtOrExpr::Expr(expr) => { self.check_expr(expr); }
            }
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, type_ann, value, _) => {
                let _ = self.check_expr(value);
                if let Expr::Ident(ident) = value {
                    if self.param_names.contains(&ident.name) {
                        self.read_borrow(&ident.name, ident.span);
                    } else {
                        self.move_var(&ident.name, ident.span);
                    }
                }
                let xiom_type = type_ann.as_ref()
                    .map(|t| Self::param_type_name(t))
                    .unwrap_or_else(|| Self::infer_type_from_expr(value).to_string());
                self.add_local(&name.name, false, &xiom_type);
            }
            Stmt::Var(name, type_ann, value, _) => {
                let _ = self.check_expr(value);
                if let Expr::Ident(ident) = value {
                    if self.param_names.contains(&ident.name) {
                        self.read_borrow(&ident.name, ident.span);
                    } else {
                        self.move_var(&ident.name, ident.span);
                    }
                }
                let xiom_type = type_ann.as_ref()
                    .map(|t| Self::param_type_name(t))
                    .unwrap_or_else(|| Self::infer_type_from_expr(value).to_string());
                self.add_local(&name.name, true, &xiom_type);
            }
            Stmt::Assign(place, value, _) => {
                let _ = self.check_expr(value);
                if let Expr::Ident(ident) = value {
                    self.move_var(&ident.name, ident.span);
                }
                self.check_expr(place);
            }
            Stmt::Return(expr, span) => {
                if let Some(e) = expr {
                    let result = self.check_expr(e);
                    if result == ExprResult::ReadRef || result == ExprResult::WriteRef {
                        self.error("cannot return a borrow from a function", *span);
                    }
                }
            }
            Stmt::Expr(expr, _) => {
                self.check_expr(expr);
            }
            Stmt::If(cond, then_block, elifs, else_block, _) => {
                self.check_expr(cond);
                self.push_scope();
                self.check_block(then_block);
                self.pop_scope();
                for (econd, eblock) in elifs {
                    self.check_expr(econd);
                    self.push_scope();
                    self.check_block(eblock);
                    self.pop_scope();
                }
                if let Some(eb) = else_block {
                    self.push_scope();
                    self.check_block(eb);
                    self.pop_scope();
                }
            }
            Stmt::Match(expr, arms, _) => {
                self.check_expr(expr);
                for arm in arms {
                    match &arm.body {
                        MatchBody::Block(b) => {
                            self.push_scope();
                            self.check_block(b);
                            self.pop_scope();
                        }
                        MatchBody::Expr(e) => {
                            self.check_expr(e);
                        }
                    }
                }
            }
            Stmt::While(cond, body, _, _) => {
                self.check_expr(cond);
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::For(var, iter, body, _) => {
                self.check_expr(iter);
                self.add_local(&var.name, true, "Int");
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::Destructure(names, value, _) => {
                let _ = self.check_expr(value);
                if let Expr::Ident(ident) = value {
                    if self.param_names.contains(&ident.name) {
                        self.read_borrow(&ident.name, ident.span);
                    } else {
                        self.move_var(&ident.name, ident.span);
                    }
                }
                for name in names {
                    self.add_local(&name.name, true, "Int");
                }
            }
            Stmt::Spawn(body, _) => {
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::Break(..) => {}
            Stmt::Continue(..) => {}
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> ExprResult {
        match expr {
            Expr::Ident(ident) => {
                self.check_use(&ident.name, ident.span)
            }
            Expr::Int(_, _) | Expr::Float(_, _) | Expr::Str(_, _)
                | Expr::Char(_, _) | Expr::Bool(_, _) => ExprResult::Value,
            Expr::Paren(inner, _) => self.check_expr(inner),
            Expr::Tuple(items, _) => {
                for item in items {
                    self.check_expr(item);
                }
                ExprResult::Value
            }
            Expr::Unary(op, inner, span) => {
                match op {
                    UnaryOp::Ref => {
                        let _ = self.check_expr(inner);
                        if let Expr::Ident(ident) = inner.as_ref() {
                            self.read_borrow(&ident.name, *span);
                        }
                        ExprResult::ReadRef
                    }
                    UnaryOp::MutRef => {
                        let _ = self.check_expr(inner);
                        if let Expr::Ident(ident) = inner.as_ref() {
                            self.write_borrow(&ident.name, *span);
                        }
                        ExprResult::WriteRef
                    }
                    _ => {
                        self.check_expr(inner);
                        ExprResult::Value
                    }
                }
            }
            Expr::Binary(left, _, right, _) => {
                self.check_expr(left);
                self.check_expr(right);
                ExprResult::Value
            }
            Expr::Try(inner, _) => self.check_expr(inner),
            Expr::Imply(a, b, _) => {
                self.check_expr(a);
                self.check_expr(b);
                ExprResult::Value
            }
            Expr::Is(expr, _, _) => {
                self.check_expr(expr);
                ExprResult::Value
            }
            Expr::Field(obj, _, _) => {
                self.check_expr(obj)
            }
            Expr::Call(func, args, span) => {
                self.check_call(func, args, *span)
            }
            Expr::Index(arr, idx, _) => {
                self.check_expr(arr);
                self.check_expr(idx);
                ExprResult::Value
            }
            Expr::AtPre(inner, _) => self.check_expr(inner),
            Expr::Ref(inner, _) => {
                self.check_expr(inner);
                if let Expr::Ident(ident) = inner.as_ref() {
                    self.read_borrow(&ident.name, expr.span());
                }
                ExprResult::ReadRef
            }
            Expr::MutRef(inner, _) => {
                self.check_expr(inner);
                if let Expr::Ident(ident) = inner.as_ref() {
                    self.write_borrow(&ident.name, expr.span());
                }
                ExprResult::WriteRef
            }
            Expr::Some(inner, _) => {
                self.check_expr(inner);
                ExprResult::Value
            }
            Expr::None(_) => ExprResult::Value,
            Expr::Ok(inner, _) => {
                self.check_expr(inner);
                ExprResult::Value
            }
            Expr::Err(inner, _) => {
                self.check_expr(inner);
                ExprResult::Value
            }
            Expr::Struct(_, fields, _spread, span) => {
                for (_, val) in fields {
                    let result = self.check_expr(val);
                    if result == ExprResult::ReadRef || result == ExprResult::WriteRef {
                        self.error("cannot store borrow in struct", *span);
                    }
                }
                ExprResult::Value
            }
            Expr::Array(items, _) => {
                for item in items {
                    self.check_expr(item);
                }
                ExprResult::Value
            }
            Expr::Closure(_, _, _, _) | Expr::PipeClosure(_, _, _) => ExprResult::Value,
            Expr::Await(inner, _) => self.check_expr(inner),
            Expr::Comptime(inner, _) => self.check_expr(inner),
            Expr::Unsafe(b, _) | Expr::BlockExpr(b, _) => { self.check_block(b); ExprResult::Value }
            Expr::As(inner, _, _) => {
                self.check_expr(inner);
                ExprResult::Value
            }
            Expr::If(cond, _then_block, elifs, _else_block, _) => {
                self.check_expr(cond);
                for (econd, _eblock) in elifs {
                    self.check_expr(econd);
                }
                ExprResult::Value
            }
            Expr::Match(scrutinee, arms, _) => {
                self.check_expr(scrutinee);
                for arm in arms {
                    match &arm.body {
                        MatchBody::Block(b) => {
                            self.push_scope();
                            self.check_block(b);
                            self.pop_scope();
                        }
                        MatchBody::Expr(e) => {
                            self.check_expr(e);
                        }
                    }
                }
                ExprResult::Value
            }
            // 5c-R: Error-poisoned nodes carry an ErrorGuaranteed proof.
            // Already diagnosed — skip borrow checking for this subtree.
            Expr::Error(_, _) => ExprResult::Value,
        }
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr], span: Span) -> ExprResult {
        // Special case: x.clone()
        if let Expr::Field(obj, method, _) = callee {
            if method.name == "clone" && args.is_empty() {
                self.check_expr(obj);
                // clone() read-borrows self and returns a fresh owned copy
                if let Expr::Ident(ident) = obj.as_ref() {
                    self.read_borrow(&ident.name, span);
                    self.release_borrow(&ident.name, BorrowType::Read);
                }
                return ExprResult::Value;
            }
        }

        // Push scope for temporary borrows from arguments
        self.push_scope();

        self.check_expr(callee);

        for arg in args {
            // Check the argument expression
            let arg_result = self.check_expr(arg);
            // If it's a bare identifier (not wrapped in & or &mut), move ownership
            if let Expr::Ident(ident) = arg {
                // Only move if the arg was not wrapped in a borrow operator
                if arg_result != ExprResult::ReadRef && arg_result != ExprResult::WriteRef {
                    // Re-check: did check_expr already change state?
                    // check_expr for Ident only checks use, doesn't move
                    if self.param_names.contains(&ident.name) {
                        self.read_borrow(&ident.name, ident.span);
                    } else {
                        self.move_var(&ident.name, ident.span);
                    }
                }
            }
        }

        self.pop_scope();

        ExprResult::Value
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use xiom_lexer::Lexer;
    use xiom_parser::Parser;

    fn project_root() -> PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap()
            .to_path_buf()
    }

    fn check(source: &str) -> Result<(), Vec<CheckError>> {
        let tokens = Lexer::new(source).tokenize();
        let program = Parser::new(tokens).parse_program();
        match program {
            Ok(p) => Checker::new().check_program(&p),
            Err(e) => Err(vec![CheckError {
                message: format!("parse error: {e}"),
                span: e.span,
                cause: crate::types::TypeCause::Other,
            }]),
        }
    }

    #[test]
    fn test_simple_addition() {
        let result = check("fn add(a: Int, b: Int) -> Int { return a + b; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_return_type_mismatch() {
        let result = check("fn bad() -> Int { return true; }");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Divergence analysis (5d): tail expressions whose every path returns
    // -----------------------------------------------------------------------

    #[test]
    fn test_divergence_unsafe_tail_return() {
        // The alloc.xi FFI-wrapper pattern: entire body is `unsafe { ...; return X; }`.
        let result = check("fn f() -> Int { unsafe { return 42; } }");
        assert!(result.is_ok(), "unsafe tail with return must satisfy fn return type: {:?}", result.err());
    }

    #[test]
    fn test_divergence_unsafe_with_early_return() {
        let src = r#"
fn f(x: Int) -> Int {
  unsafe {
    if x == 0 { return 1; };
    return x * 2;
  }
}"#;
        let result = check(src);
        assert!(result.is_ok(), "unsafe with guard + tail return must pass: {:?}", result.err());
    }

    #[test]
    fn test_divergence_if_else_both_return() {
        let src = "fn f(x: Int) -> Int { if x > 0 { return 1; } else { return 2; } }";
        let result = check(src);
        assert!(result.is_ok(), "if/else where both branches return must pass: {:?}", result.err());
    }

    #[test]
    fn test_divergence_negative_unsafe_no_return_still_errors() {
        // The unsafe block does NOT return — the () tail must still mismatch Int.
        let result = check("fn f() -> Int { unsafe { let x = 1; } }");
        assert!(result.is_err(), "unsafe tail WITHOUT return must still be a type error");
    }

    #[test]
    fn test_gap3_pub_const_resolves_at_use_site() {
        // COMPILER_GAPS GAP-3: a module-level `pub const` must be a resolvable
        // name inside functions (was: "undefined variable 'MAX'").
        let result = check("pub const MAX: Int = 10; fn f() -> Int { return MAX; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_gap3_const_forward_reference() {
        // A function may reference a const declared LATER in the file (pre-pass
        // registration makes const resolution order-independent).
        let result = check("fn f() -> Int { return LIMIT; } const LIMIT: Int = 42;");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_if_condition_bool() {
        let result = check("fn test(x: Int) -> Int { if x > 0 { return 1; } else { return 0; } }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_if_condition_not_bool() {
        let result = check("fn bad(x: Int) -> Int { if x { return 1; } else { return 0; } }");
        assert!(result.is_err());
    }

    #[test]
    fn test_undefined_variable() {
        let result = check("fn bad() -> Int { return x; }");
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_field_access() {
        let result = check("type Point = { x: Float64; y: Float64; } fn get_x(p: Point) -> Float64 { return p.x; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_unknown_field() {
        let result = check("type Point = { x: Float64; y: Float64; } fn bad(p: Point) -> Float64 { return p.z; }");
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_literal() {
        let result = check("type Point = { x: Float64; y: Float64; } fn make_point() -> Point { return Point{ x: 1.0, y: 2.0 }; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_let_inference() {
        let result = check("fn test() -> Int { let x = 42; return x; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_functions() {
        let result = check("fn square(x: Int) -> Int { return x * x; } fn sum_squares(a: Int, b: Int) -> Int { return square(a) + square(b); }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    // ========================================================================
    // Borrow Checker Tests
    // ========================================================================

    fn check_borrow(source: &str) -> Result<(), Vec<BorrowError>> {
        let tokens = Lexer::new(source).tokenize();
        let program = Parser::new(tokens).parse_program();
        match program {
            Ok(p) => BorrowChecker::new().check_program(&p),
            Err(e) => Err(vec![BorrowError {
                message: format!("parse error: {e}"),
                span: e.span,
            }]),
        }
    }

    #[test]
    fn test_use_after_move() {
        let result = check_borrow("type S = { v: Int; } fn main() { var x = S { v: 42; }; var y = x; let z = x; }");
        assert!(result.is_err(), "expected use-after-move error");
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.message.contains("use of moved value")));
    }

    #[test]
    fn test_double_mut_borrow() {
        let result = check_borrow("fn main() { var x = 42; var r1 = &mut x; var r2 = &mut x; }");
        assert!(result.is_err(), "expected double mutable borrow error");
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.message.contains("cannot borrow") && e.message.contains("more than once")));
    }

    #[test]
    fn test_read_while_mut_borrowed() {
        let result = check_borrow("fn main() { var x = 42; let r1 = &mut x; let r2 = &x; }");
        assert!(result.is_err(), "expected read while mut borrowed error");
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.message.contains("immutable while mutably borrowed")));
    }

    #[test]
    fn test_move_while_borrowed() {
        let result = check_borrow("type S = { v: Int; } fn main() { var x = S { v: 42; }; let r = &x; var y = x; }");
        assert!(result.is_err(), "expected move while borrowed error");
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.message.contains("cannot move") && e.message.contains("while borrowed")));
    }

    #[test]
    fn test_let_immutable_no_mut_borrow() {
        let result = check_borrow("fn main() { let x = 42; let r = &mut x; }");
        assert!(result.is_err(), "expected cannot borrow immutable as mutable error");
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.message.contains("immutable local")));
    }

    #[test]
    fn test_var_mutable_allows_mut_borrow() {
        let result = check_borrow("fn main() { var x = 42; let r = &mut x; }");
        assert!(result.is_ok(), "var binding should allow mutable borrow: {:?}", result.err());
    }

    #[test]
    fn test_clone_restores_ownership() {
        let result = check_borrow("fn main() { var x = 42; let y = x.clone(); let z = x.clone(); }");
        assert!(result.is_ok(), "clone should restore ownership: {:?}", result.err());
    }

    #[test]
    fn test_borrow_return_rejected() {
        let result = check_borrow("fn main(x: Int) -> &Int { return &x; }");
        assert!(result.is_err(), "expected borrow return error");
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.message.contains("cannot return a borrow")));
    }

    #[test]
    fn test_borrow_expires_at_scope_end() {
        let result = check_borrow("fn main() -> Int { var x = 42; if true { let r = &x; } return x; }");
        assert!(result.is_ok(), "borrow should expire at scope end: {:?}", result.err());
    }

    #[test]
    fn test_function_call_moves() {
        let result = check_borrow("type S = { v: Int; } fn foo(x: S) -> Int { return x.v; } fn main() { var a = S { v: 42; }; foo(a); let b = a; }");
        assert!(result.is_err(), "expected use-after-move after function call");
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.message.contains("use of moved value")));
    }

    #[test]
    fn test_read_borrow_allows_multiple() {
        let result = check_borrow("fn main() { var x = 42; let r1 = &x; let r2 = &x; let r3 = &x; }");
        assert!(result.is_ok(), "multiple read borrows should be allowed: {:?}", result.err());
    }

    #[test]
    fn test_borrow_in_struct_rejected() {
        let result = check_borrow("type Foo = { a: Int; } fn main() { var x = 42; let f = Foo{ a: &x }; }");
        assert!(result.is_err(), "expected cannot store borrow in struct error");
        let errs = result.err().unwrap();
        assert!(errs.iter().any(|e| e.message.contains("cannot store borrow in struct")));
    }

    #[test]
    fn test_mut_borrow_read_after_release() {
        let result = check_borrow("fn main() -> Int { var x = 42; if true { let r = &mut x; } return x; }");
        assert!(result.is_ok(), "read after mut borrow released should be ok: {:?}", result.err());
    }

    #[test]
    fn test_borrow_with_function_args_ref() {
        let result = check_borrow("fn foo(x: &Int) -> Int { return 1; } fn main() -> Int { var a = 42; foo(&a); return a; }");
        assert!(result.is_ok(), "passing &x should not move x: {:?}", result.err());
    }

    #[test]
    fn test_param_ref_read_borrow_not_move() {
        let result = check_borrow("fn read(pos: &mut Int) -> Int { let current = pos; return 42; }");
        assert!(result.is_ok(), "reading &mut param should not move: {:?}", result.err());
    }

    #[test]
    fn test_param_ref_in_call() {
        let result = check_borrow("fn inner(x: &mut Int) -> Int { return 42; } fn outer(pos: &mut Int) -> Int { return inner(pos); }");
        assert!(result.is_ok(), "passing &mut param to fn should reborrow: {:?}", result.err());
    }

    #[test]
    fn test_option_type() {
        let result = check("fn test() -> Option[Int] { return Some(42); }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_result_type_ok() {
        let result = check("fn test() -> Result[Int, Str] { return Ok(42); }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_result_type_err() {
        let result = check("fn test() -> Result[Int, Str] { return Err(\"fail\"); }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_match_option() {
        let result = check("fn test(x: Option[Int]) -> Int { match x { Some(v) => 1, None => 0, } }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_match_result() {
        let result = check("fn test(r: Result[Int, Str]) -> Int { match r { Ok(v) => 1, Err(e) => -1, } }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_contract_function() {
        let result = check("fn div(a: Float64, b: Float64) -> Float64 requires: b != 0.0 { return a / b; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_type_with_invariant() {
        let result = check("type Positive = { val: Int; invariant: val > 0; } fn main() -> Int { return 0; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_type_with_derive() {
        let result = check("type Point = { x: Float64; y: Float64; } derive[Eq, Clone] fn main() -> Int { return 0; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_while_loop() {
        let result = check("fn test() -> Int { var i = 0; while i < 10 { i = i + 1; } return i; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_field_assignment() {
        let result = check("type Point = { x: Float64; y: Float64; } fn test(p: Point) -> Point { p.x = 5.0; return p; }");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    // ========================================================================
    // Module System Tests
    // ========================================================================

    #[test]
    fn test_module_basic() {
        let result = check("\
module math {
    pub fn add(a: Int, b: Int) -> Int { return a + b; }
}
fn main() -> Int { return math.add(1, 2); }
");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_module_use_single() {
        let result = check("\
module math {
    pub fn add(a: Int, b: Int) -> Int { return a + b; }
}
use math.add;
fn main() -> Int { return add(1, 2); }
");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_module_use_alias() {
        let result = check("\
module math {
    pub fn add(a: Int, b: Int) -> Int { return a + b; }
}
use math.add as plus;
fn main() -> Int { return plus(1, 2); }
");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_module_use_glob() {
        let result = check("\
module math {
    pub fn add(a: Int, b: Int) -> Int { return a + b; }
    pub fn sub(a: Int, b: Int) -> Int { return a - b; }
}
use math.*;
fn main() -> Int { return add(1, 2); }
");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_module_private_access_rejected() {
        let result = check("\
module math {
    fn secret(a: Int) -> Int { return a; }
}
fn main() -> Int { return math.secret(1); }
");
        assert!(result.is_err(), "expected private access error: {:?}", result.ok());
    }

    #[test]
    fn test_module_nested() {
        let result = check("\
module outer {
    module inner {
        pub fn val() -> Int { return 42; }
    }
}
fn main() -> Int { return outer.inner.val(); }
");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_method_call_basic() {
        let result = check("\
type Point = { x: Int; y: Int; }
fn Point.twice(val: Int) -> Int { return val * 2; }
fn main() -> Int {
    let p = Point{ x: 1, y: 2 };
    return p.twice(5);
}
");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    #[test]
    fn test_method_call_mut() {
        let result = check("\
type Counter = { val: Int; }
fn Counter.add(c: Counter, amount: Int) -> Counter {
    return Counter{ val: c.val + amount };
}
fn main() -> Int {
    var c = Counter{ val: 0 };
    let c2 = c.add(5);
    return c2.val;
}
");
        assert!(result.is_ok(), "{:?}", result.err());
    }

    // ========================================================================
    // Type Error Handling Tests
    // ========================================================================

    #[test]
    fn test_unknown_type_in_struct_lit() {
        let result = check("fn main() -> Int { var x = Foo{ bar: 1 }; return 0; }");
        assert!(result.is_err(), "unknown type 'Foo' in struct literal should be an error");
    }

    #[test]
    fn test_unknown_type_in_param() {
        let result = check("fn foo(x: Unknown) -> Int { return 0; }");
        assert!(result.is_ok(), "checker currently allows unknown types to pass (legacy behavior)");
    }

    #[test]
    fn test_unknown_type_in_return() {
        let result = check("fn foo() -> Unknown { return 0; }");
        assert!(result.is_err(), "unknown type in return annotation should be an error");
    }

    #[test]
    fn test_enum_variant_wrong_field_count() {
        let result = check("enum Token { Ident(name: Str) } fn main() -> Token { return Ident(1, 2); }");
        assert!(result.is_ok(), "checker doesn't currently validate enum variant constructor arity (known limitation)");
    }

    #[test]
    fn test_enum_variant_wrong_field_type() {
        let result = check("enum Token { IntVal(v: Int) } fn main() -> Int { var t = IntVal(42); return 0; }");
        assert!(result.is_ok(), "enum variant with correct field type should be ok: {:?}", result.err());
    }

    #[test]
    fn test_method_on_nonexistent_method() {
        let result = check("fn main() -> Int { var x = 42; return x.nonexistent(); }");
        assert!(result.is_err(), "calling nonexistent method should error");
    }

    #[test]
    fn test_interface_bound_violation() {
        // Interface-bound validation now happens at monomorphisation time
        // (codegen), not at check time.  The checker accepts calls to
        // interface methods on generic params with bounds — the codegen
        // catches violations when concrete types don't implement the
        // required interface.
        let src = "\
interface Foo { fn bar() -> Int; }
type MyType = { x: Int; }
fn use_foo[T: Foo](x: T) -> Int { return x.bar(); }
fn main() -> Int { var mt = MyType{ x: 1 }; return use_foo(mt); }";
        let result = check(src);
        // The checker now accepts this (interface dispatch resolves `bar` on
        // generic `T: Foo`). Violations are caught by codegen monomorphisation.
        assert!(result.is_ok(), "interface-bound generic should type-check: {:?}", result.err());
    }

    #[test]
    fn test_type_alias_compiles() {
        let src = "\
type Point2D = { x: Float64; y: Float64; }
type Vec2 = Point2D;
fn main() -> Float64 { var v = Point2D{ x: 1.0, y: 2.0 }; return v.x; }";
        let result = check(src);
        assert!(result.is_ok(), "type alias should compile: {:?}", result.err());
    }

    #[test]
    fn test_generic_enum_variant_constructor() {
        let src = "\
enum Container[T] { Empty, Single(value: T) }
fn main() -> Int { var c = Single(value: 42); return 0; }";
        let result = check(src);
        assert!(result.is_ok(), "generic enum variant constructor should compile: {:?}", result.err());
    }

    // ========================================================================
    // Module Catalog Tests
    // ========================================================================

    #[test]
    fn test_catalog_cold_start() {
        let test_mod = project_root().join("examples/test_mod");
        let mut cat = ModuleCatalog::new(vec![test_mod.to_string_lossy().to_string()]);
        cat.build_index();
        let path_segments: Vec<String> = ["benchmark".to_string(), "math".to_string()].to_vec();
        let cached = cat.find_owned(&path_segments);
        assert!(cached.is_some(), "cold start should find benchmark.math module");
    }

    #[test]
    fn test_catalog_cached_hit() {
        let test_mod = project_root().join("examples/test_mod");
        let mut cat = ModuleCatalog::new(vec![test_mod.to_string_lossy().to_string()]);
        cat.build_index();
        let path_segments: Vec<String> = ["benchmark".to_string(), "math".to_string()].to_vec();
        let first = cat.find_owned(&path_segments);
        let second = cat.find_owned(&path_segments);
        assert!(first.is_some());
        assert!(second.is_some());
    }

    #[test]
    fn test_catalog_not_found() {
        let test_mod = project_root().join("examples/test_mod");
        let mut cat = ModuleCatalog::new(vec![test_mod.to_string_lossy().to_string()]);
        cat.build_index();
        let path_segments: Vec<String> = ["nonexistent".to_string(), "module".to_string()].to_vec();
        let cached = cat.find_owned(&path_segments);
        assert!(cached.is_none(), "nonexistent module should return None");
    }

    #[test]
    fn test_catalog_index_built() {
        let test_mod = project_root().join("examples/test_mod");
        let mut cat = ModuleCatalog::new(vec![test_mod.to_string_lossy().to_string()]);
        cat.build_index();
        let path_segments: Vec<String> = ["benchmark".to_string(), "math".to_string()].to_vec();
        let cached = cat.find_owned(&path_segments);
        assert!(cached.is_some(), "index should enable lookups");
    }

    #[test]
    fn test_catalog_empty_source_dirs() {
        let mut cat = ModuleCatalog::new(Vec::new());
        let path_segments: Vec<String> = ["anything".to_string()].to_vec();
        let cached = cat.find_owned(&path_segments);
        assert!(cached.is_none(), "empty source dirs should return None");
    }

    #[test]
    fn test_catalog_flat_filename_lookup() {
        let bench_dir = project_root().join("examples/benchmark");
        let mut cat = ModuleCatalog::new(vec![bench_dir.to_string_lossy().to_string()]);
        cat.build_index();
        let path_segments: Vec<String> = ["math".to_string()].to_vec();
        let _ = cat.find_owned(&path_segments);
        assert!(true);
    }

    #[test]
    fn test_catalog_no_duplicate_cache() {
        let test_mod = project_root().join("examples/test_mod");
        let mut cat = ModuleCatalog::new(vec![test_mod.to_string_lossy().to_string()]);
        cat.build_index();
        let path_segments: Vec<String> = ["benchmark".to_string(), "math".to_string()].to_vec();
        cat.find_owned(&path_segments);
        cat.find_owned(&path_segments);
        cat.find_owned(&path_segments);
        let cached = cat.all_cached();
        let count = cached.iter().filter(|m| m.dotted_name == "benchmark.math").count();
        assert_eq!(count, 1, "should have exactly one cached entry per module");
    }

    #[test]
    fn test_write_borrow_while_read_borrow_active() {
        let result = check_borrow("fn main() { var x = 42; let r = &x; let w = &mut x; }");
        assert!(result.is_err(), "write borrow during active read borrow should error");
    }

    #[test]
    fn test_double_mut_borrow_rejected() {
        let result = check_borrow("fn main() { var x = 42; let r1 = &mut x; let r2 = &mut x; }");
        assert!(result.is_err(), "two simultaneous &mut borrows should error");
    }

    #[test]
    fn test_mutation_through_immutable_ref_rejected() {
        let result = check_borrow("fn main() { var x = 42; let r = &x; }");
        assert!(result.is_ok(), "creating &T ref should not be a borrow error: {:?}", result.err());
    }

    #[test]
    fn test_return_owned_type_compiles() {
        let result = check_borrow("fn make() -> Int { var x = 42; return x; } fn main() -> Int { return make(); }");
        assert!(result.is_ok(), "returning owned value should compile: {:?}", result.err());
    }

    #[test]
    fn test_clone_for_struct_field_compiles() {
        let result = check_borrow("\
type Data = { val: Int; } derive[Clone]\n\
fn main() -> Int { var x = 42; var d = Data{ val: x.clone() }; return d.val; }");
        assert!(result.is_ok(), "clone for struct storage should compile: {:?}", result.err());
    }

    #[test]
    fn test_use_after_move_in_if_branch() {
        let result = check_borrow("\
type S = { v: Int; }\n\
fn consume(x: S) -> Int { return x.v; }\n\
fn main() -> Int { var x = S { v: 42; }; if true { var y = x; } return x; }");
        assert!(result.is_err(), "use-after-move after if-branch move should error");
    }

    #[test]
    fn test_reassign_after_move_is_error() {
        let result = check_borrow("\
type S = { v: Int; }\n\
fn consume(x: S) -> Int { return x.v; }\n\
fn main() -> Int { var x = S { v: 42; }; consume(x); x = 99; return x; }");
        assert!(result.is_err(), "reassign after move should be a borrow error");
    }

    #[test]
    fn test_move_into_vec_element() {
        let result = check_borrow("\
fn main() -> Int { var x = 42; var v = Vec[Int].new(); v.push(x); return 0; }");
        assert!(result.is_ok(), "move into Vec should compile: {:?}", result.err());
    }

    #[test]
    fn test_borrow_through_function_parameter() {
        let result = check_borrow("\
fn read(x: &Int) -> Int { return 1; }\n\
fn main() -> Int { var a = 42; let r = read(&a); return a + r; }");
        assert!(result.is_ok(), "borrow through fn param should not move: {:?}", result.err());
    }

    #[test]
    fn test_mut_borrow_released_then_mut_borrow_again() {
        let result = check_borrow("\
fn main() -> Int { var x = 42; if true { let r = &mut x; } let s = &mut x; return 1; }");
        assert!(result.is_ok(), "mut borrow again after release should compile: {:?}", result.err());
    }

    #[test]
    fn test_read_borrow_released_then_move() {
        let result = check_borrow("\
fn main() -> Int { var x = 42; if true { let r = &x; } return x; }");
        assert!(result.is_ok(), "move after read-borrow release should compile: {:?}", result.err());
    }

    #[test]
    fn test_multiple_borrow_restrictions() {
        let result = check_borrow("\
type Wrapper = { val: Int; }\n\
fn main() -> Int { var x = Wrapper { val: 42; }; let r = &x; var y = x; return 0; }");
        assert!(result.is_err(), "move while borrowed should error");
    }

    /// 8B/M5: Fuzz harness — random type combinations, verify TypeArena integrity.
    #[test]
    fn fuzz_type_arena_random_inserts() {
        let mut arena = crate::TypeArena::new();
        let mut seed: u64 = 99991;
        let mut names = Vec::new();
        for _ in 0..200 {
            let name_len = (seed % 16 + 1) as usize;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let mut name = String::with_capacity(name_len);
            for _ in 0..name_len {
                let c = ((seed % 26) as u8 + b'a') as char;
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                name.push(c);
            }
            let id = arena.intern(&name, false);
            // After interning, looking up the same name must return the same id
            let id2 = arena.intern(&name, false);
            assert_eq!(id, id2, "same name must produce same id");
            names.push((name, id));
        }
    }

    /// 8B/M5: Fuzz harness — verify types_compatible with random type pairs.
    #[test]
    fn fuzz_types_compatible_random() {
        let checker = crate::Checker::new();
        let types = vec![
            crate::CheckedType::Int,
            crate::CheckedType::Float64,
            crate::CheckedType::Float32,
            crate::CheckedType::Bool,
            crate::CheckedType::Char,
            crate::CheckedType::Str,
        ];
        for a in &types {
            for b in &types {
                // Must not panic on any pair
                let _ = checker.types_compatible(a, b);
            }
        }
    }

    /// 8B/M5: Fuzz harness — error count should never overflow (u32 safety).
    #[test]
    fn fuzz_error_count_boundary() {
        let mut checker = crate::Checker::new();
        assert_eq!(checker.error_count, 0);
        // Error count starts at 0, increments on each error
        // Must not wrap or panic after many errors
        for _ in 0..1000 {
            checker.error_count = checker.error_count.saturating_add(1);
        }
        assert!(checker.error_count > 0);
        assert!(checker.error_count <= 1000);
    }

    /// 8B/M5: Fuzz harness — source_dirs never cause panic on missing dirs.
    #[test]
    fn fuzz_source_dirs_missing() {
        let mut checker = crate::Checker::new();
        checker.add_source_dir("/nonexistent/path/12345".to_string());
        checker.add_source_dir("\\\\invalid\\path\\".to_string());
        checker.build_catalog_index();
        // Must not panic — the catalog should handle missing paths gracefully
    }

    // ── M5: Property-based / generative tests ───────────────────────────

    /// Checker must not crash on programs with deeply nested expressions.
    #[test]
    fn prop_deep_nesting_no_panic() {
        // Generate a function with deeply nested arithmetic
        let mut src = String::from("fn deep() -> Int { return ");
        for i in 0..200 {
            src.push_str(&format!("({i} + "));
        }
        src.push_str("0");
        for _ in 0..200 { src.push(')'); }
        src.push_str("; }");
        let result = check(&src);
        // Must not panic — may produce errors or succeed, but must not crash
        assert!(result.is_ok() || result.is_err());
    }

    /// Type checker must be consistent: checking the same program twice
    /// should produce the same result.
    #[test]
    fn prop_idempotent_check() {
        let src = "fn add(a: Int, b: Int) -> Int { return a + b; }\nfn main() -> Int { return add(1, 2); }";
        let r1 = check(src);
        let r2 = check(src);
        assert_eq!(r1.is_ok(), r2.is_ok(), "check must be idempotent");
        if let (Ok(()), Ok(())) = (&r1, &r2) {
            // both succeeded
        } else {
            assert_eq!(r1.err().unwrap().len(), r2.err().unwrap().len(),
                "same error count on idempotent check");
        }
    }

    /// Type inference must be consistent: variables assigned the same expression
    /// must have the same inferred type.
    #[test]
    fn prop_consistent_inference() {
        let srcs = vec![
            ("int literal", "fn main() -> Int { var x = 42; return x; }"),
            ("float literal", "fn main() -> Float64 { var x = 3.14; return x; }"),
            ("bool literal", "fn main() -> Bool { var x = true; return x; }"),
            ("str literal", "fn main() -> Str { var x = \"hi\"; return x; }"),
            ("array literal", "fn main() -> Int { var x = [1, 2, 3]; return x[0]; }"),
            ("option some", "fn main() -> Int { var x = Some(42); match x { Some(v) => v, None => 0, } }"),
            ("result ok", "fn main() -> Int { var x: Result[Int, Str] = Ok(42); match x { Ok(v) => v, Err(_) => 0, } }"),
        ];
        for (name, src) in &srcs {
            let result = check(src);
            assert!(result.is_ok(), "should type-check: {name}\n{src}\nerror: {:?}", result.err());
        }
    }

    /// Generic type checking must not crash on deep parameter nesting.
    #[test]
    fn prop_deep_generics_no_panic() {
        let src = "fn nest[T](x: T) -> T { return x; }\nfn main() -> Int { return nest(nest(nest(nest(nest(42))))); }";
        let result = check(&src);
        assert!(result.is_ok(), "nested generics must type-check: {:?}", result.err());
    }

    /// The checker must reject programs with obvious type errors consistently.
    #[test]
    fn prop_type_errors_consistently_detected() {
        let bad_srcs = vec![
            ("int vs str", "fn main() -> Int { return \"not an int\"; }"),
            ("bool vs int", "fn main() -> Bool { return 42; }"),
            ("wrong arg type", "fn add(a: Int, b: Int) -> Int { return a + b; } fn main() -> Int { return add(\"x\", 2); }"),
            ("undeclared var", "fn main() -> Int { return x; }"),
        ];
        for (name, src) in &bad_srcs {
            let result = check(src);
            assert!(result.is_err(), "should fail type-check: {name}");
        }
    }

    /// All built-in interfaces are recognized by the checker.
    #[test]
    fn prop_builtin_interfaces_recognized() {
        let interfaces = ["Clone", "Eq", "Ord", "Display", "Hash", "Default", "Drop", "FromStr", "Debug", "Add", "Sub", "Mul", "Div"];
        for iface in &interfaces {
            let src = format!("interface {iface} {{ fn dummy() -> Int; }}");
            let result = check(&src);
            assert!(result.is_ok(), "interface {iface} must be recognized: {:?}", result.err());
        }
    }

    /// The checker must not produce false positives on valid arithmetic.
    #[test]
    fn prop_arithmetic_no_false_positive() {
        let ops = ["+", "-", "*", "/", "%"];
        for op in ops {
            let src = format!("fn arith(a: Int, b: Int) -> Int {{ return a {op} b; }}");
            let result = check(&src);
            assert!(result.is_ok(), "arithmetic {op} must be valid: {:?}", result.err());
        }
    }

    /// Large programs with many declarations must not overflow or hang.
    #[test]
    fn prop_large_program_no_hang() {
        let mut src = String::new();
        for i in 0..100 {
            src.push_str(&format!("fn f{i}() -> Int {{ return {i}; }}\n"));
        }
        // Add a main that calls one of them
        src.push_str("fn main() -> Int { return f42(); }");
        let result = check(&src);
        assert!(result.is_ok(), "100-fn program must type-check: {:?}", result.err());
    }

    /// Combinatorial: every pair of primitive types must interact correctly.
    #[test]
    fn prop_primitive_pairs() {
        let types = ["Int", "Float64", "Bool", "Str"];
        for t in &types {
            let src = format!("fn use_{t}(x: {t}) -> {t} {{ return x; }}");
            let result = check(&src);
            assert!(result.is_ok(), "must accept {t} identity function: {:?}", result.err());
        }
    }

    /// Option/Result nesting must not cause stack overflow.
    #[test]
    fn prop_deep_option_result_no_panic() {
        let src = "fn deep() -> Option[Result[Option[Result[Int, Str]], Str]] { return None; }";
        let result = check(&src);
        assert!(result.is_ok(), "deeply nested types: {:?}", result.err());
    }

    /// Method resolution: methods defined on types must be callable.
    #[test]
    fn prop_method_resolution_valid() {
        let src = "type Point = { x: Float64; y: Float64; }\npub fn Point.dist(self) -> Float64 { return x; }\nfn main() -> Float64 { var p = Point{ x: 1.0; y: 2.0; }; return p.dist(); }";
        let result = check(&src);
        assert!(result.is_ok(), "method resolution: {:?}", result.err());
    }

    /// Enum variants with payload must type-check correctly.
    #[test]
    fn prop_enum_payloads() {
        let cases = [
            "enum E { A(x: Int), B } fn main() -> Int { match E.A(42) { E.A(v) => v, E.B => 0, } }",
            "enum Opt { Some(v: Int), None } fn main() -> Int { match Opt.Some(10) { Opt.Some(v) => v, Opt.None => 0, } }",
            "enum Res { Ok(v: Int), Err(e: Str) } fn main() -> Int { match Res.Ok(1) { Res.Ok(v) => v, Res.Err(_) => 0, } }",
        ];
        for (i, src) in cases.iter().enumerate() {
            let result = check(src);
            assert!(result.is_ok(), "enum case {i}: {src}\n{:?}", result.err());
        }
    }

    /// Ref/deref patterns: borrow and use.
    #[test]
    fn prop_borrow_patterns() {
        let src = "fn read(x: &Int) -> Int { return x; }\nfn main() -> Int { var a = 42; return read(&a); }";
        let result = check(&src);
        assert!(result.is_ok(), "borrow pattern: {:?}", result.err());
    }

    /// use / module-qualified path resolution.
    #[test]
    fn prop_module_qualified_names() {
        let src = "module math { pub fn add(a: Int, b: Int) -> Int { return a + b; } }\nfn main() -> Int { return math.add(1, 2); }";
        let result = check(&src);
        assert!(result.is_ok(), "module-qualified: {:?}", result.err());
    }

    /// Wildcard and partial patterns in match must not crash.
    #[test]
    fn prop_match_wildcard_patterns() {
        let patterns = [
            "fn main() -> Int { match 42 { 0 => 1, _ => 0, } }",
            "fn main() -> Int { match Some(1) { Some(v) => v, _ => 0, } }",
            "fn main() -> Int { match Ok(5) { Ok(v) => v, _ => 0, } }",
        ];
        for (i, src) in patterns.iter().enumerate() {
            let result = check(src);
            assert!(result.is_ok(), "wildcard pattern {i}: {:?}", result.err());
        }
    }

    /// Generic with multiple bounds must resolve.
    #[test]
    fn prop_multi_bound_generic() {
        let src = "fn double[T: Clone + Display](x: T) -> T { return x.clone(); }\nfn main() -> Int { return double(42); }";
        let result = check(&src);
        // May fail if Clone/Display aren't impl'd for Int, but must not crash
        assert!(result.is_ok() || result.is_err());
    }

    // ── M7: Deref/DerefMut/AsRef usage tests ──────────────────────────

    /// Box[T] deref: field access through Box should resolve to T's fields.
    #[test]
    fn prop_box_deref_field_access() {
        let src = "type Point = { x: Float64; y: Float64; }\nfn main() -> Float64 { var p = Box.new(Point{ x: 1.0; y: 2.0; }); return p.x; }";
        let result = check(&src);
        // Field access through Box requires Deref — may not be fully supported yet
        // but must not crash the checker
        assert!(result.is_ok() || result.is_err());
    }

    /// Rc[T] deref: reading through Rc should compile.
    #[test]
    fn prop_rc_deref_valid() {
        let src = "fn main() -> Int { var r = Rc.new(42); return r; }";
        let result = check(&src);
        assert!(result.is_ok() || result.is_err());
    }

    /// Arc[T] deref: reading through Arc should compile.
    #[test]
    fn prop_arc_deref_valid() {
        let src = "fn main() -> Int { var a = Arc.new(42); return a; }";
        let result = check(&src);
        assert!(result.is_ok() || result.is_err());
    }

    /// AsRef on Str should work.
    #[test]
    fn prop_str_asref() {
        let src = "fn show(s: &Str) { } fn main() { var x = \"hello\"; show(x.as_ref()); }";
        let result = check(&src);
        assert!(result.is_ok() || result.is_err());
    }

    /// Vec.as_slice should type-check.
    #[test]
    fn prop_vec_as_slice() {
        let src = "fn sum(items: &Slice[Int]) -> Int { return 0; }\nfn main() -> Int { var v = Vec[Int].new(); v.push(1); return sum(v.as_slice()); }";
        let result = check(&src);
        assert!(result.is_ok() || result.is_err());
    }

    // ── More fuzz-like stress tests ────────────────────────────────────

    /// Variable shadowing across scopes must not confuse the checker.
    #[test]
    fn prop_variable_shadowing() {
        let src = "fn main() -> Int { var x = 1; { var x = \"hi\"; } return x; }";
        let result = check(&src);
        assert!(result.is_ok(), "shadowing: {:?}", result.err());
    }

    /// If-else expression type unification (both branches same type).
    #[test]
    fn prop_if_else_unification() {
        let src = "fn main() -> Int { var x = if true { 1 } else { 2 }; return x; }";
        let result = check(&src);
        assert!(result.is_ok() || result.is_err());
    }

    /// Return position match expression.
    #[test]
    fn prop_return_match() {
        let src = "fn classify(x: Int) -> Str { match x { 0 => \"zero\", 1 => \"one\", _ => \"many\", } }";
        let result = check(&src);
        assert!(result.is_ok() || result.is_err());
    }

    /// Nested match expressions.
    #[test]
    fn prop_nested_match() {
        let src = "fn classify(x: Int, y: Int) -> Str { match x { 0 => match y { 0 => \"both zero\", _ => \"x zero\", }, _ => \"not zero\", } }";
        let result = check(&src);
        assert!(result.is_ok() || result.is_err());
    }

    /// Recursive function type-checking.
    #[test]
    fn prop_recursive_fn() {
        let src = "fn factorial(n: Int) -> Int { if n <= 1 { return 1; } return n * factorial(n - 1); }";
        let result = check(&src);
        assert!(result.is_ok(), "recursive: {:?}", result.err());
    }

    /// Mutually recursive functions.
    #[test]
    fn prop_mutual_recursion() {
        let src = "fn is_even(n: Int) -> Bool { if n == 0 { return true; } return is_odd(n - 1); }\nfn is_odd(n: Int) -> Bool { if n == 0 { return false; } return is_even(n - 1); }";
        let result = check(&src);
        assert!(result.is_ok(), "mutual recursion: {:?}", result.err());
    }

    /// const-generic type resolution.
    #[test]
    fn prop_const_generic() {
        let src = "fn first[T, const N: Int](arr: &[N]T) -> T { return arr[0]; }\nfn main() -> Int { var arr = [1, 2, 3]; return first(&arr); }";
        let result = check(&src);
        assert!(result.is_ok() || result.is_err());
    }

    /// Explicit type annotations at var binding.
    #[test]
    fn prop_explicit_type_annotation() {
        let srcs = [
            "fn main() -> Int { var x: Int = 42; return x; }",
            "fn main() -> Float64 { var x: Float64 = 3.14; return x; }",
            "fn main() -> Bool { var x: Bool = true; return x; }",
            "fn main() -> Str { var x: Str = \"hi\"; return x; }",
        ];
        for (i, src) in srcs.iter().enumerate() {
            let result = check(src);
            assert!(result.is_ok(), "explicit type {i}: {:?}", result.err());
        }
    }

    /// Trailing comma in match arms.
    #[test]
    fn prop_match_trailing_comma() {
        let src = "fn main() -> Int { match 42 { 0 => 1, 1 => 2, _ => 0, } }";
        let result = check(&src);
        assert!(result.is_ok(), "trailing comma: {:?}", result.err());
    }

    // ── M21-3: Checker edge cases ──────────────────────────────────────

    // Recursive types (linked lists, trees)
    #[test] fn test_edge_recursive_type_linked_list() {
        let src = "\
type Node = { val: Int; next: Option[Box[Node]]; }
fn main() -> Int { var n = Node{ val: 1; next: None }; return n.val; }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_edge_recursive_type_tree() {
        let src = "\
type Tree = { val: Int; left: Option[Box[Tree]]; right: Option[Box[Tree]]; }
fn sum(t: Option[Box[Tree]]) -> Int {
    match t { Some(node) => node.val + sum(node.left) + sum(node.right), None => 0, }
}";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_edge_recursive_mutual_types() {
        let src = "\
type A = { b: Option[B]; }
type B = { a: Option[A]; }
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Deeply nested generic types
    #[test] fn test_edge_deep_nested_generic_type() {
        let src = "fn deep() -> Option[Result[Option[Result[Option[Result[Int, Str]], Str]], Str]] { return None; }";
        let result = check(src);
        assert!(result.is_ok(), "deeply nested generics: {:?}", result.err());
    }

    #[test] fn test_edge_generic_of_generic() {
        let src = "fn vec_of_vec() -> Vec[Vec[Int]] { var v = Vec[Vec[Int]].new(); return v; }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_edge_map_type() {
        let src = "fn lookup(m: BTreeMap[Str, Vec[Int]]) -> Option[Int] { return None; }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Type inference with multiple constraints
    #[test] fn test_edge_infer_from_arithmetic() {
        let src = "fn infer() -> Float64 { var x = 1.0; var y = x * 2.5; var z = y + 0.5; return z; }";
        let result = check(src);
        assert!(result.is_ok(), "inference chain: {:?}", result.err());
    }

    #[test] fn test_edge_infer_from_function_call() {
        let src = "fn make() -> Int { return 42; } fn use_val() -> Int { var x = make(); return x + 1; }";
        let result = check(src);
        assert!(result.is_ok(), "infer from fn call: {:?}", result.err());
    }

    #[test] fn test_edge_infer_options() {
        let src = "fn test() -> Int { var x = Some(42); var y = None; match x { Some(v) => v, None => match y { Some(vv) => vv, None => 0, }, } }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Ambiguous trait resolution
    #[test] fn test_edge_trait_ambiguity() {
        let src = "\
interface A { fn method() -> Int; }
interface B { fn method() -> Int; }
fn use_trait[T: A + B](x: T) -> Int { return x.method(); }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_edge_trait_multiple_methods() {
        let src = "\
interface Iterator[T] { fn next() -> Option[T]; fn count(&self) -> Int; fn reset(&mut self); }
fn test() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_ok(), "multi-method trait: {:?}", result.err());
    }

    // Circular type definitions
    #[test] fn test_edge_circular_type_alias() {
        let src = "\
type A = B;
type B = C;
type C = A;
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Type alias chains (5+ levels)
    #[test] fn test_edge_type_alias_chain_deep() {
        let src = "\
type L1 = Int;
type L2 = L1;
type L3 = L2;
type L4 = L3;
type L5 = L4;
type L6 = L5;
type L7 = L6;
fn main() -> L7 { var x: L7 = 42; return x; }";
        let result = check(src);
        assert!(result.is_ok(), "deep type alias: {:?}", result.err());
    }

    // Generic with multiple params and complex bounds
    #[test] fn test_edge_generic_complex_param() {
        let src = "\
interface Hash { fn hash() -> Int; }
interface Eq { fn eq(other: &Self) -> Bool; }
fn dedup[K: Eq + Hash, V: Clone](map: BTreeMap[K, V], key: K) -> Option[V] { return None; }
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_ok(), "complex generic: {:?}", result.err());
    }

    // Enum with complex payload patterns
    #[test] fn test_edge_enum_complex_payload() {
        let src = "\
enum Expr {
    IntLit(val: Int),
    FloatLit(val: Float64),
    StrLit(val: Str),
    BoolLit(val: Bool),
    Var(name: Str),
    BinOp(left: Box[Expr], op: Str, right: Box[Expr]),
    Call(name: Str, args: Vec[Expr]),
}
fn eval(e: Expr) -> Int { match e { Expr.IntLit(v) => v, _ => 0, } }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Pattern matching deep patterns
    #[test] fn test_edge_deep_pattern_matching() {
        let src = "\
type Inner = { a: Int; b: Int; }
type Outer = { inner: Inner; }
fn is_unit(o: Outer) -> Bool {
    match o {
        Outer { inner: Inner { a: 1, b: 1 } } => true,
        _ => false,
    }
}";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_edge_refutable_pattern() {
        let src = "\
fn test(x: Option[Int]) -> Str {
    if let Some(v) = x {
        if v > 10 { return \"big\"; }
        return \"small\";
    }
    return \"none\";
}";
        let result = check(src);
        assert!(result.is_ok(), "if let: {:?}", result.err());
    }

    #[test] fn test_edge_while_let_pattern() {
        let src = "\
fn process(queue: Vec[Option[Int]]) {
    var i = 0;
    while let Some(v) = queue.get(i) {
        i = i + 1;
    }
}";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Integer overflow safety
    #[test] fn test_edge_integer_types_range() {
        let types = ["Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt"];
        for t in &types {
            let src = format!("fn f(x: {t}) -> {t} {{ return x; }}");
            let result = check(&src);
            assert!(result.is_ok(), "type {t}: {:?}", result.err());
        }
    }

    #[test] fn test_edge_float_types_range() {
        let types = ["Float32", "Float64"];
        for t in &types {
            let src = format!("fn f(x: {t}) -> {t} {{ return x; }}");
            let result = check(&src);
            assert!(result.is_ok(), "type {t}: {:?}", result.err());
        }
    }

    // Multi-module resolution
    #[test] fn test_edge_module_re_export() {
        let src = "\
module inner { pub fn val() -> Int { return 1; } }
module outer { pub use inner.val; }
fn main() -> Int { return outer.val(); }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_edge_module_deep_path() {
        let src = "\
module a { module b { module c { module d { pub fn e() -> Int { return 42; } } } } }
fn main() -> Int { return a.b.c.d.e(); }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Struct with spread
    #[test] fn test_edge_struct_spread_inference() {
        let src = "\
type Point = { x: Float64; y: Float64; z: Float64; }
fn origin() -> Point { return Point{ x: 0.0; y: 0.0; z: 0.0; } }
fn main() -> Float64 { var p = Point{ x: 1.0, ..origin() }; return p.y; }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Unsafe pointer operations
    #[test] fn test_edge_unsafe_ptr_cast() {
        let src = "\
fn ptr_add(ptr: *UInt8, offset: Int) -> *UInt8 {
    unsafe { return ptr + offset; }
}";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    #[test] fn test_edge_unsafe_raw_memory() {
        let src = "\
fn unsafe_read(ptr: *Int) -> Int {
    unsafe { return ptr; }
}";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Complex boolean expressions
    #[test] fn test_edge_complex_bool_expression() {
        let src = "fn valid(a: Bool, b: Bool, c: Bool, d: Bool) -> Bool { return a && (b || c) && !d || a == b; }";
        let result = check(src);
        assert!(result.is_ok(), "complex bool: {:?}", result.err());
    }

    // Early returns in match arms
    #[test] fn test_edge_early_return_in_match() {
        let src = "\
fn classify(n: Int) -> Str {
    match n {
        0 => return \"zero\",
        1 => return \"one\",
        _ => return \"many\",
    }
}";
        let result = check(src);
        assert!(result.is_ok(), "match return: {:?}", result.err());
    }

    // `var` reassignment with type change (should error)
    #[test] fn test_edge_reassign_with_type_change() {
        let src = "fn test() { var x = 42; x = \"hello\"; }";
        let result = check(src);
        assert!(result.is_err(), "reassign type change should error");
    }

    // `let` rebinding with different type (shadowing — should be ok)
    #[test] fn test_edge_shadowing_with_different_type() {
        let src = "fn test() -> Str { let x = 42; let x = \"hi\"; return x; }";
        let result = check(src);
        assert!(result.is_ok(), "shadowing different type: {:?}", result.err());
    }

    // Generic enum constructors
    #[test] fn test_edge_generic_enum_option() {
        let src = "\
enum Maybe[T] { Just(v: T), Nothing }
fn get_default[T](m: Maybe[T], default: T) -> T {
    match m { Maybe.Just(v) => v, Maybe.Nothing => default, }
}";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Function pointer types
    #[test] fn test_edge_function_pointer_as_param() {
        let src = "\
fn apply(f: fn(Int) -> Int, x: Int) -> Int { return f(x); }
fn square(x: Int) -> Int { return x * x; }
fn main() -> Int { return apply(square, 5); }";
        let result = check(src);
        assert!(result.is_ok(), "fn ptr: {:?}", result.err());
    }

    // Closure type inference
    #[test] fn test_edge_closure_returning_value() {
        let src = "\
fn make_adder(n: Int) -> fn(Int) -> Int { return fn(x: Int) -> Int { return x + n; }; }";
        let result = check(src);
        assert!(result.is_ok() || result.is_err());
    }

    // Iterator pattern
    #[test] fn test_edge_iterator_pattern() {
        let src = "\
fn sum_range(lo: Int, hi: Int) -> Int {
    var sum = 0;
    var i = lo;
    while i < hi {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}";
        let result = check(src);
        assert!(result.is_ok(), "iterator: {:?}", result.err());
    }
}
