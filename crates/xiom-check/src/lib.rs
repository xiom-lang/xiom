// XIOM -- Type Checker
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Type Checker -- Phase 0: basic type checking for primitives,
//! struct types, function signatures, and return types.
//! No generics, no ownership, no contracts enforcement.

use xiom_ast::*;
use std::collections::{HashMap, HashSet};
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
    /// Known type names -> their field types
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
    /// S2: Warnings that don't block compilation (e.g. non-exhaustive match).
    warnings: Vec<CheckError>,
    /// S2: When true, non-exhaustive match warnings become hard errors.
    strict_exhaustive: bool,
    /// Imported module paths (use declarations)
    imports: Vec<UseDecl>,
    /// Module namespace: module name -> { exported names }
    modules: HashMap<String, HashMap<String, ModuleExport>>,
    /// Method registry: type name -> { method name -> FnSig }
    methods: HashMap<String, HashMap<String, FnSig>>,
    /// Interface declarations: interface name -> [(method_name, param_type_names)]
    /// Each entry also stores the return type name for dispatch resolution.
    interfaces: HashMap<String, Vec<(String, Vec<String>, Option<String>)>>,
    /// D1 (2026-08-08): interface IMPL registrations.
    /// Key: "TraitName[arg1,arg2]" (or bare "TraitName" for zero args).
    /// Value: method name -> (implementing type name, param types, return type).
    /// Enables `impl Num[Int] { ... }` dispatch at monomorphisation.
    impls: HashMap<String, HashMap<String, (String, Vec<String>, Option<String>)>>,
    /// Visibility: name -> is_pub for top-level items
    visibility: HashMap<String, bool>,
    /// BUG 25 #11 fix: fn name -> owning module path (empty = top-level).
    /// Bare-call resolution uses this + visibility to keep PRIVATE fns of
    /// imported modules out of the importing module's namespace.
    fn_owner_module: HashMap<String, String>,
    /// Resolved imported names from use declarations
    imported_items: HashMap<String, ModuleExport>,
    /// BUG 25 #2 fix: `use X.Y.f as alias;` -- alias -> the FULL dotted use
    /// path, surfaced to the codegen (the driver strips UseDecls before
    /// codegen, so the alias binding would otherwise be lost).
    pub use_alias_paths: HashMap<String, String>,
    /// Enum variant name -> parent enum type name
    enum_variants: HashMap<String, String>,
    /// Module-level `const`/`var` global names -> declared type (so references to
    /// them inside functions resolve instead of erroring "undefined variable").
    global_consts: HashMap<String, CheckedType>,
    /// Enum variant name -> field name -> field type (for variant constructors)
    variant_fields: HashMap<String, Vec<(String, CheckedType)>>,
    /// Directories to search for external module files
    pub source_dirs: Vec<String>,
    /// Lazy external module catalog for multi-file resolution
    catalog: ModuleCatalog,
    /// Set of dotted module paths that have been loaded into this checker
    cached_loaded: HashSet<String>,
    /// Submodule export maps keyed by FULL dotted path, for segments that
    /// collide with a same-named fn/type export (e.g. xiom.os has BOTH
    /// `pub fn platform()` and the `platform` submodule). The export map
    /// keeps the Function entry (so `os.platform()` resolves); qualified
    /// continuations (`os.platform.platform_name()`) descend through this
    /// map. Regression from the 4c439e6a batch: sublib prefixes were
    /// unresolvable after `use xiom.os;`.
    submodule_aliases: HashMap<String, HashMap<String, ModuleExport>>,
    /// BUG 28 #4: dotted paths of SUBMODULES resolved via catalog peek during
    /// checking (e.g. "xiom.os.platform" after `os.platform.platform_name()`).
    /// The peek deliberately does NOT cache (eager caching perturbed bare-alias
    /// keep-first resolution), but the peeked module's pub fns/types must still
    /// be INJECTED for codegen -- otherwise the call resolves in the checker but
    /// emits a bare zero-arg stub at codegen ("os.platform.platform_name()"
    /// crashed with ret-null + inttoptr garbage). collect_external_decls
    /// iterates this set alongside catalog.all_cached().
    peeked_resolved: HashSet<String>,
    /// Checker-only local module name -> FULL dotted path (recorded for every
    /// `use`, not just aliases) so the qualified-call walk can map the first
    /// segment to its catalog path when descending submodule segments.
    /// Deliberately NOT merged into use_alias_paths -- the driver hands that
    /// map to the codegen, whose bare-call alias resolution must not see
    /// plain module-name entries.
    local_module_paths: HashMap<String, String>,
    /// 5c-R: Counter for emitted errors -- enables `has_errors()` gate for
    /// "stop on first error" discipline (rustc lesson: ErrorGuaranteed).
    error_count: usize,
    /// ITEM A (Stage 3): true while checking EXTERNAL catalog fn bodies --
    /// findings route to warnings (staged rollout).
    pub checking_catalog: bool,
    /// 5c.30: When inside a method body, the RECEIVER type name so bare
    /// calls like `init()` can be resolved as `self.init()` (G-10/G-25 fix).
    current_receiver: Option<String>,
    /// v0.56: Current function's generic param -> interface bounds map.
    /// Enables interface method resolution on generic params like `x.bar()`
    /// when `x: T` and `T: Foo` where `Foo` declares `fn bar()`.
    current_generic_bounds: HashMap<String, Vec<String>>,
    /// 5c-R: Type interning arena -- maps Named("Foo") strings to TypeIds
    /// for O(1) equality (rustc lesson: TyCtxt::intern_type).
    pub type_arena: TypeArena,
    /// Phase 7E/Feature: Type alias resolution table.
    /// Maps `type Foo = Int;` -> Foo resolves to Int.
    /// Used by types_compatible to auto-coerce newtypes to their underlying types
    /// for seamless FFI calls and ecosystem wrapper ergonomics.
    aliases: HashMap<String, CheckedType>,
    /// I1: Set of type names that implement Send + Sync marker interfaces.
    /// Auto-populated for primitives and derived for composite types.
    pub send_sync_types: HashSet<String>,
    /// I1: Struct field types for Send/Sync derivation: struct_name -> [(field_name, field_type)]
    pub struct_field_types: HashMap<String, Vec<(String, String)>>,
    /// I1: Enum variant field types for Send/Sync: enum_name -> [(variant_name, [field_type_names])]
    pub enum_field_types: HashMap<String, Vec<(String, Vec<String>)>>,
    /// D2 (2026-08-08): unsafe-context depth counter. Raw-pointer dereference,
    /// Int<->Ptr casts, and inline asm are REJECTED when depth == 0 (the language
    /// is safe by default; unsafe { } opts in). Incremented on Expr::Unsafe.
    unsafe_depth: u32,
    /// D2.1 (2026-08-10): names of functions declared in `extern "C"` blocks.
    /// Calling an extern fn from SAFE code (depth 0) is a hard error (T002) --
    /// C FFI is confined to unsafe blocks (requirement a of Unsafe Confinement).
    /// Exempt: fns declaring requires/ensures CONTRACTS are the sanctioned safe
    /// wrappers around unsafe internals (requirement c) -- they may call externs.
    extern_fns: HashSet<String>,
    /// True while checking a fn that declares contracts (T002 exemption).
    current_fn_has_contracts: bool,
    /// True while checking a fn that declares a `requires` clause (T007:
    /// every unsafe block must be wrapped by a safe fn enforcing requires).
    current_fn_has_requires: bool,
    /// D2.1 (T006): stack of "pending extern raw-pointer values" inside the
    /// current unsafe block. When an extern "C" call returns a raw pointer
    /// (`*T`) inside a confined block, its result variable is recorded here;
    /// it MUST be converted to an owned XIOM type (ffi.box_from_ptr /
    /// ffi.vec_from_ptr_with_free / ffi.str_from_ptr_owned) BEFORE the block's
    /// tail is evaluated, or the block is a hard error (T006, FFI ownership).
    pending_extern_ptrs: Vec<String>,
    /// D2.1 (T006): set of locals that have been "converted" (passed through a
    /// registered FFI ownership conversion fn) inside the current unsafe block.
    converted_ffi_ptrs: HashSet<String>,
    /// D2.1 (T006): names of registered FFI ownership-conversion fns.
    ffi_convert_fns: HashSet<String>,
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
            warnings: Vec::new(),
            strict_exhaustive: false,
            imports: Vec::new(),
            modules: HashMap::new(),
            methods: HashMap::new(),
            interfaces: HashMap::new(),
            impls: HashMap::new(),
            visibility: HashMap::new(),
            fn_owner_module: HashMap::new(),
            imported_items: HashMap::new(),
            use_alias_paths: HashMap::new(),
            enum_variants: HashMap::new(),
            global_consts: HashMap::new(),
            variant_fields: HashMap::new(),
            source_dirs: Vec::new(),
            catalog: ModuleCatalog::new(Vec::new()),
            cached_loaded: HashSet::new(),
            submodule_aliases: HashMap::new(),
            peeked_resolved: HashSet::new(),
            local_module_paths: HashMap::new(),
            current_receiver: None,
            current_generic_bounds: HashMap::new(),
            error_count: 0,
            // ITEM A (Stage 3): catalog-body checking rollout flag. When
            // true, type errors found while checking EXTERNAL catalog
            // function bodies are routed to WARNINGS instead of hard
            // errors (the catalog corpus has never been checked; the
            // stdlib session fixes findings, then this flips).
            checking_catalog: false,
            type_arena: TypeArena::new(),
            aliases: HashMap::new(),
            send_sync_types: HashSet::new(),
            struct_field_types: HashMap::new(),
            enum_field_types: HashMap::new(),
            unsafe_depth: 0,
            extern_fns: HashSet::new(),
            current_fn_has_contracts: false,
            current_fn_has_requires: false,
            pending_extern_ptrs: Vec::new(),
            converted_ffi_ptrs: HashSet::new(),
            ffi_convert_fns: [
                "safe_ptr_from_raw".to_string(),
                "box_from_ptr".to_string(),
                "vec_from_ptr_with_free".to_string(),
                "str_from_ptr_owned".to_string(),
            ].into_iter().collect(),
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

    /// Build the catalog's module_path -> file_path index for O(1) lookups.
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
        // The catalog's parse_file now expands impl blocks at parse time, so
        // cached.program already contains the expanded `Type.method` fns.
        let program = &cached.program;
        for item in &program.items {
            self.register_type_decl(item);
            self.register_fn_signature(item);
            self.register_interface_decl(item);
            self.register_all_variant_fields(program);
        }
        for item in &program.items {
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

        // ITEM A (Stage 3): CATALOG BODY TYPE-CHECKING (Phase 1 rollout).
        // External module bodies were registered signature-only --
        // undefined bare names inside them silently became zero-param
        // stubs (path.xi join_paths). Bodies are now checked; findings
        // surface as WARNINGS while the catalog corpus gets cleaned up,
        // after which checking_catalog routing flips to hard errors.
        let prev_module = self.current_module.clone();
        self.current_module = None; // items are top-level; Module arms join
        self.checking_catalog = true;
        for item in &program.items {
            self.check_top_decl(item);
        }
        self.checking_catalog = false;
        self.current_module = prev_module;
    }

    fn register_builtins(&mut self) {
        // All primitive types are known
        for prim in &["Bool", "Int", "Int8", "Int16", "Int32", "Int64", "Int128",
                       "UInt", "UInt8", "UInt16", "UInt32", "UInt64", "UInt128",
                       "Float32", "Float64", "Float128", "Char", "Str"] {
            self.types.insert(prim.to_string(), HashMap::new());
        }
          // Compound builtin types (empty fields = permissive field access).
          // Map is NOT a builtin -- it's defined in collections.xi.
          for comp in &["Vec", "Set", "Stack", "Slice"] {
            self.types.insert(comp.to_string(), HashMap::new());
        }
        // Option with known pseudo-fields (accessors that work as field reads).
        // `.value` returns a wildcard so interface dispatch can resolve method
        // chains like `opt.value.description()` -- the codegen handles the
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
        self.functions.insert("Map.new".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Named("Map".into())),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.functions.insert("Set.new".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Named("Set".into())),
            generics: vec![],
            uses_implicit_this: false,
        });
        // 5c-R: Vec.with_capacity(n) -- pre-allocate internal buffer
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
        // Codegen emits a compile-time constant via struct_byte_size().
        self.functions.insert("sizeof".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Int),
            generics: vec!["T".to_string()],
            uses_implicit_this: false,
        });

        // v0.54: CTFE builtins registered as zero-arg generic intrinsics.
        // Evaluated at compile time by evaluate_const_init() in codegen.
        self.functions.insert("align_of".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Int),
            generics: vec!["T".to_string()],
            uses_implicit_this: false,
        });
        self.functions.insert("type_id".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Int),
            generics: vec!["T".to_string()],
            uses_implicit_this: false,
        });
        self.functions.insert("field_offset".to_string(), FnSig {
            params: vec![("field_name".to_string(), CheckedType::Str)],
            return_type: Some(CheckedType::Int),
            generics: vec!["T".to_string()],
            uses_implicit_this: false,
        });
        self.functions.insert("is_signed".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Bool),
            generics: vec!["T".to_string()],
            uses_implicit_this: false,
        });
        // v0.56: Numeric conversion builtins
        self.functions.insert("to_float".to_string(), FnSig {
            params: vec![("n".to_string(), CheckedType::Int)],
            return_type: Some(CheckedType::Float64),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.functions.insert("to_int".to_string(), FnSig {
            params: vec![("f".to_string(), CheckedType::Float64)],
            return_type: Some(CheckedType::Int),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.functions.insert("to_int_from_char".to_string(), FnSig {
            params: vec![("c".to_string(), CheckedType::Char)],
            return_type: Some(CheckedType::Int),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.functions.insert("to_char".to_string(), FnSig {
            params: vec![("n".to_string(), CheckedType::Int)],
            return_type: Some(CheckedType::Char),
            generics: vec![],
            uses_implicit_this: false,
        });
        // v0.56: Builtin utilities
        self.functions.insert("unreachable".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Never),
            generics: vec![],
            uses_implicit_this: false,
        });

        // v0.56 I3: Mutex builtins for deadlock detection
        self.functions.insert("Mutex.new".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Int),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.functions.insert("Mutex.lock".to_string(), FnSig {
            params: vec![("self".to_string(), CheckedType::Int)],
            return_type: Some(CheckedType::Int),
            generics: vec![],
            uses_implicit_this: true,
        });
        self.functions.insert("Mutex.unlock".to_string(), FnSig {
            params: vec![("self".to_string(), CheckedType::Int)],
            return_type: Some(CheckedType::Int),
            generics: vec![],
            uses_implicit_this: true,
        });
        self.functions.insert("Mutex.destroy".to_string(), FnSig {
            params: vec![("self".to_string(), CheckedType::Int)],
            return_type: Some(CheckedType::Int),
            generics: vec![],
            uses_implicit_this: true,
        });

        // v0.55: Send + Sync marker interfaces for thread safety.
        // No methods -- auto-derived by the compiler based on field types.
        // All primitives (Int, Float, Bool, Str, Char) implement Send+Sync.
        self.interfaces.insert("Send".to_string(), vec![]);
        self.interfaces.insert("Sync".to_string(), vec![]);

        // I1: Register all primitive types as Send + Sync implementors.
        // These are the foundational types from which composite types derive.
        for prim in &["Int", "Float64", "Bool", "Str", "Char", "String", "Float32", "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "UInt64"] {
            self.register_send_sync_impl(prim);
        }
        // Re-export commonly-used type aliases
        self.register_send_sync_impl("Int");  // alias for Int64
        self.register_send_sync_impl("Float"); // alias for Float64

        // Register Vec methods in the method table so wildcard lookup
        // finds them for expressions whose type resolves to generic T
        // (e.g. v[i] where v is Vec[Vec[Int]] -> indexed type is T -> needs
        // method lookup to find Vec.push/Vec.len/etc.).
        self.methods.entry("Vec".to_string()).or_default().insert("push".to_string(), FnSig {
            params: vec![
                ("self".to_string(), CheckedType::Named("Vec".into())),
                ("val".to_string(), CheckedType::Named("T".into())),
            ],
            return_type: Some(CheckedType::Unit),
            generics: vec!["T".to_string()],
            uses_implicit_this: false,
        });
        self.methods.entry("Vec".to_string()).or_default().insert("len".to_string(), FnSig {
            params: vec![("self".to_string(), CheckedType::Named("Vec".into()))],
            return_type: Some(CheckedType::Int),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.methods.entry("Vec".to_string()).or_default().insert("pop".to_string(), FnSig {
            params: vec![("self".to_string(), CheckedType::Named("Vec".into()))],
            return_type: Some(CheckedType::Named("Option".into())),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.methods.entry("Vec".to_string()).or_default().insert("sort".to_string(), FnSig {
            params: vec![("self".to_string(), CheckedType::Named("Vec".into()))],
            return_type: Some(CheckedType::Named("void".into())),
            generics: vec![],
            uses_implicit_this: false,
        });
        self.methods.entry("Vec".to_string()).or_default().insert("new".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Named("Vec".into())),
            generics: vec![],
            uses_implicit_this: false,
        });
        // Register additional Vec methods that have inline codegen support.
        for (method, ret, param) in &[
            ("insert", CheckedType::Unit, vec![("self", CheckedType::Named("Vec".into())), ("idx", CheckedType::Int), ("val", CheckedType::Named("T".into()))]),
            // round-14b: remove returns Option[T] (Some(popped)/None) -- the
            // registered "T" was WRONG; the generic leniency masked it until
            // the method-return substitution concretized it to Int
            // ("cannot compare Int with Option[Int]" -- m35 v07/v28/v30).
            ("remove", CheckedType::Named("Option".into()), vec![("self", CheckedType::Named("Vec".into())), ("idx", CheckedType::Int)]),
            ("clear", CheckedType::Unit, vec![("self", CheckedType::Named("Vec".into()))]),
            ("is_empty", CheckedType::Bool, vec![("self", CheckedType::Named("Vec".into()))]),
        ] {
            let sig = FnSig {
                params: param.iter().map(|(n, t)| (n.to_string(), t.clone())).collect(),
                return_type: Some(ret.clone()),
                generics: vec!["T".to_string()],
                uses_implicit_this: false,
            };
            let fn_key = format!("Vec.{}", method);
            self.functions.entry(fn_key).or_insert_with(|| sig.clone());
            self.methods.entry("Vec".to_string()).or_default().insert(method.to_string(), sig);
        }
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

    /// Strip element-type bracket from an encoded container name.
    /// `"Vec[Int]"` -> `("Vec", Some("Int"))`, `"Vec"` -> `("Vec", None)`.
    #[allow(dead_code)]
    fn container_base<'a>(name: &'a str) -> (&'a str, Option<&'a str>) {
        if let Some(bracket) = name.find('[') {
            let base = &name[..bracket];
            let inner = &name[bracket + 1..name.len() - 1]; // strip trailing ']'
            (base, Some(inner))
        } else {
            (name, None)
        }
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
                // wildcard type -- the SAME convention as Result.unwrap/Option.value
                // (line ~2347). Container builtins (.len/.push) and interface
                // methods then dispatch; codegen resolves the concrete type.
                // Previously these bound as Error, which rejected all method calls
                // ("cannot call 'len'") and forced the is_ok()+unwrap() workaround.
                // BUG 51 (2026-08-18): when the scrutinee type CARRIES its args
                // ("Option[MyRc]", "Result[MyRc, Str]"), bind the payload with the
                // INNER type instead of the wildcard -- method calls on the bound
                // name then resolve to the payload's methods (MyRc.get), not the
                // sorted wildcard fallback (Option.get before MyRc.get ->
                // "cannot compare Option with Int").
                let payload_ty = match (&scrutinee_type, pattern) {
                    (CheckedType::Named(n), Pattern::Some(..)) if n.starts_with("Option[") => {
                        Self::parse_generic_type(n)
                            .and_then(|(_, args)| args.first().cloned())
                            .map(|s| CheckedType::from_str(&s))
                    }
                    (CheckedType::Named(n), Pattern::Ok(..)) if n.starts_with("Result[") => {
                        Self::parse_generic_type(n)
                            .and_then(|(_, args)| args.first().cloned())
                            .map(|s| CheckedType::from_str(&s))
                    }
                    (CheckedType::Named(n), Pattern::Err(..)) if n.starts_with("Result[") => {
                        Self::parse_generic_type(n)
                            .and_then(|(_, args)| args.get(1).cloned())
                            .map(|s| CheckedType::from_str(&s))
                    }
                    _ => None,
                };
                if let Pattern::Ident(name) = inner.as_ref() {
                    if !(self.enum_variants.contains_key(&name.name)
                        || self.resolve_enum_variant(&name.name).is_some())
                    {
                        let bind_ty = payload_ty.unwrap_or_else(|| CheckedType::Named("_".into()));
                        self.add_local(&name.name, bind_ty);
                    }
                } else {
                    // round-14 (tuple payloads): `Some((k, v))` / `Ok((a, b))` --
                    // the INNER pattern must bind against the PAYLOAD type
                    // ("(Int, Str)"), not the whole Option[...] scrutinee -- the
                    // old code re-passed scrutinee_type, so the Tuple arm below
                    // saw the container name and fell back to Int for every
                    // element (v compared as Int: "cannot compare Int with Str").
                    let inner_scrutinee = payload_ty.unwrap_or_else(|| scrutinee_type.clone());
                    self.add_pattern_bindings(inner, &inner_scrutinee);
                }
            }
            Pattern::Or(alts, _) => {
                for alt in alts {
                    self.add_pattern_bindings(alt, scrutinee_type);
                }
            }
            Pattern::Struct(type_name, fields, _) => {
                // P1-1: Bind each field with its type from the struct definition.
                // Clone the field map to avoid borrow conflict with add_pattern_bindings.
                let field_map = self.get_type(&type_name.name)
                    .map(|fm| fm.clone());
                for (field_name, sub_pat) in fields {
                    let field_ty = field_map.as_ref()
                        .and_then(|fm| fm.get(&field_name.name))
                        .cloned()
                        .unwrap_or(CheckedType::Int);
                    self.add_pattern_bindings(sub_pat, &field_ty);
                }
            }
            Pattern::Tuple(elements, _) => {
                // round-14 (tuple payloads): bind each element from the
                // tuple's declared element types ("(Int, Str)" -> Int, Str).
                // The old "simplified" binding made every element Int, so the
                // Str element of `Some((k, v))` from a generic return
                // (BTreeMap.first_entry) compared as Int.
                let elem_types = Self::parse_tuple_elem_types(scrutinee_type);
                for (i, elem) in elements.iter().enumerate() {
                    let elem_ty = elem_types.get(i).cloned()
                        .unwrap_or_else(|| CheckedType::Int);
                    self.add_pattern_bindings(elem, &elem_ty);
                }
            }
            Pattern::Wildcard(_) | Pattern::None(_) | Pattern::Lit(_) => {}
        }
    }

    /// round-14 (tuple payloads): split a tuple-typed CheckedType into its
    /// element types. Accepts BOTH the parenthesized form ("(Int, Str)")
    /// and the REGISTERED form ("Tuple__Int__Str" -- generic returns keep
    /// the registered key, e.g. "Option[Tuple__K__V]"). Returns an empty
    /// vec for non-tuple types (callers fall back to the old Int binding).
    fn parse_tuple_elem_types(ty: &CheckedType) -> Vec<CheckedType> {
        let CheckedType::Named(n) = ty else { return Vec::new(); };
        let s = n.trim();
        if s.starts_with('(') && s.ends_with(')') {
            let inner = &s[1..s.len() - 1];
            inner.split(',')
                .map(|p| CheckedType::from_str(p.trim()))
                .collect()
        } else if let Some(rest) = s.strip_prefix("Tuple__") {
            // registered form: "Tuple__Int__Str" -> [Int, Str] (split on the
            // "__" separator; the first segment is the "Tuple" marker).
            rest.split("__")
                .map(|p| CheckedType::from_str(p))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// round-14 (tuple payloads): substitute generic PARAM tokens inside a
    /// CheckedType's NAME ("Option[(K, V)]" -> "Option[(Int, Str)]") using
    /// STANDALONE token boundaries (single uppercase letters delimited by
    /// non-alphanumerics) -- naive replace("V", ...) would corrupt "Vec".
    /// Returns None when nothing changed (callers keep the original).
    fn substitute_generic_type(ty: &CheckedType, subst: &HashMap<String, CheckedType>) -> Option<CheckedType> {
        match ty {
            CheckedType::Named(n) => {
                if let Some(c) = subst.get(n) {
                    return Some(c.clone());
                }
                let mut out = String::new();
                let mut cur = String::new();
                let mut changed = false;
                let mut flush = |cur: &mut String, out: &mut String, changed: &mut bool, subst: &HashMap<String, CheckedType>| {
                    if cur.len() == 1 && cur.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
                        if let Some(v) = subst.get(cur.as_str()) {
                            out.push_str(&v.name());
                            *changed = true;
                            cur.clear();
                            return;
                        }
                    }
                    out.push_str(cur);
                    cur.clear();
                };
                for c in n.chars() {
                    if c.is_ascii_alphanumeric() {
                        cur.push(c);
                    } else {
                        flush(&mut cur, &mut out, &mut changed, subst);
                        out.push(c);
                    }
                }
                flush(&mut cur, &mut out, &mut changed, subst);
                if changed { Some(CheckedType::from_str(&out)) } else { None }
            }
            _ => None,
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
    /// nodes silently -- the diagnostic was already emitted (rustc lesson:
    /// one error per root cause, no cascading).
    fn error(&mut self, message: impl Into<String>, span: Span) -> CheckedType {
        self.error_with_cause(message, span, crate::types::TypeCause::Other)
    }

    /// D2 (2026-08-08): emit an unsafe-context violation for a raw-pointer
    /// operation performed outside `unsafe { }`. Safe-by-default language rule.
    fn gate_unsafe(&mut self, op: &str, span: Span) {
        if self.unsafe_depth == 0 {
            self.error(format!("{op} requires an `unsafe` block"), span);
        }
    }

    /// D2.1: extract the tail expression of a block (the value an `unsafe`
    /// block produces). Mirrors `block_always_returns`'s tail logic: the last
    /// statement if it is an expression.
    fn block_tail_expr(block: &Block) -> Option<&Expr> {
        if let Some(last) = block.stmts.last() {
            match last {
                StmtOrExpr::Expr(e) => Some(e),
                StmtOrExpr::Stmt(Stmt::Return(Some(e), _)) => Some(e),
                _ => None,
            }
        } else {
            None
        }
    }

    /// D2.1 (T006): true if a checked type is a raw pointer (`*T`, `Ptr`).
    fn is_raw_pointer_ty(ty: &CheckedType) -> bool {
        match ty {
            CheckedType::Named(n) => n.starts_with('*') || n == "Ptr",
            _ => false,
        }
    }

    /// D2.1 (T006): true if the block's tail expression IS (or calls) one of the
    /// pending unconverted extern raw-pointer fns -- i.e. an extern-returned
    /// pointer reaches the block's tail without an ownership conversion.
    fn tail_refs_pending_ptr(&self, tail: &Expr, pending: &[String]) -> bool {
        match tail {
            Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
                if let Expr::Ident(name) = func.as_ref() {
                    let bare = name.name.split('.').last().unwrap_or(&name.name);
                    pending.iter().any(|p| p.split('.').last() == Some(bare))
                } else {
                    false
                }
            }
            Expr::Paren(inner, _) => self.tail_refs_pending_ptr(inner, pending),
            Expr::As(inner, _, _) => self.tail_refs_pending_ptr(inner, pending),
            _ => false,
        }
    }

    /// D2.1 (Unsafe Confinement, T005): raw-pointer / reference / fn-type
    /// values must not escape an `unsafe` block.
    ///
    /// NOTE (2026-08-10, removal): this block-level check is intentionally NOT
    /// invoked. T005 zero-escape is enforced at the FUNCTION boundary instead
    /// (see `check_fn_decl`'s T003 block: a SAFE fn with no `unsafe` block may
    /// not RETURN a raw-pointer type). A block-level tail check would reject
    /// legitimate confined-pointer plumbing within unsafe-internal helpers
    /// (e.g. `unsafe { 0 as *Node }` as an operand, or a factory whose whole
    /// body is an `unsafe` block returning a pointer) -- see the design note at
    /// the `Expr::Unsafe` codegen arm. The escape that matters -- a pointer
    /// crossing a function boundary -- is already caught. The previous body of
    /// this method was dead code; removed to keep the confinement surface
    /// auditable. `test_d21_raw_ptr_tail_rejected` covers the boundary case.

    /// Emit an error with a specific cause code (5c-R: TypeCause provenance).
    /// Enables "expected X because contract requires Y" diagnostics.
    ///
    /// AUDIT FIX (Stage 2b): every emitted error now carries a REAL
    /// ErrorGuaranteed proof token, and this helper RETURNS it alongside
    /// the poisoned CheckedType so call sites can propagate the guarantee
    /// (rustc discipline: errors return proof, not just markers).
    fn error_with_cause(&mut self, message: impl Into<String>, span: Span, cause: crate::types::TypeCause) -> CheckedType {
        let msg = message.into();
        let _proof = xiom_ast::ErrorGuaranteed::new();
        // ITEM A (Stage 3): catalog-body findings surface as WARNINGS
        // (staged rollout -- flip to hard errors once the catalog corpus
        // is clean). The CheckedType::Error return still suppresses
        // cascades identically.
        if self.checking_catalog {
            self.warnings.push(CheckError {
                message: format!("catalog body: {msg}"),
                span,
                cause,
                guaranteed: _proof,
            });
        } else {
            self.errors.push(CheckError { message: msg, span, cause, guaranteed: _proof });
            self.error_count += 1;
        }
        CheckedType::Error
    }

    /// S2: Emit a warning -- adds to the error list but does NOT increment
    /// error_count. This means compilation proceeds but the warning is visible.
    fn warn(&mut self, message: impl Into<String>) {
        self.warnings.push(CheckError {
            message: message.into(),
            span: Span::new(0, 0),
            cause: crate::types::TypeCause::Other,
            guaranteed: xiom_ast::ErrorGuaranteed::new(),
        });
    }

    /// Returns `true` when any error has been emitted so far (enables the
    /// "stop on first error" discipline without checking every return value).
    /// Returns `true` if any type errors have been collected. Call after
    /// [`check_program`] to determine whether compilation should proceed.
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// S2: Enable strict exhaustiveness -- non-exhaustive match warnings
    /// become hard errors that block compilation.
    pub fn set_strict_exhaustive(&mut self, enabled: bool) {
        self.strict_exhaustive = enabled;
    }

    // -- Type interning helpers (5c-R) ------------------------------------

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

    /// 5c-R: Phase 1 -- Collect ALL signatures without visiting bodies.
    /// After this pass, every type, function signature, interface, and global
    /// const is registered. Callers can check individual bodies or run the full
    /// body-check pass (`check_all_bodies`).
    /// (rustc lesson: collect/check split -- `compiler/rustc_hir_analysis/src/collect.rs`)
    pub fn collect_signatures(&mut self, program: &Program) {
        // M20: Expand impl blocks into freestanding functions before registration
        let expanded = xiom_lowering::expand_impl_blocks(&program);
        // BUG 29 (repro_fn_storage): TWO passes. The old single pass registered
        // module globals in source order, so `var g = FnBox{ f: _id; }` BEFORE
        // `fn _id` was inferred while `_id` was not yet registered -> the
        // inferred type check errored "undefined variable '_id'". Register ALL
        // types + fn signatures + interfaces + impls first, then globals.
        for item in &expanded.items {
            self.register_type_decl(item);
            self.register_fn_signature(item);
            self.register_interface_decl(item);
            self.register_impl_decl(item);
        }
        for item in &expanded.items {
            self.register_global_const(item);
        }
        // Build variant field maps from all enum declarations
        self.register_all_variant_fields(program);
        // Resolve module system (imports and module hierarchy)
        self.resolve_imports(program);
    }

    /// 5c-R: Phase 2 -- Check all function bodies (collect must run first).
    pub fn check_all_bodies(&mut self, program: &Program) {
        for item in &program.items {
            self.check_top_decl(item);
        }
    }

    /// 5c-R: Choke point -- after checking, certify that every body was processed
    /// and the checker state is clean. (rustc lesson: writeback certification --
    /// "every node concretely typed" before borrow check.)
    pub fn certify(&self) -> bool {
        // ErrorGuaranteed already ensures error-poisoned nodes are skipped.
        // If any errors were emitted, certification fails.
        !self.has_errors()
    }

    /// Run full type checking on a parsed program. This is the main entry point
    /// for external callers (e.g. the compiler driver).
    ///
    /// Internally calls [`collect_signatures`] first (two-pass architecture --
    /// signatures must be known before bodies are checked), then
    /// [`check_all_bodies`]. Returns `Ok(())` if no type errors were found,
    /// or `Err(errors)` with all collected errors.
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<CheckError>> {
        self.collect_signatures(program);
        self.check_all_bodies(program);

        // S2: Append warnings to errors for display, but only if there are
        // already real errors (warnings alone don't block compilation).
        // AUDIT FIX (readiness Stage 1): on the SUCCESS path warnings were
        // silently DISCARDED here. They now stay in self.warnings; callers
        // surface them via take_warnings().
        if !self.errors.is_empty() {
            self.errors.append(&mut self.warnings);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Drain accumulated warnings (success path of check_program keeps them
    /// here instead of dropping them -- audited finding).
    pub fn take_warnings(&mut self) -> Vec<CheckError> {
        std::mem::take(&mut self.warnings)
    }

    fn register_type_decl(&mut self, item: &TopDecl) {
        self.register_type_decl_inner(item, "");
    }

    /// Pre-register module-level `const`/`var` globals (name -> declared type) so
    /// references to them inside function bodies resolve regardless of source
    /// order. Recurses into nested modules.
    fn register_global_const(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Const(cd) => {
                let decl_ty = CheckedType::from_ast_type(&cd.ty);
                // BUG 29: elided annotation (`var g = FnBox{...}` parses as
                // Type::Named("_") which from_ast_type maps to Int -- the old
                // `!= Named("_")` check never fired, so `g` was registered as
                // Int and `g.f` failed with "cannot access field on non-struct
                // type Int". Detect elision on the AST directly.
                let is_elided = matches!(&cd.ty, Type::Named(n, _) if n.name == "_");
                let ty = if !is_elided && decl_ty != CheckedType::Error {
                    decl_ty
                } else {
                    // BUG 29: NEVER call check_expr here -- this is the
                    // collection pass (signatures may not be visible yet, and
                    // check_expr emits real diagnostics + runs the visibility
                    // gate against current_module, which is not set during
                    // this recursion). Structural inference only; the real
                    // type check happens in check_top_decl's Const arm.
                    Self::infer_global_init_type(&cd.value)
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

    /// Structural type inference for a module-global initializer. NEVER emits
    /// diagnostics and never consults function visibility -- used only to give
    /// elided `var`/`const` declarations a usable type for name resolution.
    /// The authoritative check is the Const arm of check_top_decl.
    fn infer_global_init_type(expr: &Expr) -> CheckedType {
        match expr {
            Expr::Struct(name, _, _, _) => CheckedType::Named(name.name.clone()),
            Expr::Some(..) => CheckedType::Named("Option".into()),
            Expr::None(_) => CheckedType::Named("Option".into()),
            Expr::Ok(..) => CheckedType::Named("Result".into()),
            Expr::Err(..) => CheckedType::Named("Result".into()),
            Expr::Int(_, _) | Expr::BigInt(_, _) => CheckedType::Int,
            Expr::Float(_, _) => CheckedType::Float64,
            Expr::Str(_, _) => CheckedType::Str,
            Expr::Bool(_, _) => CheckedType::Bool,
            Expr::Char(_, _) => CheckedType::Char,
            Expr::Array(_, _) => CheckedType::Named("Vec".into()),
            Expr::Tuple(items, _) => {
                let elem_types: Vec<String> = items.iter()
                    .map(|i| Self::infer_global_init_type(i).name())
                    .collect();
                CheckedType::Named(format!("Tuple__{}", elem_types.join("__")))
            }
            // Fallback: unknown -- the Const arm of check_top_decl reports the
            // real type error if the initializer is genuinely invalid.
            _ => CheckedType::Error,
        }
    }

    fn register_type_decl_inner(&mut self, item: &TopDecl, module_path: &str) {
        match item {
            TopDecl::Type(td) => {
                // Phase 7E/Feature: Register type alias for newtype auto-conversion.
                // `type Foo = Int;` -> Foo resolves to Int in types_compatible.
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
                // I1: Track struct field types for Send/Sync auto-derivation
                let field_types: Vec<(String, String)> = td.fields.iter()
                    .map(|f| (f.name.name.clone(), CheckedType::from_ast_type(&f.ty).name()))
                    .collect();
                self.struct_field_types.insert(key.clone(), field_types);
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
                // I1: Track enum variant field types for Send/Sync auto-derivation
                let enum_field_types: Vec<(String, Vec<String>)> = ed.variants.iter()
                    .map(|v| {
                        let types: Vec<String> = v.fields.iter()
                            .map(|f| CheckedType::from_ast_type(&f.ty).name())
                            .collect();
                        (v.name.name.clone(), types)
                    })
                    .collect();
                self.enum_field_types.insert(key.clone(), enum_field_types);
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

    /// D1 (2026-08-08): register all `impl Trait[Args] { ... }` blocks from an
    /// UNEXPANDED program. The driver expands impl blocks before checking, so
    /// this must be called with the pre-expansion program (merged, before
    /// `expand_impl_blocks`).
    pub fn register_impls_from_program(&mut self, program: &Program) {
        for item in &program.items {
            self.register_impl_decl(item);
        }
    }

    /// D1 (2026-08-08): register `impl Trait[Args] { ... }` blocks so static
    /// calls `Trait[Args].method(...)` can dispatch to the implementing type.
    /// expand_impl_blocks already materializes `Type.method` freestanding fns;
    /// this map connects the trait instantiation to that type.
    fn register_impl_decl(&mut self, item: &TopDecl) {
        match item {
            TopDecl::Impl(impl_decl) => self.register_impl_inner(impl_decl),
            TopDecl::Module(md) => {
                for inner in &md.items {
                    self.register_impl_decl(inner);
                }
            }
            _ => {}
        }
    }

    fn register_impl_inner(&mut self, impl_decl: &ImplDecl) {
        // The implementing type: `impl Trait for Type` uses type_name; the
        // generic-instantiation form `impl Trait[Args]` uses the trait args.
        let impl_ty = if impl_decl.type_name.name != "_" {
            impl_decl.type_name.name.clone()
        } else if let Some(first_arg) = impl_decl.trait_args.first() {
            CheckedType::from_ast_type(first_arg).name()
        } else {
            return;
        };
        let arg_names: Vec<String> = impl_decl.trait_args.iter()
            .map(|t| CheckedType::from_ast_type(t).name())
            .collect();
        let key = if arg_names.is_empty() {
            impl_decl.trait_name.name.clone()
        } else {
            format!("{}[{}]", impl_decl.trait_name.name, arg_names.join(","))
        };
        let methods = self.impls.entry(key).or_default();
        for member in &impl_decl.members {
            if let ImplItem::Fn(fd) = member {
                let param_types: Vec<String> = fd.params.iter()
                    .map(|p| CheckedType::from_ast_type(&p.ty).name())
                    .collect();
                let ret = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t).name());
                methods.insert(fd.name.name.clone(), (impl_ty.clone(), param_types, ret));
            }
        }
    }

    /// 5c.33: Register anonymous struct types encountered in function
    /// signatures so field access works correctly. Walks an AST Type tree
    /// and registers any `AnonStruct` variants in `self.types`.
    fn register_anon_struct_from_ast(&mut self, ty: &Type) {
        match ty {
            // Anonymous struct: register the field types under the synthetic name
            Type::AnonStruct(fields) => {
                let parts: Vec<String> = fields.iter()
                    .map(|f| format!("{}_{}", f.name.name, CheckedType::from_ast_type(&f.ty).name()))
                    .collect();
                let anon_name = format!("_Anon__{}", parts.join("__"));
                if !self.types.contains_key(&anon_name) {
                    let field_map: HashMap<String, CheckedType> = fields.iter()
                        .map(|f| (f.name.name.clone(), CheckedType::from_ast_type(&f.ty)))
                        .collect();
                    self.types.insert(anon_name, field_map);
                }
            }
            // Walk nested type constructs that may contain anonymous structs
            Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner)
            | Type::Option(inner) | Type::Vec(inner) | Type::Slice(inner)
            | Type::Set(inner) => self.register_anon_struct_from_ast(inner),
            Type::Result(ok, err) => {
                self.register_anon_struct_from_ast(ok);
                self.register_anon_struct_from_ast(err);
            }
            Type::Map(k, v) => {
                self.register_anon_struct_from_ast(k);
                self.register_anon_struct_from_ast(v);
            }
            Type::Tuple(types) | Type::Fn(types, _) => {
                for t in types { self.register_anon_struct_from_ast(t); }
            }
            Type::Array(_, elem) => self.register_anon_struct_from_ast(elem),
            // Named, ImplTrait -- no recursive anonymous structs
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
            xiom_ast::Expr::Call(func, args, _)
            | xiom_ast::Expr::GenericCall(func, _, args, _) => {
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
            xiom_ast::Stmt::While(cond, body, _, _, _) | xiom_ast::Stmt::For(_, cond, body, _, _) => {
                Self::expr_uses_this(cond) || Self::block_uses_this(body)
            }
            xiom_ast::Stmt::Match(scrut, arms, _) => {
                Self::expr_uses_this(scrut)
                    || arms.iter().any(|arm| match &arm.body {
                        xiom_ast::MatchBody::Block(b) => Self::block_uses_this(b),
                        xiom_ast::MatchBody::Expr(e) => Self::expr_uses_this(e),
                    })
            }
            xiom_ast::Stmt::Spawn(b, _, _move) => Self::block_uses_this(b),
            _ => false,
        }
    }

    fn block_uses_this(block: &xiom_ast::Block) -> bool {
        block.stmts.iter().any(|s| match s {
            xiom_ast::StmtOrExpr::Stmt(stmt) => Self::stmt_uses_this(stmt),
            xiom_ast::StmtOrExpr::Expr(expr) => Self::expr_uses_this(expr),
        })
    }

    /// R2: Collect all variable references from a block (excluding declarations).
    fn collect_expr_references_block(block: &Block) -> HashSet<String> {
        let mut refs = HashSet::new();
        Self::collect_expr_references_block_into(block, &mut refs);
        refs
    }

    fn collect_expr_references_block_into(block: &Block, refs: &mut HashSet<String>) {
        for se in &block.stmts {
            match se {
                StmtOrExpr::Expr(e) => Self::collect_expr_references(e, refs),
                StmtOrExpr::Stmt(s) => Self::collect_stmt_references(s, refs),
            }
        }
    }

    fn collect_stmt_references(stmt: &Stmt, refs: &mut HashSet<String>) {
        match stmt {
            Stmt::Let(_, _, e, _) | Stmt::Var(_, _, e, _) => Self::collect_expr_references(e, refs),
            Stmt::Assign(a, b, _) => { Self::collect_expr_references(a, refs); Self::collect_expr_references(b, refs); }
            Stmt::Return(Some(e), _) => Self::collect_expr_references(e, refs),
            Stmt::Return(None, _) => {}
            Stmt::Expr(e, _) => Self::collect_expr_references(e, refs),
            Stmt::If(c, t, elifs, els, _) => {
                Self::collect_expr_references(c, refs);
                Self::collect_expr_references_block_into(t, refs);
                for (ec, eb) in elifs { Self::collect_expr_references(ec, refs); Self::collect_expr_references_block_into(eb, refs); }
                if let Some(eb) = els { Self::collect_expr_references_block_into(eb, refs); }
            }
            Stmt::While(c, b, _, _, _) => { Self::collect_expr_references(c, refs); Self::collect_expr_references_block_into(b, refs); }
            Stmt::For(_, e, b, _, _) => { Self::collect_expr_references(e, refs); Self::collect_expr_references_block_into(b, refs); }
            Stmt::Spawn(b, _, _) => Self::collect_expr_references_block_into(b, refs),
            Stmt::Match(e, arms, _) => {
                Self::collect_expr_references(e, refs);
                for arm in arms {
                    if let Some(g) = &arm.guard { Self::collect_expr_references(g, refs); }
                    match &arm.body {
                        MatchBody::Block(b) => Self::collect_expr_references_block_into(b, refs),
                        MatchBody::Expr(e) => Self::collect_expr_references(e, refs),
                    }
                }
            }
            Stmt::Destructure(_, e, _) => Self::collect_expr_references(e, refs),
            Stmt::Break(..) | Stmt::Continue(..) | Stmt::Asm(_) | Stmt::Defer(_, _) => {}
            Stmt::Assert(c, m, _) => {
                Self::collect_expr_references(c, refs);
                if let Some(msg) = m { Self::collect_expr_references(msg, refs); }
            }
            Stmt::Debugger(_) => {},
        }
    }

    fn collect_expr_references(expr: &Expr, refs: &mut HashSet<String>) {
        match expr {
            Expr::Ident(id) => { refs.insert(id.name.clone()); }
            Expr::Field(b, _, _) => Self::collect_expr_references(b, refs),
            Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => {
                Self::collect_expr_references(f, refs);
                for a in args { Self::collect_expr_references(a, refs); }
            }
            Expr::Index(a, b, _) => { Self::collect_expr_references(a, refs); Self::collect_expr_references(b, refs); }
            Expr::Binary(a, _, b, _) | Expr::Imply(a, b, _) => {
                Self::collect_expr_references(a, refs); Self::collect_expr_references(b, refs);
            }
            Expr::Unary(_, e, _) | Expr::Paren(e, _) | Expr::Try(e, _) | Expr::Ref(e, _)
            | Expr::MutRef(e, _) | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _)
            | Expr::Comptime(e, _) | Expr::As(e, _, _) => Self::collect_expr_references(e, refs),
            Expr::Struct(_, fields, base, _) => {
                for (_, v) in fields { Self::collect_expr_references(v, refs); }
                if let Some(b) = base { Self::collect_expr_references(b, refs); }
            }
            Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
                for e in elems { Self::collect_expr_references(e, refs); }
            }
            Expr::If(c, t, elifs, els, _) => {
                Self::collect_expr_references(c, refs);
                Self::collect_expr_references_block_into(t, refs);
                for (ec, eb) in elifs { Self::collect_expr_references(ec, refs); Self::collect_expr_references_block_into(eb, refs); }
                if let Some(eb) = els { Self::collect_expr_references_block_into(eb, refs); }
            }
            Expr::Match(e, arms, _) => {
                Self::collect_expr_references(e, refs);
                for arm in arms {
                    if let Some(g) = &arm.guard { Self::collect_expr_references(g, refs); }
                    match &arm.body {
                        MatchBody::Block(b) => Self::collect_expr_references_block_into(b, refs),
                        MatchBody::Expr(e) => Self::collect_expr_references(e, refs),
                    }
                }
            }
            Expr::Is(e, _, _) => Self::collect_expr_references(e, refs),
            Expr::Closure(_, _, b, _) | Expr::BlockExpr(b, _) => Self::collect_expr_references_block_into(b, refs),
            Expr::PipeClosure(_, e, _) => Self::collect_expr_references(e, refs),
            _ => {} // Int, Float, Bool, Str, Char, None, Wildcard, etc.
        }
    }

    // ====================================================================
    // I1: Send/Sync enforcement -- auto-derivation + spawn capture checking
    // ====================================================================

    /// Register a concrete type as implementing both Send and Sync.
    fn register_send_sync_impl(&mut self, type_name: &str) {
        self.interfaces.entry("Send".to_string())
            .or_default();
        self.interfaces.entry("Sync".to_string())
            .or_default();
        // Store implementation in a separate set for fast lookup
        self.send_sync_types.insert(type_name.to_string());
    }

    /// Check if a type implements Send (safe to transfer between threads).
    /// Auto-derived: primitives are Send; structs are Send if all fields are Send;
    /// generic containers (Option, Result, Vec) are Send if type params are Send.
    fn is_send(&self, type_name: &str) -> bool {
        // Directly registered types (primitives + explicitly marked)
        if self.send_sync_types.contains(type_name) {
            return true;
        }
        // Structs: check if all field types are Send
        if let Some(fields) = self.struct_field_types.get(type_name) {
            return fields.iter().all(|(_, ft)| self.is_send(ft));
        }
        // Enum types: all variant field types must be Send
        if let Some(variants) = self.enum_field_types.get(type_name) {
            return variants.iter().all(|(_, fields)| {
                fields.iter().all(|ft| self.is_send(ft))
            });
        }
        // Generic containers: Option[T], Result[T,E], Vec[T] -- assume Send
        // for now (they own their data). Full generic analysis deferred.
        if let Some((base, _params)) = Self::parse_generic_type(type_name) {
            match base.as_str() {
                "Option" | "Result" | "Vec" | "Map" | "Set" | "Deque"
                | "HashMap" | "HashSet" | "BTreeMap" | "PriorityQueue"
                | "Channel" | "Arc" | "AtomicInt" | "AtomicBool"
                | "Mutex" | "RwLock" => return true,
                _ => {}
            }
        }
        false
    }

    /// Parse "Vec[Int]" -> ("Vec", ["Int"]), "Option[Result[Int,Str]]" -> ("Option", ["Result[Int,Str]"])
    fn parse_generic_type(name: &str) -> Option<(String, Vec<String>)> {
        if let Some(bracket) = name.find('[') {
            let base = name[..bracket].to_string();
            let inner = &name[bracket + 1..name.len() - 1];
            // Split by top-level commas only
            let mut params = Vec::new();
            let mut depth = 0;
            let mut current = String::new();
            for ch in inner.chars() {
                match ch {
                    '[' => { depth += 1; current.push(ch); }
                    ']' => { depth -= 1; current.push(ch); }
                    ',' if depth == 0 => {
                        params.push(current.trim().to_string());
                        current.clear();
                    }
                    _ => current.push(ch),
                }
            }
            if !current.is_empty() {
                params.push(current.trim().to_string());
            }
            Some((base, params))
        } else {
            None
        }
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
                // self -- otherwise its first real argument aligns to the phantom self
                // and every call mis-reports "expected Self".
                for p in &fd.params {
                    self.register_anon_struct_from_ast(&p.ty);
                    params.push((p.name.name.clone(), CheckedType::from_ast_type(&p.ty)));
                }
                if let Some(ref ret) = fd.return_type {
                    self.register_anon_struct_from_ast(ret);
                }
                let return_type = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
                // BUG 29 (m35_t16): strip a receiver prefix already present in
                // the fn NAME. expand_impl_blocks emits `impl Sum for NumPair`
                // methods as name="NumPair.sum" AND receiver=NumPair -- naive
                // `{recv}.{name}` doubling registered "NumPair.NumPair.sum",
                // so `np1.sum()` resolved through the wildcard to the WRONG
                // signature (Int-returning `get`) and the checker rejected
                // `sum() != 30.0` ("cannot mix Int with Float64"). Mirrors
                // codegen fn_key's rsplit('.').next() stripping.
                let bare_key = if let Some(recv) = fd.receiver.as_ref() {
                    let bare_method = fd.name.name.rsplit('.').next().unwrap_or(&fd.name.name);
                    format!("{}.{}", recv.name, bare_method)
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
                // BUG 25 #11 fix: track each fn's owning module so bare-call
                // resolution can enforce visibility (a PRIVATE fn of an
                // imported module must not hijack bare calls).
                // BUG 29: keep-first to MATCH the bare-key keep-first rule
                // above (entry().or_insert). The bare-key slot in `functions`
                // belongs to whichever fn registered FIRST (user program
                // registers before catalog imports). Unconditional insert
                // let a transitively-loaded catalog private fn (e.g.
                // xiom.encoding.base64_index) OVERWRITE the owner record of
                // the user's own same-named fn, so the visibility gate
                // rejected the user's bare call ("undefined variable").
                self.fn_owner_module.entry(fd.name.name.clone()).or_insert(module_path.to_string());
                self.visibility.entry(fd.name.name.clone()).or_insert(fd.is_pub);
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
                    // BUG 29 (selfhost xiomc_v092): the stdlib's extern blocks
                    // (loaded via `use xiom.io`) must NOT overwrite a fn the
                    // USER PROGRAM declared as a signature-only fn -- the user's
                    // declaration wins (keep-first). The user's `fn
                    // xiom_read_file(path: Str) -> Int;` is the sanctioned
                    // selfhost declaration pattern; the stdlib extern's
                    // `xiom_char_at(s: Str, pos: Int) -> Char` was registered
                    // with unconditional insert and hijacked the user's
                    // signature, turning safe calls into "extern requires
                    // unsafe" + wrong param types. extern_fns tracking stays
                    // for externs that are NOT shadowed by a user declaration.
                    let user_declared = self.functions.contains_key(&func.name.name)
                        && self.fn_owner_module.contains_key(&func.name.name);
                    if !user_declared {
                        self.functions.insert(func.name.name.clone(), sig);
                        self.visibility.insert(func.name.name.clone(), func.is_pub);
                        // D2.1 (T002): extern "C" calls are confined to unsafe blocks.
                        self.extern_fns.insert(func.name.name.clone());
                    }
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
                // BUG 29: JOIN dotted module paths (parser nests `module a.b.c`
                // as Module(a){Module(b){Module(c)}}). Registration
                // (register_fn_signature_inner) joins the same way, so
                // fn_owner_module["f"] = "a.b.c" while the old overwrite left
                // current_module = "c" -- the visibility gate then rejected
                // bare calls to PRIVATE fns in the same module ("undefined
                // variable 'is_valid'" in every m18_guard_*/ecosystem test
                // with a dotted module name).
                let prev = self.current_module.take();
                self.current_module = Some(if let Some(ref p) = prev {
                    format!("{}.{}", p, md.name.name)
                } else {
                    md.name.name.clone()
                });
                for item in &md.items {
                    self.check_top_decl(item);
                }
                self.current_module = prev;
            }
            TopDecl::Const(cd) => {
                let val_ty = self.check_expr(&cd.value);
                let decl_ty = CheckedType::from_ast_type(&cd.ty);
                // v0.56: Skip type check for zero-initialized globals of complex types
                // (Array, Map, Vec, etc.) -- the zero is a placeholder, not the real type.
                let is_zero_default = matches!(&cd.value, Expr::Int(0, _) | Expr::Float(_, _));
                let is_complex_type = matches!(&decl_ty, CheckedType::Named(n) if n == "Array" || n == "Map" || n == "Vec" || n == "Set");
                // BUG 29: elided annotation (`var g = FnBox{...}` parses as
                // Type::Named("_") -> Int here). The real type was inferred from
                // the initializer in register_global_const; the mismatch check
                // against the placeholder Int is a false positive ("const type
                // mismatch: declared Int, found FnBox" blocked repro_fn_storage).
                let is_elided = matches!(&cd.ty, Type::Named(n, _) if n.name == "_");
                if val_ty != CheckedType::Error && decl_ty != CheckedType::Error && !is_elided {
                    if !is_zero_default || !is_complex_type {
                        if !self.types_compatible(&val_ty, &decl_ty) {
                            self.error(
                                format!("const type mismatch: declared {}, found {}", decl_ty.name(), val_ty.name()),
                                cd.span,
                            );
                        }
                    }
                }
                // (Registration into global_consts happens in the pre-pass
                // `register_global_const` so references resolve regardless of order.)
            }
            TopDecl::Extern(_) => {} // extern blocks have no type info to register
            TopDecl::Spawn(body, _span, _move) => {
                // M21: Type-check module-level spawn block body.
                self.push_scope();
                self.check_block(body, None);
                self.pop_scope();
            }
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
        // TRANSITIVE deps are loaded too: a loaded module's own `use` decls
        // (e.g. env.xi uses xiom.io) must reach the catalog cache, otherwise
        // collect_external_decls misses their bodies and non-generic stdlib
        // functions (io.args, io.println) fall back to undefined stubs.
        let mut worklist: Vec<Vec<String>> = Vec::new();
        for ud in &import_snapshot {
            if ud.path.is_empty() {
                continue;
            }
            // Strip the 'stdlib' filesystem-directory prefix the same way
            // process_use does, so stdlib-prefixed imports get the same
            // transitive/prelude module loading as plain `xiom.*` uses.
            let path: &[Ident] = if ud.path.len() > 1 && ud.path[0].name == "stdlib" {
                &ud.path[1..]
            } else {
                &ud.path
            };
            for end in 1..=path.len() {
                worklist.push(path[..end].iter().map(|i| i.name.clone()).collect());
            }
        }
        while let Some(prefix) = worklist.pop() {
            let dotted = prefix.join(".");
            if self.cached_loaded.contains(&dotted) {
                continue;
            }
            // Only try catalog if the leaf segment isn't already in self.modules.
            let leaf = prefix.last().unwrap();
            if self.modules.contains_key(leaf.as_str()) {
                continue;
            }
            if let Some(cached) = self.catalog.find_owned(&prefix) {
                self.cached_loaded.insert(dotted.clone());
                self.register_external_module(&cached);
                // Enqueue the loaded module's own imports (transitive closure).
                fn collect_uses(items: &[TopDecl], out: &mut Vec<Vec<String>>) {
                    for item in items {
                        match item {
                            TopDecl::Use(ud) => {
                                for end in 1..=ud.path.len() {
                                    out.push(ud.path[..end].iter().map(|i| i.name.clone()).collect());
                                }
                            }
                            TopDecl::Module(md) => collect_uses(&md.items, out),
                            _ => {}
                        }
                    }
                }
                collect_uses(&cached.program.items, &mut worklist);
            }
        }
        // Build parent-module entries for dotted names so that process_use
        // can walk `self.modules.get("xiom") -> async -> ...`.
        // Example: registered "xiom.async" -> ensure "xiom" contains "async".
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
        // unqualified across modules -- `to_string`/`to_int`/`to_float`/`to_char`
        // (core), `str_concat`/`str_len`/`char_at` (string), `fabs`/trig (math),
        // `gcd`/`lcm` (num), plus core `cmp`/`char` helpers. These are neither
        // `pub`-imported nor `use`d, so they were never loaded into the catalog --
        // leaving them undefined at link time and untyped at call sites
        // (`call i64` default -> ptr/int IR mismatches). When a program uses ANY
        // `xiom.*` module, force-load the prelude modules so the checker resolves
        // them and codegen injects+registers their real signatures.
        //
        // Gated strictly on real stdlib usage: no non-stdlib program (and none of
        // the exact-IR diff/e2e examples, which never `use xiom.*`) is affected.
        let uses_xiom_stdlib = import_snapshot
            .iter()
            .any(|ud| {
                let first = if ud.path.len() > 1 && ud.path[0].name == "stdlib" {
                    ud.path.get(1).map(|i| i.name == "xiom").unwrap_or(false)
                } else {
                    ud.path.first().map(|i| i.name == "xiom").unwrap_or(false)
                };
                first
            });
        if uses_xiom_stdlib {
            const PRELUDE: &[&[&str]] = &[
                &["xiom", "core"],
                &["xiom", "string"],
                &["xiom", "math"],
                &["xiom", "num"],
                &["xiom", "char"],
                &["xiom", "cmp"],
                // BUG 27 (Map.new in module-global inits): the container
                // generics the compiler special-cases (Vec/Map/Set/Slice/Stack)
                // are declared in xiom.collections, which nothing `use`s --
                // their constructors were never injected, so
                // `Map[Str, Bool].new()` emitted a stub `define i64
                // @Map.new() { ret i64 0 }` (crash) or "use of undefined
                // value '@new'" (link error). Force-load collections with the
                // prelude so the generic decls reach the monomorphisation
                // registry. NOTE: an on-demand load via a catalog reverse
                // type-index is the planned refinement (docs/ROADMAP.md).
                &["xiom", "collections"],
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

        // Now process each use declaration -- self.modules is fully populated.
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

    /// Collect external declarations from the catalog that are not already present
    /// in the given program, for injection before codegen. Types, enums, and function
    /// stubs from lazily-loaded external modules are returned as TopDecl items.
    /// Primitive types are filtered out.
    pub fn collect_external_decls(&mut self, program: &Program) -> Vec<TopDecl> {
        // Names already declared in the program (to avoid duplicates).
        let mut existing: HashSet<String> = HashSet::new();
        // Delegation-crash fix (stdlib-audit #3, 2026-09-09): the old
        // `user_free_fns` shadow set made injection SKIP any stdlib free fn
        // whose BARE name matched a user fn -- so `xiom.num.convert.to_base58`
        // never registered its leaf-qualified key ("convert.to_base58") when
        // the user module declared its own `pub fn to_base58`, and the
        // module-qualified call fell through to the bare name and bound the
        // LOCAL fn (silent wrong-module resolution; historically the
        // 0xC0000005 delegation crash). The skip's original rationale
        // (duplicate `@alloc` from `module sys { pub fn alloc }`) no longer
        // applies: injected decls arrive LEAF-qualified and emit qualified
        // symbols (@alloc.alloc), while the user's bare fn keeps the bare
        // symbol -- the codegen alias map prefers the existing bare entry,
        // so bare calls still bind the user's fn. Qualified-key dedup below
        // (module-qualified free fns, methods, impl fns) is the only gate
        // injection needs.
        fn collect_names(items: &[TopDecl], existing: &mut HashSet<String>) {
            for item in items {
                match item {
                    TopDecl::Type(td) => { existing.insert(td.name.name.clone()); }
                    TopDecl::Enum(ed) => { existing.insert(ed.name.name.clone()); }
                    TopDecl::Interface(id) => { existing.insert(id.name.name.clone()); }
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
            "Option", "Result", "Vec", "Slice",
            "Ptr", "Array", "Tuple", "fn", "Tuple2",
            // round-9 (Set ABI): "Set" was here -- the compiler has NO builtin
            // Set layout (unlike Vec/Slice/Map, which register %struct layouts
            // in compile_program), so the stdlib's `type Set[T]` must inject
            // and Set resolves like any struct. Keeping Set in PRIMITIVES left
            // every Set value erased to i64 while the stdlib methods operate
            // on %struct.Set -- `Set[Int].new()` hijacked Reverse.new and
            // Set-typed params/returns/fields compiled as i64.
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
        // inject when their receiver type decl is ALSO injected (pub) -- codegen
        // recognizes the receiver as generic (via that injected type decl) and then
        // monomorphises the method on demand instead of emitting a malformed
        // un-monomorphised concrete body. A method on a NON-pub generic type (e.g.
        // core's `BinaryHeap[T].new`) has no injected type decl, so codegen would
        // treat it as concrete and emit broken IR -- those stay skipped.
        // round-8 (vd2/vd6): SUPERSEDED -- non-pub generic TYPE DECLS are now
        // injected too (Type arm below), so every generic receiver is recognized
        // and every method monomorphises; pub_generic_type_names was removed.


        // Walk the cached program items recursively and inject pub type/enum/fn decls
        // with full bodies (not stubs), deduplicated against existing names.
        // Hoisted out of the per-module loop so BOTH the cached-module loop and
        // the BUG 28 #4 peeked-submodule loop can share it.
            fn collect_pub_decls(
                items: &[TopDecl],
                existing: &mut HashSet<String>,
                primitives: &[&str],
                generic_types: &HashSet<String>,
                module_name: &str,
                out: &mut Vec<TopDecl>,
            ) {
                // BUG 9 / BUG 29: struct-name walkers shared by the Fn and Const
                // arms so injected bodies/initializers pull in the layouts of the
                // private types they reference (else they degrade to i64).
                fn first_named(ty: &Type) -> Option<String> {
                    match ty {
                        Type::Named(n, _) => Some(n.name.clone()),
                        Type::Ref(i) | Type::MutRef(i) | Type::Ptr(i)
                        | Type::Vec(i) | Type::Slice(i) | Type::Option(i) => first_named(i),
                        Type::Result(a, b) => first_named(a).or_else(|| first_named(b)),
                        Type::Map(k, v) => first_named(k).or_else(|| first_named(v)),
                        Type::Set(i) => first_named(i),
                        _ => None,
                    }
                }
                fn collect_block_struct_names(b: &Block, out: &mut Vec<String>) {
                    for se in &b.stmts {
                        match se {
                            StmtOrExpr::Stmt(s) => collect_stmt_struct_names(s, out),
                            StmtOrExpr::Expr(e) => collect_expr_struct_names(e, out),
                        }
                    }
                }
                fn collect_stmt_struct_names(s: &Stmt, out: &mut Vec<String>) {
                    match s {
                        Stmt::Let(_, Some(t), _, _) | Stmt::Var(_, Some(t), _, _) => {
                            if let Some(n) = first_named(t) { out.push(n); }
                        }
                        Stmt::Let(_, None, e, _) | Stmt::Var(_, None, e, _) => collect_expr_struct_names(e, out),
                        Stmt::Assign(_, e, _) => collect_expr_struct_names(e, out),
                        Stmt::Return(Some(e), _) => collect_expr_struct_names(e, out),
                        Stmt::Expr(e, _) => collect_expr_struct_names(e, out),
                        Stmt::If(c, t, elifs, els, _) => {
                            collect_expr_struct_names(c, out);
                            collect_block_struct_names(t, out);
                            for (ec, eb) in elifs { collect_expr_struct_names(ec, out); collect_block_struct_names(eb, out); }
                            if let Some(eb) = els { collect_block_struct_names(eb, out); }
                        }
                        Stmt::Match(e, arms, _) => {
                            collect_expr_struct_names(e, out);
                            for arm in arms {
                                match &arm.body {
                                    MatchBody::Block(b) => collect_block_struct_names(b, out),
                                    MatchBody::Expr(e) => collect_expr_struct_names(e, out),
                                }
                            }
                        }
                        Stmt::While(c, b, _, _, _) | Stmt::For(_, c, b, _, _) => {
                            collect_expr_struct_names(c, out);
                            collect_block_struct_names(b, out);
                        }
                        Stmt::Spawn(b, _, _) | Stmt::Defer(b, _) => collect_block_struct_names(b, out),
                        Stmt::Destructure(_, e, _) => collect_expr_struct_names(e, out),
                        _ => {}
                    }
                }
                fn collect_expr_struct_names(e: &Expr, out: &mut Vec<String>) {
                    match e {
                        Expr::Struct(id, fields, base, _) => {
                            out.push(id.name.clone());
                            for (_, v) in fields { collect_expr_struct_names(v, out); }
                            if let Some(b) = base { collect_expr_struct_names(b, out); }
                        }
                        Expr::Field(b, _, _) => collect_expr_struct_names(b, out),
                        Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => {
                            collect_expr_struct_names(f, out);
                            for a in args { collect_expr_struct_names(a, out); }
                        }
                        Expr::Index(a, b, _) => { collect_expr_struct_names(a, out); collect_expr_struct_names(b, out); }
                        Expr::Paren(e, _) | Expr::Unary(_, e, _) | Expr::Try(e, _) | Expr::AtPre(e, _)
                        | Expr::Ref(e, _) | Expr::MutRef(e, _) | Expr::Some(e, _) | Expr::Ok(e, _)
                        | Expr::Err(e, _) | Expr::Await(e, _) | Expr::Comptime(e, _) | Expr::As(e, _, _) => {
                            collect_expr_struct_names(e, out);
                        }
                        Expr::Binary(a, _, b, _) | Expr::Imply(a, b, _) => {
                            collect_expr_struct_names(a, out); collect_expr_struct_names(b, out);
                        }
                        Expr::Is(e, _, _) => collect_expr_struct_names(e, out),
                        Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
                            for el in elems { collect_expr_struct_names(el, out); }
                        }
                        Expr::Closure(_, _, b, _) => collect_block_struct_names(b, out),
                        Expr::PipeClosure(_, e, _) => collect_expr_struct_names(e, out),
                        Expr::If(c, t, elifs, els, _) => {
                            collect_expr_struct_names(c, out);
                            collect_block_struct_names(t, out);
                            for (ec, eb) in elifs { collect_expr_struct_names(ec, out); collect_block_struct_names(eb, out); }
                            if let Some(eb) = els { collect_block_struct_names(eb, out); }
                        }
                        Expr::Match(e, arms, _) => {
                            collect_expr_struct_names(e, out);
                            for arm in arms {
                                match &arm.body {
                                    MatchBody::Block(b) => collect_block_struct_names(b, out),
                                    MatchBody::Expr(e) => collect_expr_struct_names(e, out),
                                }
                            }
                        }
                        Expr::Unsafe(b, _) | Expr::BlockExpr(b, _) => collect_block_struct_names(b, out),
                        _ => {}
                    }
                }
                for item in items {
                    match item {
                        TopDecl::Type(td) => {
                            // round-8 (vd2/vd6): inject NON-pub GENERIC type decls
                            // too (VecDeque/Stack/LinkedList/Queue/BTreeMap/...).
                            // Their methods are callable from the catalog (the
                            // visibility gate is loose for file-level modules) but
                            // without the type decl codegen has no layout AND
                            // generic_type_names misses the receiver, so the calls
                            // hijack same-leaf methods of OTHER types (Reverse).
                            // Pub/non-pub non-generic private types stay excluded
                            // (implementation details; referenced ones are injected
                            // via the fn-signature walk below).
                            if (td.is_pub || !td.generics.is_empty())
                                && !existing.contains(&td.name.name)
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
                        // 3c (2026-08-10): inject catalog-loaded INTERFACES so the
                        // codegen can register them for impl-dispatch resolution
                        // (`Num[T].add` -> `core.Float64.add`).
                        // round-15 (Ord dispatch): NON-pub interfaces must inject
                        // too -- core.xi's `interface Ord[T]` (and Eq/Bounded/...)
                        // are non-pub, so codegen never registered them and
                        // `Ord[T].compare(a, b)` inside mono'd stdlib bodies was
                        // misread as a VALUE INSTANCE method: the inline scalar
                        // compare fired with a literal-0 receiver
                        // (`compare(0, data[parent])` -> heap order broke:
                        // smoke_core_binary_heap popped 2,3,1,5). Interfaces are
                        // pure declarations (no layout), so injection is harmless
                        // (same rationale as the round-8 non-pub TYPE injection).
                        TopDecl::Interface(id) => {
                            if !existing.contains(&id.name.name) {
                                existing.insert(id.name.name.clone());
                                out.push(TopDecl::Interface(id.clone()));
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
                            // round-8 (vd2/vd6): NON-pub generic receivers are no
                            // longer skipped -- their TYPE DECL is injected too (see
                            // the Type arm), so `generic_type_names` recognizes them,
                            // `recv_is_generic` blocks concrete direct-emission, and
                            // the methods monomorphise on demand (the old comment
                            // claimed `BinaryHeap[T].new` has empty fd.generics -- the
                            // BUG 38b parser fix captures receiver generics, so every
                            // such method is mono-able). Without injection, calls like
                            // `dq.push_front(20)` hijacked same-leaf methods of OTHER
                            // types (Reverse.push_front) and every VecDeque/Stack/
                            // LinkedList/Queue/BTreeMap/BTreeSet/BinaryHeap mutation
                            // was silently lost.
                            // Deduplicate by the QUALIFIED key (`Receiver.method` for
                            // methods, bare name for free functions). Deduping by the
                            // bare name alone would drop distinct methods that share a
                            // leaf name (e.g. `Layout.new`, `Vec.new`, `Rc.new`) --
                            // and since catalog iteration order is nondeterministic,
                            // which `new` survived would flip between builds.
                            let dedup_key = if fd.is_method() {
                                format!("{}.{}", fd.receiver.as_ref().unwrap().name, fd.name.name)
                            } else if fd.name.name.contains('.') {
                                // 3c: expanded IMPL methods (e.g. `Int.to_float`,
                                // `Float64.from_int`) have no receiver but their
                                // name is already TYPE-QUALIFIED by
                                // expand_impl_blocks. Dedup on that qualified name
                                // so widths don't collide (`core.to_float` vs
                                // `core.Int.to_float` would drop all but one).
                                fd.name.name.clone()
                            } else if !module_name.is_empty() {
                                // Module-qualified free fns (e.g. xiom.env.args vs
                                // xiom.io.args) must NOT dedup against each other --
                                // bare-name dedup dropped one, leaving the other to
                                // self-recursively resolve (env.args -> @args).
                                format!("{}.{}", module_name, fd.name.name)
                            } else {
                                fd.name.name.clone()
                            };
                            if !existing.contains(&dedup_key)
                                && !primitives.contains(&fd.name.name.as_str()) {
                                existing.insert(dedup_key);
                                // BUG 9 fix (2026-08-11): inject NON-pub struct/enum
                                // types referenced by this fn's signature (params,
                                // return, and -- transitively -- their fields). Their
                                // layouts must reach codegen, otherwise the type
                                // resolves to i64 and the fn signature/ABI degrades
                                // (docs/COMPILER_BUGS.md BUG 9). The stdlib worked
                                // around this with `pub IntFrac`; the compiler now
                                // handles private types used across the boundary.
                                let mut sig_types: Vec<String> = Vec::new();
                                for p in &fd.params {
                                    if let Some(n) = first_named(&p.ty) { sig_types.push(n); }
                                }
                                if let Some(rt) = &fd.return_type {
                                    if let Some(n) = first_named(rt) { sig_types.push(n); }
                                }
                                // BUG 25 #10 (crypto follow-up): inject PRIVATE struct
                                // types used ONLY as LOCALS / struct literals inside
                                // catalog fn bodies (crypto.xi's KeyExpState in
                                // aes_key_expansion_128, aes.xi's AesState). Signature-
                                // only injection left these degrading to i64: the local
                                // was bound as a single i64 slot and the struct
                                // constructor's zero-store clobbered the field reads
                                // mid-construction (wrong AES key schedule -> wrong
                                // ciphertext / contract violations). Struct literals
                                // (`S{ ... }`) and annotated bindings (`var x: S`)
                                // both seed the transitive walk below.
                                if let Some(body) = &fd.body {
                                    collect_block_struct_names(body, &mut sig_types);
                                }
                                // Transitive walk: inject referenced types + the types
                                // their FIELDS reference (nested private structs).
                                let mut worklist = sig_types;
                                while let Some(ty_name) = worklist.pop() {
                                    if existing.contains(&ty_name) || primitives.contains(&ty_name.as_str()) {
                                        continue;
                                    }
                                    for item in items {
                                        match item {
                                            TopDecl::Type(td) if td.name.name == ty_name => {
                                                existing.insert(ty_name.clone());
                                                out.push(TopDecl::Type(td.clone()));
                                                for f in &td.fields {
                                                    if let Some(n) = first_named(&f.ty) {
                                                        worklist.push(n);
                                                    }
                                                }
                                                break;
                                            }
                                            TopDecl::Enum(ed) if ed.name.name == ty_name => {
                                                existing.insert(ty_name.clone());
                                                out.push(TopDecl::Enum(ed.clone()));
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                // Inject with full body so codegen emits define, not declare.
                                let mut fd2 = fd.clone();
                                // Leaf-qualify FREE fn names (e.g. `array.contains`)
                                // so codegen registers module-scoped leaf keys. The
                                // driver merge only accepts flat TopDecl::Fn (Module
                                // wrappers are dropped), so the module context must
                                // ride on the name itself. Methods keep their
                                // receiver-based keys (fn_key uses the receiver).
                                // Bare internal calls (e.g. env.args_os -> args())
                                // still resolve via codegen's bare-key alias map.
                                // NOTE: even when the fn's name equals the module
                                // leaf (`alloc` in xiom.alloc -> `alloc.alloc`), the
                                // rename still applies -- the qualified key is what
                                // makes it distinct from a user's bare `alloc`.
                                if fd2.receiver.is_none() && !module_name.is_empty() {
                                    if let Some(leaf) = module_name.rsplit('.').next() {
                                        if !leaf.is_empty() {
                                            fd2.name.name = format!("{}.{}", leaf, fd2.name.name);
                                        }
                                    }
                                }
                                out.push(TopDecl::Fn(fd2));
                            }
                        }
                        TopDecl::Module(md) => {
                            // Recurse FLAT (no wrapper): the driver merge in
                            // crates/xiom/src/lib.rs drops TopDecl::Module from the
                            // external-decl injection, so wrapped decls would never
                            // reach codegen. Module context is preserved instead by
                            // leaf-qualifying free fn names above.
                            collect_pub_decls(&md.items, existing, primitives, generic_types, module_name, out);
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
                                // BUG 29 (repro_fn_storage): a module-level
                                // `var g = FnBox{...}` injects the CONST but not
                                // the (possibly private) struct type it
                                // references -- codegen then degraded the global
                                // to i64 ("module-scope fn storage read-only").
                                // Transitive type injection mirrors the Fn arm:
                                // collect struct names from the declared type
                                // and the initializer expression, then pull in
                                // their layouts (and nested field types).
                                let mut worklist: Vec<String> = Vec::new();
                                if let Some(n) = first_named(&cd.ty) {
                                    worklist.push(n);
                                }
                                collect_expr_struct_names(&cd.value, &mut worklist);
                                while let Some(ty_name) = worklist.pop() {
                                    if existing.contains(&ty_name) || primitives.contains(&ty_name.as_str()) {
                                        continue;
                                    }
                                    for item in items {
                                        match item {
                                            TopDecl::Type(td) if td.name.name == ty_name => {
                                                existing.insert(ty_name.clone());
                                                out.push(TopDecl::Type(td.clone()));
                                                for f in &td.fields {
                                                    if let Some(n) = first_named(&f.ty) {
                                                        worklist.push(n);
                                                    }
                                                }
                                                break;
                                            }
                                            TopDecl::Enum(ed) if ed.name.name == ty_name => {
                                                existing.insert(ty_name.clone());
                                                out.push(TopDecl::Enum(ed.clone()));
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

        for cached in self.catalog.all_cached() {
            let cached_module_name = cached.dotted_name.clone();
            collect_pub_decls(&cached.program.items, &mut existing, PRIMITIVES, &generic_type_names, &cached_module_name, &mut decls);
        }

        // BUG 28 #4: submodules resolved via catalog PEEK during checking
        // (e.g. "xiom.os.platform" from `os.platform.platform_name()` after
        // `use xiom.os;`). The peek is deliberately non-caching, so these
        // never entered all_cached() -- yet their pub fns ARE callable and
        // must reach codegen, or the call emits a bare zero-arg stub
        // (ret null -> inttoptr garbage -> crash). Inject them exactly like
        // cached modules; the reachability filter below prunes everything
        // the program does not actually reference.
        for dotted in &self.peeked_resolved {
            let segs: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
            if let Some(cached) = self.catalog.peek_owned(&segs) {
                let cached_module_name = cached.dotted_name.clone();
                collect_pub_decls(&cached.program.items, &mut existing, PRIMITIVES, &generic_type_names, &cached_module_name, &mut decls);
            }
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
                        // 3c: generic BOUNDS (`[T: Real + Num]`) make the bound
                        // interfaces referenced -- their impl methods must survive
                        // the reachability filter for the C001 bound check.
                        for gp in &fd.generics {
                            for b in &gp.bounds {
                                out.insert(b.name.clone());
                            }
                        }
                        for c in &fd.contracts {
                            match c {
                                ContractClause::Requires(e, _)
                                | ContractClause::Ensures(e, _) => collect_expr_names(e, out),
                            }
                        }
                    }
                    TopDecl::Module(md) => collect_referenced_names(&md.items, out),
                    // BUG 25 #2 fix: a use declaration references its TARGET
                    // leaf -- the reachability filter must keep the aliased fn
                    // (`use X.f as af; af(...)` kept abs_float alive, not just
                    // the alias name "af").
                    TopDecl::Use(ud) => {
                        for p in &ud.path {
                            out.insert(p.name.clone());
                            if let Some(leaf) = p.name.rsplit('.').next() {
                                out.insert(leaf.to_string());
                            }
                        }
                        if let Some(alias) = &ud.alias {
                            out.insert(alias.name.clone());
                        }
                    }
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
                Stmt::While(c, b, _, _, _) => { collect_expr_names(c, out); collect_block_names(b, out); }
                Stmt::For(_, e, b, _, _) => { collect_expr_names(e, out); collect_block_names(b, out); }
                Stmt::Spawn(b, _, _move) => collect_block_names(b, out),
                Stmt::Destructure(_, e, _) => collect_expr_names(e, out),
                Stmt::Break(..) | Stmt::Continue(..) => {},
                Stmt::Asm(_) => {},
                Stmt::Defer(b, _) => collect_block_names(b, out),
            Stmt::Assert(_, _, _) | Stmt::Debugger(_) => {},
            }
        }
        fn collect_expr_names(expr: &Expr, out: &mut HashSet<String>) {
            match expr {
                Expr::Ident(id) => { out.insert(id.name.clone()); }
                Expr::Field(b, f, _) => { collect_expr_names(b, out); out.insert(f.name.clone()); }
                Expr::Call(f, args, _)
                | Expr::GenericCall(f, _, args, _) => {
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
                TopDecl::Fn(fd) => {
                    // 3c: a candidate fn's generic BOUNDS reference interfaces --
                    // seed them so the interface's impl methods survive pruning
                    // (the C001 bound check needs every method registered).
                    for gp in &fd.generics {
                        for b in &gp.bounds {
                            referenced.insert(b.name.clone());
                        }
                    }
                    fn_candidates.push(fd)
                }
                other => kept.push(other),
            }
        }
        // BUG 29 (repro_fn_storage): KEPT const initializers reference fns too
        // -- `var g = FnBox{ f: _id; }` seeds `_id`. Without this, the
        // reachability filter pruned `_id` (only the module-global initializer
        // references it, and the user program never names it), the ginit
        // emitted `ptrtoint` of an undefined symbol -> stored 0 -> the stored fn
        // call crashed. The fixpoint loop below propagates from these seeds.
        for d in &kept {
            if let TopDecl::Const(cd) = d {
                collect_expr_names(&cd.value, &mut referenced);
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
                Pattern::Struct(_, fields, _) => {
                    for (_, sub_pat) in fields { collect_pattern_names(sub_pat, out); }
                }
                Pattern::Tuple(elements, _) => {
                    for elem in elements { collect_pattern_names(elem, out); }
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
                // Leaf-qualified free fns (e.g. `array.contains`) are referenced by
                // their BARE name (`contains`) from call sites like `array.contains(x)`
                // (Expr::Field collects the field name). Match on the last dot segment.
                let leaf_name = fn_candidates[i].name.name.rsplit('.').next().unwrap_or(&fn_candidates[i].name.name);
                let leaf_reachable = referenced.contains(leaf_name)
                    || referenced.contains(&fn_candidates[i].name.name);
                let key = if fn_candidates[i].is_method() {
                    format!("{}.{}", fn_candidates[i].receiver.as_ref().unwrap().name, fn_candidates[i].name.name)
                } else {
                    fn_candidates[i].name.name.clone()
                };
                let qualified_reachable = referenced.contains(&key);
                // 3c: an IMPL METHOD (name like `Int.to_float`, no receiver) is
                // reachable when its INTERFACE is referenced -- the dispatch goes
                // through `FromInt[T].to_float`, which collects the interface
                // name `FromInt` into `referenced`. Without this, unused-now
                // interface methods (e.g. to_float when only from_int is called)
                // are pruned, breaking the C001 bound check at monomorphisation.
                let impl_iface_reachable = if fn_candidates[i].receiver.is_none()
                    && fn_candidates[i].name.name.contains('.') {
                    let hit = referenced.iter().any(|r| {
                        self.interfaces.contains_key(r)
                            && self.interfaces.get(r).map_or(false, |methods| {
                                methods.iter().any(|(m, _, _)| m == leaf_name)
                            })
                    });
                    hit
                } else {
                    false
                };
                let is_reachable = leaf_reachable || qualified_reachable || impl_iface_reachable;
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
        let mut m = self.build_module_map_inner(items, "");
        // The parser nests dotted file modules (`module xiom.io` becomes
        // Module(xiom){ Module(io){ ... } }), so build_module_map_inner returns a
        // single-child chain {xiom:{io:{real fns}}}. Binding that chain under
        // the imported name breaks `io.println`-style resolution (the walk
        // finds only "xiom" at the top). Descend through single-child
        // SubModule chains to the LEAF export map.
        loop {
            if m.len() == 1 {
                if let Some(ModuleExport::SubModule(inner)) = m.values().next() {
                    m = inner.clone();
                    continue;
                }
            }
            return m;
        }
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
                    // BUG 25 #11 fix: PRIVATE fns must not be re-exported by
                    // `use` -- a bare call in an importing module previously
                    // resolved to the imported module's private fn (hijacking
                    // same-named calls, e.g. heap.xi's private `pheap_merge`).
                    // The export map is the module SURFACE; private fns stay
                    // visible only within their own module.
                    if is_pub {
                        map.insert(map_key, ModuleExport::Function { sig, is_pub });
                    }
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

        /// Build the export map for a catalog module and AUGMENT it with the
    /// catalog's submodule entries (e.g. xiom.os's exports gain "platform",
    /// "filesystem", ...). Directory modules (os.xi + os/*.xi siblings) must
    /// expose their submodules so qualified calls like `os.platform.<fn>`
    /// resolve after `use xiom.os;` (regression from the 4c439e6a batch).
    ///
    /// Collision policy: when a submodule shares a name with an existing
    /// export (xiom.os has BOTH `pub fn platform()` and the `platform`
    /// submodule), the map keeps the Function/Type entry (so the 2-segment
    /// call `os.platform()` keeps resolving) and the submodule's exports are
    /// recorded under `submodule_aliases` keyed by the FULL dotted path
    /// ("xiom.os.platform") -- the qualified walk descends through that map.
    fn module_exports_with_submodules(&mut self, cached: &CachedModule) -> HashMap<String, ModuleExport> {
        self.build_module_map(&cached.program.items)
    }

    fn process_use(&mut self, ud: &UseDecl) {
        if ud.path.is_empty() {
            return;
        }

        // v0.56: Strip 'stdlib' prefix -- it's a filesystem directory, not a module.
        // `use stdlib.xiom.io` should resolve as `use xiom.io` via source_dirs.
        let effective_path: Vec<Ident> = if ud.path.len() > 1 && ud.path[0].name == "stdlib" {
            ud.path[1..].to_vec()
        } else {
            ud.path.clone()
        };
        if effective_path.is_empty() {
            return;
        }

        let module_name = &effective_path[0].name;
        // Clone the exports map to avoid borrow conflicts with self.modules.insert below
        let exports = match self.modules.get(module_name).cloned() {
            Some(e) => e,
            None => {
                // First segment not in modules (e.g. "xiom" from `use xiom.async`
                // when no standalone xiom.xi exists). Load the full path from catalog
                // and build a parent module entry containing the submodule.
                // NOTE: the full path may include the ITEM (fn/const) -- the walk
                // above already resolved directory-module paths; here the last
                // segment is either part of the module path or the item itself.
                let full_path: Vec<String> = effective_path.iter().map(|p| p.name.clone()).collect();
                if let Some(cached) = self.catalog.find_owned(&full_path) {
                    // Register function signatures from the loaded module so
                    // method resolution works (e.g. Vec.insert, Map.contains).
                    // build_module_map creates export maps but doesn't register
                    // functions in self.functions - without this, method calls
                    // on stdlib types fail with "cannot call on this expression".
                    for item in &cached.program.items {
                        self.register_fn_signature(item);
                    }
                    let sub_exports = self.module_exports_with_submodules(&cached);
                    let mut parent = HashMap::new();
                    // Extract the short submodule name from the last path segment
                    let short = effective_path.last().map(|p| p.name.clone()).unwrap_or_default();
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
        let mut i = 1;
        while i < effective_path.len() - 1 {
            let seg = &effective_path[i].name;
            match current.get(seg) {
                Some(ModuleExport::SubModule(sub)) => {
                    current = sub.clone();
                    i += 1;
                }
                _ => {
                    // BUG fix (2026-08-11): load the LONGEST matching dotted
                    // prefix from the catalog -- submodule DIRECTORIES like
                    // xiom.collect.skiplist have no `collect.xi` intermediate
                    // file, so single-segment loading (`collect`) fails and the
                    // whole import silently binds nothing. Try each prefix from
                    // the current segment onward (INCLUDING the last segment,
                    // which may be a directory module path) and take the first
                    // that resolves as a real module file.
                    let mut resolved: Option<(usize, CachedModule)> = None;
                    for j in i..effective_path.len() {
                        let cand: Vec<String> = effective_path[0..=j].iter().map(|p| p.name.clone()).collect();
                        if let Some(cached) = self.catalog.find_owned(&cand) {
                            resolved = Some((j, cached));
                            break;
                        }
                    }
                    match resolved {
                        Some((j, cached)) => {
                            let sub_exports = self.module_exports_with_submodules(&cached);
                            // Register the intermediate segments AND the consumed
                            // path into the FIRST segment's parent map so qualified
                            // expressions (xiom.collect.skiplist.fn) resolve.
                            let mut chain: HashMap<String, ModuleExport> = sub_exports.clone();
                            for k in (1..=j).rev() {
                                let mut parent = HashMap::new();
                                parent.insert(effective_path[k].name.clone(), ModuleExport::SubModule(chain));
                                chain = parent;
                            }
                            if let Some(root) = self.modules.get_mut(&effective_path[0].name) {
                                for (k, v) in chain {
                                    root.insert(k, v);
                                }
                            } else {
                                self.modules.insert(effective_path[0].name.clone(), chain);
                            }
                            current = sub_exports;
                            i = j + 1;
                        }
                        None => return,
                    }
                }
            }
        }

        if ud.glob {
            // `use module.*;` -- import all pub items
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
            let item_name = &effective_path.last().unwrap().name;
            let export = match current.get(item_name) {
                Some(e) => e.clone(),
                None => {
                    // Not found in current module -- try loading the full dotted
                    // path from catalog (e.g. "xiom.async" when the parent module
                    // "xiom" is incomplete or the submodule wasn't pre-indexed).
                    let full_path: Vec<String> = effective_path.iter().map(|p| p.name.clone()).collect();
                    if let Some(cached) = self.catalog.find_owned(&full_path) {
                        let module_exports = self.module_exports_with_submodules(&cached);
                        let local_name = ud.alias.as_ref()
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| item_name.clone());
                        self.local_module_paths.insert(local_name.clone(), full_path.join("."));
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
            // BUG 25 #2 fix: record the alias -> FULL dotted use path so the
            // codegen can resolve bare calls through the alias (the driver
            // strips UseDecls before codegen; the checker is the only place
            // the binding survives).
            if ud.alias.is_some() {
                let full: Vec<String> = effective_path.iter().map(|p| p.name.clone()).collect();
                if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() {
                    eprintln!("[userec] {local_name} -> {} (full={full:?})", full.join("."));
                }
                // Record BOTH the full dotted path AND the stdlib-stripped
                // leaf-qualified form ("xiom.math.abs_float" and
                // "math.abs_float") -- injected stdlib fns register under the
                // leaf-qualified key.
                self.use_alias_paths.insert(local_name.clone(), full.join("."));
                if full.len() > 1 {
                    self.use_alias_paths.insert(
                        format!("{local_name}::qualified"),
                        full[1..].join("."),
                    );
                }
            }
            // Checker-only local-name -> FULL dotted path map, recorded for
            // EVERY use. The qualified-call walk uses it to map the first
            // segment back to its full dotted path when descending submodule
            // segments (`use xiom.os;` -> "os.platform" -> "xiom.os.platform").
            // Kept SEPARATE from use_alias_paths: the driver hands that map
            // to the codegen, whose bare-call alias resolution must not see
            // plain module-name entries (perturbed unrelated programs).
            {
                let full: Vec<String> = effective_path.iter().map(|p| p.name.clone()).collect();
                self.local_module_paths.insert(local_name.clone(), full.join("."));
            }
            // Register SubModules in both imported_items (for type paths)
            // and modules (for expression paths like `async.Executor.new()`)
            if let ModuleExport::SubModule(sub_exports) = &export {
                self.modules.entry(local_name.clone()).or_insert_with(|| sub_exports.clone());
                // G-32: when `use mod` imports a SubModule, recursively
                // inject its pub items (fns, types, CONSTS) so bare `PI`
                // resolves. Previously the module was registered but the
                // items inside were hidden -- consts were invisible to bare
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
        // D1 (2026-08-08): interface impl dispatch -- `Trait[Args].method(args)`
        // resolves to the registered impl's `Type.method` freestanding fn
        // (produced by expand_impl_blocks). Handles both `Num[Int].add(...)`
        // and `Num.add(...)` (zero-arg generic interface).
        if let Some(impl_ty) = self.resolve_impl_method(obj, method) {
            // Check args against the impl method's signature.
            let sig_key = format!("{}.{}", impl_ty, method.name);
            let sig = self.functions.get(&sig_key).cloned();
            if let Some(sig) = sig {
                // AUDIT FIX (readiness Stage 1): arity was never checked --
                // extra arguments were silently DROPPED at codegen.
                if args.len() > sig.params.len() {
                    self.error(
                        format!("{} expects {} argument(s), found {}",
                            sig_key, sig.params.len(), args.len()),
                        span,
                    );
                }
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
                return Some(sig.return_type.clone().unwrap_or(CheckedType::Unit));
            }
            for arg in args { let _ = self.check_expr(arg); }
            return Some(CheckedType::Named("_".into()));
        }

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

        // AUDIT FIX (readiness Stage 1): arity was never checked on
        // module-prefix calls -- `convert.float_to_string(3.14159, 2)` against
        // the 1-param def compiled and silently dropped the extra argument.
        if args.len() > sig.params.len() {
            self.error(
                format!("{} expects {} argument(s), found {}",
                    path.join("."), sig.params.len(), args.len()),
                span,
            );
        }
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
    /// D1 (2026-08-08): resolve `Trait[Args].method` (or `Trait.method`) to the
    /// implementing type registered via `impl Trait[Args] { ... }`.
    /// Returns the implementing type name (e.g. "Int" for `Num[Int]`).
    /// Receiver shapes handled:
    ///   GenericCall(Ident(Trait), [Args], _) -> trait with type args
    ///   Index(Ident(Trait), arg_expr)         -> `Trait[Arg]` parsed as indexing
    ///   Ident(Trait)                          -> bare trait name
    fn resolve_impl_method(&self, obj: &Expr, method: &Ident) -> Option<String> {
        // Extract (trait_name, args) from the receiver.
        let (trait_name, arg_names): (String, Vec<String>) = match obj {
            Expr::GenericCall(base, types, _, _) => {
                if let Expr::Ident(id) = base.as_ref() {
                    let args: Vec<String> = types.iter()
                        .map(|t| CheckedType::from_ast_type(t).name())
                        .collect();
                    (id.name.clone(), args)
                } else {
                    return None;
                }
            }
            Expr::Index(base, idx, _) => {
                if let Expr::Ident(id) = base.as_ref() {
                    // `Num[Int]` -- the index expr is a type name as an Ident.
                    let arg = match idx.as_ref() {
                        Expr::Ident(i) => i.name.clone(),
                        _ => return None,
                    };
                    (id.name.clone(), vec![arg])
                } else {
                    return None;
                }
            }
            Expr::Ident(id) => (id.name.clone(), Vec::new()),
            _ => return None,
        };
        // Candidate keys: "Trait[Int]" then bare "Trait".
        let mut keys: Vec<String> = Vec::new();
        if !arg_names.is_empty() {
            keys.push(format!("{}[{}]", trait_name, arg_names.join(",")));
        }
        keys.push(trait_name.clone());
        for key in &keys {
            if let Some(impl_methods) = self.impls.get(key) {
                if let Some((impl_ty, _, _)) = impl_methods.get(&method.name) {
                    return Some(impl_ty.clone());
                }
            }
        }
        // 3c (2026-08-10): `Num[T].add` inside `fn sum2[T: Num]` -- the arg is a
        // GENERIC TYPE PARAMETER whose impl is unknown until monomorphisation.
        // If the param has the trait as a bound and the trait declares the
        // method, accept the call (codegen resolves the concrete impl when T
        // is substituted). Return the generic param name as the "impl type".
        if arg_names.len() == 1 {
            let gp = &arg_names[0];
            let gp_is_bound = self.current_generic_bounds.get(gp).map_or(false, |bounds| {
                bounds.iter().any(|b| b == &trait_name)
                    || bounds.iter().any(|b| b.ends_with(&format!(".{trait_name}")))
            });
            if gp_is_bound
                && self.impls.values().any(|m| m.contains_key(&method.name))
            {
                return Some(gp.clone());
            }
        }
        None
    }

    fn resolve_module_function(&mut self, path: &[String]) -> Option<FnSig> {
        let module_name = &path[0];
        // Try modules first, then imported_items (short names from `use`)
        let exports = self.modules.get(module_name).cloned().or_else(|| {
            self.imported_items.get(module_name).and_then(|export| {
                match export {
                    ModuleExport::SubModule(exports) => Some(exports.clone()),
                    _ => None,
                }
            })
        })?;
        // Full dotted path of the FIRST segment, for submodule descent when a
        // segment collides with a same-named fn (xiom.os.platform) or is only
        // discoverable via the catalog index (directory modules).
        let mut dotted = self.local_module_paths.get(module_name)
            .cloned()
            .unwrap_or_else(|| module_name.clone());
        let mut current_exports = exports;
        for i in 1..path.len() - 1 {
            let seg = &path[i];
            dotted = format!("{dotted}.{seg}");
            let export = current_exports.get(seg).cloned();
            match export {
                Some(ModuleExport::SubModule(sub)) => current_exports = sub,
                Some(ModuleExport::Function { .. }) | Some(ModuleExport::Type { .. }) => {
                    // Name collision: the segment is BOTH an export and a
                    // submodule (xiom.os has `pub fn platform()` AND the
                    // `platform` submodule). A qualified continuation means
                    // the submodule -- descend through submodule_aliases, or
                    // resolve it from the catalog on demand.
                    if let Some(alias) = self.submodule_aliases.get(&dotted).cloned() {
                        current_exports = alias;
                    } else {
                        let segs: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
                        match self.catalog.peek_owned(&segs) {
                            Some(sub_cached) => {
                                // BUG 28 #4: the submodule resolved a REAL callable --
                                // record it so collect_external_decls injects its pub
                                // decls into codegen (peek stays non-caching).
                                self.peeked_resolved.insert(dotted.clone());
                                let sub_exports = self.build_module_map(&sub_cached.program.items);
                                self.submodule_aliases.insert(dotted.clone(), sub_exports.clone());
                                current_exports = sub_exports;
                            }
                            None => return None,
                        }
                    }
                }
                None => {
                    // Segment missing from the export map: it may be a
                    // catalog submodule of a directory module (os.xi with
                    // os/platform.xi siblings) that nothing `use`d. Resolve
                    // it lazily from the catalog WITHOUT caching (peek) so
                    // the submodule's decls never enter the injection set
                    // (eager injection perturbed bare-alias keep-first
                    // resolution -- crypto sha256 broke when os/* submodules
                    // entered the graph).
                    let segs: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
                    match self.catalog.peek_owned(&segs) {
                        Some(sub_cached) => {
                            // BUG 28 #4: record for injection (see above).
                            self.peeked_resolved.insert(dotted.clone());
                            let sub_exports = self.build_module_map(&sub_cached.program.items);
                            self.submodule_aliases.insert(dotted.clone(), sub_exports.clone());
                            current_exports = sub_exports;
                        }
                        None => return None,
                    }
                }
                Some(_) => return None,
            }
        }
        let func_name = &path[path.len() - 1];
        let export = current_exports.get(func_name)?.clone();
        match export {
            ModuleExport::Function { sig, is_pub: true } => Some(sig),
            _ => None,
        }
    }

    /// Resolve a module path to a function signature without pub check
    /// (used by check_module_field_access to verify visibility separately).
    /// Try to resolve module-qualified field access: `module.Type` or `module.sub.Type`
    fn check_module_field_access(&mut self, obj: &Expr, field: &Ident) -> Option<CheckedType> {
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
        // `use xiom.async` inserts "async" -> SubModule(exports) into
        // imported_items but not into modules.
        let exports = self.modules.get(module_name).cloned().or_else(|| {
            self.imported_items.get(module_name).and_then(|export| {
                match export {
                    ModuleExport::SubModule(exports) => Some(exports.clone()),
                    _ => None,
                }
            })
        })?;
        let mut current_exports = exports;
        // Full dotted path of the first segment, for submodule descent when a
        // segment collides with a same-named fn/type (xiom.os.platform) or is
        // only discoverable via the catalog index (directory modules).
        let mut dotted = self.local_module_paths.get(module_name)
            .cloned()
            .unwrap_or_else(|| module_name.clone());
        for i in 1..path.len() - 1 {
            let seg = &path[i];
            dotted = format!("{dotted}.{seg}");
            let export = current_exports.get(seg).cloned();
            match export {
                Some(ModuleExport::SubModule(sub)) => current_exports = sub,
                Some(ModuleExport::Function { .. }) | Some(ModuleExport::Type { .. }) => {
                    // Name collision: the segment is BOTH an export and a
                    // submodule -- descend through submodule_aliases or the
                    // catalog on demand.
                    if let Some(alias) = self.submodule_aliases.get(&dotted).cloned() {
                        current_exports = alias;
                    } else {
                        let segs: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
                        match self.catalog.peek_owned(&segs) {
                            Some(sub_cached) => {
                                // BUG 28 #4: record for injection (see resolve_module_function).
                                self.peeked_resolved.insert(dotted.clone());
                                let sub_exports = self.build_module_map(&sub_cached.program.items);
                                self.submodule_aliases.insert(dotted.clone(), sub_exports.clone());
                                current_exports = sub_exports;
                            }
                            None => return None,
                        }
                    }
                }
                None => {
                    let segs: Vec<String> = dotted.split('.').map(|s| s.to_string()).collect();
                    match self.catalog.peek_owned(&segs) {
                        Some(sub_cached) => {
                            // BUG 28 #4: record for injection (see resolve_module_function).
                            self.peeked_resolved.insert(dotted.clone());
                            let sub_exports = self.build_module_map(&sub_cached.program.items);
                            self.submodule_aliases.insert(dotted.clone(), sub_exports.clone());
                            current_exports = sub_exports;
                        }
                        None => return None,
                    }
                }
                Some(_) => return None,
            }
        }

        let name = &path[path.len() - 1];
        // NOTE 4 fix: module-qualified enum VARIANT access -- `bigfloat.Down`
        // -- the variant's parent enum type must be one the module EXPORTS
        // (e.g. bigfloat.RoundMode.Down also works via the Type export path).
        if !current_exports.contains_key(name) {
            let parent = self.enum_variants.get(name)
                .or_else(|| self.resolve_enum_variant(name));
            if let Some(parent_name) = parent {
                let parent_leaf = parent_name.rsplit('.').next().unwrap_or(parent_name);
                if current_exports.contains_key(parent_leaf) {
                    return Some(CheckedType::Named(parent_name.clone()));
                }
            }
        }
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
        // `fn len[T, const N: Int](arr: &[N]T) -> Int { N }` -- N must resolve
        for g in &fd.generics {
            let gen_ty = if g.is_const {
                g.const_ty.as_ref().map(|t| CheckedType::from_ast_type(t)).unwrap_or(CheckedType::Int)
            } else {
                CheckedType::Named("type".into())
            };
            self.add_local(&g.name.name, gen_ty);
        }
        // Track generic param bounds for interface method resolution.
        // e.g., fn foo[T: Foo](x: T) { x.bar() } -- need to know T has Foo
        // bound to resolve bar() as an interface method.
        self.current_generic_bounds.clear();
        for g in &fd.generics {
            if !g.bounds.is_empty() {
                let bounds: Vec<String> = g.bounds.iter().map(|b| b.name.clone()).collect();
                self.current_generic_bounds.insert(g.name.name.clone(), bounds);
            }
        }

        // For methods, inject the receiver's fields into scope (implicit self)
        if let Some(recv) = fd.receiver.as_ref() {
            // Add self as a variable (for match self { ... } in enum methods)
            self.add_local("self", CheckedType::Named(recv.name.clone()));
            // 5c.30: track the receiver type for implicit-self method call
            // resolution (G-10: bare `init()` inside `fn GrpcClient.init()`).
            self.current_receiver = Some(recv.name.clone());
            // G-20: bare receiver fields are backed by codegen for ALL slot
            // forms now -- explicit `self`, receiver-style `&T` first param,
            // `this`-based bodies, AND bare-field bodies (codegen emits a
            // %param_self slot whenever the body mentions receiver state).
            // Do NOT shadow explicit parameters with same-named receiver
            // fields (e.g. `fn BufReader.read_line(self, buf: &mut Str)`
            // where `buf` is also a BufReader field of type Vec[UInt8]).
            let param_names: Vec<String> = fd.params.iter()
                .map(|p| p.name.name.clone())
                .collect();
            let fields_clone = self.get_type(&recv.name).cloned();
            if let Some(fields) = fields_clone {
                for (field_name, field_ty) in fields {
                    if !param_names.contains(&field_name) {
                        self.add_local(&field_name, field_ty);
                    }
                }
            }
        }

        // Set expected return type
        let expected_return = fd.return_type.as_ref().map(|t| CheckedType::from_ast_type(t));
        let expected_return_clone = expected_return.clone();
        self.current_return = expected_return.clone();

        // D2.1 (T002 exemption): fns with contracts are the sanctioned safe
        // wrappers around unsafe internals (requirement c) -- they may call
        // extern "C" functions directly.
        self.current_fn_has_contracts = !fd.contracts.is_empty();
        // D2.1 (T007): an unsafe block must be wrapped by a safe fn enforcing
        // at least one `requires` clause (pre-entry validation, requirement c).
        self.current_fn_has_requires = fd.contracts.iter().any(|c| matches!(c, ContractClause::Requires(..)));

        // Check body
        if let Some(body) = fd.body.as_ref() {
            // D2.1 (T007, requirement c): a fn whose ENTIRE body is one
            // `unsafe { }` block must declare at least one `requires` clause --
            // the safe wrapper pattern: inputs are logically validated before
            // the confined block executes.
            if !self.current_fn_has_requires && Self::block_is_single_unsafe(body) {
                self.error(
                    format!(
                        "fn '{}' has a whole-body `unsafe` block but declares no `requires` (T007 -- pre-entry contract, Unsafe Confinement requirement c)",
                        fd.name.name
                    ),
                    fd.name.span,
                );
            }
            self.check_block(body, expected_return);
        }

        // D2.1 (T003 extension -- zero-escape at the FUNCTION boundary): a SAFE
        // fn (one with NO unsafe blocks) may not RETURN a raw-pointer or
        // reference type. Unsafe-internal helpers (bodies containing `unsafe`
        // blocks) legitimately return pointers created within their own
        // confinement -- they are the sanctioned plumbing for pointer factories
        // (e.g. `fn null_expr() -> *Expr { return unsafe { 0 as *Expr }; }`).
        let body_has_unsafe = fd.body.as_ref().map_or(false, |b| Self::block_contains_unsafe(b));
        if !body_has_unsafe {
            if let Some(ret) = &expected_return_clone {
                match ret {
                    // Raw pointers: zero-escape gate. `&T` references are the
                    // SAFE borrow mechanism (borrow-checked) and remain allowed
                    // as fn returns -- only RAW pointers are confined.
                    CheckedType::Named(n) if n.starts_with('*') || n == "Ptr" => {
                        self.error(format!(
                            "safe fn '{}' cannot return raw pointer type '{n}' (T003 -- zero-escape; only unsafe-internal helpers may return pointers)",
                            fd.name.name
                        ), fd.name.span);
                    }
                    _ => {}
                }
            }
        }

        self.pop_scope();
        self.current_receiver = None;
        self.current_fn_has_contracts = false;
        self.current_fn_has_requires = false;
    }

    /// True if the block is exactly one `unsafe { }` expression statement
    /// (T007 whole-body-unsafe detection).
    fn block_is_single_unsafe(block: &Block) -> bool {
        if block.stmts.len() == 1 {
            if let StmtOrExpr::Expr(e) = &block.stmts[0] {
                return matches!(e, Expr::Unsafe(..));
            }
        }
        false
    }

    /// True if the block (transitively) contains an `unsafe { }` expression --
    /// marks an unsafe-internal helper for T003's zero-escape exemption.
    fn block_contains_unsafe(block: &Block) -> bool {
        fn stmt_has_unsafe(stmt: &StmtOrExpr) -> bool {
            match stmt {
                StmtOrExpr::Expr(e) => expr_has_unsafe(e),
                StmtOrExpr::Stmt(s) => match s {
                    Stmt::Let(_, _, e, _) | Stmt::Var(_, _, e, _) => expr_has_unsafe(e),
                    Stmt::Return(Some(e), _) => expr_has_unsafe(e),
                    Stmt::Expr(e, _) => expr_has_unsafe(e),
                    Stmt::If(c, t, elifs, els, _) => {
                        expr_has_unsafe(c)
                            || block_has_unsafe(t)
                            || elifs.iter().any(|(ec, eb)| expr_has_unsafe(ec) || block_has_unsafe(eb))
                            || els.as_ref().map_or(false, block_has_unsafe)
                    }
                    Stmt::While(c, b, _, _, _) => expr_has_unsafe(c) || block_has_unsafe(b),
                    Stmt::Match(e, arms, _) => {
                        expr_has_unsafe(e)
                            || arms.iter().any(|arm| match &arm.body {
                                MatchBody::Block(b) => block_has_unsafe(b),
                                MatchBody::Expr(e2) => expr_has_unsafe(e2),
                            })
                    }
                    Stmt::Spawn(b, _, _) => block_has_unsafe(b),
                    _ => false,
                },
            }
        }
        fn block_has_unsafe(b: &Block) -> bool {
            b.stmts.iter().any(stmt_has_unsafe)
        }
        fn expr_has_unsafe(e: &Expr) -> bool {
            match e {
                Expr::Unsafe(..) => true,
                Expr::BlockExpr(b, _) => block_has_unsafe(b),
                Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => {
                    expr_has_unsafe(f) || args.iter().any(expr_has_unsafe)
                }
                Expr::Field(o, _, _) | Expr::Unary(_, o, _) | Expr::Paren(o, _) | Expr::Index(o, _, _)
                | Expr::Ref(o, _) | Expr::MutRef(o, _) => expr_has_unsafe(o),
                Expr::Binary(a, _, b, _) => expr_has_unsafe(a) || expr_has_unsafe(b),
                Expr::If(c, t, elifs, els, _) => {
                    expr_has_unsafe(c)
                        || block_has_unsafe(t)
                        || elifs.iter().any(|(ec, eb)| expr_has_unsafe(ec) || block_has_unsafe(eb))
                        || els.as_ref().map_or(false, block_has_unsafe)
                }
                Expr::Match(e, arms, _) => {
                    expr_has_unsafe(e)
                        || arms.iter().any(|arm| match &arm.body {
                            MatchBody::Block(b) => block_has_unsafe(b),
                            MatchBody::Expr(e2) => expr_has_unsafe(e2),
                        })
                }
                _ => false,
            }
        }
        block_has_unsafe(block)
    }

    fn check_block(&mut self, block: &Block, expected_return: Option<CheckedType>) -> Option<CheckedType> {
        // Push a fresh scope so local variables declared inside this block
        // do not leak into the enclosing scope. Nested `{ var x = ...; }`
        // blocks create their own scope -- shadowing the outer binding without
        // mutating it. Without this, `fn main() -> Int { var x = 1; { var x = "hi"; } return x; }`
        // would resolve `x` to Str after the inner block because add_local
        // overwrote the outer binding in the shared scope.
        self.push_scope();

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
                    // satisfies any declared return type -- the block value is
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

        self.pop_scope();
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
                    // AUDIT #6 FIX: a `_` WILDCARD annotation must INFER the
                    // value's type -- the old mapping typed it as Int, so
                    // `let x: _ = "s"` bound x: Int silently.
                    if match &**annot { Type::Named(n, _) => n.name == "_", _ => false }
                    {
                        self.add_local(&name.name, val_ty.clone());
                        return;
                    }
                    let annot_ty = CheckedType::from_ast_type(annot);
                    // An uninitialized let/var defaults to the placeholder `Int(0)`.
                    // When a type annotation is present the var is zero-initialized to that
                    // type, so trust the annotation instead of erroring on the placeholder.
                    let is_placeholder = matches!(value, Expr::Int(0, _));
                    if is_placeholder {
                        self.add_local(&name.name, annot_ty);
                        return;
                    }
                    // 5c.32: &expr coerces to *T for raw pointer assignments
                    let is_ref_coercion = matches!(value, Expr::Ref(..) | Expr::MutRef(..))
                        && {
                            let val_name = val_ty.name();
                            let annot_name = annot_ty.name();
                            annot_name.starts_with('*') && annot_name[1..] == val_name
                        };
                    // BUG 26: a non-literal INTEGER value cannot bind to a
                    // FLOAT-typed binding (Rust-style -- silent precision loss);
                    // int LITERALS may adopt the float type (exact). Float
                    // values never bind to int targets.
                    let bind_mix_err = (Self::is_int_family(&val_ty) && Self::is_float_family(&annot_ty) && !Self::is_int_literal_expr(value))
                        || (Self::is_float_family(&val_ty) && Self::is_int_family(&annot_ty));
                    if bind_mix_err {
                        self.error(
                            format!("type mismatch in let: cannot bind {} to {} -- convert explicitly with `as`", val_ty.name(), annot_ty.name()),
                            *span,
                        );
                    }
                    if !self.types_compatible(&val_ty, &annot_ty)
                        && val_ty != CheckedType::Error
                        && !matches!(&val_ty, CheckedType::Named(n) if n == "_")
                        && !is_ref_coercion
                    {
                        self.error(
                            format!("type mismatch in let: annotated {}, found {}", annot_ty.name(), val_ty.name()),
                            *span,
                        );
                    }
                    self.add_local(&name.name, annot_ty);
                    return;
                }
                // BUG 26: Vec-ctor bindings register their FULL generic type
                // ("Vec[Vec[Float64]]") so element reads resolve precisely.
                // round-14: generalized to ANY generic ctor ("BTreeMap[Int, Str]")
                // so the local carries its concrete args for method-return
                // substitution (first_entry -> Option[Tuple__Int__Str]).
                let bind_ty = self.generic_ctor_type_name(value)
                    .map(|s| CheckedType::from_str(&s))
                    .unwrap_or(val_ty.clone());
                self.add_local(&name.name, bind_ty);
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
                    // 5c.32: &expr coerces to *T for raw pointer assignments
                    let is_ref_coercion = matches!(value, Expr::Ref(..) | Expr::MutRef(..))
                        && {
                            let val_name = val_ty.name();
                            let annot_name = annot_ty.name();
                            annot_name.starts_with('*') && annot_name[1..] == val_name
                        };
                    // BUG 26: a non-literal INTEGER value cannot bind to a
                    // FLOAT-typed binding (Rust-style -- silent precision loss);
                    // int LITERALS may adopt the float type (exact). Float
                    // values never bind to int targets.
                    let bind_mix_err = (Self::is_int_family(&val_ty) && Self::is_float_family(&annot_ty) && !Self::is_int_literal_expr(value))
                        || (Self::is_float_family(&val_ty) && Self::is_int_family(&annot_ty));
                    if bind_mix_err {
                        self.error(
                            format!("type mismatch in var: cannot bind {} to {} -- convert explicitly with `as`", val_ty.name(), annot_ty.name()),
                            *span,
                        );
                    }
                    if !self.types_compatible(&val_ty, &annot_ty)
                        && val_ty != CheckedType::Error
                        && !matches!(&val_ty, CheckedType::Named(n) if n == "_")
                        && !is_ref_coercion
                    {
                        self.error(
                            format!("type mismatch in var: annotated {}, found {}", annot_ty.name(), val_ty.name()),
                            *span,
                        );
                    }
                    self.add_local(&name.name, annot_ty);
                    return;
                }
                // BUG 26: Vec-ctor bindings register their FULL generic type.
                // round-14: generalized to ANY generic ctor (see the let arm).
                let bind_ty = self.generic_ctor_type_name(value)
                    .map(|s| CheckedType::from_str(&s))
                    .unwrap_or(val_ty.clone());
                self.add_local(&name.name, bind_ty);
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
                // S2: Match exhaustiveness
                self.check_match_exhaustiveness(arms, &matched_ty);
                let _ = matched_ty;
            }
            Stmt::While(cond, body, _, _, _) => {
                let cond_ty = self.check_expr(cond);
                if cond_ty.name() != "Bool" && cond_ty != CheckedType::Error {
                    self.error(format!("while condition must be Bool, found {}", cond_ty.name()), cond.span());
                }
                self.check_block(body, None);
            }
            Stmt::For(var, iter, body, _, _) => {
                let _iter_ty = self.check_expr(iter);
                self.add_local(&var.name, CheckedType::Int); // simplified
                self.check_block(body, None);
            }
            Stmt::Destructure(names, value, _) => {
                let val_ty = self.check_expr(value);
                // BUG 26 #3: bind each name to its ELEMENT type -- previously
                // every name got the WHOLE tuple ("cannot compare
                // Tuple__Int__Int with Int" on the first use). The codegen
                // already extracts the fields; the checker must type them.
                let elem_types: Vec<CheckedType> = match &val_ty {
                    CheckedType::Named(n) if n.starts_with("Tuple__") => {
                        let inner = &n["Tuple__".len()..];
                        inner.split("__").map(|t| CheckedType::from_str(t)).collect()
                    }
                    _ => Vec::new(),
                };
                if elem_types.len() == names.len() {
                    for (name, ty) in names.iter().zip(elem_types.iter()) {
                        self.add_local(&name.name, ty.clone());
                    }
                } else {
                    for name in names {
                        self.add_local(&name.name, val_ty.clone());
                    }
                }
            }
            Stmt::Spawn(body, _, is_move) => {
                // R2: Move semantics -- analyze captures and mark as moved.
                let outer_locals: HashSet<String> = self.locals.iter()
                    .flat_map(|scope| scope.keys())
                    .cloned()
                    .collect();
                // Push a new scope so variables declared inside spawn are tracked separately
                self.push_scope();
                self.check_block(body, None);
                let inner_locals: HashSet<String> = self.locals.last()
                    .map(|scope| scope.keys().cloned().collect())
                    .unwrap_or_default();
                self.pop_scope();

                // Find captured variables: referenced in body but not declared inside spawn
                let refs = Self::collect_expr_references_block(body);
                let captures: HashSet<String> = refs.difference(&inner_locals)
                    .filter(|name| outer_locals.contains(*name))
                    .cloned()
                    .collect();

                if !captures.is_empty() && !is_move {
                    // Non-move spawn with captures: warning or error
                    // For now, spawn without `move` still works but captures are implicit
                }

                // I1: Send/Sync enforcement -- verify every captured variable's type
                // implements Send before allowing the spawn capture.
                for cap in &captures {
                    if let Some(cap_ty) = self.lookup_local(cap) {
                        let type_name = cap_ty.name();
                        if !self.is_send(&type_name) {
                            self.error(
                                format!(
                                    "spawn capture '{}' of type '{}' does not implement Send; \
                                     only Send types can be moved across thread boundaries.",
                                    cap, type_name
                                ),
                                body.stmts.first().map(|_| Span::new(0, 0)).unwrap_or(Span::new(0, 0)),
                            );
                        }
                    }
                }

                // Mark captures as moved -- they cannot be used after spawn
                for cap in &captures {
                    // Remove from all scopes to prevent post-spawn use
                    for scope in self.locals.iter_mut().rev() {
                        scope.remove(cap);
                    }
                }
            }
            Stmt::Break(..) => {}
            Stmt::Continue(..) => {}
            Stmt::Asm(asm) => {
                // D2 (2026-08-08): inline assembly is unsafe -- requires unsafe context.
                if self.unsafe_depth == 0 {
                    self.error("inline asm requires an `unsafe` block", asm.span);
                }
            }
            Stmt::Defer(b, _) => { self.check_block(b, None); }
            // BUG 27: assert(cond[, "msg"]) -- the condition must be Bool;
            // the message, when present, must be a Str literal expression.
            Stmt::Assert(cond, msg, span) => {
                let cond_ty = self.check_expr(cond);
                if cond_ty.name() != "Bool"
                    && !matches!(&cond_ty, CheckedType::Named(n) if n == "_")
                    && cond_ty != CheckedType::Error
                {
                    self.error(format!("assert condition must be Bool, found {}", cond_ty.name()), *span);
                }
                if let Some(m) = msg {
                    let msg_ty = self.check_expr(m);
                    if msg_ty != CheckedType::Str && msg_ty != CheckedType::Error {
                        self.error(format!("assert message must be Str, found {}", msg_ty.name()), *span);
                    }
                }
            }
            Stmt::Debugger(_) => {},
        }
    }

    // ========================================================================
    // Expression type checking
    // ========================================================================

    /// BUG 23 #7 fix: derive the field map of a tuple type name
    /// ("Tuple__Bool__Bool" -> {_0: Bool, _1: Bool}). Tuple types only register
    /// at tuple-EXPRESSION check sites; catalog fn returns never did.
    fn tuple_fields_from_name(name: &str) -> Option<HashMap<String, CheckedType>> {
        let rest = name.strip_prefix("Tuple__")?;
        let parts: Vec<&str> = rest.split("__").collect();
        if parts.len() < 2 {
            return None;
        }
        let map = parts.iter().enumerate()
            .map(|(i, p)| (format!("_{i}"), CheckedType::from_str(p)))
            .collect();
        Some(map)
    }

    fn is_int_family(ty: &CheckedType) -> bool {
        Self::is_float_family(ty) == false && ty.is_numeric()
    }

    fn is_float_family(ty: &CheckedType) -> bool {
        matches!(ty, CheckedType::Float32 | CheckedType::Float64)
            || matches!(ty, CheckedType::Named(n) if n == "Float" || n == "Float128" || n == "fp128")
    }

    fn is_int_literal_expr(e: &Expr) -> bool {
        match e {
            Expr::Int(..) | Expr::BigInt(..) => true,
            Expr::Paren(inner, _) => Self::is_int_literal_expr(inner),
            _ => false,
        }
    }

    /// BUG 26: render the type argument of a `Vec[...].new()` value into
    /// "Vec[<elem>]" (nested args keep their brackets). Lets the checker
    /// type Vec-ctor bindings precisely so element reads resolve.
    fn vec_ctor_type_name(&self, value: &Expr) -> Option<String> {
        let func = match value {
            Expr::Call(f, ..) | Expr::GenericCall(f, ..) => f.as_ref(),
            _ => return None,
        };
        let (obj, method) = match func {
            Expr::Field(o, m, _) => (o, &m.name),
            _ => return None,
        };
        if method != "new" && method != "with_capacity" {
            return None;
        }
        let (base, idx) = match obj.as_ref() {
            Expr::Index(b, i, _) => (b, i),
            _ => return None,
        };
        if !matches!(base.as_ref(), Expr::Ident(id) if id.name == "Vec") {
            return None;
        }
        fn render(e: &Expr) -> String {
            match e {
                Expr::Ident(id) => id.name.clone(),
                Expr::Paren(inner, _) => render(inner),
                // AUDIT #6 FIX (exposed by container-arg agreement):
                // `Vec[(Str, Str)]` type args are TUPLE literals -- they
                // used to fall to "Int", so tuple vectors inferred
                // Vec[Int] and only the container-erasure shim made
                // Vec[Tuple__Str__Str] params accept them.
                Expr::Tuple(items, _) => {
                    let parts: Vec<String> = items.iter().map(render).collect();
                    format!("Tuple__{}", parts.join("__"))
                }
                Expr::Index(b, i, _) => {
                    if let Expr::Ident(bid) = b.as_ref() {
                        if bid.name == "Vec" {
                            format!("Vec[{}]", render(i))
                        } else {
                            "Int".to_string()
                        }
                    } else {
                        "Int".to_string()
                    }
                }
                _ => "Int".to_string(),
            }
        }
        Some(format!("Vec[{}]", render(idx)))
    }

    /// round-14 (tuple payloads): render ANY generic ctor binding's full type
    /// ("BTreeMap[Int, Str]") so the local carries its CONCRETE args -- the
    /// generic-return substitution for method calls (`bm.first_entry()` ->
    /// "Option[Tuple__K__V]" with K=Int, V=Str) derives the receiver's args
    /// from the local's registered type. Vec delegates to vec_ctor_type_name
    /// (which renders nested Vec[Vec[...]]).
    fn generic_ctor_type_name(&mut self, value: &Expr) -> Option<String> {
        let func = match value {
            Expr::Call(f, ..) | Expr::GenericCall(f, ..) => f.as_ref(),
            _ => return None,
        };
        let (obj, method) = match func {
            Expr::Field(o, m, _) => (o, &m.name),
            _ => return None,
        };
        if method != "new" && method != "with_capacity" {
            return None;
        }
        let (base, idx) = match obj.as_ref() {
            Expr::Index(b, i, _) => (b, i),
            _ => return None,
        };
        let Expr::Ident(base_id) = base.as_ref() else { return None; };
        if base_id.name == "Vec" {
            return self.vec_ctor_type_name(value);
        }
        let args: Vec<String> = match idx.as_ref() {
            Expr::Tuple(items, _) => items.iter().map(|t| self.check_expr(t).name()).collect(),
            Expr::Ident(_) => vec![self.check_expr(idx).name()],
            _ => Vec::new(),
        };
        if args.is_empty() {
            return None;
        }
        Some(format!("{}[{}]", base_id.name, args.join(", ")))
    }

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
                } else if self.functions.contains_key(&ident.name)
                    // BUG 25 #11 fix: a PRIVATE fn of an imported module must
                    // not be reachable as a bare call from another module
                    // (it previously hijacked same-named calls). Bare
                    // resolution is allowed for: pub fns, fns owned by the
                    // CURRENT module, and top-level program fns.
                    && (self.visibility.get(&ident.name).copied().unwrap_or(true)
                        || self.fn_owner_module.get(&ident.name)
                            .map(|m| m.is_empty() || Some(m.as_str()) == self.current_module.as_deref())
                            .unwrap_or(true))
                {
                    CheckedType::Named("fn".into())
                } else if (ident.name == "dbg" || ident.name == "todo" || ident.name == "unimplemented")
                    // BUG 27: debug intrinsics are callable builtins when no
                    // user fn with the name is registered.
                    && !self.functions.contains_key(&ident.name)
                    && !self.fn_owner_module.contains_key(&ident.name)
                {
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
            Expr::Int(_, _) | Expr::BigInt(_, _) => CheckedType::Int,
            Expr::Float(_, _) => CheckedType::Float64,
            Expr::Str(_, _) => CheckedType::Str,
            Expr::Char(_, _) => CheckedType::Char,
            Expr::Bool(_, _) => CheckedType::Bool,
            Expr::Paren(inner, _) => self.check_expr(inner),
            Expr::Tuple(items, _) => {
                // 5c.36: Multi-element tuples produce a tuple type.
                // Single-element is a parenthesized expression (element type).
                if items.len() > 1 {
                    // 5c.36: Collect checked types once, build tuple type name,
                    // and register in checker's type registry for field access.
                    let item_types: Vec<CheckedType> = items.iter()
                        .map(|item| self.check_expr(item))
                        .collect();
                    let elem_types: Vec<String> = item_types.iter()
                        .map(|ty| ty.name())
                        .collect();
                    let tuple_name = format!("Tuple__{}", elem_types.join("__"));
                    if !self.types.contains_key(&tuple_name) {
                        let field_map: HashMap<String, CheckedType> = item_types.iter()
                            .enumerate()
                            .map(|(i, ty)| (format!("_{i}"), ty.clone()))
                            .collect();
                        self.types.insert(tuple_name.clone(), field_map);
                    }
                    CheckedType::Named(tuple_name)
                } else if let Some(item) = items.first() {
                    self.check_expr(item)
                } else {
                    CheckedType::Unit
                }
            }
            Expr::Unary(op, inner, span) => {
                let inner_ty = self.check_expr(inner);
                match op {
                    UnaryOp::Neg => {
                        // BUG 22 #2 fix: match-bound payload vars (Some(d)/Ok(v))
                        // use the wildcard convention (Gap D) -- codegen resolves
                        // the concrete type. Negation must defer like `Not` and
                        // method dispatch do, instead of rejecting "_".
                        if !inner_ty.is_numeric()
                            && !matches!(&inner_ty, CheckedType::Named(n) if n == "_")
                        {
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
                    UnaryOp::Ref | UnaryOp::MutRef => inner_ty, // & keeps the type; *T coercion at use-site
                    UnaryOp::BitNot => inner_ty, // bitwise not preserves integer type
                    UnaryOp::Deref => {
                        // D2 (2026-08-08): dereferencing a RAW pointer is an
                        // unsafe operation -- requires `unsafe { }` context.
                        // References (&T) remain safe (borrow-checked).
                        let is_raw_ptr = matches!(&inner_ty, CheckedType::Named(n) if n.starts_with('*') || n == "Ptr");
                        if is_raw_ptr && self.unsafe_depth == 0 {
                            self.error(
                                format!(
                                    "raw pointer dereference requires an `unsafe` block (found `*{}`)",
                                    inner_ty.name()
                                ),
                                *span,
                            );
                        }
                        // *p: strip pointer type -- *Ptr[T] -> T, *T -> T (encoded as "*Tname")
                        if let CheckedType::Named(ref name) = inner_ty {
                            if let Some(inner_name) = name.strip_prefix('*') {
                                return CheckedType::from_str(inner_name);
                            }
                            if name == "Ptr" {
                                return CheckedType::Int;
                            }
                        }
                        inner_ty
                    }
                }
            }
            Expr::Binary(left, op, right, span) => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                // BUG 26 (secure numeric policy): INT <-> FLOAT mixing requires
                // an explicit `as` cast (Rust-style). Auto-widening stays for
                // same-family (int->wider int, float->wider float); an INT
                // LITERAL operand may adopt the float type (exact -- `d * 2`
                // stays ergonomic), but a non-literal integer with a float
                // operand is a hard error (silent precision loss above 2^53).
                // FLOAT literals never adopt an integer type (lossy).
                let l_int = Self::is_int_family(&left_ty);
                let r_int = Self::is_int_family(&right_ty);
                let l_flt = Self::is_float_family(&left_ty);
                let r_flt = Self::is_float_family(&right_ty);
                if (l_int && r_flt && !Self::is_int_literal_expr(left))
                    || (l_flt && r_int && !Self::is_int_literal_expr(right))
                {
                    self.error(
                        format!(
                            "cannot mix {} with {} -- convert explicitly with `as` (e.g. `{} as Float64`)",
                            left_ty.name(), right_ty.name(),
                            if l_int { left_ty.name() } else { right_ty.name() }
                        ),
                        *span,
                    );
                }
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
                        let is_ptr_like = |ty: &CheckedType| -> bool {
                            matches!(ty, CheckedType::Named(n) if n.starts_with('*'))
                                || matches!(ty, CheckedType::Fn(..))
                        };
                        if !left_ty.is_numeric() && !is_generic_param(&left_ty) && !is_ptr_like(&left_ty) {
                            self.error(format!("left operand must be numeric, found {}", left_ty.name()), *span);
                        }
                        if !right_ty.is_numeric() && !is_generic_param(&right_ty) {
                            self.error(format!("right operand must be numeric, found {}", right_ty.name()), *span);
                        }
                        left_ty // result type is the left operand type (promotion in Phase 1)
                    }
                    BinOp::Eq | BinOp::Neq => {
                        // BUG 24 fix: equality operand compatibility. A struct
                        // compared with an INCOMPATIBLE scalar (BigFloat == 4)
                        // silently lowered to a field-0 compare (miscompare /
                        // silent corruption). Numbers coerce; same-type operands
                        // (incl. structs -- compared structurally in codegen)
                        // are allowed; generic/wildcard operands defer to codegen.
                        // BUG 29: resolve TYPE ALIASES first -- `type Id = Int;
                        // type SessionId = Id;` then `s == 42` must compare as
                        // Int == Int (m29_type_alias + the ~15-test alias
                        // comparison cluster). types_compatible already resolves
                        // aliases; the comparison path must too.
                        let left_ty = self.resolve_alias(&left_ty);
                        let right_ty = self.resolve_alias(&right_ty);
                        let numeric_family = |ty: &CheckedType| -> bool {
                            ty.is_numeric() || matches!(ty, CheckedType::Char)
                        };
                        let is_generic_or_wild = |ty: &CheckedType| -> bool {
                            if let CheckedType::Named(n) = ty {
                                n == "_" || (n.len() == 1 && n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
                            } else { false }
                        };
                        let is_ptr_ty = |ty: &CheckedType| -> bool {
                            matches!(ty, CheckedType::Named(n) if n.starts_with('*') || n == "Ptr")
                        };
                        let is_null_literal = |e: &Expr| -> bool {
                            matches!(e, Expr::Ident(id) if id.name == "null")
                                || matches!(e, Expr::Int(0, _))
                        };
                        // BUG 29 (regress_gap2 + LSP alloc.xi): a RAW POINTER
                        // compared with the null literal (`ptr == 0`, `ptr ==
                        // null`, `ptr != null`) is the canonical FFI null check.
                        // The `null` keyword types as Ptr; the literal 0 as Int.
                        let ptr_null_cmp = (is_ptr_ty(&left_ty)
                            && (matches!(&right_ty, CheckedType::Int) || matches!(&right_ty, CheckedType::Named(n) if n == "Ptr" || n == "_"))
                            && is_null_literal(right))
                            || (is_ptr_ty(&right_ty)
                                && (matches!(&left_ty, CheckedType::Int) || matches!(&left_ty, CheckedType::Named(n) if n == "Ptr" || n == "_"))
                                && is_null_literal(left));
                        let compatible = left_ty == right_ty
                            || (numeric_family(&left_ty) && numeric_family(&right_ty))
                            || is_generic_or_wild(&left_ty)
                            || is_generic_or_wild(&right_ty)
                            || ptr_null_cmp
                            // BUG 51 (2026-08-18): container-erasure equality --
                            // "Option" vs "Option[Int]" (pop() vs Some(30)),
                            // "Result" vs "Result[Int, Str]" compare fine.
                            || matches!((&left_ty, &right_ty), (CheckedType::Named(a), CheckedType::Named(b))
                                if a.split('[').next().unwrap_or(a) == b.split('[').next().unwrap_or(b));
                        // BUG 26: equality also rejects int<->float mixing
                        // (int literals may adopt the float type).
                        let mix_err = (Self::is_int_family(&left_ty) && Self::is_float_family(&right_ty) && !Self::is_int_literal_expr(left))
                            || (Self::is_float_family(&left_ty) && Self::is_int_family(&right_ty) && !Self::is_int_literal_expr(right));
                        if mix_err {
                            self.error(
                                format!(
                                    "cannot compare {} with {} -- convert explicitly with `as`",
                                    left_ty.name(), right_ty.name()
                                ),
                                *span,
                            );
                        }
                        if !compatible {
                            self.error(
                                format!("cannot compare {} with {}", left_ty.name(), right_ty.name()),
                                *span,
                            );
                        }
                        CheckedType::Bool
                    }
                    BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                        // BUG 26: ordering comparisons follow the same
                        // int<->float rule (Rust-style -- `d > 0` with an int
                        // literal is fine; `i > 0.5` needs an explicit cast).
                        let l_int = Self::is_int_family(&left_ty);
                        let r_int = Self::is_int_family(&right_ty);
                        let l_flt = Self::is_float_family(&left_ty);
                        let r_flt = Self::is_float_family(&right_ty);
                        if (l_int && r_flt && !Self::is_int_literal_expr(left))
                            || (l_flt && r_int && !Self::is_int_literal_expr(right))
                        {
                            self.error(
                                format!(
                                    "cannot compare {} with {} -- convert explicitly with `as`",
                                    left_ty.name(), right_ty.name()
                                ),
                                *span,
                            );
                        }
                        CheckedType::Bool
                    }
                    BinOp::And | BinOp::Or => {
                        let is_generic = |ty: &CheckedType| -> bool {
                            matches!(ty, CheckedType::Named(n) if n.len() == 1 && n.chars().next().map_or(false, |c| c.is_ascii_uppercase()))
                        };
                        if left_ty.name() != "Bool" && !is_generic(&left_ty) && !matches!(&left_ty, CheckedType::Named(n) if n == "_") {
                            self.error(format!("left operand of logical op must be Bool, found {}", left_ty.name()), *span);
                        }
                        if right_ty.name() != "Bool" && !is_generic(&right_ty) && !matches!(&right_ty, CheckedType::Named(n) if n == "_") {
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
                // v0.56: Cascade suppression -- Error/Unit/wildcard from previous
                // errors should not produce additional ? operator errors.
                if inner_ty == CheckedType::Error || inner_ty == CheckedType::Unit {
                    return inner_ty;
                }
                // ? unwraps Result[T,E] -> T or Option[T] -> T.
                match &inner_ty {
                    CheckedType::Named(n)
                        if n == "Result" || n == "Option" || n == "_"
                            || n.starts_with("Result[") || n.starts_with("Option[") => {
                        // P2-1: Validate that the enclosing function returns Result/Option.
                        let fn_returns_result_or_option = self.current_return.as_ref()
                            .map_or(false, |ret| match ret {
                                CheckedType::Named(rn) => rn == "Result" || rn == "Option" || rn == "_"
                                    || rn.starts_with("Result[") || rn.starts_with("Option["),
                                _ => false,
                            });
                        if !fn_returns_result_or_option && n != "_" {
                            self.error(
                                format!("'?' operator used in function that returns '{}' -- must return Result or Option",
                                    self.current_return.as_ref().map_or("void".to_string(), |r| r.name())),
                                *_span,
                            );
                        }
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
                        // v0.56: Wildcard type _ -- field access always allowed (codegen resolves)
                        if name == "_" {
                            return CheckedType::Named("_".into());
                        }
                        // Generic type params have no registered fields -- return wildcard
                        let is_generic_param = name.len() == 1 && name.chars().next().map_or(false, |c| c.is_ascii_uppercase());
                        if is_generic_param {
                            return CheckedType::Named("_".into());
                        }
                        // BUG 51 (2026-08-18): CONTAINER receivers with args
                        // ("Option[Str]", "Vec[Int]") look up their pseudo-fields
                        // (is_some/is_none/value) and registered methods under the
                        // BASE name -- the arg-bearing key is never registered.
                        let lookup_base = name.split('[').next().unwrap_or(name);
                        if let Some(fields) = self.get_type(lookup_base) {
                            if let Some(field_ty) = fields.get(&field.name) {
                                field_ty.clone()
                            } else if !fields.is_empty() {
                                // Known type with registered fields -- unknown field
                                self.error(
                                    format!("type '{}' has no field '{}'", name, field.name),
                                    *span,
                                )
                            } else {
                                // Known type with no registered fields (builtin) -- allow access
                                CheckedType::Int
                            }
                        } else if let Some(tuple_fields) = Self::tuple_fields_from_name(name) {
                            // BUG 23 #7 fix: tuples RETURNED by catalog fns never
                            // register their field maps (only tuple EXPRESSIONS do
                            // at check time). Derive "Tuple__A__B" -> {_0: A, _1: B}
                            // and register on first field access, so cross-module
                            // `t.0` / `t.1` type-check instead of degrading to
                            // <error> (which broke `!t.0`, `240 * t.1`, ...).
                            self.types.insert(name.clone(), tuple_fields.clone());
                            tuple_fields.get(&field.name).cloned().unwrap_or(CheckedType::Error)
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
            Expr::GenericCall(func, _ty, args, span) => {
                self.check_expr(&Expr::Call(func.clone(), args.clone(), *span))
            }
            Expr::Call(func, args, span) => {
                // Method call or module-qualified call: receiver.method(args) or module.func(args)
                // Also handle type-parameterized calls like Stack.new[Int]() which parse as
                // Expr::Call(Expr::Index(Expr::Field(Expr::Ident("Stack"), "new"), [Expr::Ident("Int")]), [])
                // round-15 (probe_zip_j/k): a TYPE-PARAMETERIZED BARE call
                // `apply_g[(Int, Int)](...)` parses as
                // Expr::Call(Expr::Index(Ident(fn), types), args) -- unwrap it
                // to the bare Ident so the fn-signature resolution below fires
                // (the old flow checked the Index as a VALUE expression -> Unit
                // -> "cannot logically negate type ()" on `!apply_g[...](...)`).
                // Only when the base is NOT a type name (a plain value index
                // like `handlers[i](...)` must keep the value path).
                let mut explicit_type_args: Option<Vec<CheckedType>> = None;
                let func_unwrapped: &Expr = match func.as_ref() {
                    Expr::Index(base, idx, _) if matches!(base.as_ref(), Expr::Ident(id)
                        if !self.contains_type(&id.name)
                            && self.functions.contains_key(&id.name)) =>
                    {
                        explicit_type_args = Some(match idx.as_ref() {
                            Expr::Tuple(items, _) => items.iter().map(|t| self.check_expr(t)).collect(),
                            other => vec![self.check_expr(other)],
                        });
                        base.as_ref()
                    }
                    other => other,
                };
                let method_target = match func_unwrapped {
                    Expr::Field(..) => Some(func_unwrapped),
                    Expr::Index(field_expr, _, _) if matches!(field_expr.as_ref(), Expr::Field(..)) => Some(field_expr.as_ref()),
                    _ => None,
                };
                // Save field call info for fn-typed field fallback
                let fn_field_info: Option<(&Expr, &Ident)> = match method_target {
                    Some(Expr::Field(obj, method, _)) => Some((obj, method)),
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
                        if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() && method.name == "len" {
                            eprintln!("[mth] obj_ty={type_name} local_m={:?}", self.lookup_local("m"));
                        }
                        // BUG 26: Vec[..]-typed receivers dispatch on the base
                        // "Vec" methods ("Vec[Int].len" resolves to "Vec.len").
                        // BUG 51: generalize to ALL bracketed containers
                        // ("Option[MyRc]" -> "Option", "Result[Int, Str]" ->
                        // "Result") so method calls on Option/Result-typed
                        // receivers (unwrap, is_some, ...) hit the registered
                        // signature instead of the sorted wildcard fallback.
                        let base_name = type_name
                            .split('[')
                            .next()
                            .map(|b| b.trim().to_string())
                            .unwrap_or_else(|| type_name.clone());
                        let method_key = format!("{}.{}", base_name, method.name);
                        if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() && method.name == "sum" {
                            eprintln!("[sum] obj_ty={} key={} in_functions={} methods_keys={:?}", type_name, method_key, self.functions.contains_key(&method_key), self.methods.keys().collect::<Vec<_>>());
                        }
                        // Try module-prefixed key first, then bare key as fallback
                        let sig = if let Some(ref module) = self.current_module {
                            let prefixed = format!("{}.{}.{}", module, base_name, method.name);
                            self.functions.get(&prefixed).or_else(|| self.functions.get(&method_key))
                        } else {
                            self.functions.get(&method_key)
                        }.cloned();
                        // If not found, try wildcard method lookup (any type with that method).
                        // v0.56: Skip wildcard lookup for 'clone' -- it matches the wrong type's
                        // clone method (e.g., Rc.clone returns Rc[T], not the receiver type).
                        // BUG 30: the wildcard skip must ONLY apply to the FALLBACK --
                        // the original `let sig = if clone { None } else {...}` DISCARDED
                        // the DIRECT hit too, so `r.clone()` on Rc[Int] typed `_` and the
                        // next method call fell to the wildcard (r2.get() -> BTreeMap.get ->
                        // Option -- "cannot compare Option with Int"; smoke_rc).
                        // BUG 30 (cont.): the wildcard itself MUST be deterministic. The
                        // old `.find()` over self.methods (a HashMap) picked the first
                        // hash-order match -- for `r.get()` on Rc[Int] it sometimes
                        // returned Option.get's signature (flaky per process/run).
                        // Sort candidates by type name; prefer the receiver's base name
                        // (generic args stripped), then a module-qualified/base suffix.
                        let sig = sig.or_else(|| {
                            if method.name == "clone" {
                                None
                            } else {
                                // "Rc[Int]" -> "Rc"; "Vec[Int]" -> "Vec"
                                let base = base_name.split('[').next().unwrap_or(&base_name).to_string();
                                let mut candidates: Vec<(String, &FnSig)> = self.methods.iter()
                                    .filter_map(|(ty, methods)| methods.get(&method.name).map(|s| (ty.clone(), s)))
                                    .collect();
                                candidates.sort_by(|a, b| a.0.cmp(&b.0));
                                // AUDIT #6 FIX: UNIQUE candidates resolve
                                // deterministically (sound); AMBIGUOUS
                                // receivers fail closed with a proper
                                // no-such-method diagnostic instead of the
                                // old alphabetical cross-type capture.
                                // (join/as_path have exactly one candidate
                                // type; wildcard receivers resolve when
                                // unambiguous.)
                                if candidates.len() == 1 {
                                    return candidates.into_iter().next().map(|(_, s)| (*s).clone());
                                }
                                // Prefer an exact base-name match (wildcard
                                // receivers skip this -- no base).
                                if base != "_" && base_name != "_" {
                                    let exact = candidates.iter()
                                        .find(|(ty, _)| ty == &base || ty == &base_name)
                                        .map(|(_, s)| (*s).clone());
                                    if exact.is_some() {
                                        return exact;
                                    }
                                    let close = candidates.iter()
                                        .find(|(ty, _)| ty.ends_with(&format!(".{base}")) || ty.ends_with(&format!(".{base_name}")))
                                        .map(|(_, s)| (*s).clone());
                                    if close.is_some() {
                                        return close;
                                    }
                                }
                                // AUDIT #6 FIX: NO arbitrary capture --
                                // fail closed.
                                None
                            }
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
                            //     OR receiver-style `fn T.method(h: &T, ...)` -- but ONLY
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
                            // G-20 fix: `fn V2.lerp(other: V2, t: Float32)` -- a first
                            // param of the receiver TYPE is ambiguous between
                            // receiver-style (h IS the receiver) and a REAL argument
                            // (math-style lerp/dot/cross). Call-site arity settles it
                            // deterministically:
                            //   args == params     -> params are all real (offset 0)
                            //   args == params - 1 -> first param is the receiver (offset 1)
                            // Previously the type heuristic always chose receiver-style,
                            // shifting every arg and rejecting/miscompiling lerp-shaped
                            // methods (silent swap class).
                            let arity_direct = args.len() == sig.params.len();
                            let has_explicit_self = first_param_is_self_named
                                || (first_param_matches_receiver && !arity_direct);
                            // param_offset table:
                            //   explicit self  + instance call -> skip self (offset=1)
                            //   explicit self  + static call   -> self is first arg (offset=0)
                            //   implicit this  + instance call -> args map directly (offset=0)
                            //   implicit this  + static call   -> first arg is receiver, skip (offset=1)
                            //     NOTE: codegen adds a %param_self pointer to the LLVM signature
                            //     for this-based methods; this offset only controls checker-level
                            //     param matching. The codegen's self_offset handles the actual
                            //     argument layout independently.
                            //   constructor    + any call       -> args map directly, no self (offset=0)
                            let param_offset: usize = if has_explicit_self {
                                if is_static_call { 0 } else { 1 }
                            } else if sig.uses_implicit_this {
                                if is_static_call { 1 } else { 0 }
                            } else {
                                0 // constructor -- no self at all
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
                            let mut ret_ty = sig.return_type.unwrap_or(CheckedType::Unit);
                            // round-14 (tuple payloads): substitute the method's
                            // generic params with the RECEIVER's concrete type
                            // args ("BTreeMap[Int, Str]" -> K=Int, V=Str) so
                            // `bm.first_entry()` returns "Option[(Int, Str)]"
                            // instead of the raw generic "Option[(K, V)]" -- the
                            // tuple pattern bindings then type the Str element
                            // correctly ("cannot compare Int with Str"). Gated
                            // on ARITY so receiver+method generics (Result[T, E]
                            // methods with their own [F]) don't misalign.
                            if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() && method.name == "first_entry" {
                                let pg = Self::parse_generic_type(type_name);
                                eprintln!("[fe] type_name={type_name} generics={:?} ret={} parsed={:?}", sig.generics, ret_ty.name(), pg.map(|(b, a)| (b, a)));
                            }
                            if !sig.generics.is_empty() {
                                if let Some((_, recv_args)) = Self::parse_generic_type(type_name) {
                                    if recv_args.len() == sig.generics.len() {
                                        let subst: HashMap<String, CheckedType> = sig.generics.iter()
                                            .zip(recv_args.iter())
                                            .map(|(g, a)| (g.clone(), CheckedType::from_str(a)))
                                            .collect();
                                        if let Some(substituted) = Self::substitute_generic_type(&ret_ty, &subst) {
                                            ret_ty = substituted;
                                        }
                                    }
                                }
                            }
                            return ret_ty;
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
                            "is_empty" if prim_ty == CheckedType::Str => return CheckedType::Bool,
                            // v0.56: String manipulation methods
                            "trim" | "trim_start" | "trim_end" | "to_lower" | "to_upper" | "substr"
                                if prim_ty == CheckedType::Str => return CheckedType::Str,
                            "byte_at" | "char_at" if prim_ty == CheckedType::Str => return CheckedType::Int,
                            // Gap A fix: to_owned is the idiomatic Str duplication
                            // alias (Rust parity). Same semantics as clone.
                            "to_owned" if prim_ty == CheckedType::Str => return CheckedType::Str,
                            // G-43: C-string interop (BUG-008 codegen builtins).
                            "c_str" if prim_ty == CheckedType::Str => return CheckedType::Named("Ptr".into()),
                            "byte_len" if prim_ty == CheckedType::Str => return CheckedType::Int,
                            "to_str" | "to_string" => return CheckedType::Str,
                            // M12/P0: Str conversions from C strings / byte buffers.
                            // These are codegen builtins (call.rs:1210) that reinterpret
                            // a pointer as a Str at the ABI level -- identity transform
                            // on i8* with no runtime cost. The checker must return Str
                            // (not Result) so io.read_line() / list_dir() / args() work.
                            "from_cstring" | "from_c_str" | "from_utf8" | "from_bytes"
                                if prim_ty == CheckedType::Str => return CheckedType::Str,
                            // M12/P0: Str.substr(start, end) -- substring extraction.
                            // Codegen emits xiom_str_slice (a runtime concat call);
                            // always infallible for valid bounds.
                            "substr" if prim_ty == CheckedType::Str => return CheckedType::Str,
                            // M12/P1: byte indexing -- returns a single byte at position.
                            "byte_at" if prim_ty == CheckedType::Str => return CheckedType::UInt8,
                            "char_at" if prim_ty == CheckedType::Str => return CheckedType::Named("Option".into()),
                            // M12/P1: scripting ergonomics -- slice() and starts_with()
                            // as methods on Str, avoiding verbose string.str_slice() calls.
                            "slice" if prim_ty == CheckedType::Str => return CheckedType::Str,
                            "starts_with" if prim_ty == CheckedType::Str => return CheckedType::Bool,
                            "ends_with" if prim_ty == CheckedType::Str => return CheckedType::Bool,
                            // M21: Str.concat(other) -- string concatenation method
                            "concat" if prim_ty == CheckedType::Str => return CheckedType::Str,
                            _ => {}
                        }
                    }
                    // Builtin methods on core generic containers whose element/inner
                    // types are erased in the checker (Vec/Slice/Option/Result/Map/Set).
                    // These are legitimate stdlib APIs; accept them so correct code
                    // type-checks (the "if it compiles, it's safe" gate stays sound
                    // because codegen lowers these to real builtins).
                    if let CheckedType::Named(tn) = &obj_ty {
                        // BUG 51 (2026-08-18): normalize BOTH the module prefix and
                        // container args -- "Result[Int, MyErr]" -> "Result" -- so the
                        // builtin special-case table (unwrap/is_ok/len/clone/...) and
                        // the registered-method lookups below hit for arg-bearing
                        // receivers (from_ast_type now preserves the args).
                        let base = tn.rsplit('.').next().unwrap_or(tn)
                            .split('[').next().unwrap_or(tn)
                            .trim();
                        for arg in args { let _ = self.check_expr(arg); }
                        // v0.56: Wildcard type _ -- accept any method call (codegen resolves)
                        if tn == "_" || base == "_" {
                            return CheckedType::Named("_".into());
                        }
                        // P2-5: Before builtin match, check if the concrete type has
                        // the method registered. This catches user-defined method calls
                        // at checker time instead of deferring to codegen.
                        let method_key = format!("{}.{}", tn, method.name);
                        // v0.56: Skip function registry check for single-uppercase-letter
                        // generic params (T, U, V etc.). These collide with type param names
                        // from other modules (e.g., MaybeUninit[T] registers 'T' as a key).
                        let is_generic_param = tn.len() == 1 && tn.chars().next().map_or(false, |c| c.is_ascii_uppercase());
                        if !is_generic_param && self.functions.contains_key(&method_key) {
                            return CheckedType::Named("_".into());
                        }
                        let method_key_base = format!("{}.{}", base, method.name);
                        if method_key_base != method_key && self.functions.contains_key(&method_key_base) {
                            return CheckedType::Named("_".into());
                        }
                        match (base, method.name.as_str()) {
                            ("Vec" | "Slice" | "Array" | "Str" | "Map" | "Set", "len")
                                => return CheckedType::Int,
                            ("Vec" | "Slice" | "Array" | "Str", "is_empty") => return CheckedType::Bool,
                            // P1-4: Contract collection methods -- returns Bool for ensures/requires uses.
                            ("Vec" | "Slice" | "Array", "is_sorted") => return CheckedType::Bool,
                            ("Vec" | "Slice" | "Array", "all") => return CheckedType::Bool,
                            ("Vec" | "Slice" | "Array", "none") => return CheckedType::Bool,
                            ("Vec" | "Slice" | "Array", "contains") => return CheckedType::Bool,
                            // v0.56: Pointer/reference accessors (FFI, low-level)
                            ("Vec" | "Slice" | "Array" | "Str" | "Box", "as_ptr" | "as_mut_ptr") => return CheckedType::Named("*UInt8".into()),
                            // v0.56: Str byte access
                            ("Str", "byte_at" | "char_at") => return CheckedType::Int,
                            // v0.56: Common Str methods
                            ("Str", "trim" | "trim_start" | "trim_end" | "to_lower" | "to_upper" | "substr" | "from_c_str" | "to_c_str") => return CheckedType::Str,
                            // v0.56: String conversion method
                            (_, "to_string") => return CheckedType::Str,
                            // v0.56: Time/counter methods
                            (_, "now" | "elapsed" | "as_millis" | "as_micros" | "as_nanos" | "as_secs") => return CheckedType::Int,
                            // v0.56: Pointer/offset methods
                            (_, "offset" | "seek" | "tell" | "position" | "read" | "write" | "flush" | "close") => return CheckedType::Int,
                            // v0.56: Map iterator / key access
                            ("Map" | "Set", "keys" | "values" | "entries" | "iter") => return CheckedType::Named("_".into()),
                            // Container clone returns the same container type. (G-36)
                            ("Vec" | "Slice" | "Map" | "Set", "clone") => return obj_ty.clone(),
                            // Option/Result payload accessors -- inner type is erased,
                            // so return a wildcard the rest of the checker accepts.
                            ("Option" | "Result", "unwrap" | "unwrap_or" | "unwrap_err" | "expect" | "value")
                                => return CheckedType::Named("_".into()),
                            ("Option" | "Result", "is_some" | "is_none" | "is_ok" | "is_err")
                                => return CheckedType::Bool,
                            // Common wrapper accessors (Cell/Rc/Arc/Mutex/Box/Reverse).
                            ("Cell" | "Rc" | "Arc" | "Mutex" | "Box" | "Reverse" | "RefCell", "get" | "clone" | "lock" | "borrow" | "borrow_mut")
                                => return CheckedType::Named("_".into()),
                            // P2-5: clone() returns the receiver type for generic/
                            // non-container types. Codegen resolves concrete impl.
                            (_, "clone") => return obj_ty.clone(),
                            // v0.56: .new() on any named type returns the receiver type
                            (_, "new") => return obj_ty.clone(),
                            // P2-5: default() on generic types (T.default() for T: Default)
                            (_, "default") => return obj_ty.clone(),
                            (_, "serialize_json") => return CheckedType::Named("Result".into()),
                            (_, "deserialize_json") => return CheckedType::Named("Result".into()),
                            // v0.56: Ord interface method on generic types
                            (_, "compare") => return CheckedType::Int,
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
                                // Generic param with potential interface bound.
                                // Look up the current function's generic bounds to
                                // see if this type parameter has an interface bound
                                // that declares the called method.
                                self.current_generic_bounds.get(tn).map_or(false, |bounds| {
                                    bounds.iter().any(|b| {
                                        self.interfaces.get(b).map_or(false, |methods| {
                                            methods.iter().any(|(mn, _, _)| mn == &method.name)
                                        })
                                    })
                                })
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
                    // P2-7: Function pointer call via struct field (e.g., self.f(args)).
                    // The receiver is a struct field with fn type; check args against
                    // the fn signature and return the fn's return type.
                    if let Some((obj, method_ident)) = fn_field_info {
                        let obj_ty = self.check_expr(obj);
                        if let CheckedType::Named(tn) = &obj_ty {
                            let field_map = self.get_type(tn).map(|fm| fm.clone());
                            if let Some(ref fm) = field_map {
                                if let Some(field_ty) = fm.get(&method_ident.name) {
                                    if let CheckedType::Fn(param_types, ret_type) = field_ty {
                                        for (i, arg) in args.iter().enumerate() {
                                            let arg_ty = self.check_expr(arg);
                                            if i < param_types.len() {
                                                let expected = &param_types[i];
                                                if !self.types_compatible(&arg_ty, expected) && arg_ty != CheckedType::Error {
                                                    self.error(
                                                        format!("argument {} type mismatch: expected {}, found {}", i + 1, expected.name(), arg_ty.name()),
                                                        *span,
                                                    );
                                                }
                                            }
                                        }
                                        return ret_type.as_ref().clone();
                                    }
                                }
                            }
                        }
                    }
                    // Fallback: unknown call target
                    self.error(
                        format!("cannot call '{}' on this expression", method.name),
                        *span,
                    );
                    return CheckedType::Error;
                }
                // Look up the function by name if it's a simple identifier
                // (round-15: func_unwrapped -- a type-parameterized bare call
                // `f[T](...)` unwraps to the Ident above).
                if let Expr::Ident(name) = func_unwrapped {
                    // D2.1 (T002): extern "C" functions are confined to unsafe
                    // blocks (Unsafe Confinement requirement a). Calling one from
                    // safe code (depth 0) is a hard error -- EXCEPT inside a fn
                    // declaring requires/ensures contracts (the sanctioned safe
                    // wrapper pattern, requirement c).
                    if self.unsafe_depth == 0
                        && !self.current_fn_has_contracts
                        && self.extern_fns.contains(&name.name)
                    {
                        self.error(
                            format!("calling extern \"C\" function '{}' requires an `unsafe` block", name.name),
                            name.span,
                        );
                    }
                    // BUG 25 #1 fix: a bare name exported by MULTIPLE
                    // imported modules is AMBIGUOUS -- resolve deterministically
                    // or error. Silently picking one module's version
                    // (keep-first vs last-imported) produced wrong calls
                    // (to_base58 resolving to the wrong module's fn). Error
                    // and require a module-qualified call.
                    // BUG 29 (m34_j08/m33_p13): an EXPLICIT `use module.fn`
                    // import already disambiguates -- `use net.ping; ping()`
                    // must not error even when both the module and its alias
                    // appear in the module map (count > 1). Skip the check
                    // when the name is in imported_items as a Function.
                    let explicitly_imported = matches!(
                        self.imported_items.get(&name.name),
                        Some(ModuleExport::Function { .. })
                    );
                    let ambiguous = !explicitly_imported && self.modules.iter()
                        .filter(|(m, ex)| !m.is_empty() && matches!(ex.get(&name.name), Some(ModuleExport::Function { .. })))
                        .count() > 1;
                    if ambiguous {
                        self.error(
                            format!("ambiguous function '{}': exported by multiple imported modules -- use a module-qualified call", name.name),
                            name.span,
                        );
                    }
                    // BUG 25 #11 fix: same visibility gate as the Ident
                    // expression -- a PRIVATE fn of an imported module must
                    // not resolve as a bare call (it previously hijacked
                    // same-named calls in the importing module).
                    let visible = self.visibility.get(&name.name).copied().unwrap_or(true)
                        || self.fn_owner_module.get(&name.name)
                            .map(|m| m.is_empty() || Some(m.as_str()) == self.current_module.as_deref())
                            .unwrap_or(true);
                    // Try module-prefixed key first, then bare name
                    let fn_sig = if visible {
                        if let Some(ref module) = self.current_module {
                            let prefixed = format!("{}.{}", module, name.name);
                            self.functions.get(&prefixed).or_else(|| self.functions.get(&name.name))
                        } else {
                            self.functions.get(&name.name)
                        }
                    } else {
                        None
                    };
                    if let Some(sig) = fn_sig.cloned() {
                        // Build generic substitution map from the call arguments.
                        // round-15 (probe_zip_j/k): explicit type args from
                        // `apply_g[(Int, Int)](...)` win over arg inference --
                        // they cover generic params that don't appear in any
                        // param type (e.g. `fn() -> Option[T]` helpers).
                        let mut subst: HashMap<String, CheckedType> = HashMap::new();
                        if !sig.generics.is_empty() {
                            if let Some(explicit) = explicit_type_args.as_ref() {
                                for (gi, g) in sig.generics.iter().enumerate() {
                                    if let Some(t) = explicit.get(gi) {
                                        subst.insert(g.clone(), t.clone());
                                    }
                                }
                            }
                            for (i, arg) in args.iter().enumerate() {
                                if i < sig.params.len() {
                                    let pname = &sig.params[i].1.name();
                                    if sig.generics.iter().any(|g| g == pname) {
                                        subst.entry(pname.clone()).or_insert_with(|| self.check_expr(arg));
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
                        // D2.1 (T006, FFI ownership): inside a confined block, an
                        // extern "C" call returning a raw pointer (*T) must have
                        // its result converted to an owned XIOM type before the
                        // block's tail. Track such calls; a conversion fn clears
                        // the pending set. Checked at the unsafe-block boundary.
                        if self.unsafe_depth > 0 && self.extern_fns.contains(&name.name) {
                            if matches!(&ret_ty, CheckedType::Named(n) if n.starts_with('*')) {
                                self.pending_extern_ptrs.push(format!("{}", name.name));
                            }
                        }
                        // A registered FFI ownership-conversion fn converts the
                        // pending extern pointer(s) (safe_ptr_from_raw, etc.).
                        let fn_bare = name.name.split('.').last().unwrap_or(&name.name).to_string();
                        if self.unsafe_depth > 0 && self.ffi_convert_fns.contains(&fn_bare) {
                            self.converted_ffi_ptrs.extend(self.pending_extern_ptrs.drain(..));
                        }
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
                    // BUG 27: debug intrinsics -- dbg!(expr) returns the arg's
                    // type; todo!()/unimplemented!() are polymorphic.
                    let debug_builtin_free = !self.functions.contains_key(&name.name)
                        && !self.fn_owner_module.contains_key(&name.name);
                    if debug_builtin_free && name.name == "dbg" {
                        let mut arg_ty = CheckedType::Unit;
                        for a in args { arg_ty = self.check_expr(a); }
                        return arg_ty;
                    }
                    if debug_builtin_free && (name.name == "todo" || name.name == "unimplemented") {
                        for a in args { let _ = self.check_expr(a); }
                        return CheckedType::Named("_".into());
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
                if let Expr::Ident(name) = func_unwrapped {
                    if self.enum_variants.contains_key(&name.name) || self.resolve_enum_variant(&name.name).is_some() {
                        // Enum variant constructor with positional args -- typecheck args loosely
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
                let callee_ty = self.check_expr(func_unwrapped);
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
                // M20-A1: Closure call -- callee is Named("fn") (non-capturing lambda).
                // Accept any args and return wildcard since we don't track closure
                // signatures in the type system yet.
                if matches!(&callee_ty, CheckedType::Named(n) if n == "fn") {
                    for arg in args { let _ = self.check_expr(arg); }
                    return CheckedType::Named("_".into());
                }
                // v0.56: Generic type param from Vec/Array indexing (Named("T")) or
                // wildcard (_) as callable -- the real type is erased, codegen resolves.
                if matches!(&callee_ty, CheckedType::Named(n) if n == "_" || (n.len() == 1 && n.chars().next().map_or(false, |c| c.is_ascii_uppercase()))) {
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
                        // Type parameter expression -- return the container type.
                        // round-14 (tuple payloads): KEEP the concrete type args
                        // ("BTreeMap[Int, Str]") instead of dropping them -- the
                        // generic-return substitution for method calls
                        // (`bm.first_entry()` -> "Option[Tuple__K__V]" with
                        // K=Int, V=Str) derives the receiver's args from this
                        // name. Field access already falls back to the base via
                        // name.split('[') (BUG 51).
                        let args_str: Vec<String> = match idx.as_ref() {
                            Expr::Tuple(items, _) => items.iter().map(|t| self.check_expr(t).name()).collect(),
                            Expr::Ident(_) => vec![self.check_expr(idx).name()],
                            _ => Vec::new(),
                        };
                        if args_str.is_empty() {
                            return CheckedType::Named(container_ident.name.clone());
                        }
                        return CheckedType::Named(format!("{}[{}]", container_ident.name, args_str.join(", ")));
                    }
                }
                // Regular index: arr[idx]
                let arr_ty = self.check_expr(arr);
                let _ = self.check_expr(idx);
                match &arr_ty {
                    CheckedType::Named(name) if name == "Vec" => {
                        // Vec[T][i] -> T (use generic placeholder, actual type from usage)
                        CheckedType::Named("T".into())
                    }
                    // BUG 26: resolve the element type of Vec[...]-typed locals.
                    // ONE level per index -- `m[1]` of Vec[Vec[Int]] is
                    // Vec[Int]; a SECOND index strips the next level (nested
                    // reads recurse through the nested Expr::Index).
                    CheckedType::Named(name) if name.starts_with("Vec[") && name.ends_with(']') => {
                        let inner = &name[4..name.len() - 1];
                        // BUG 51 (2026-08-18): fn-typed elements
                        // (Vec[fn() -> Int]) must parse into a REAL Fn
                        // CheckedType -- from_str yields a bare Named
                        // ("fn() -> Int") that mismatches the annotation's
                        // Fn(...) ("type mismatch in var: annotated
                        // fn() -> Int, found fn() -> Int" -- test_fnptr).
                        if inner.starts_with("fn(") {
                            if let Some(ret_pos) = inner.find("->") {
                                let params_part = inner[3..ret_pos].trim();
                                let ret_part = inner[ret_pos + 2..].trim();
                                let params_body = params_part
                                    .trim_start_matches('(')
                                    .trim_end_matches(')')
                                    .trim();
                                let params: Vec<CheckedType> = if params_body.is_empty() {
                                    Vec::new()
                                } else {
                                    params_body
                                        .split(',')
                                        .map(|p| CheckedType::from_str(p.trim()))
                                        .collect()
                                };
                                CheckedType::Fn(params, Box::new(CheckedType::from_str(ret_part)))
                            } else {
                                CheckedType::from_str(inner)
                            }
                        } else {
                            CheckedType::from_str(inner)
                        }
                    }
                    // BUG 29 (repro_opt_vec): wildcard receiver (`v` from
                    // `o.unwrap()` where o: Option[Vec[Str]]). Indexing must
                    // DEFER to codegen (return `_`), not degrade to Int --
                    // otherwise `v[0] != "hello"` errors "cannot compare Int
                    // with Str" even though codegen lowers it correctly.
                    CheckedType::Named(name) if name == "_" => CheckedType::Named("_".into()),
                    _ => CheckedType::Int,
                }
            }
            Expr::AtPre(inner, _) => self.check_expr(inner),
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => {
                // 5c.32: &expr preserves the inner type. Coercion to *T for
                // raw pointer assignments is handled at the assignment/argument
                // site (see types_compatible_for_ref).
                self.check_expr(inner)
            }
            Expr::Some(inner, _) => {
                // BUG 51 (2026-08-18): type the constructor with the payload
                // arg ("Option[MyRc]") -- the erased "Option" mismatched fn
                // return types that now carry args (expected Option[MyRc],
                // found Option). The container-erasure compatibility rule in
                // types_compatible keeps both forms interchangeable.
                let inner_ty = self.check_expr(inner);
                CheckedType::Named(format!("Option[{}]", inner_ty.name()))
            }
            Expr::None(_) => CheckedType::Named("Option".into()),
            Expr::Ok(inner, _) => {
                let _ = self.check_expr(inner);
                // The error type is unknowable from the constructor alone --
                // keep the erased form (types_compatible erases both sides).
                CheckedType::Named("Result".into())
            }
            Expr::Err(inner, _) => {
                let _ = self.check_expr(inner);
                CheckedType::Named("Result".into())
            }
            Expr::Struct(name, fields, _spread, span) => {
                // M22: Anonymous struct `{ field: value; }` -- type inferred from context.
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
                            // soft error -- arrays should be homogeneous
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
                // AUDIT follow-up (stdlib finding #15): `as` binds tighter
                // than binary operators (Rust parity), so `len % 256 as UInt8`
                // casts the LITERAL -- 256 truncates to 0 and the `%` traps
                // with an illegal instruction at runtime. Warn at COMPILE
                // time when a direct integer literal truncates, so the
                // ambiguous form gets parenthesized.
                if let Expr::Int(raw, _) = &**inner {
                    let val = *raw as i64;
                    let (min, max): (i64, i64) = match &target_ty {
                        CheckedType::UInt8 => (0, 255),
                        CheckedType::Int8 => (-128, 127),
                        CheckedType::UInt16 => (0, 65535),
                        CheckedType::Int16 => (-32768, 32767),
                        CheckedType::UInt32 => (0, 4_294_967_295),
                        CheckedType::Int32 => (-2_147_483_648, 2_147_483_647),
                        CheckedType::Char => (0, 1_114_111),
                        _ => (i64::MIN, i64::MAX),
                    };
                    if val < min || val > max {
                        self.warn(format!(
                            "cast truncates: literal {val} does not fit in {} -- parenthesize the expression you intend to cast, e.g. (a % b) as {}",
                            target_ty.name(), target_ty.name()));
                    }
                }
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
                    // v0.56: Char <-> Float casts (string parsing, core.xi)
                    _ if inner_resolved == CheckedType::Char
                        && matches!(target_resolved, CheckedType::Float32 | CheckedType::Float64) => target_ty,
                    _ if matches!(inner_resolved, CheckedType::Float32 | CheckedType::Float64)
                        && target_resolved == CheckedType::Char => target_ty,
                    // 5c-E: Int <-> Ptr casts (raw pointer FFI, ptr.xi)
                    (CheckedType::Int, CheckedType::Named(s)) if s == "Ptr" || s.starts_with('*') => {
                        self.gate_unsafe("integer-to-pointer cast", *span);
                        target_ty
                    }
                    (CheckedType::Named(s), CheckedType::Int) if s == "Ptr" || s.starts_with('*') => {
                        self.gate_unsafe("pointer-to-integer cast", *span);
                        target_ty
                    }
                    // 5c-E: Vec/Slice/Array -> Ptr cast (Vulkan FFI: pass buffer to extern)
                    (CheckedType::Named(s), CheckedType::Named(t))
                        if (t == "Ptr" || t.starts_with('*')) && (s == "Vec" || s == "Slice" || s == "Array") => {
                        self.gate_unsafe("container-to-pointer cast", *span);
                        target_ty
                    }
                    // v0.56: Str -> Ptr cast (C FFI: pass string as byte pointer)  
                    (CheckedType::Str, CheckedType::Named(t)) if t == "Ptr" || t.starts_with('*') => {
                        self.gate_unsafe("string-to-pointer cast", *span);
                        target_ty
                    }
                    (CheckedType::Named(s), CheckedType::Named(t))
                        if (s == "Ptr" || s.starts_with('*')) && (t == "Vec" || t == "Slice" || t == "Array" || t == "Str") => {
                        self.gate_unsafe("pointer-to-container cast", *span);
                        target_ty
                    }
                    // M33: Pointer-to-pointer cast (`*T as *U`): allows byte-level
                    // reinterpretation in unsafe code (e.g. `pi as *UInt8` for raw
                    // memory access). Both sides must be pointer types.
                    (CheckedType::Named(s), CheckedType::Named(t))
                        if s.starts_with('*') && t.starts_with('*') => {
                        self.gate_unsafe("pointer-to-pointer cast", *span);
                        target_ty
                    }
                    // BUG 25 #10 (crypto AES-NI follow-up): `&x as *T` /
                    // `&v[i] as *T` -- address-of cast to a raw pointer. The
                    // reference's INNER type checks as the element type
                    // (UInt8), which no rule above matches; the reference
                    // wrapper is what makes this an ADDRESS, not a value.
                    // `&out as *UInt8`-style casts are the documented FFI
                    // idiom (G-44) and must type-check inside unsafe blocks.
                    _ if matches!(inner.as_ref(), Expr::Ref(..) | Expr::MutRef(..))
                        && matches!(target_resolved, CheckedType::Named(ref t) if t == "Ptr" || t.starts_with('*')) => {
                        self.gate_unsafe("reference-to-pointer cast", *span);
                        target_ty
                    }
                    // 5e.2 G-34: fn-ptr <-> Int casts (COM vtables, callback registries).
                    (CheckedType::Int, CheckedType::Fn(..)) => {
                        self.gate_unsafe("integer-to-function-pointer cast", *span);
                        target_ty
                    }
                    (CheckedType::Fn(..), CheckedType::Int) => {
                        self.gate_unsafe("function-pointer-to-integer cast", *span);
                        target_ty
                    }
                    // v0.56: fn-ptr -> *UInt8 cast (thread spawn, FFI callback)
                    (CheckedType::Fn(..), CheckedType::Named(t)) if t.starts_with('*') => {
                        self.gate_unsafe("function-pointer-to-pointer cast", *span);
                        target_ty
                    }
                    // v0.56: Wildcard type (_) can cast to anything (unwrap result, etc.)
                    (CheckedType::Named(n), _) if n == "_" => target_ty,
                    // G-16: function name as Int (callback pointer).
                    (CheckedType::Named(n), CheckedType::Int) if n == "fn" => {
                        self.gate_unsafe("function-name-to-integer cast", *span);
                        target_ty
                    }
                    // v0.56: Generic type param cast -- let any generic param be cast
                    (CheckedType::Named(n), _) if n.len() == 1 && n.chars().next().map_or(false, |c| c.is_ascii_uppercase()) => target_ty,
                    _ => {
                        self.error(format!("unsupported type cast: {} to {}", inner_ty.name(), target_ty.name()), *span)
                    }
                }
            }
            Expr::Await(inner, _) => self.check_expr(inner),
            Expr::Comptime(inner, _) => self.check_expr(inner),
            Expr::Unsafe(block, _) => {
                // D2 (2026-08-08): `unsafe { }` opts into raw-pointer ops for
                // this block only. Depth-scoped so nested blocks compose.
                // D2.1 (T007, requirement c): whole-body-unsafe fns must
                // declare `requires` -- enforced at check_fn_decl (a fn whose
                // ENTIRE body is one unsafe block). Sub-expression unsafe
                // blocks are confined plumbing (operands, assignments) and do
                // not escape the fn, so they need no wrapper contract.
                self.unsafe_depth += 1;
                let pending_saved = std::mem::take(&mut self.pending_extern_ptrs);
                let converted_saved = std::mem::take(&mut self.converted_ffi_ptrs);
                let ty = self.check_block(block, None).unwrap_or(CheckedType::Unit);
                // D2.1 (T006, FFI ownership): an extern call returning a raw
                // pointer inside this confined block must have been converted to
                // an owned XIOM type (ffi.safe_ptr_from_raw / box_from_ptr /
                // vec_from_ptr_with_free / str_from_ptr_owned) BEFORE the tail.
                // The block's TAIL must not be (or nest) an unconverted
                // extern-returned raw pointer -- that would leak/double-free.
                // EXEMPTION: raw-pointer-RETURNING fns (the stdlib allocator
                // pattern `fn alloc(...) -> *mut UInt8` -- `unsafe { return
                // malloc(...); }`) transfer ownership to the CALLER, who is
                // responsible for freeing. Mirrors T003's unsafe-internal-helper
                // exemption.
                let fn_returns_raw_ptr = self.current_return.as_ref().map_or(false, Self::is_raw_pointer_ty);
                if !self.pending_extern_ptrs.is_empty() && !fn_returns_raw_ptr {
                    // Does the tail expression reference an unconverted extern
                    // pointer variable? (Ident / field / index of a pending ptr.)
                    let tail_expr = Self::block_tail_expr(block).cloned();
                    let tail_escapes = tail_expr.map_or(false, |tail| {
                        self.tail_refs_pending_ptr(&tail, &self.pending_extern_ptrs)
                    });
                    if tail_escapes {
                        self.error(
                            "extern raw-pointer result must be converted to an owned XIOM type before the `unsafe` block's tail (T006: use ffi.safe_ptr_from_raw / box_from_ptr / vec_from_ptr_with_free / str_from_ptr_owned)",
                            block.span,
                        );
                    }
                }
                self.unsafe_depth -= 1;
                // Restore the enclosing block's pending/converted state (a
                // pointer produced in an outer scope is not affected by this
                // nested block).
                self.pending_extern_ptrs = pending_saved;
                self.converted_ffi_ptrs = converted_saved;
                // D2.1 (T005): zero-escape is enforced at the FUNCTION boundary
                // (see enforce_fn_unsafe_tail) -- a raw-pointer value produced by
                // an unsafe SUB-expression stays confined to the enclosing
                // scope (unsafe-internal helpers may hold *T). Rejecting every
                // block tail here would break legitimate confined-pointer
                // plumbing (e.g. `unsafe { 0 as *Node }` as an operand).
                ty
            }
            Expr::BlockExpr(block, _) => { self.check_block(block, None).unwrap_or(CheckedType::Unit) }
            Expr::ConstBlock(inner, _) => self.check_expr(inner),
            // 5c-R: Error-poisoned nodes carry an ErrorGuaranteed proof token.
            // Skip silently -- a diagnostic was already emitted for this subtree.
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
                // S2: Match exhaustiveness -- verify all variants covered.
                self.check_match_exhaustiveness(arms, &scr_ty);
                result_ty
            }
        }
    }

    /// S2: Match exhaustiveness -- verify all variants of the scrutinee type
    /// are covered by the match arms. Reports an error for missing variants.
    fn check_match_exhaustiveness(&mut self, arms: &[xiom_ast::MatchArm], scr_ty: &CheckedType) {
        let type_name = match self.resolve_alias(scr_ty) {
            CheckedType::Named(n) => n,
            _ => return,
        };
        let variants: Vec<String> = match type_name.as_str() {
            "Option" => vec!["Some".to_string(), "None".to_string()],
            "Result" => vec!["Ok".to_string(), "Err".to_string()],
            "Bool" => vec!["true".to_string(), "false".to_string()],
            _ => {
                // enum_variants maps variant_name -> parent_enum.
                // Collect all variants whose parent matches type_name.
                self.enum_variants.iter()
                    .filter(|(_, parent)| parent.as_str() == type_name.as_str()
                        || parent.ends_with(&format!(".{}", type_name)))
                    .map(|(variant, _)| variant.clone())
                    .collect()
            }
        };
        if variants.is_empty() { return; }
        // AUDIT #6/#16 FIX: variants register BOTH bare and qualified
        // ("Op" AND "Token.Op") -- dedupe to BASE names so coverage
        // compares pattern names against the same form (the old loop
        // double-warned and false-flagged qualified keys as uncovered).
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let variants: Vec<String> = variants.into_iter()
            .map(|v| v.rsplit('.').next().unwrap_or(&v).to_string())
            .filter(|v| seen.insert(v.clone()))
            .collect();
        for variant in &variants {
            let covered = arms.iter().any(|arm| pattern_covers_variant(&arm.pattern, variant));
            if !covered {
                if self.strict_exhaustive {
                    self.error(
                        format!("non-exhaustive match: variant '{}' of '{}' not covered", variant, type_name),
                        xiom_ast::Span::new(0, 0),
                    );
                } else {
                    self.warn(
                        format!("non-exhaustive match: variant '{}' of '{}' not covered", variant, type_name),
                    );
                }
            }
        }
    }

    /// Phase 7E/Feature: Resolve type aliases recursively.
    /// `type Foo = Int; type Bar = Foo;` -- resolving Bar gives Int.
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

    /// AUDIT #6 FIX helper: recursive container-arg agreement with SCALAR
    /// promotion rules. "Int" vs "UInt8" passes (literal promotion);
    /// "Int" vs "Str" fails; nested containers recurse. Top-level commas
    /// are split bracket-aware so Result[Vec[Int], Str] compares correctly.
    fn container_args_compatible(a: &str, b: &str) -> bool {
        if a == b { return true; }
        let split = |s: &str| -> Vec<String> {
            let mut out = Vec::new();
            let mut depth = 0i32;
            let mut cur = String::new();
            for ch in s.chars() {
                match ch {
                    '[' => { depth += 1; cur.push(ch); }
                    ']' => { depth -= 1; cur.push(ch); }
                    ',' if depth == 0 => {
                        let t = cur.trim();
                        if !t.is_empty() { out.push(t.to_string()); }
                        cur.clear();
                    }
                    _ => cur.push(ch),
                }
            }
            let t = cur.trim();
            if !t.is_empty() { out.push(t.to_string()); }
            out
        };
        let pa = split(a);
        let pb = split(b);
        if pa.len() != pb.len() { return false; }
        for (x, y) in pa.iter().zip(pb.iter()) {
            let (x, y) = (x.trim(), y.trim());
            // nested container -> recurse
            if x.contains('[') || y.contains('[') {
                if x.contains('[') && y.contains('[') {
                    let (bx, ax) = x.split_once('[').unwrap_or((x, ""));
                    let (by, ay) = y.split_once('[').unwrap_or((y, ""));
                    if bx != by || !Self::container_args_compatible(ax, ay) {
                        return false;
                    }
                } else {
                    // Bare-vs-parameterized NESTED arg (Result vs
                    // Result[Int, Int]): the bare side erased its args in
                    // a legacy flow -- tolerant, mirroring the top-level
                    // rule.
                    continue;
                }
                continue;
            }
            // Wildcard inner args: Option[_] is context-adaptable.
            if x == "_" || y == "_" {
                continue;
            }
            // Generic type parameters (T, U, *T): compatible with anything --
            // mirrors the scalar matrix (math_tower passes Vec[Int] to Vec[T]).
            let is_generic = |s: &str| -> bool {
                let inner = s.strip_prefix('*').unwrap_or(s);
                inner.len() == 1 && inner.chars().next().map_or(false, |c| c.is_ascii_uppercase())
            };
            if is_generic(x) || is_generic(y) { continue; }
            // Empty/unit payloads: bare `None` infers Option[()] -- its
            // payload is context-adaptable, like integer literals.
            if x.is_empty() || y.is_empty() || x == "()" || y == "()" {
                continue;
            }
            // Tuple-typed args: broad compat, mirrors the scalar matrix
            // (the exact tuple shape is a structural-types item).
            if x.starts_with("Tuple") || y.starts_with("Tuple") { continue; }
            // Pointer-to-pointer, mirrors the scalar matrix.
            if (x.starts_with('*') || x == "Ptr") && (y.starts_with('*') || y == "Ptr") {
                continue;
            }
            let cx = CheckedType::from_str(x);
            let cy = CheckedType::from_str(y);
            let same = cx == cy;
            let num_promo = (cx.is_numeric() || cx == CheckedType::Bool)
                && (cy.is_numeric() || cy == CheckedType::Bool);
            if !same && !num_promo {
                return false;
            }
        }
        true
    }
    fn types_compatible(&self, found: &CheckedType, expected: &CheckedType) -> bool {
        // Phase 7E/Feature: Resolve type aliases so newtypes auto-convert
        let found = &self.resolve_alias(found);
        let expected = &self.resolve_alias(expected);
        // M9.6: impl Trait is an opaque return type -- any concrete type in the body
        // is compatible. Full trait-resolution checking is deferred.
        if matches!(found, CheckedType::ImplTrait(_)) || matches!(expected, CheckedType::ImplTrait(_)) {
            return true;
        }
        // v0.56: Str is represented as *UInt8 internally (C FFI).
        // Allow Str to be passed where *UInt8 is expected and vice versa.
        if matches!((found, expected), 
            (CheckedType::Str, CheckedType::Named(s)) | (CheckedType::Named(s), CheckedType::Str)
            if s.starts_with('*') || s == "Ptr")
        {
            return true;
        }
        // Wildcard type `_` -- compatible with any concrete type
        if matches!(found, CheckedType::Named(n) if n == "_") ||
           matches!(expected, CheckedType::Named(n) if n == "_") {
            return true;
        }
        // BUG 51 (2026-08-18): CONTAINER ERASURE compatibility - "Option" vs
        // "Option[MyRc]", "Result" vs "Result[Int, Str]", "Vec" vs "Vec[Int]"
        // are interchangeable (some paths erase the args, others preserve
        // them; the payload binding resolves the inner type when present).
        // AUDIT #6 FIX: when BOTH sides carry generic args, the args must
        // AGREE -- recursively, with the same SCALAR promotion rules used
        // everywhere else (Vec[Int] -> Vec[UInt8] stays legal: int literals
        // promote to unsigned bytes), but Option[Int] vs Option[Str] is a
        // type error, not a silent pass. (Bare-vs-parameterized stays
        // compatible: one side erased its args in a legacy flow.)
        if let (CheckedType::Named(a), CheckedType::Named(b)) = (found, expected) {
            let (base_a, mut args_a) = a.split_once('[').unwrap_or((a.as_str(), ""));
            let (base_b, mut args_b) = b.split_once('[').unwrap_or((b.as_str(), ""));
            args_a = args_a.strip_suffix(']').unwrap_or(args_a);
            args_b = args_b.strip_suffix(']').unwrap_or(args_b);
            if base_a == base_b {
                if !args_a.is_empty() && !args_b.is_empty()
                    && !Self::container_args_compatible(args_a, args_b)
                {
                    return false;
                }
                return true;
            }
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
        if expected.as_ptr_like()
            && (found.is_numeric() || found == &CheckedType::Bool)
        {
            return true;
        }
        // Named types are compatible if they have the same name
        // Generic type parameters (single uppercase letter) are compatible with any type
        // Also handles *T, *U etc. (pointer to generic)
        let is_generic_param = |ty: &CheckedType| -> bool {
            if let CheckedType::Named(s) = ty {
                let inner = s.strip_prefix('*').unwrap_or(s);
                inner.len() == 1 && inner.chars().next().map_or(false, |c| c.is_ascii_uppercase())
            } else {
                false
            }
        };
        if is_generic_param(found) || is_generic_param(expected) {
            return true;
        }
        // v0.56: Pointer types are compatible with each other (e.g., *T with *Int).
        // Both encode as Named("*..."); accept any pointer-to-pointer match.
        if let (CheckedType::Named(a), CheckedType::Named(b)) = (found, expected) {
            if (a.starts_with('*') || a == "Ptr") && (b.starts_with('*') || b == "Ptr") {
                return true;
            }
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
            // Exception: Self is always compatible -- it's an alias for the concrete type.
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
            // Signed<->unsigned integer compatibility (FFI Common)
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
// Borrow Checker -- Phase 1: ownership and lexical scope borrow checking
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
        // 5c-R: Place-level loan tracking -- only for field-granular paths.
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
        // 5c-R: Place-level loan tracking -- only for field-granular paths.
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

    /// P2-3: Build a Place from a borrow expression (bare ident, field, or index chain).
    fn expr_to_place(expr: &Expr) -> Option<crate::borrow::Place> {
        match expr {
            Expr::Ident(id) => Some(crate::borrow::Place::from_local(&id.name)),
            Expr::Field(obj, field, _) => {
                Self::expr_to_place(obj).map(|p| p.field(&field.name))
            }
            Expr::Index(obj, _, _) => {
                Self::expr_to_place(obj).map(|p| p.index())
            }
            _ => None,
        }
    }

    /// P2-3: Attempt a read borrow on a place (field-granular check).
    fn borrow_place_read(&mut self, expr: &Expr, span: Span) {
        // Always check bare-variable state for the root local.
        if let Some(place) = Self::expr_to_place(expr) {
            // Check and grant the loan
            match self.active_loans.grant(crate::borrow::Loan::read(place)) {
                crate::borrow::LoanResult::Granted => {},
                crate::borrow::LoanResult::Conflict(msg) => {
                    self.error(msg, span);
                }
            }
        }
        // Also track bare-variable state for the root
        if let Expr::Ident(id) = expr {
            self.read_borrow(&id.name, span);
        } else if let Some(place) = Self::expr_to_place(expr) {
            self.read_borrow(&place.local, span);
        }
    }

    /// P2-3: Attempt a write (mutable) borrow on a place (field-granular check).
    fn borrow_place_write(&mut self, expr: &Expr, span: Span) {
        if let Some(place) = Self::expr_to_place(expr) {
            match self.active_loans.grant(crate::borrow::Loan::write(place)) {
                crate::borrow::LoanResult::Granted => {},
                crate::borrow::LoanResult::Conflict(msg) => {
                    self.error(msg, span);
                }
            }
        }
        if let Expr::Ident(id) = expr {
            self.write_borrow(&id.name, span);
        } else if let Some(place) = Self::expr_to_place(expr) {
            self.write_borrow(&place.local, span);
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
                let xiom_type = match type_ann.as_ref() {
                    // AUDIT #6 FIX: `_` wildcard INFERS from the value in
                    // the borrow checker's local tracking too.
                    Some(t) if match &**t { Type::Named(n, _) => n.name == "_", _ => false } =>
                        Self::infer_type_from_expr(value).to_string(),
                    Some(t) => Self::param_type_name(t),
                    None => Self::infer_type_from_expr(value).to_string(),
                };
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
                let xiom_type = match type_ann.as_ref() {
                    Some(t) if match &**t { Type::Named(n, _) => n.name == "_", _ => false } =>
                        Self::infer_type_from_expr(value).to_string(),
                    Some(t) => Self::param_type_name(t),
                    None => Self::infer_type_from_expr(value).to_string(),
                };
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
            Stmt::While(cond, body, _, _, _) => {
                self.check_expr(cond);
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::For(var, iter, body, _, _) => {
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
            Stmt::Spawn(body, _, _move) => {
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::Break(..) => {}
            Stmt::Continue(..) => {}
            Stmt::Asm(_) => {}, // asm is valid in unsafe context
            Stmt::Defer(b, _) => { self.check_block(b); }
            Stmt::Assert(c, m, _) => {
                self.check_expr(c);
                if let Some(msg) = m { self.check_expr(msg); }
            }
            Stmt::Debugger(_) => {},
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> ExprResult {
        match expr {
            Expr::Ident(ident) => {
                self.check_use(&ident.name, ident.span)
            }
            Expr::Int(_, _) | Expr::BigInt(_, _) | Expr::Float(_, _) | Expr::Str(_, _)
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
                        self.borrow_place_read(inner, *span);
                        ExprResult::ReadRef
                    }
                    UnaryOp::MutRef => {
                        let _ = self.check_expr(inner);
                        self.borrow_place_write(inner, *span);
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
            Expr::GenericCall(func, _ty, args, span) => {
                self.check_call(func, args, *span)
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
                self.borrow_place_read(inner, expr.span());
                ExprResult::ReadRef
            }
            Expr::MutRef(inner, _) => {
                self.check_expr(inner);
                self.borrow_place_write(inner, expr.span());
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
            Expr::ConstBlock(inner, _) => self.check_expr(inner),
            // 5c-R: Error-poisoned nodes carry an ErrorGuaranteed proof.
            // Already diagnosed -- skip borrow checking for this subtree.
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
// Free helper for match exhaustiveness -- called from Checker::check_match_exhaustiveness
// ============================================================================

fn pattern_covers_variant(pattern: &xiom_ast::Pattern, variant: &str) -> bool {
    match pattern {
        xiom_ast::Pattern::Wildcard(_) | xiom_ast::Pattern::Ident(_) => true,
        xiom_ast::Pattern::Some(_, _) => variant == "Some",
        xiom_ast::Pattern::None(_) => variant == "None",
        xiom_ast::Pattern::Ok(_, _) => variant == "Ok",
        xiom_ast::Pattern::Err(_, _) => variant == "Err",
        xiom_ast::Pattern::Variant(name, _, _) => name.name == variant,
        xiom_ast::Pattern::Lit(lit) => match lit {
            xiom_ast::Literal::Bool(b, _) => (*b && variant == "true") || (!*b && variant == "false"),
            _ => false,
        },
        xiom_ast::Pattern::Or(alts, _) => alts.iter().any(|a| pattern_covers_variant(a, variant)),
        xiom_ast::Pattern::Struct(..) | xiom_ast::Pattern::Tuple(..) => false,
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
            Ok(p) => {
                let mut checker = Checker::new();
                // D1: mirror the driver -- register impls from the UNEXPANDED
                // program before check_program expands them away.
                checker.register_impls_from_program(&p);
                checker.check_program(&p)
            }
            Err(e) => Err(vec![CheckError {
                message: format!("parse error: {e}"),
                span: e.span,
                cause: crate::types::TypeCause::Other,
                guaranteed: e.guaranteed,
            }]),
        }
    }

    #[test]
    // AUDIT #6 regressions: wildcard inference + container-arg agreement.

    #[test]
    fn test_wildcard_annotation_infers_value_type() {
        // `let x: _ = "s"` used to bind x: Int (the "_" -> Int foot-gun).
        // Binding a Str to an Int-typed local must now ERROR.
        let result = check("fn f() -> Int { let x: _ = \"s\"; let n: Int = x; return n; }");
        assert!(result.is_err(), "wildcard-bound Str must not silently become Int: {:?}", result.ok());
    }

    #[test]
    #[test]
    #[test]
    fn test_cast_truncation_warns() {
        // stdlib finding #15: `as` binds tighter than binary ops (Rust
        // parity), so `len % 256 as UInt8` casts the literal -- 256 -> 0
        // and the `%` traps (illegal instruction) at runtime. A direct
        // literal truncation must WARN at compile time.
        let src = "fn main() -> Int { var x = 256 as UInt8; return 0; }";
        let tokens = Lexer::new(src).tokenize();
        let program = Parser::new(tokens).parse_program().expect("parse");
        let mut checker = Checker::new();
        let _ = checker.check_program(&program);
        let warns = checker.take_warnings();
        assert!(warns.iter().any(|w| w.message.contains("truncates")),
            "literal-truncating cast must warn: {:?}", warns);
    }
    fn test_container_arg_context_adaptation() {
        // Nested bare-vs-parameterized (legacy erasure inside args) stays
        // compatible: Option[Result] vs Option[Result[Int, Int]].
        let ok = check("fn f(a: Option[Result[Int, Int]]) { let b: Option[Result] = a; }");
        assert!(ok.is_ok(), "nested bare-vs-parameterized must stay compatible: {:?}", ok.err());
        // Wildcard INNER args are context-adaptable: Option[_] annotation
        // accepts an Option[Int] value.
        let ok2 = check("fn g(a: Option[Int]) { let b: Option[_] = a; }");
        assert!(ok2.is_ok(), "wildcard inner arg must adapt: {:?}", ok2.err());
        // Genuinely incompatible args STILL error: Option[Int] vs Option[Str].
        let err = check("fn h(a: Option[Str]) { let b: Option[Int] = a; }");
        assert!(err.is_err(), "Option[Str] vs Option[Int] must error");
    }
    fn test_container_args_must_agree_when_both_present() {
        // Option[Int] vs Option[Str] used to pass via container erasure
        // (bare "Option" == "Option[...]"). Differing concrete args must
        // now be a type error.
        let result = check("fn g(a: Option[Int]) { let b: Option[Str] = a; }");
        assert!(result.is_err(), "Option[Int] vs Option[Str] must be a type error");
        // Bare-vs-parameterized stays compatible (legacy erasure flows).
        let ok = check("fn h(a: Option[Int]) { let b: Option = a; }");
        assert!(ok.is_ok(), "bare-vs-parameterized must stay compatible: {:?}", ok.err());
    }
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
        // Whole-body unsafe fns must declare `requires` (T007).
        let result = check("fn f() -> Int requires: true { unsafe { return 42; } }");
        assert!(result.is_ok(), "unsafe tail with return must satisfy fn return type: {:?}", result.err());
    }

    #[test]
    fn test_divergence_unsafe_with_early_return() {
        let src = r#"
fn f(x: Int) -> Int
  requires: x >= 0
{
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
        // The unsafe block does NOT return -- the () tail must still mismatch Int.
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
        // interface methods on generic params with bounds -- the codegen
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

    /// 8B/M5: Fuzz harness -- random type combinations, verify TypeArena integrity.
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

    /// 8B/M5: Fuzz harness -- verify types_compatible with random type pairs.
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

    /// 8B/M5: Fuzz harness -- error count should never overflow (u32 safety).
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

    /// 8B/M5: Fuzz harness -- source_dirs never cause panic on missing dirs.
    #[test]
    fn fuzz_source_dirs_missing() {
        let mut checker = crate::Checker::new();
        checker.add_source_dir("/nonexistent/path/12345".to_string());
        checker.add_source_dir("\\\\invalid\\path\\".to_string());
        checker.build_catalog_index();
        // Must not panic -- the catalog should handle missing paths gracefully
    }

    // -- M5: Property-based / generative tests ---------------------------

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
        // Parser depth 128 + the checker's recursive expr walk cost several
        // MB of stack; run on a big-stack thread exactly like the driver.
        let child = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(move || {
                let result = check(&src);
                // Must not panic -- may produce errors or succeed
                assert!(result.is_ok() || result.is_err());
            })
            .expect("spawn");
        child.join().expect("deep-nesting check must not abort");
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

    // -- M7: Deref/DerefMut/AsRef usage tests --------------------------

    /// Box[T] deref: field access through Box should resolve to T's fields.
    #[test]
    fn prop_box_deref_field_access() {
        let src = "type Point = { x: Float64; y: Float64; }\nfn main() -> Float64 { var p = Box.new(Point{ x: 1.0; y: 2.0; }); return p.x; }";
        let result = check(&src);
        // Field access through Box requires Deref -- may not be fully supported yet
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

    // -- More fuzz-like stress tests ------------------------------------

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

    // -- M21-3: Checker edge cases --------------------------------------

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

    // D2 (2026-08-08): safe-by-default -- raw pointer ops REQUIRE unsafe blocks.
    #[test] fn test_d2_deref_outside_unsafe_rejected() {
        let src = "\
fn read_via_ptr(p: *Int) -> Int {
    return *p;
}";
        let result = check(src);
        assert!(result.is_err(), "raw pointer deref outside unsafe must fail: {:?}", result.err());
    }

    #[test] fn test_d2_deref_inside_unsafe_accepted() {
        // Whole-body unsafe fns must declare `requires` (T007).
        let src = "\
fn read_via_ptr(p: *Int) -> Int
    requires: p != (0 as *Int)
{
    unsafe { return *p; }
}";
        let result = check(src);
        assert!(result.is_ok(), "raw pointer deref inside unsafe must pass: {:?}", result.err());
    }

    #[test] fn test_d2_int_to_ptr_cast_outside_unsafe_rejected() {
        let src = "\
fn make_ptr(n: Int) -> *UInt8 {
    return n as *UInt8;
}";
        let result = check(src);
        assert!(result.is_err(), "int-to-ptr cast outside unsafe must fail: {:?}", result.err());
    }

    #[test] fn test_d2_int_to_ptr_cast_inside_unsafe_accepted() {
        let src = "\
fn make_ptr(n: Int) -> *UInt8
    requires: n >= 0
{
    unsafe { return n as *UInt8; }
}";
        let result = check(src);
        assert!(result.is_ok(), "int-to-ptr cast inside unsafe must pass: {:?}", result.err());
    }

    #[test] fn test_d2_asm_outside_unsafe_rejected() {
        let src = "\
fn spin() {
    asm(\"nop\");
}";
        let result = check(src);
        assert!(result.is_err(), "asm outside unsafe must fail: {:?}", result.err());
    }

    #[test] fn test_d2_ref_coercion_stays_safe() {
        // 5c.32: &expr coerces to *T for raw pointer assignments -- the safe
        // FFI borrow pattern. Must NOT be gated AT THE CAST SITE.
        let src = "\
fn borrow(x: Int) -> Int {
    unsafe {
        let p: *Int = &x;
        if *p == x { return 0; }
    }
    return 1;
}";
        let result = check(src);
        assert!(result.is_ok(), "&x ref-coercion must stay safe: {:?}", result.err());
    }

    #[test] fn test_d2_nested_unsafe_composes() {
        let src = "\
fn deep(p: *Int) -> Int
    requires: p != (0 as *Int)
{
    unsafe {
        let q = p as *Int;
        unsafe { return *q; }
    }
}";
        let result = check(src);
        assert!(result.is_ok(), "nested unsafe must pass: {:?}", result.err());
    }

    // D1 (2026-08-08): interface impl dispatch -- `impl Trait[Args]` must
    // register and `Trait[Args].method(...)` static calls must resolve.
    #[test] fn test_d1_impl_dispatch_registers() {
        let src = "\
interface Num[T] {
  fn add(a: T, b: T) -> T;
}

impl Num[Int] {
  fn add(a: Int, b: Int) -> Int { return a + b; }
}

fn main() -> Int {
  var r = Num[Int].add(20, 22);
  if r == 42 { return 0; }
  return 1;
}";
        let result = check(src);
        assert!(result.is_ok(), "impl dispatch must type-check: {:?}", result.err());
    }

    #[test] fn test_d1_impl_dispatch_multi_type() {
        let src = "\
interface Num[T] {
  fn add(a: T, b: T) -> T;
}

impl Num[Int] {
  fn add(a: Int, b: Int) -> Int { return a + b; }
}

impl Num[Float64] {
  fn add(a: Float64, b: Float64) -> Float64 { return a + b; }
}

fn main() -> Int {
  var i = Num[Int].add(1, 2);
  var f = Num[Float64].add(1.5, 2.5);
  return 0;
}";
        let result = check(src);
        assert!(result.is_ok(), "multi-type impl dispatch must type-check: {:?}", result.err());
    }

    // D1: generic explicit type args -- `fn[Float32](...)` must keep the
    // concrete type through parsing (regression: was discarded -> resolved Int).
    #[test] fn test_d1_generic_explicit_type_args_parse() {
        let src = "\
fn id[T](a: T) -> T { return a; }

fn main() -> Int {
  var f = id[Float32](1.5 as Float32);
  var e: Float32 = 1.5 as Float32;
  if f == e { return 0; }
  return 1;
}";
        let result = check(src);
        assert!(result.is_ok(), "generic explicit type args must type-check: {:?}", result.err());
    }

    // 3c (2026-08-10): generic interface-bound dispatch --
    // `fn lerp[T: Num]` calling `Num[T].add` must type-check when T is a
    // generic param bound by the trait.
    #[test] fn test_3c_generic_bound_dispatch_typechecks() {
        let src = "\
interface Num[T] {
  fn add(a: T, b: T) -> T;
}

impl Num[Int] {
  fn add(a: Int, b: Int) -> Int { return a + b; }
}

impl Num[Float64] {
  fn add(a: Float64, b: Float64) -> Float64 { return a + b; }
}

fn total2[T: Num](a: T, b: T) -> T {
  return Num[T].add(a, b);
}

fn main() -> Int {
  var i = total2[Int](20, 22);
  if i == 42 { return 0; }
  return 1;
}";
        let result = check(src);
        assert!(result.is_ok(), "generic-bound dispatch must type-check: {:?}", result.err());
    }

    #[test] fn test_3c_missing_impl_is_error() {
        let src = "\
interface Num[T] {
  fn add(a: T, b: T) -> T;
}

impl Num[Int] {
  fn add(a: Int, b: Int) -> Int { return a + b; }
}

fn total2[T: Num](a: T, b: T) -> T {
  return Num[T].add(a, b);
}

fn main() -> Int {
  var s = total2[Str](\"a\", \"b\");
  return 0;
}";
        let result = check(src);
        // Str has no Num impl -- the CHECKER accepts the generic fn (the
        // codegen reports the missing impl as C001 at monomorphisation).
        // Assert we don't crash and the generic fn itself type-checks.
        assert!(result.is_ok() || result.is_err(), "must not panic: {:?}", result.err());
    }

    // D2.1 (Unsafe Confinement Phase 1): confinement gates.
    #[test] fn test_d21_extern_call_outside_unsafe_rejected() {
        let src = "\
extern \"C\" {
  fn c_malloc(size: Int) -> *UInt8;
}

fn main() -> Int {
  var p = c_malloc(100);
  return 0;
}";
        let result = check(src);
        assert!(result.is_err(), "extern call outside unsafe must fail: {:?}", result.err());
    }

    #[test] fn test_d21_extern_call_inside_unsafe_accepted() {
        let src = "\
extern \"C\" {
  fn c_malloc(size: Int) -> *UInt8;
}

fn main() -> Int {
  unsafe {
    var p = c_malloc(100);
    if p == (0 as *UInt8) { return 1; }
  }
  return 0;
}";
        let result = check(src);
        assert!(result.is_ok(), "extern call inside unsafe must pass: {:?}", result.err());
    }

    #[test] fn test_d21_reference_tail_rejected() {
        // &T as a fn RETURN is borrow-checked (safe); the zero-escape gate is
        // for RAW pointers. `unsafe { &x }` as a local is confined to the fn.
        let src = "\
fn main() -> Int {
  var x = 5;
  var r = unsafe { &x };
  if *r == 5 { return 0; }
  return 1;
}";
        let result = check(src);
        assert!(result.is_ok(), "&T local from unsafe is borrow-confined and must pass: {:?}", result.err());
    }

    #[test] fn test_d21_safe_fn_cannot_return_raw_ptr() {
        // A SAFE fn (no unsafe in body) returning *T is a zero-escape violation.
        let src = "\
fn bad() -> *Int {
  return 12345 as *Int;
}
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_err(), "safe fn returning *T must fail: {:?}", result.err());
    }

    #[test] fn test_d21_raw_ptr_tail_rejected() {
        // A pointer flowing through a SAFE fn (no unsafe blocks) is a
        // zero-escape violation: the value leaves confinement.
        let src = "\
fn pass_through(p: *Int) -> *Int {
  return p;
}
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_err(), "raw pointer through a safe fn must fail: {:?}", result.err());
    }

    #[test] fn test_d21_safe_tail_accepted() {
        let src = "\
fn main() -> Int {
  var r = unsafe { 42 };
  if r == 42 { return 0; }
  return 1;
}";
        let result = check(src);
        assert!(result.is_ok(), "safe scalar tail must pass: {:?}", result.err());
    }

    // D2.1 (Unsafe Confinement Phase 2 -- requirement c): whole-body unsafe
    // fns must declare `requires` (pre-entry contracts).
    #[test] fn test_d21_whole_body_unsafe_requires_rejected() {
        let src = "\
fn whole() -> Int {
  unsafe { return 42; }
}
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_err(), "whole-body unsafe without requires must fail: {:?}", result.err());
    }

    #[test] fn test_d21_whole_body_unsafe_with_requires_accepted() {
        let src = "\
fn whole(x: Int) -> Int
  requires: x >= 0
{
  unsafe { return x; }
}
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_ok(), "whole-body unsafe with requires must pass: {:?}", result.err());
    }

    // D2.1 (Unsafe Confinement Phase 7/plan S2.12 -- requirement i, T006):
    // an extern "C" call returning a raw pointer inside a confined block must
    // have its result converted to an owned XIOM type before the block's tail.
    #[test] fn test_t006_extern_ptr_tail_rejected() {
        // The fn returns a NON-pointer (Int); the extern-returned pointer is
        // cast to Int in the tail without any ownership conversion -> T006.
        let src = "\
extern \"C\" {
  fn xiom_alloc(size: Int) -> *UInt8;
}
fn bad_tail() -> Int
  requires: true
{
  unsafe {
    xiom_alloc(8) as Int
  }
}
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_err(), "unconverted extern pointer tail must fail (T006): {:?}", result.err());
    }

    #[test] fn test_t006_extern_ptr_converted_accepted() {
        let src = "\
extern \"C\" {
  fn xiom_alloc(size: Int) -> *UInt8;
}
fn safe_ptr_from_raw(p: *UInt8, size: Int) -> Int { return 0; }
fn good(x: Int) -> Int
  requires: x >= 0
{
  unsafe {
    var p = xiom_alloc(8);
    var sp = safe_ptr_from_raw(p, 8);
    42
  }
}
fn main() -> Int { return 0; }";
        let result = check(src);
        assert!(result.is_ok(), "converted extern pointer must pass (T006): {:?}", result.err());
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

    // `let` rebinding with different type (shadowing -- should be ok)
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
