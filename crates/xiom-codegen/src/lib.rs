// XIOM — LLVM IR Codegen
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Codegen — Phase 0: AST → LLVM IR text.
//! Emits human-readable LLVM IR that can be compiled with `llc`.
//! No external dependencies — pure string emission.
//! Handles: functions, arithmetic, control flow (if/else/while/match),
//! let/var bindings, structs, function calls.

use xiom_ast::*;
use std::collections::HashMap;
use std::collections::HashSet;

// ============================================================================
// Metadata for user-defined types
// ============================================================================

#[derive(Clone)]
#[allow(dead_code)]
struct TypeMeta {
    fields: Vec<(String, String)>,  // (field_name, type_name)
    derives: Vec<DeriveTrait>,
    invariants: Vec<Expr>,
}

// ============================================================================
// LLVM IR Emitter
// ============================================================================

pub struct IrEmitter {
    output: String,
    /// Counter for unique temporary names
    tmp_counter: u32,
    /// Counter for unique block labels
    block_counter: u32,
    /// Counter for unique string constants
    str_counter: u32,
    /// Local variables: name → (alloca_register, llvm_type)
    locals: Vec<HashMap<String, (String, String)>>,
    /// Known function signatures: name → (param_llvm_types, return_llvm_type_or_empty)
    functions: HashMap<String, (Vec<String>, String)>,
    /// Known type structures: name → field names (for struct type definition)
    types: HashMap<String, Vec<String>>,
    /// Full type metadata: name → TypeMeta
    type_meta: HashMap<String, TypeMeta>,
    /// Current function name (for labels)
    current_fn: Option<String>,
    /// Return type of current function (empty = void)
    current_return_type: String,
    /// String constants to emit at the top
    strings: Vec<String>,
    /// Current function's param LLVM types (index → type)
    current_param_llvm_types: Vec<String>,
    /// Whether to emit contract runtime checks
    check_contracts: bool,
    /// Generic function ASTs stored for later monomorphisation
    generic_fn_decls: Vec<FnDecl>,
    /// Tracked generic instantiations: (fn_original_name, vec![concrete_type_names])
    generic_instantiations: Vec<(String, Vec<String>)>,
    /// Whether @llvm.trap has been declared
    #[allow(dead_code)]
    has_llvm_trap_decl: bool,
    /// Pre-state value of self (for self@pre in ensures)
    #[allow(dead_code)]
    self_pre_value: Option<String>,
    /// Current function's ensures clauses (for contract checking at return points)
    current_ensures: Vec<Expr>,
    /// Alloca for the result value in ensures expressions
    result_ptr: Option<String>,
    /// Alloca for match result in expression position
    match_result_ptr: Option<String>,
    /// Interface registry: interface name → vec of (method_name, param_type_names)
    interfaces: HashMap<String, Vec<(String, Vec<String>)>>,
    /// Enum variants registry: enum name → vec of (variant_name, field_names)
    enum_variants: HashMap<String, Vec<(String, Vec<String>)>>,
    /// Scrutinee info for match arm field extraction: (alloca_name, type_name)
    #[allow(dead_code)]
    scrutinee_info: Option<(String, String)>,
    /// Builtin types whose impls have been referenced by the program
    used_builtins: HashSet<String>,
    /// Current type substitution map for monomorphisation: generic_name → concrete_type
    current_type_map: HashMap<String, String>,
    /// Maps variable name to concrete type for generic params in monomorphised functions
    param_concrete_types: HashMap<String, String>,
    /// LLVM target triple (default: x86_64-pc-windows-msvc)
    target_triple: String,
    /// Maximum allowed recursion depth (emitted into LLVM IR as constant)
    max_recursion_depth: u32,
    /// Tracks emitted function names to avoid duplicate definitions
    emitted_fns: HashSet<String>,
    /// Current module prefix for scoped type resolution (e.g., "types" or "derive")
    current_module: Option<String>,
    /// Maps function pointer parameter names to their LLVM return types
    fn_ptr_return_types: HashMap<String, String>,
}

impl IrEmitter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            tmp_counter: 0,
            block_counter: 0,
            str_counter: 0,
            locals: vec![HashMap::new()],
            functions: HashMap::new(),
            types: HashMap::new(),
            type_meta: HashMap::new(),
            current_fn: None,
            current_return_type: String::new(),
            strings: Vec::new(),
            current_param_llvm_types: Vec::new(),
            check_contracts: true,
            max_recursion_depth: 500,

            // Remaining fields use defaults
            generic_fn_decls: Vec::new(),
            generic_instantiations: Vec::new(),
            has_llvm_trap_decl: false,
            self_pre_value: None,
            current_ensures: Vec::new(),
            result_ptr: None,
            match_result_ptr: None,
            interfaces: HashMap::new(),
            enum_variants: HashMap::new(),
            scrutinee_info: None,
            used_builtins: HashSet::new(),
            current_type_map: HashMap::new(),
            param_concrete_types: HashMap::new(),
            target_triple: "x86_64-pc-windows-msvc".to_string(),
            emitted_fns: HashSet::new(),
            current_module: None,
            fn_ptr_return_types: HashMap::new(),
        }
    }

    pub fn set_target_triple(&mut self, triple: &str) {
        self.target_triple = triple.to_string();
    }

    pub fn set_check_contracts(&mut self, enabled: bool) {
        self.check_contracts = enabled;
    }

    pub fn set_max_recursion_depth(&mut self, depth: u32) {
        self.max_recursion_depth = depth;
    }

    fn fresh_tmp(&mut self) -> String {
        let n = self.tmp_counter;
        self.tmp_counter += 1;
        format!("%tmp{n}")
    }

    fn fresh_block(&mut self, label: &str) -> String {
        let n = self.block_counter;
        self.block_counter += 1;
        format!("{label}{n}")
    }

    fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.locals.pop();
    }

    fn add_local(&mut self, name: &str, reg: String, llvm_ty: &str) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), (reg, llvm_ty.to_string()));
        }
    }

    fn lookup_local(&self, name: &str) -> Option<&(String, String)> {
        for scope in self.locals.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }

    fn emitln(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    /// Convert literal "0" to "zeroinitializer" for aggregate (struct) types
    fn zero_val_for(&self, val: &str, llvm_ty: &str) -> String {
        if val == "0" && llvm_ty.starts_with("%struct.") {
            "zeroinitializer".to_string()
        } else {
            val.to_string()
        }
    }

    fn xiom_to_llvm_type(xiom_ty: &str) -> &'static str {
        match xiom_ty {
            "Int8" | "UInt8" | "Char" => "i8",
            "Bool" => "i64",
            "Int16" | "UInt16" => "i16",
            "Int32" | "UInt32" => "i32",
            "Int" | "Int64" | "UInt" | "UInt64" => "i64",
            "Float32" => "float",
            "Float64" => "double",
            "Str" => "i8*",
            "()" => "void",
            _ => "i64", // Default: treat unknown types as i64
        }
    }

    fn xiom_type_name_from_llvm(llvm_ty: &str) -> String {
        let base = llvm_ty
            .trim_start_matches("%struct.")
            .trim_start_matches('%')
            .trim_end_matches('*')
            .trim();
        match base {
            "i64" => "Int".to_string(),
            "i32" => "Int32".to_string(),
            "i16" => "Int16".to_string(),
            "i8" => "Int8".to_string(),
            "double" => "Float64".to_string(),
            "float" => "Float32".to_string(),
            "i1" => "Bool".to_string(),
            "i8*" => "Str".to_string(),
            _ => base.to_string(),
        }
    }

    fn type_from_ast(ty: &Type) -> String {
        match ty {
            Type::Named(ident, _) => ident.name.clone(),
            Type::Ref(inner) => Self::type_from_ast(inner),
            Type::MutRef(inner) => Self::type_from_ast(inner),
            Type::Option(_) => "Option".to_string(),
            Type::Result(_, _) => "Result".to_string(),
            Type::Vec(_) => "Vec".to_string(),
            Type::Map(_, _) => "Map".to_string(),
            Type::Set(_) => "Set".to_string(),
            Type::Tuple(types) => {
                let parts: Vec<String> = types.iter().map(Self::type_from_ast).collect();
                format!("Tuple_{}", parts.join("_"))
            }
            _ => "Int".to_string(),
        }
    }

    fn llvm_type_for(&self, type_name: &str) -> String {
        // Try current module's qualified name first (e.g., "types.Person")
        if let Some(ref module) = self.current_module {
            let qualified = format!("{}.{}", module, type_name);
            if self.types.contains_key(&qualified) || self.type_meta.contains_key(&qualified) {
                return format!("%struct.{qualified}");
            }
        }
        // Try exact match
        if self.types.contains_key(type_name) || self.type_meta.contains_key(type_name) {
            return format!("%struct.{type_name}");
        }
        // Search for any module-qualified variant ending with .type_name
        for (key, _) in &self.type_meta {
            if key.ends_with(&format!(".{type_name}")) {
                return format!("%struct.{key}");
            }
        }
        // Check builtin types first (match known Axiom type names, NOT the default i64 fallback)
        let builtin = Self::xiom_to_llvm_type(type_name);
        match type_name {
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
            | "Bool" | "Float32" | "Float64" | "Str" | "Char" | "()" => return builtin.to_string(),
            _ => {}
        }
        // If type_name is an enum variant (e.g., "Image"), find its parent enum type
        for (enum_key, variants) in &self.enum_variants {
            if variants.iter().any(|(v, _)| v == type_name) {
                return format!("%struct.{enum_key}");
            }
        }
        builtin.to_string()
    }

    fn field_llvm_type(&self, struct_name: &str, field_idx: usize) -> String {
        let meta = self.type_meta.get(struct_name)
            .or_else(|| {
                // Try current module's qualified name first (deterministic)
                if let Some(ref module) = self.current_module {
                    let qualified = format!("{}.{}", module, struct_name);
                    self.type_meta.get(&qualified)
                } else {
                    None
                }
            })
            .or_else(|| {
                // Fallback: search all qualified keys
                self.type_meta.iter()
                    .find(|(k, _)| k.ends_with(&format!(".{struct_name}")))
                    .map(|(_, v)| v)
            });
        if let Some(meta) = meta {
            if let Some((_, ty_name)) = meta.fields.get(field_idx) {
                return self.llvm_type_for(ty_name);
            }
        }
        "i64".to_string()
    }

    // ========================================================================
    // Program compilation
    // ========================================================================

    pub fn compile_program(&mut self, program: &Program) -> Result<String, String> {
        // Register builtin types for Option and Result
        if !self.types.contains_key("Option") {
            self.types.insert("Option".to_string(), vec!["discriminant".to_string(), "value".to_string()]);
            self.type_meta.insert("Option".to_string(), TypeMeta {
                fields: vec![("discriminant".to_string(), "Int".to_string()), ("value".to_string(), "Int".to_string())],
                derives: vec![],
                invariants: vec![],
            });
        }
        if !self.types.contains_key("Result") {
            self.types.insert("Result".to_string(), vec!["discriminant".to_string(), "value".to_string(), "error".to_string()]);
            self.type_meta.insert("Result".to_string(), TypeMeta {
                fields: vec![
                    ("discriminant".to_string(), "Int".to_string()),
                    ("value".to_string(), "Int".to_string()),
                    ("error".to_string(), "Int".to_string()),
                ],
                derives: vec![],
                invariants: vec![],
            });
        }
        if !self.enum_variants.contains_key("Result") {
            self.enum_variants.insert("Result".to_string(), vec![
                ("Err".to_string(), vec!["error".to_string()]),
                ("Ok".to_string(), vec!["value".to_string()]),
            ]);
        }
        // Register Vec type for runtime operations
        if !self.types.contains_key("Vec") {
            self.types.insert("Vec".to_string(), vec!["data".to_string(), "len".to_string(), "cap".to_string()]);
        }

        // Register type structures
        for item in &program.items {
            self.register_type_layout(item);
        }

        // Register function signatures
        for item in &program.items {
            self.register_functions(item);
        }

        // Emit module header
        self.emitln("; XIOM Phase 1 — LLVM IR");
        self.emitln("; Auto-generated by xiomc\n");
        self.emitln(&format!("target triple = \"{}\"", self.target_triple));
        self.emitln("");

        // Emit builtin struct types FIRST so user types can reference them
        self.emitln("%struct.Vec = type { i8*, i64, i64 }");
        self.emitln("");

        // Emit struct type definitions using actual field types from type_meta
        for (name, meta) in &self.type_meta.clone() {
            let struct_ref = format!("%struct.{name}");
            let field_types: Vec<String> = meta.fields.iter()
                .map(|(_, ty_name)| {
                    let t = self.llvm_type_for(ty_name);
                    if t == struct_ref { format!("{t}*") } else { t }
                })
                .collect();
            self.emitln(&format!("%struct.{name} = type {{ {} }}", field_types.join(", ")));
        }
        if !self.types.is_empty() {
            self.emitln("");
        }

        // Declare external C functions + LLVM intrinsics
        self.emitln("declare i32 @printf(i8*, ...)");
        self.emitln("declare i32 @puts(i8*)");
        self.emitln("declare void @llvm.trap()");
        self.emitln("@xiom_recursion_counter = internal thread_local global i64 0");
        self.emitln("declare i8* @malloc(i64)");
        self.emitln("declare i8* @realloc(i8*, i64)");
        self.emitln("declare void @free(i8*)");
        self.emitln("declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)");
        self.emitln("declare i64 @xiom_is_sorted(i8*, i64)");
        self.emitln("declare i64 @xiom_all(i8*, i64, i8*)");
        self.emitln("declare i64 @xiom_none(i8*, i64, i8*)");
        self.emitln("declare i64 @xiom_contains(i8*, i64)");
        self.emitln("declare i8* @xiom_read_file(i8*)");
        self.emitln("declare i64 @xiom_file_size(i8*)");
        self.emitln("declare void @xiom_free(i8*)");
        self.emitln("declare i8 @xiom_char_at(i8*, i64)");
        self.emitln("declare i64 @xiom_str_len(i8*)");
        // String interning
        self.emitln("declare i64 @xiom_intern(i8*, i64, i64)");
        self.emitln("declare i8* @xiom_lookup(i64)");
        // IR emission
        self.emitln("declare i64 @xiom_ir_open(i8*)");
        self.emitln("declare void @xiom_ir_close()");
        self.emitln("declare void @xiom_ir_header()");
        self.emitln("declare void @xiom_ir_define(i64, i64)");
        self.emitln("declare void @xiom_ir_param(i64, i64)");
        self.emitln("declare void @xiom_ir_entry()");
        self.emitln("declare void @xiom_ir_alloca(i64, i64)");
        self.emitln("declare void @xiom_ir_store(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_load(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_binop(i8*, i64, i64, i64, i64)");
        self.emitln("declare void @xiom_ir_call(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_call_arg(i64, i64)");
        self.emitln("declare void @xiom_ir_call_lit(i8*)");
        self.emitln("declare void @xiom_ir_call_end()");
        self.emitln("declare void @xiom_ir_ret(i64, i64)");
        self.emitln("declare void @xiom_ir_ret_void()");
        self.emitln("declare void @xiom_ir_endfn()");
        self.emitln("declare void @xiom_ir_raw(i8*)");
        self.emitln("declare void @xiom_ir_emit_program(i64)");
        // v0.9.4 string-based IR emission
        self.emitln("declare void @xiom_ir_define_s(i8*, i8*)");
        self.emitln("declare void @xiom_ir_param_int(i64)");
        self.emitln("declare void @xiom_ir_param_double(i64)");
        self.emitln("declare void @xiom_ir_alloca_s(i64)");
        self.emitln("declare void @xiom_ir_store_param(i64, i64)");
        self.emitln("declare void @xiom_ir_load_s(i64, i64)");
        self.emitln("declare void @xiom_ir_add(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_fmul(i64, i64, i64)");
        self.emitln("declare void @xiom_ir_call_fn(i64, i8*, i8*)");
        self.emitln("declare void @xiom_ir_call_arg_lit(i8*, i8*)");
        self.emitln("declare void @xiom_ir_ret_reg(i64)");
        self.emitln("declare void @xiom_ir_ret_lit(i64)");
        // v0.10.0 function table
        self.emitln("declare void @xiom_fn_table_init()");
        self.emitln("declare void @xiom_set_source(i64)");
        self.emitln("declare void @xiom_fn_table_add(i64, i64, i64, i64, i64)");
        self.emitln("declare i64 @xiom_fn_table_count()");
        self.emitln("declare i64 @xiom_fn_name_id(i64)");
        self.emitln("declare i64 @xiom_fn_ret_type_id(i64)");
        self.emitln("declare i64 @xiom_fn_param_count(i64)");
        self.emitln("declare i64 @xiom_fn_body_start(i64)");
        self.emitln("declare i64 @xiom_fn_body_end(i64)");
        self.emitln("declare void @xiom_fn_emit_all()");
        self.emitln("");

        // Emit derive implementations for types with derive clauses
        self.compile_derive_impls(&program.items)?;

        // Define all non-generic function bodies, tracking generic instantiations
        for item in &program.items {
            self.compile_top_decl(item)?;
        }

        // Emit monomorphised generic function bodies
        self.compile_generic_monomorphisations()?;

        // Emit builtin runtime implementations (Option, Result, alloc, etc.)
        self.compile_builtin_impls();

        // Emit string constants collected during compilation
        for s in &self.strings.clone() {
            self.emitln(&s);
        }
        if !self.strings.is_empty() {
            self.emitln("");
        }

        Ok(self.output.clone())
    }

     fn register_type_layout(&mut self, item: &TopDecl) {
        self.register_type_layout_impl(item, "");
    }

    fn register_type_layout_impl(&mut self, item: &TopDecl, prefix: &str) {
        if let TopDecl::Type(td) = item {
            if td.fields.is_empty() { return; }
            let fields: Vec<String> = td.fields.iter()
                .map(|f| f.name.name.clone())
                .collect();
            let bare_name = td.name.name.clone();
            let type_name = if prefix.is_empty() { bare_name.clone() } else { format!("{}.{}", prefix, bare_name) };
            let full_fields: Vec<(String, String)> = td.fields.iter()
                .map(|f| (f.name.name.clone(), Self::type_from_ast(&f.ty)))
                .collect();
            self.types.insert(type_name.clone(), fields);
            self.type_meta.insert(type_name, TypeMeta {
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
            for variant in &ed.variants {
                let vname = variant.name.name.clone();
                let mut vfields = Vec::new();
                for field in &variant.fields {
                    let fname = field.name.name.clone();
                    if !all_fields.contains(&fname) {
                        all_fields.push(fname.clone());
                        all_meta.push((fname.clone(), Self::type_from_ast(&field.ty)));
                    }
                    vfields.push(fname);
                }
                variants_info.push((vname, vfields));
            }
            self.types.insert(enum_name.clone(), all_fields);
            self.type_meta.insert(enum_name.clone(), TypeMeta {
                fields: all_meta,
                derives: ed.derives.clone(),
                invariants: Vec::new(),
            });
            self.enum_variants.insert(enum_name, variants_info);
        }
        if let TopDecl::Module(md) = item {
            let new_prefix = if prefix.is_empty() { md.name.name.clone() } else { format!("{}.{}", prefix, md.name.name) };
            for sub in &md.items {
                self.register_type_layout_impl(sub, &new_prefix);
            }
        }
    }

    fn ensure_tuple_type_registered(&mut self, ty: &Type) {
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

    fn substitute_concrete_name(ty: &Type, type_map: &HashMap<String, String>) -> String {
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

    fn ensure_concrete_tuple_type_registered(&mut self, ty: &Type, type_map: &HashMap<String, String>) {
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

    fn register_functions(&mut self, item: &TopDecl) {
        if let TopDecl::Fn(fd) = item {
            // Register tuple types used in function signature before resolving LLVM types
            fd.return_type.as_ref().map(|t| self.ensure_tuple_type_registered(t));
            for p in &fd.params {
                self.ensure_tuple_type_registered(&p.ty);
            }
            let mut param_types: Vec<String> = Vec::new();
            // For methods, self is the first parameter
            if let Some(recv) = fd.receiver.as_ref() {
                param_types.push(self.llvm_type_for(&recv.name));
            }
            let explicit_params: Vec<String> = fd.params.iter()
                .map(|p| self.llvm_type_for(&Self::type_from_ast(&p.ty)))
                .collect();
            param_types.extend(explicit_params);
            let ret_type = fd.return_type.as_ref()
                .map(|t| self.llvm_type_for(&Self::type_from_ast(t)))
                .unwrap_or_else(|| "void".to_string());
            let key = self.fn_key(fd);
            self.functions.insert(key, (param_types, ret_type));
            if !fd.generics.is_empty() {
                self.generic_fn_decls.push(fd.clone());
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

    fn fn_key(&self, fd: &FnDecl) -> String {
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
    fn fn_symbol(&self, fd: &FnDecl) -> String {
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
    fn resolve_module_call(&self, receiver: &Expr, fn_name: &str) -> String {
        if let Expr::Ident(id) = receiver {
            let module_name = &id.name;
            // Try leaf-qualified: "math.run_all"
            let leaf_key = format!("{}.{}", module_name, fn_name);
            if self.functions.contains_key(&leaf_key) {
                return leaf_key;
            }
            // Try parent-qualified: "benchmark.math.run_all" (current_module parent + module_name)
            if let Some(ref cur_mod) = self.current_module {
                if let Some(parent) = cur_mod.rsplitn(2, '.').last() {
                    let parent_key = format!("{}.{}.{}", parent, module_name, fn_name);
                    if self.functions.contains_key(&parent_key) {
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
        }
        fn_name.to_string()
    }

    fn compile_top_decl(&mut self, item: &TopDecl) -> Result<(), String> {
        match item {
            TopDecl::Fn(fd) => {
                // Skip generic functions — they will be monomorphised later
                if fd.generics.is_empty() {
                    if fd.body.is_some() {
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
            TopDecl::Interface(_) | TopDecl::Enum(_) | TopDecl::Const(_) | TopDecl::Type(_) | TopDecl::Use(_) => Ok(()),
        }
    }

    fn compile_fn(&mut self, fd: &FnDecl) -> Result<(), String> {
        self.push_scope();
        self.block_counter = 0;
        self.tmp_counter = 0;
        self.fn_ptr_return_types.clear();

        let ret_llvm = fd.return_type.as_ref()
            .map(|t| self.llvm_type_for(&Self::type_from_ast(t)))
            .unwrap_or_else(|| "void".to_string());
        self.current_return_type = ret_llvm.clone();
        self.current_param_llvm_types = fd.params.iter()
            .map(|p| self.llvm_type_for(&Self::type_from_ast(&p.ty)))
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

        // For methods, prepend the self struct parameter
        let self_llvm_ty = fd.receiver.as_ref().map(|r| {
            self.llvm_type_for(&r.name)
        });
        let self_offset: usize = if self_llvm_ty.is_some() { 1 } else { 0 };

        let mut params_str: Vec<String> = Vec::new();
        if let Some(ref st) = self_llvm_ty {
            params_str.push(format!("{st} %param_self"));
        }
        let explicit_params: Vec<String> = fd.params.iter()
            .enumerate()
            .map(|(i, p)| {
                let llvm_ty = self.llvm_type_for(&Self::type_from_ast(&p.ty));
                format!("{llvm_ty} %param{}", i + self_offset)
            })
            .collect();
        params_str.extend(explicit_params);

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
            self.emitln(&format!("  {self_alloca} = alloca {st}"));
            self.emitln(&format!("  store {st} %param_self, {st}* {self_alloca}"));
            self.add_local("self", self_alloca.clone(), st);
            // Also add struct fields as locals for direct access
            let fields_clone = self.types.get(&recv.name).cloned();
            if let Some(fields) = fields_clone {
                let alloca_ref = self_alloca;
                for (idx, field_name) in fields.iter().enumerate() {
                    let field_llvm_ty = self.field_llvm_type(&recv.name, idx);
                    let gep = self.fresh_tmp();
                    self.emitln(&format!("  {gep} = getelementptr {st}, {st}* {alloca_ref}, i32 0, i32 {idx}"));
                    self.add_local(field_name, gep, &field_llvm_ty);
                }
            }
        }
        // Then allocate explicit parameters
        for (i, param) in fd.params.iter().enumerate() {
            let llvm_ty = self.llvm_type_for(&Self::type_from_ast(&param.ty));
            let alloca = self.fresh_tmp();
            let param_idx = i + self_offset;
            self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
            self.emitln(&format!("  store {llvm_ty} %param{param_idx}, {llvm_ty}* {alloca}"));
            self.add_local(&param.name.name, alloca, &llvm_ty);
            // Track function pointer return types for function pointer parameters
            if let Type::Fn(_, ret) = &param.ty {
                let ret_ty_name = Self::type_from_ast(ret);
                let ret_llvm = self.llvm_type_for(&ret_ty_name);
                self.fn_ptr_return_types.insert(param.name.name.clone(), ret_llvm);
            }
        }

        // Capture self@pre for ensures (method functions with self@pre references)
        if self.check_contracts && !self.current_ensures.is_empty() {
            if let Some(recv) = fd.receiver.as_ref() {
                if let Some((ptr, llvm_ty)) = self.lookup_local(&recv.name).cloned() {
                    let pre_alloca = self.fresh_tmp();
                    self.emitln(&format!("  {pre_alloca} = alloca {llvm_ty}"));
                    let loaded = self.fresh_tmp();
                    self.emitln(&format!("  {loaded} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                    self.emitln(&format!("  store {llvm_ty} {loaded}, {llvm_ty}* {pre_alloca}"));
                    self.add_local("__self_pre", pre_alloca, &llvm_ty);
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
        }

        self.emitln("}\n");
        self.pop_scope();
        self.current_fn = None;
        self.current_ensures.clear();
        self.result_ptr = None;
        Ok(())
    }

    // ========================================================================
    // Contract checks (requires / ensures / invariant)
    // ========================================================================

    /// Emit a runtime check for a single boolean contract expression.
    /// If the expression evaluates to false (i64 0), emit a panic and trap.
    fn compile_contract_check(&mut self, expr: &Expr, clause_type: &str) {
        let cond_val = match self.compile_expr(expr) {
            Ok(v) => v,
            Err(_) => return,
        };
        let ok_label = self.fresh_block("contract_ok");
        let fail_label = self.fresh_block("contract_fail");
        // Ensure we have an i1 for the branch — some expressions (or/and) return i64
        let expr_ty = self.infer_llvm_type(expr);
        let cond_i1 = if expr_ty == "i1" {
            cond_val
        } else if expr_ty == "i64" {
            let tmp = self.fresh_tmp();
            self.emitln(&format!("  {tmp} = icmp ne i64 {cond_val}, 0"));
            tmp
        } else {
            let ext = self.fresh_tmp();
            self.emitln(&format!("  {ext} = zext {expr_ty} {cond_val} to i64"));
            let tmp = self.fresh_tmp();
            self.emitln(&format!("  {tmp} = icmp ne i64 {ext}, 0"));
            tmp
        };
        self.emitln(&format!("  br i1 {cond_i1}, label %{ok_label}, label %{fail_label}"));
        self.emitln(&format!("\n{fail_label}:"));
        let msg_ptr = self.fresh_tmp();
        let msg = format!("contract violated: {clause_type} at {}", expr.span());
        let str_id = self.str_counter;
        self.str_counter += 1;
        let label = format!("@.contract_str{str_id}");
        let escaped = msg.replace('\\', "\\\\").replace('"', "\\22");
        self.strings.push(format!(
            "{label} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"",
            len = msg.len() + 1
        ));
        self.emitln(&format!("  {msg_ptr} = getelementptr [{len} x i8], [{len} x i8]* {label}, i64 0, i64 0",
            len = msg.len() + 1));
        self.emitln(&format!("  call i32 @puts(i8* {msg_ptr})"));
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln(&format!("\n{ok_label}:"));
    }

    /// Emit checks for all ensures clauses of the current function.
    /// Called just before a return instruction.
    fn compile_ensures_checks(&mut self) {
        for expr in &self.current_ensures.clone() {
            self.compile_contract_check(expr, "ensures");
        }
    }

    /// Generate an invariant check function for a struct type.
    fn compile_invariant_check(&mut self, type_name: &str) -> Result<(), String> {
        let invariants = match self.type_meta.get(type_name) {
            Some(m) => m.invariants.clone(),
            None => return Ok(()),
        };
        if invariants.is_empty() {
            return Ok(());
        }
        // Look up field names for this type
        let field_names = match self.types.get(type_name) {
            Some(f) => f.clone(),
            None => return Ok(()),
        };
        let struct_ty = format!("%struct.{type_name}");
        let fn_name = format!("{type_name}.invariant_check");
        self.emitln(&format!("define void @{fn_name}({struct_ty} %__obj) {{"));
        // Store the struct value in an alloca for field access
        let alloca = self.fresh_tmp();
        self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %__obj, {struct_ty}* {alloca}"));
        // Register field locals for invariant expression compilation
        for (i, fname) in field_names.iter().enumerate() {
            let gep = self.fresh_tmp();
            let loaded = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
            // Create an alloca so compile_expr can load from it via the pointer
            let field_alloca = self.fresh_tmp();
            self.emitln(&format!("  {field_alloca} = alloca {field_llvm_ty}"));
            self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {field_alloca}"));
            // Push a synthetic scope for invariant compilation
            // Since compile_contract_check will call compile_expr which uses lookup_local,
            // we need to register these field names temporarily
            if self.locals.is_empty() {
                self.locals.push(HashMap::new());
            }
            self.add_local(fname, field_alloca, &field_llvm_ty);
        }
        for inv in &invariants {
            self.compile_contract_check(inv, "invariant");
        }
        self.emitln("  ret void");
        self.emitln("}\n");
        // Clean up the temporary field locals
        // We added them to the current scope, they'll be cleaned on pop_scope
        // But since we're not actually pushing a real scope, let's just clear
        if let Some(scope) = self.locals.last_mut() {
            for fname in &field_names {
                scope.remove(fname);
            }
        }
        Ok(())
    }

    /// Emit a call to a type's invariant check function.
    /// Helper: given an expression and its compiled value register, emit invariant
    /// check if the expression evaluates to a struct type that has invariants.
    fn maybe_check_value_invariants(&mut self, value: &Expr, val_reg: &str) {
        let type_name = self.struct_type_from_expr(value);
        if let Some(ref tn) = type_name {
            if self.type_meta.get(tn).map(|m| !m.invariants.is_empty()).unwrap_or(false) {
                self.compile_invariant_call(tn, val_reg);
            }
        }
    }

    /// Infer the struct type name from an expression (if it produces a struct value).
    fn struct_type_from_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Struct(ident, _, _) => Some(ident.name.clone()),
            Expr::Ident(ident) => {
                if let Some((_, llvm_ty)) = self.lookup_local(&ident.name) {
                    if llvm_ty.starts_with("%struct.") {
                        return Some(llvm_ty[8..].to_string());
                    }
                }
                None
            }
            Expr::Call(func, _, _) => {
                let fn_name = match &**func {
                    Expr::Ident(name) => Some(name.name.clone()),
                    Expr::Field(obj, field, _) => {
                        let bare = field.name.clone();
                        if let Some(recv_type) = self.infer_struct_type_name(obj) {
                            let qualified = format!("{}.{}", recv_type, field.name);
                            if self.functions.contains_key(&qualified) {
                                Some(qualified)
                            } else {
                                Some(bare)
                            }
                        } else {
                            Some(bare)
                        }
                    }
                    _ => None,
                };
                if let Some(ref name) = fn_name {
                    // Check if the known return type is a struct
                    if self.type_meta.contains_key(name) {
                        return Some(name.clone());
                    }
                    // Also check the return type from the function registry
                    if let Some((_, ret_ty)) = self.functions.get(name) {
                        if ret_ty.starts_with("%struct.") {
                            return Some(ret_ty[8..].to_string());
                        }
                    }
                }
                None
            }
            Expr::Tuple(_, _) => None,
            Expr::Some(_, _) => {
                // Some(x) produces Option[T] — not a struct with user invariants
                None
            }
            Expr::Ok(_, _) | Expr::Err(_, _) => {
                None
            }
            _ => None,
        }
    }

    /// Convert a value to i64 via bitcast or ptrtoint if needed
    fn val_to_i64(&mut self, val: &str, ty: &str) -> String {
        if ty == "void" {
            return "0".to_string();
        }
        if ty == "double" {
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = bitcast double {val} to i64"));
            bc
        } else if ty == "i8*" || ty.contains('*') {
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = ptrtoint {ty} {val} to i64"));
            bc
        } else if ty.starts_with('%') {
            let ptr = self.fresh_tmp();
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {ptr} = alloca {ty}"));
            self.emitln(&format!("  store {ty} {val}, {ty}* {ptr}"));
            self.emitln(&format!("  {bc} = ptrtoint {ty}* {ptr} to i64"));
            bc
        } else {
            val.to_string()
        }
    }

    /// Convert a value to i8* via bitcast (for pointers) or inttoptr (for ints)
    fn val_to_i8ptr(&mut self, val: &str, ty: &str) -> String {
        if ty == "i8*" {
            val.to_string()
        } else if ty.contains('*') {
            let cast = self.fresh_tmp();
            self.emitln(&format!("  {cast} = bitcast {ty} {val} to i8*"));
            cast
        } else {
            let i64_val = self.val_to_i64(val, ty);
            let cast = self.fresh_tmp();
            self.emitln(&format!("  {cast} = inttoptr i64 {i64_val} to i8*"));
            cast
        }
    }

    fn val_to_struct(&mut self, val: &str, val_ty: &str, struct_ty: &str) -> String {
        let alloca = self.fresh_tmp();
        self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
        if val_ty.starts_with('%') {
            let ptr = self.fresh_tmp();
            self.emitln(&format!("  {ptr} = bitcast {struct_ty}* {alloca} to i64*"));
            let i64_val = self.val_to_i64(val, val_ty);
            self.emitln(&format!("  store i64 {i64_val}, i64* {ptr}"));
        } else if val_ty == "i64" {
            let ptr = self.fresh_tmp();
            self.emitln(&format!("  {ptr} = bitcast {struct_ty}* {alloca} to i64*"));
            self.emitln(&format!("  store {val_ty} {val}, {val_ty}* {ptr}"));
        } else {
            let ptr = self.fresh_tmp();
            self.emitln(&format!("  {ptr} = bitcast {struct_ty}* {alloca} to {val_ty}*"));
            self.emitln(&format!("  store {val_ty} {val}, {val_ty}* {ptr}"));
        }
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
    loaded
}

    fn compile_invariant_call(&mut self, type_name: &str, struct_val_reg: &str) {
        let meta = match self.type_meta.get(type_name) {
            Some(m) => m,
            None => return,
        };
        if meta.invariants.is_empty() {
            return;
        }
        let fn_name = format!("{type_name}.invariant_check");
        let struct_ty = format!("%struct.{type_name}");
        self.emitln(&format!("  call void @{fn_name}({struct_ty} {struct_val_reg})"));
    }

    // ========================================================================
    // Derive Code Generation
    // ========================================================================

    fn compile_derive_impls(&mut self, items: &[TopDecl]) -> Result<(), String> {
        for item in items {
            self.compile_derive_for_item(item)?;
        }
        Ok(())
    }

    fn compile_derive_for_item(&mut self, item: &TopDecl) -> Result<(), String> {
        match item {
            TopDecl::Type(td) => {
                let bare_name = &td.name.name;
                let type_name = if let Some(ref module) = self.current_module {
                    let qualified = format!("{}.{}", module, bare_name);
                    if self.type_meta.contains_key(&qualified) { qualified } else { bare_name.clone() }
                } else {
                    bare_name.clone()
                };
                let field_names: Vec<String> = td.fields.iter().map(|f| f.name.name.clone()).collect();
                let struct_ty = self.llvm_type_for(&type_name);
                for derive in &td.derives {
                    match derive {
                        DeriveTrait::Eq => self.compile_eq_impl(&type_name, &struct_ty, &field_names, &td.fields)?,
                        DeriveTrait::Clone => self.compile_clone_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Display => self.compile_display_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Hash => self.compile_hash_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Ord => self.compile_ord_impl(&type_name, &struct_ty, &field_names, &td.fields)?,
                    }
                }
                if !td.invariants.is_empty() {
                    self.compile_invariant_check(&type_name)?;
                }
            }
            TopDecl::Enum(ed) => {
                if ed.derives.is_empty() {
                    return Ok(());
                }
                let bare_name = &ed.name.name;
                let type_name = if let Some(ref module) = self.current_module {
                    let qualified = format!("{}.{}", module, bare_name);
                    if self.type_meta.contains_key(&qualified) { qualified } else { bare_name.clone() }
                } else {
                    bare_name.clone()
                };
                if !self.types.contains_key(&type_name) {
                    self.types.insert(type_name.clone(), vec!["discriminant".to_string()]);
                    self.type_meta.insert(type_name.clone(), TypeMeta {
                        fields: vec![("discriminant".to_string(), "Int".to_string())],
                        derives: ed.derives.clone(),
                        invariants: Vec::new(),
                    });
                }
                let struct_ty = self.llvm_type_for(&type_name);
                let field_names: Vec<String> = vec!["discriminant".to_string()];
                for derive in &ed.derives {
                    match derive {
                        DeriveTrait::Eq => self.compile_eq_impl(&type_name, &struct_ty, &field_names, &[])?,
                        DeriveTrait::Clone => self.compile_clone_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Hash => self.compile_hash_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Ord => self.compile_ord_impl(&type_name, &struct_ty, &field_names, &[])?,
                        _ => {}
                    }
                }
            }
            TopDecl::Module(md) => {
                let saved_module = self.current_module.clone();
                self.current_module = Some(if let Some(ref prev) = saved_module {
                    format!("{}.{}", prev, md.name.name)
                } else {
                    md.name.name.clone()
                });
                for sub in &md.items {
                    self.compile_derive_for_item(sub)?;
                }
                self.current_module = saved_module;
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_clone_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.clone");
        if self.emitted_fns.contains(&fn_name) { return Ok(()); }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], struct_ty.to_string()));
        self.emitln(&format!("define {struct_ty} @{fn_name}({struct_ty} %self) {{"));
        let self_alloca = self.fresh_tmp();
        let result_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));
        self.emitln(&format!("  {result_alloca} = alloca {struct_ty}"));
        for (i, fname) in field_names.iter().enumerate() {
            let src_gep = self.fresh_tmp();
            let src_val = self.fresh_tmp();
            let dst_gep = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {src_gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {src_val} = load {field_llvm_ty}, {field_llvm_ty}* {src_gep}"));
            self.emitln(&format!("  {dst_gep} = getelementptr {struct_ty}, {struct_ty}* {result_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  store {field_llvm_ty} {src_val}, {field_llvm_ty}* {dst_gep}"));
            let _ = fname;
        }
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {result_alloca}"));
        self.emitln(&format!("  ret {struct_ty} {loaded}"));
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string()], struct_ty.to_string()));
        Ok(())
    }

    fn compile_eq_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String], _fields: &[FieldDecl]) -> Result<(), String> {
        let fn_name = format!("{type_name}.eq");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        self.emitln(&format!("define i64 @{fn_name}({struct_ty} %self, {struct_ty} %other) {{"));
        let self_alloca = self.fresh_tmp();
        let other_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));
        self.emitln(&format!("  {other_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %other, {struct_ty}* {other_alloca}"));
        let mut last_cmp = String::new();
        for (i, _fname) in field_names.iter().enumerate() {
            let self_gep = self.fresh_tmp();
            let self_val = self.fresh_tmp();
            let other_gep = self.fresh_tmp();
            let other_val = self.fresh_tmp();
            let cmp = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {self_gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {self_val} = load {field_llvm_ty}, {field_llvm_ty}* {self_gep}"));
            self.emitln(&format!("  {other_gep} = getelementptr {struct_ty}, {struct_ty}* {other_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {other_val} = load {field_llvm_ty}, {field_llvm_ty}* {other_gep}"));
            let is_float = field_llvm_ty == "double";
            if field_llvm_ty.starts_with("%struct.") {
                let field_type_name = &field_llvm_ty[8..];
                let eq_fn = format!("{field_type_name}.eq");
                self.emitln(&format!("  {cmp} = call i64 @{eq_fn}({field_llvm_ty} {self_val}, {field_llvm_ty} {other_val})"));
                if last_cmp.is_empty() { last_cmp = cmp; } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {cmp}"));
                    last_cmp = and_tmp;
                }
            } else if is_float {
                self.emitln(&format!("  {cmp} = fcmp oeq {field_llvm_ty} {self_val}, {other_val}"));
                let ze = self.fresh_tmp();
                self.emitln(&format!("  {ze} = zext i1 {cmp} to i64"));
                if last_cmp.is_empty() { last_cmp = ze; } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {ze}"));
                    last_cmp = and_tmp;
                }
            } else {
                self.emitln(&format!("  {cmp} = icmp eq {field_llvm_ty} {self_val}, {other_val}"));
                let ze = self.fresh_tmp();
                self.emitln(&format!("  {ze} = zext i1 {cmp} to i64"));
                if last_cmp.is_empty() { last_cmp = ze; } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {ze}"));
                    last_cmp = and_tmp;
                }
            }
        }
        if last_cmp.is_empty() { self.emitln("  ret i64 1"); }
        else { self.emitln(&format!("  ret i64 {last_cmp}")); }
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }
