// AXIOM — Type Checker
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! AXIOM Type Checker — Phase 0: basic type checking for primitives,
//! struct types, function signatures, and return types.
//! No generics, no ownership, no contracts enforcement.

use axiom_ast::*;
use axiom_lexer::Lexer;
use axiom_parser::Parser;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// ============================================================================
// Type representation for the checker
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CheckedType {
    Bool,
    Int, Int8, Int16, Int32, Int64,
    UInt, UInt8, UInt16, UInt32, UInt64,
    Float32, Float64,
    Char,
    Str,
    Unit,
    Never,
    /// A user-defined type by name
    Named(String),
    /// A generic type parameter (still unresolved)
    Generic(String),
    /// Function pointer type: fn(T, U) -> V
    Fn(Vec<CheckedType>, Box<CheckedType>),
    /// Error type — used when type checking fails
    Error,
}

impl CheckedType {
    /// Convert from AST Type to checked type representation
    pub fn from_ast_type(ty: &Type) -> Self {
        match ty {
            Type::Named(ident, _) => Self::from_str(&ident.name),
            Type::Ref(inner) => CheckedType::from_ast_type(inner),
            Type::MutRef(inner) => CheckedType::from_ast_type(inner),
            Type::Option(_) => CheckedType::Named("Option".into()),
            Type::Result(_, _) => CheckedType::Named("Result".into()),
            Type::Vec(_) => CheckedType::Named("Vec".into()),
            Type::Slice(_) => CheckedType::Named("Slice".into()),
            Type::Map(_, _) => CheckedType::Named("Map".into()),
            Type::Set(_) => CheckedType::Named("Set".into()),
            Type::Tuple(types) => {
                // In Phase 0, represent tuples as Named for simplicity
                CheckedType::Named(format!("Tuple{}", types.len()))
            }
            Type::Ptr(_) => CheckedType::Named("Ptr".into()),
            Type::Array(_, _) => CheckedType::Named("Array".into()),
            Type::Fn(params, ret) => CheckedType::Fn(
                params.iter().map(CheckedType::from_ast_type).collect(),
                Box::new(CheckedType::from_ast_type(ret)),
            ),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Bool" => CheckedType::Bool,
            "Int" => CheckedType::Int,
            "Int8" => CheckedType::Int8,
            "Int16" => CheckedType::Int16,
            "Int32" => CheckedType::Int32,
            "Int64" => CheckedType::Int64,
            "UInt" => CheckedType::UInt,
            "UInt8" => CheckedType::UInt8,
            "UInt16" => CheckedType::UInt16,
            "UInt32" => CheckedType::UInt32,
            "UInt64" => CheckedType::UInt64,
            "Float32" => CheckedType::Float32,
            "Float64" => CheckedType::Float64,
            "Char" => CheckedType::Char,
            "Str" => CheckedType::Str,
            "()" => CheckedType::Unit,
            "_" => CheckedType::Int, // wildcard placeholder
            _ => CheckedType::Named(s.to_string()),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self,
            CheckedType::Int | CheckedType::Int8 | CheckedType::Int16 |
            CheckedType::Int32 | CheckedType::Int64 |
            CheckedType::UInt | CheckedType::UInt8 | CheckedType::UInt16 |
            CheckedType::UInt32 | CheckedType::UInt64 |
            CheckedType::Float32 | CheckedType::Float64
        )
    }

    pub fn is_integer(&self) -> bool {
        matches!(self,
            CheckedType::Int | CheckedType::Int8 | CheckedType::Int16 |
            CheckedType::Int32 | CheckedType::Int64 |
            CheckedType::UInt | CheckedType::UInt8 | CheckedType::UInt16 |
            CheckedType::UInt32 | CheckedType::UInt64
        )
    }

    pub fn name(&self) -> String {
        match self {
            CheckedType::Bool => "Bool".into(),
            CheckedType::Int => "Int".into(),
            CheckedType::Int8 => "Int8".into(),
            CheckedType::Int16 => "Int16".into(),
            CheckedType::Int32 => "Int32".into(),
            CheckedType::Int64 => "Int64".into(),
            CheckedType::UInt => "UInt".into(),
            CheckedType::UInt8 => "UInt8".into(),
            CheckedType::UInt16 => "UInt16".into(),
            CheckedType::UInt32 => "UInt32".into(),
            CheckedType::UInt64 => "UInt64".into(),
            CheckedType::Float32 => "Float32".into(),
            CheckedType::Float64 => "Float64".into(),
            CheckedType::Char => "Char".into(),
            CheckedType::Str => "Str".into(),
            CheckedType::Unit => "()".into(),
            CheckedType::Never => "!".into(),
            CheckedType::Named(s) => s.clone(),
            CheckedType::Fn(params, ret) => {
                let params_str: Vec<String> = params.iter().map(|p| p.name()).collect();
                format!("fn({}) -> {}", params_str.join(", "), ret.name())
            }
            CheckedType::Generic(s) => s.clone(),
            CheckedType::Error => "<error>".into(),
        }
    }
}

// ============================================================================
// Module export representation
// ============================================================================

#[derive(Debug, Clone)]
pub enum ModuleExport {
    Type { fields: HashMap<String, CheckedType>, is_pub: bool },
    Function { sig: FnSig, is_pub: bool },
    SubModule(HashMap<String, ModuleExport>),
}

// ============================================================================
// Type Checker
// ============================================================================

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
    /// Visibility: name → is_pub for top-level items
    visibility: HashMap<String, bool>,
    /// Resolved imported names from use declarations
    imported_items: HashMap<String, ModuleExport>,
    /// Enum variant name → parent enum type name
    enum_variants: HashMap<String, String>,
    /// Enum variant name → field name → field type (for variant constructors)
    variant_fields: HashMap<String, Vec<(String, CheckedType)>>,
    /// Directories to search for external module files
    pub source_dirs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FnSig {
    pub params: Vec<(String, CheckedType)>,
    pub return_type: Option<CheckedType>,
    pub generics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CheckError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type error at {}: {}", self.span, self.message)
    }
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
            visibility: HashMap::new(),
            imported_items: HashMap::new(),
            enum_variants: HashMap::new(),
            variant_fields: HashMap::new(),
            source_dirs: Vec::new(),
        };
        // Register built-in types
        checker.register_builtins();
        checker
    }

    fn register_builtins(&mut self) {
        // All primitive types are known
        for prim in &["Bool", "Int", "Int8", "Int16", "Int32", "Int64",
                       "UInt", "UInt8", "UInt16", "UInt32", "UInt64",
                       "Float32", "Float64", "Char", "Str"] {
            self.types.insert(prim.to_string(), HashMap::new());
        }
        // Compound builtin types (empty fields = permissive field access)
        for comp in &["Vec", "Map", "Set", "Stack", "Slice"] {
            self.types.insert(comp.to_string(), HashMap::new());
        }
        // Option with known fields
        let mut opt = HashMap::new();
        opt.insert("is_some".to_string(), CheckedType::Named("Bool".into()));
        opt.insert("is_none".to_string(), CheckedType::Named("Bool".into()));
        opt.insert("value".to_string(), CheckedType::Int);
        self.types.insert("Option".to_string(), opt);
        // Result with known fields
        let mut res = HashMap::new();
        res.insert("is_ok".to_string(), CheckedType::Named("Bool".into()));
        res.insert("is_err".to_string(), CheckedType::Named("Bool".into()));
        res.insert("value".to_string(), CheckedType::Int);
        res.insert("error".to_string(), CheckedType::Int);
        self.types.insert("Result".to_string(), res);

        // Vec builtin methods
        self.functions.insert("Vec.new".to_string(), FnSig {
            params: vec![],
            return_type: Some(CheckedType::Named("Vec".into())),
            generics: vec![],
        });
        self.functions.insert("Vec.push".to_string(), FnSig {
            params: vec![
                ("self".to_string(), CheckedType::Named("Vec".into())),
                ("val".to_string(), CheckedType::Named("T".into())),
            ],
            return_type: Some(CheckedType::Unit),
            generics: vec!["T".to_string()],
        });
        self.functions.insert("Vec.len".to_string(), FnSig {
            params: vec![
                ("self".to_string(), CheckedType::Named("Vec".into())),
            ],
            return_type: Some(CheckedType::Int),
            generics: vec![],
        });
        self.functions.insert("Vec.pop".to_string(), FnSig {
            params: vec![
                ("self".to_string(), CheckedType::Named("Vec".into())),
            ],
            return_type: Some(CheckedType::Named("Option".into())),
            generics: vec![],
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

    fn add_pattern_bindings(&mut self, pattern: &Pattern) {
        if let Pattern::Variant(name, _, _) = pattern {
        }
        match pattern {
            Pattern::Ident(name) => {
                // Don't add bindings for unit enum variants (like Empty, None)
                if self.enum_variants.contains_key(&name.name) || self.resolve_enum_variant(&name.name).is_some() {
                    return;
                }
                // Use Error type to suppress cascade errors (actual type resolved later)
                self.add_local(&name.name, CheckedType::Error);
            }
            Pattern::Variant(name, fields, _) => {
                // Look up variant field types for correct binding types
                let field_types = self.variant_fields.get(&name.name)
                    .or_else(|| self.variant_fields.get(&name.name))
                    .cloned()
                    .unwrap_or_default();
                for (i, field) in fields.iter().enumerate() {
                    if field.name == "_" { continue; } // skip wildcard placeholders
                    let field_ty = field_types.get(i)
                        .map(|(_, ty)| ty.clone())
                        .unwrap_or(CheckedType::Int);
                    self.add_local(&field.name, field_ty);
                }
            }
            Pattern::Ok(inner, _) => {
                self.add_pattern_bindings(inner);
            }
            Pattern::Err(inner, _) => {
                self.add_pattern_bindings(inner);
            }
            Pattern::Some(inner, _) => {
                self.add_pattern_bindings(inner);
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

    fn error(&mut self, message: impl Into<String>, span: Span) -> CheckedType {
        self.errors.push(CheckError { message: message.into(), span });
        CheckedType::Error
    }

    fn register_derived_method(&mut self, type_name: &str, module_path: &str, derive_trait: &DeriveTrait) {
        let method_name = match derive_trait {
            DeriveTrait::Clone => "clone",
            DeriveTrait::Eq => "eq",
            DeriveTrait::Display => "to_str",
            DeriveTrait::Hash => "hash",
            DeriveTrait::Ord => "compare",
            _ => return,
        };
        let ret_type = match derive_trait {
            DeriveTrait::Clone => CheckedType::Named(type_name.to_string()),
            DeriveTrait::Eq => CheckedType::Bool,
            DeriveTrait::Display => CheckedType::Str,
            DeriveTrait::Hash => CheckedType::Int,
            DeriveTrait::Ord => CheckedType::Int,
            _ => return,
        };
        let sig = FnSig {
            params: vec![], // no explicit params, self is implicit
            return_type: Some(ret_type),
            generics: vec![],
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
                        let mut vfields: Vec<(String, CheckedType)> = Vec::new();
                        for field in &variant.fields {
                            vfields.push((field.name.name.clone(), CheckedType::from_ast_type(&field.ty)));
                        }
                        self.variant_fields.insert(variant_key.clone(), vfields.clone());
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

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<CheckError>> {
        // Register all type declarations first
        for item in &program.items {
            self.register_type_decl(item);
        }

        // Build variant field maps from all enum declarations
        self.register_all_variant_fields(program);

        // Register all function signatures
        for item in &program.items {
            self.register_fn_signature(item);
        }

        // Resolve module system (imports and module hierarchy)
        self.resolve_imports(program);

        // Check all function bodies
        for item in &program.items {
            self.check_top_decl(item);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn register_type_decl(&mut self, item: &TopDecl) {
        self.register_type_decl_inner(item, "");
    }

    fn register_type_decl_inner(&mut self, item: &TopDecl, module_path: &str) {
        match item {
            TopDecl::Type(td) => {
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
                        self.enum_variants.entry(variant.name.name.clone()).or_insert(parent);
                    }
                    // Store variant fields for constructor field validation
                    let mut vfields: Vec<(String, CheckedType)> = Vec::new();
                    for field in &variant.fields {
                        vfields.push((field.name.name.clone(), CheckedType::from_ast_type(&field.ty)));
                    }
                    self.variant_fields.entry(variant_key.clone()).or_insert(vfields.clone());
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

    fn register_fn_signature(&mut self, item: &TopDecl) {
        self.register_fn_signature_inner(item, "");
    }

    fn register_fn_signature_inner(&mut self, item: &TopDecl, module_path: &str) {
        match item {
            TopDecl::Fn(fd) => {
                let mut params: Vec<_> = Vec::new();
                // Add implicit self for methods that don't have an explicit self param
                if let Some(recv) = fd.receiver.as_ref() {
                    let has_explicit_self = fd.params.first()
                        .map(|p| CheckedType::from_ast_type(&p.ty).name() == recv.name)
                        .unwrap_or(false);
                    if !has_explicit_self {
                        params.push(("self".to_string(), CheckedType::Named(recv.name.clone())));
                    }
                }
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
                let sig = FnSig { params, return_type, generics };
                self.functions.insert(key.clone(), sig.clone());
                // Also register with bare key as fallback (don't overwrite existing)
                if key != bare_key {
                    self.functions.entry(bare_key).or_insert(sig.clone());
                }
                self.visibility.insert(fd.name.name.clone(), fd.is_pub);
                // Track methods separately
                if let Some(recv) = fd.receiver.as_ref() {
                    let method_key = format!("{}.{}", recv.name, fd.name.name);
                    self.methods
                        .entry(recv.name.clone())
                        .or_default()
                        .insert(fd.name.name.clone(), sig);
                }
            }
            TopDecl::Module(md) => {
                let new_path = if module_path.is_empty() { md.name.name.clone() } else { format!("{}.{}", module_path, md.name.name) };
                for item in &md.items {
                    self.register_fn_signature_inner(item, &new_path);
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
            }
            _ => {}
        }
    }

    // ========================================================================
    // Module system
    // ========================================================================

    fn resolve_imports(&mut self, program: &Program) {
        // Build module hierarchy from all module declarations
        for item in &program.items {
            if let TopDecl::Module(md) = item {
                let exports = self.build_module_map_inner(&md.items, &md.name.name);
                self.modules.insert(md.name.name.clone(), exports);
            }
        }

        // Flatten submodules into self.modules for short-name resolution (e.g. "math" instead of "benchmark.math")
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

        // Process each use declaration — try filesystem resolution for missing modules
        let import_snapshot = std::mem::take(&mut self.imports);
        for ud in &import_snapshot {
            if !ud.path.is_empty() {
                let module_name = &ud.path[0].name;
                if !self.modules.contains_key(module_name) {
                    if let Some(exports) = self.load_external_module(module_name) {
                        self.modules.insert(module_name.clone(), exports);
                    }
                }
            }
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
                self.modules.insert(md.name.name.clone(), exports);
                self.flatten_submodules_inner(&md.items, &new_prefix);
            }
        }
    }

    /// Try to load an external module file `{source_dir}/{module_name}.ax` from the
    /// configured source directories.  Returns the module's export map if found.
    fn load_external_module(&mut self, module_name: &str) -> Option<HashMap<String, ModuleExport>> {
        for dir in &self.source_dirs {
            let file_path = format!("{}/{}.ax", dir, module_name);
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

    /// Try to load a module from `{source_dir}/{parent_path}/{module_name}.ax`.
    /// Used when walking dotted module paths (e.g., `benchmark.main` → `benchmark/main.ax`).
    fn load_external_module_path(&mut self, parent_path: &str, module_name: &str) -> Option<HashMap<String, ModuleExport>> {
        for dir in &self.source_dirs {
            let file_path = format!("{}/{}/{}.ax", dir, parent_path, module_name);
            if !Path::new(&file_path).exists() {
                // Also try without .ax extension for directory-based modules
                let dir_path = format!("{}/{}/{}", dir, parent_path, module_name);
                if Path::new(&dir_path).is_dir() {
                    // Try package.ax inside the subdirectory
                    let pkg_path = format!("{}/package.ax", dir_path);
                    if Path::new(&pkg_path).exists() {
                        if let Ok(source) = fs::read_to_string(&pkg_path) {
                            let tokens = Lexer::new(&source).tokenize();
                            if let Ok(program) = Parser::new(tokens).parse_program() {
                                return Some(self.build_module_map(&program.items));
                            }
                        }
                    }
                }
                continue;
            }
            let source = fs::read_to_string(&file_path).ok()?;
            let tokens = Lexer::new(&source).tokenize();
            let program = Parser::new(tokens).parse_program().ok()?;
            return Some(self.build_module_map(&program.items));
        }
        None
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
                    let key = if fd.is_method() {
                        format!("{}.{}", fd.receiver.as_ref().unwrap().name, fd.name.name)
                    } else {
                        fd.name.name.clone()
                    };
                    let prefixed_key = if prefix.is_empty() { key.clone() } else { format!("{}.{}", prefix, key) };
                    if let Some(sig) = self.functions.get(&prefixed_key).or_else(|| self.functions.get(&key)) {
                        let is_pub = fd.is_pub;
                        map.insert(fd.name.name.clone(), ModuleExport::Function { sig: sig.clone(), is_pub });
                    }
                }
                TopDecl::Module(md) => {
                    let new_prefix = if prefix.is_empty() { md.name.name.clone() } else { format!("{}.{}", prefix, md.name.name) };
                    let sub = self.build_module_map_inner(&md.items, &new_prefix);
                    map.insert(md.name.name.clone(), ModuleExport::SubModule(sub));
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
            None => return,
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
                    // The path is relative to the source directory: {source_dir}/{seg}.ax
                    if let Some(mut sub_exports) = self.load_external_module(seg) {
                        // The loaded file may have nested module wrappers
                        // (e.g., main.ax contains `module benchmark.main { ... }`).
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
                None => return,
            };
            let local_name = ud.alias.as_ref()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| item_name.clone());
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
        let exports = self.modules.get(module_name)?;
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
        let exports = self.modules.get(module_name)?;
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

        // For methods, inject the receiver's fields into scope (implicit self)
        if let Some(recv) = fd.receiver.as_ref() {
            // Add self as a variable (for match self { ... } in enum methods)
            self.add_local("self", CheckedType::Named(recv.name.clone()));
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
    }

    fn check_block(&mut self, block: &Block, expected_return: Option<CheckedType>) -> Option<CheckedType> {
        let mut last_expr_ty = None;
        let mut has_return = false;

        for item in &block.stmts {
            match item {
                StmtOrExpr::Stmt(stmt) => {
                    self.check_stmt(stmt);
                    if matches!(stmt, Stmt::Return(..)) {
                        has_return = true;
                        last_expr_ty = None; // return already checked, don't double-check
                    }
                }
                StmtOrExpr::Expr(expr) => {
                    last_expr_ty = Some(self.check_expr(expr));
                }
            }
        }

        // If this block is the function body and has a return, skip the return type check
        // (return statements are already checked individually)
        if let Some(expected) = expected_return {
            if !has_return {
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

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, ty_annot, value, span) => {
                let val_ty = self.check_expr(value);
                if let Some(annot) = ty_annot {
                    let annot_ty = CheckedType::from_ast_type(annot);
                    if !self.types_compatible(&val_ty, &annot_ty) && val_ty != CheckedType::Error {
                        self.error(
                            format!("type mismatch in let: annotated {}, found {}", annot_ty.name(), val_ty.name()),
                            *span,
                        );
                    }
                }
                self.add_local(&name.name, val_ty);
            }
            Stmt::Var(name, ty_annot, value, span) => {
                let val_ty = self.check_expr(value);
                if let Some(annot) = ty_annot {
                    let annot_ty = CheckedType::from_ast_type(annot);
                    if !self.types_compatible(&val_ty, &annot_ty) && val_ty != CheckedType::Error {
                        self.error(
                            format!("type mismatch in var: annotated {}, found {}", annot_ty.name(), val_ty.name()),
                            *span,
                        );
                    }
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
                    self.add_pattern_bindings(&arm.pattern);
                    match &arm.body {
                        MatchBody::Block(b) => { self.check_block(b, None); }
                        MatchBody::Expr(e) => { self.check_expr(e); }
                    }
                    self.pop_scope();
                }
                let _ = matched_ty;
            }
            Stmt::While(cond, body, _) => {
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
        }
    }

    // ========================================================================
    // Expression type checking
    // ========================================================================

    fn check_expr(&mut self, expr: &Expr) -> CheckedType {
        match expr {
            Expr::Ident(ident) => {
                if ident.name == "_" {
                    CheckedType::Int // wildcard placeholder type
                } else if let Some(ty) = self.lookup_local(&ident.name) {
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
                        ModuleExport::SubModule(_) => CheckedType::Named("module".into()),
                    }
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
                        if inner_ty.name() != "Bool" {
                            self.error(format!("cannot logically negate type {}", inner_ty.name()), *span);
                        }
                        CheckedType::Bool
                    }
                    UnaryOp::Ref | UnaryOp::MutRef => inner_ty, // reference keeps the type
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
                        if left_ty.name() != "Bool" {
                            self.error(format!("left operand of logical op must be Bool, found {}", left_ty.name()), *span);
                        }
                        if right_ty.name() != "Bool" {
                            self.error(format!("right operand of logical op must be Bool, found {}", right_ty.name()), *span);
                        }
                        CheckedType::Bool
                    }
                    BinOp::Assign => right_ty,
                }
            }
            Expr::Try(inner, _span) => {
                let inner_ty = self.check_expr(inner);
                // ? unwraps Result or Option — return the inner type
                // Phase 0 simplified: just pass through
                inner_ty
            }
            Expr::Imply(_, _, _) => CheckedType::Bool,
            Expr::Is(_, _, _) => CheckedType::Bool,
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
                            for (i, arg) in args.iter().enumerate() {
                                let arg_ty = self.check_expr(arg);
                                let param_idx = i + 1; // skip receiver
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
                    // Also check imported items for function aliases
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
            Expr::Struct(name, fields, span) => {
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
            Expr::PipeClosure(_, body, _) => {
                let _ = self.check_expr(body);
                CheckedType::Named("fn".into())
            }
            Expr::As(inner, ty, span) => {
                let inner_ty = self.check_expr(inner);
                let target_ty = CheckedType::from_ast_type(ty);
                match (&inner_ty, &target_ty) {
                    (CheckedType::Int, CheckedType::Float64) => target_ty,
                    (CheckedType::Float64, CheckedType::Int) => target_ty,
                    _ if inner_ty == target_ty => target_ty,
                    _ if inner_ty == CheckedType::Error => CheckedType::Error,
                    _ => self.error(format!("unsupported type cast: {} to {}", inner_ty.name(), target_ty.name()), *span),
                }
            }
            Expr::Await(inner, _) => self.check_expr(inner),
            Expr::Comptime(inner, _) => self.check_expr(inner),
            Expr::If(cond, then_block, elifs, else_block, _) => {
                self.check_expr(cond);
                self.check_block(then_block, None);
                for (econd, eblock) in elifs {
                    self.check_expr(econd);
                    self.check_block(eblock, None);
                }
                if let Some(eb) = else_block { self.check_block(eb, None); }
                CheckedType::Named("_".into())
            }
        }
    }

    fn types_compatible(&self, found: &CheckedType, expected: &CheckedType) -> bool {
        if found == &CheckedType::Error || expected == &CheckedType::Error {
            return true; // Don't cascade errors
        }
        if found == expected {
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
            // Named types: allow compatible across different names (e.g., Range vs Vec)
            (CheckedType::Named(_), CheckedType::Named(_)) => true,
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

pub struct BorrowChecker {
    ownership: Vec<HashMap<String, OwnershipInfo>>,
    borrow_stack: Vec<Vec<ScopeBorrow>>,
    errors: Vec<BorrowError>,
    param_names: HashSet<String>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            ownership: vec![HashMap::new()],
            borrow_stack: vec![Vec::new()],
            errors: Vec::new(),
            param_names: HashSet::new(),
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

    fn add_local(&mut self, name: &str, is_mutable: bool) {
        if let Some(scope) = self.ownership.last_mut() {
            scope.insert(name.to_string(), OwnershipInfo {
                state: BorrowState::Owned,
                read_borrow_count: 0,
                is_mutable,
            });
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
            self.add_local(&param.name.name, true);
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
            Stmt::Let(name, _, value, _) => {
                let _ = self.check_expr(value);
                if let Expr::Ident(ident) = value {
                    if self.param_names.contains(&ident.name) {
                        self.read_borrow(&ident.name, ident.span);
                    } else {
                        self.move_var(&ident.name, ident.span);
                    }
                }
                self.add_local(&name.name, false);
            }
            Stmt::Var(name, _, value, _) => {
                let _ = self.check_expr(value);
                if let Expr::Ident(ident) = value {
                    if self.param_names.contains(&ident.name) {
                        self.read_borrow(&ident.name, ident.span);
                    } else {
                        self.move_var(&ident.name, ident.span);
                    }
                }
                self.add_local(&name.name, true);
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
            Stmt::While(cond, body, _) => {
                self.check_expr(cond);
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::For(var, iter, body, _) => {
                self.check_expr(iter);
                self.add_local(&var.name, true);
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
                    self.add_local(&name.name, true);
                }
            }
            Stmt::Spawn(body, _) => {
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
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
            Expr::Struct(_, fields, span) => {
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

    #[allow(dead_code)]
    fn release_borrows_for(&mut self, name: &str) {
        let to_release: Vec<(String, BorrowType)> = self.borrow_stack.iter()
            .flat_map(|scope| scope.iter())
            .filter(|b| b.var_name == name)
            .map(|b| (b.var_name.clone(), b.borrow_type))
            .collect();
        for (n, ty) in to_release {
            self.release_borrow(&n, ty);
        }
        for borrows in self.borrow_stack.iter_mut() {
            borrows.retain(|b| b.var_name != name);
        }
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
    use axiom_lexer::Lexer;
    use axiom_parser::Parser;

    fn check(source: &str) -> Result<(), Vec<CheckError>> {
        let tokens = Lexer::new(source).tokenize();
        let program = Parser::new(tokens).parse_program();
        match program {
            Ok(p) => Checker::new().check_program(&p),
            Err(e) => Err(vec![CheckError {
                message: format!("parse error: {e}"),
                span: e.span,
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
        let result = check_borrow("fn main() { var x = 42; var y = x; let z = x; }");
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
        let result = check_borrow("fn main() { var x = 42; let r = &x; var y = x; }");
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
        let result = check_borrow("fn foo(x: Int) -> Int { return x; } fn main() { var a = 42; foo(a); let b = a; }");
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
}
