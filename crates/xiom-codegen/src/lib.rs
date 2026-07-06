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
    /// LLVM type used when storing an arm body into `match_result_ptr`.
    /// When `None`, falls back to `current_return_type` (tail-position match).
    match_result_ty: Option<String>,
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
    /// Stack of active loop labels: (continue_label, break_label)
    loop_stack: Vec<(String, String)>,
    /// Set of function names already declared via `declare` (to avoid duplicates)
    already_declared: HashSet<String>,
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
            match_result_ty: None,
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
            loop_stack: Vec::new(),
            already_declared: HashSet::new(),
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

    /// True if the most recently emitted line in the current function body is a
    /// basic-block terminator. Used to decide whether a fallback terminator must
    /// be appended so every block is terminated and the IR stays valid.
    ///
    /// Returns `false` when the last meaningful line is a bare label (a freshly
    /// opened, still-empty block) or a non-terminator instruction.
    fn current_block_terminated(&self) -> bool {
        for line in self.output.lines().rev() {
            let t = line.trim();
            if t.is_empty() || t.starts_with(';') {
                continue;
            }
            // A bare label line ("entry:", "endif7:") opens a fresh block that has
            // no terminator yet.
            if t.ends_with(':') && !t.contains(' ') {
                return false;
            }
            return t == "unreachable"
                || t == "ret void"
                || t.starts_with("ret ")
                || t.starts_with("br ")
                || t.starts_with("switch ");
        }
        false
    }

    /// A valid default constant of `llvm_ty`, used only for fallback terminators
    /// on control-flow paths that fall off the end of a value-returning function.
    fn default_const_for(llvm_ty: &str) -> String {
        match llvm_ty {
            "i1" | "i8" | "i16" | "i32" | "i64" => "0".to_string(),
            "float" | "double" => "0.0".to_string(),
            _ if llvm_ty.ends_with('*') => "null".to_string(),
            _ => "zeroinitializer".to_string(), // aggregates / structs
        }
    }

    fn is_primitive_type_name(type_name: &str) -> bool {
        matches!(
            type_name,
            "Bool" | "Int" | "Int8" | "Int16" | "Int32" | "Int64"
                | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
                | "Float32" | "Float64" | "Char" | "Str"
        )
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

    /// Map an AST Type to its LLVM type string, handling pointer types (`*T` -> `<T>*`),
    /// ref types (`&T` -> `<T>*`), and named/builtin types.
    fn extern_type_to_llvm(&self, ty: &Type) -> String {
        match ty {
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                let inner_llvm = self.extern_type_to_llvm(inner);
                format!("{}*", inner_llvm)
            }
            Type::Named(id, _) => {
                self.llvm_type_for(&id.name).unwrap_or_else(|_| {
                    Self::xiom_to_llvm_type(&id.name).to_string()
                })
            }
            Type::Tuple(_) => "i64".to_string(),
            _ => "i64".to_string(),
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

    fn llvm_type_for(&self, type_name: &str) -> Result<String, String> {
        // Try current module's qualified name first (e.g., "types.Person")
        if let Some(ref module) = self.current_module {
            let qualified = format!("{}.{}", module, type_name);
            if self.types.contains_key(&qualified) || self.type_meta.contains_key(&qualified) {
                return Ok(format!("%struct.{qualified}"));
            }
        }
        // Try exact match
        if self.types.contains_key(type_name) || self.type_meta.contains_key(type_name) {
            return Ok(format!("%struct.{type_name}"));
        }
        // Search for any module-qualified variant ending with .type_name
        for (key, _) in &self.type_meta {
            if key.ends_with(&format!(".{type_name}")) {
                return Ok(format!("%struct.{key}"));
            }
        }
        // Check builtin types first (match known xiom type names, NOT the default i64 fallback)
        let builtin = Self::xiom_to_llvm_type(type_name);
        match type_name {
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
            | "Bool" | "Float32" | "Float64" | "Str" | "Char" | "()" => return Ok(builtin.to_string()),
            _ => {}
        }
        // If type_name is an enum variant (e.g., "Image"), find its parent enum type
        for (enum_key, variants) in &self.enum_variants {
            if variants.iter().any(|(v, _)| v == type_name) {
                return Ok(format!("%struct.{enum_key}"));
            }
        }
        match type_name {
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
            | "Bool" | "Float32" | "Float64" | "Str" | "Char" | "()" => Ok(builtin.to_string()),
            _ => Err(format!("unknown type '{}' — not a registered struct, enum, or builtin", type_name)),
        }
    }

    /// Resolve type name to LLVM type, with suffix-search fallback for module-qualified types.
    /// Use this when the exact type registration is uncertain (e.g., type aliases from other modules).
    fn llvm_type_for_fallback(&self, type_name: &str) -> String {
        match self.llvm_type_for(type_name) {
            Ok(t) => t,
            Err(_) => {
                // Try suffix search across type_meta
                let search = format!(".{}", type_name);
                for key in self.type_meta.keys() {
                    if key.ends_with(&search) {
                        return format!("%struct.{key}");
                    }
                }
                for key in self.types.keys() {
                    if key.ends_with(&search) {
                        return format!("%struct.{key}");
                    }
                }
                "i64".to_string()
            }
        }
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
                return self.llvm_type_for(ty_name).unwrap_or_else(|_| "i64".to_string());
            }
        }
        "i64".to_string()
    }

    /// Returns `true` if `name` is the name of a variant of the enum currently
    /// being matched on. Used to decide whether a bare `Pattern::Ident` should
    /// be compiled as a runtime discriminant check (rather than a variable
    /// binding / wildcard).
    fn ident_is_enum_variant(&self, scrutinee_type: &Option<String>, name: &str) -> bool {
        if let Some(type_name) = scrutinee_type {
            if let Some(variants) = self.enum_variants.get(type_name) {
                return variants.iter().any(|(v, _)| v.as_str() == name);
            }
        }
        false
    }

    /// Single source of truth for "does this match arm need a runtime check
    /// block?". Both the check-label build loop and the check-block emit loop
    /// in `Stmt::Match` codegen call this, so they can never disagree about
    /// which arms consume a `check_labels` slot. A previous inconsistency
    /// between those loops desynchronized `check_idx` from `check_labels.len()`
    /// and caused an out-of-bounds panic.
    fn pattern_needs_check(&self, pattern: &Pattern, scrutinee_type: &Option<String>) -> bool {
        match pattern {
            Pattern::Lit(Literal::Int(..)) | Pattern::Lit(Literal::Bool(..)) => true,
            Pattern::Variant(..) => true,
            Pattern::Ident(ident) => self.ident_is_enum_variant(scrutinee_type, &ident.name),
            _ => false,
        }
    }

    /// Emits a discriminant comparison for an enum-variant match arm.
    ///
    /// Loads field 0 (the discriminant) of the scrutinee struct and branches to
    /// `arm_label` when it equals the variant's index, otherwise to `next`.
    /// Falls back to a literal comparison on the raw scrutinee value when no
    /// struct/alloca information is available. Always emits a terminator so the
    /// block is well-formed.
    fn emit_variant_discriminant_check(
        &mut self,
        variant_name: &str,
        scrutinee_alloca_info: &Option<(String, String, String)>,
        val: &str,
        arm_label: &str,
        next: &str,
    ) {
        if let Some((alloca, type_name, struct_ty)) = scrutinee_alloca_info {
            let variant_idx = self
                .enum_variants
                .get(type_name)
                .and_then(|variants| variants.iter().position(|(vn, _)| vn.as_str() == variant_name))
                .unwrap_or(0);
            let disc_gep = self.fresh_tmp();
            let disc_val = self.fresh_tmp();
            self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
            self.emitln(&format!("  {disc_val} = load i64, i64* {disc_gep}"));
            let check = self.fresh_tmp();
            self.emitln(&format!("  {check} = icmp eq i64 {disc_val}, {variant_idx}"));
            self.emitln(&format!("  br i1 {check}, label %{arm_label}, label %{next}"));
        } else {
            // No struct type info: fall back to a literal comparison on `val`.
            let variant_idx = 0;
            let check = self.fresh_tmp();
            self.emitln(&format!("  {check} = icmp eq i64 {val}, {variant_idx}"));
            self.emitln(&format!("  br i1 {check}, label %{arm_label}, label %{next}"));
        }
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
                    let t = self.llvm_type_for(ty_name).unwrap_or_else(|_| "i64".to_string());
                    if t == struct_ref { format!("{t}*") } else { t }
                })
                .collect();
            self.emitln(&format!("%struct.{name} = type {{ {} }}", field_types.join(", ")));
        }
        if !self.types.is_empty() {
            self.emitln("");
        }

        // Pre-seed already_declared with all hardcoded names so user extern
        // blocks cannot duplicate them.
        self.already_declared = Self::hardcoded_declare_names();

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
        self.emitln("declare i64 @xiom_fn_emit_all()");
        self.emitln("");

        // Emit declares for user-defined extern "C" functions
        // (skips names already in self.already_declared, e.g. malloc)
        self.emit_extern_declares(&program.items);

        // Additive metadata emission: RTTI for the `reflect` stdlib module and a
        // contract table for the `contracts` stdlib module. This is a NEW,
        // self-contained step appended alongside the runtime `declare`s above.
        // It NEVER alters any existing lowering path — it only reads
        // already-registered type metadata (`self.type_meta` / `self.enum_variants`)
        // and the program AST (function contract clauses), then emits new globals
        // and `@xiom_*` function definitions. Emission is gated on the presence of
        // the matching `extern "C"` declarations (added only in reflect.xi /
        // contracts.xi), so every other program is byte-for-byte unaffected.
        self.emit_metadata_tables(program);

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

    /// Collect the set of all hardcoded `declare` names so user extern blocks
    /// never duplicate them. This set is pre-seeded before emitting user extern
    /// function declares.
    fn hardcoded_declare_names() -> HashSet<String> {
        let mut s = HashSet::new();
        s.insert("printf".to_string());
        s.insert("puts".to_string());
        s.insert("llvm.trap".to_string());
        s.insert("malloc".to_string());
        s.insert("realloc".to_string());
        s.insert("free".to_string());
        s.insert("llvm.memcpy.p0i8.p0i8.i64".to_string());
        s.insert("xiom_is_sorted".to_string());
        s.insert("xiom_all".to_string());
        s.insert("xiom_none".to_string());
        s.insert("xiom_contains".to_string());
        s.insert("xiom_read_file".to_string());
        s.insert("xiom_file_size".to_string());
        s.insert("xiom_free".to_string());
        s.insert("xiom_char_at".to_string());
        s.insert("xiom_str_len".to_string());
        s.insert("xiom_intern".to_string());
        s.insert("xiom_lookup".to_string());
        s.insert("xiom_ir_open".to_string());
        s.insert("xiom_ir_close".to_string());
        s.insert("xiom_ir_header".to_string());
        s.insert("xiom_ir_define".to_string());
        s.insert("xiom_ir_param".to_string());
        s.insert("xiom_ir_entry".to_string());
        s.insert("xiom_ir_alloca".to_string());
        s.insert("xiom_ir_store".to_string());
        s.insert("xiom_ir_load".to_string());
        s.insert("xiom_ir_binop".to_string());
        s.insert("xiom_ir_call".to_string());
        s.insert("xiom_ir_call_arg".to_string());
        s.insert("xiom_ir_call_lit".to_string());
        s.insert("xiom_ir_call_end".to_string());
        s.insert("xiom_ir_ret".to_string());
        s.insert("xiom_ir_ret_void".to_string());
        s.insert("xiom_ir_endfn".to_string());
        s.insert("xiom_ir_raw".to_string());
        s.insert("xiom_ir_emit_program".to_string());
        s.insert("xiom_ir_define_s".to_string());
        s.insert("xiom_ir_param_int".to_string());
        s.insert("xiom_ir_param_double".to_string());
        s.insert("xiom_ir_alloca_s".to_string());
        s.insert("xiom_ir_store_param".to_string());
        s.insert("xiom_ir_load_s".to_string());
        s.insert("xiom_ir_add".to_string());
        s.insert("xiom_ir_fmul".to_string());
        s.insert("xiom_ir_call_fn".to_string());
        s.insert("xiom_ir_call_arg_lit".to_string());
        s.insert("xiom_ir_ret_reg".to_string());
        s.insert("xiom_ir_ret_lit".to_string());
        s.insert("xiom_fn_table_init".to_string());
        s.insert("xiom_set_source".to_string());
        s.insert("xiom_fn_table_add".to_string());
        s.insert("xiom_fn_table_count".to_string());
        s.insert("xiom_fn_name_id".to_string());
        s.insert("xiom_fn_ret_type_id".to_string());
        s.insert("xiom_fn_param_count".to_string());
        s.insert("xiom_fn_body_start".to_string());
        s.insert("xiom_fn_body_end".to_string());
        s.insert("xiom_fn_emit_all".to_string());
        // strcmp is declared in emit_metadata_tables (conditional)
        s.insert("strcmp".to_string());
        s
    }

    /// Walk all top-level declarations (recursing into modules) and emit
    /// `declare` statements for every `extern "C"` function whose name is not
    /// already present in `self.already_declared`. Skips functions whose names
    /// are already in the hardcoded set or already declared by another extern block.
    fn emit_extern_declares(&mut self, items: &[TopDecl]) {
        for item in items {
            match item {
                TopDecl::Extern(eb) => {
                    for fd in &eb.functions {
                        let name = &fd.name.name;
                        if self.already_declared.contains(name) {
                            continue;
                        }
                        self.already_declared.insert(name.clone());
                        // Map return type
                        let ret_llvm = fd.return_type.as_ref()
                            .map(|t| self.extern_type_to_llvm(t))
                            .unwrap_or_else(|| "void".to_string());
                        // Map param types
                        let param_llvm: Vec<String> = fd.params.iter()
                            .map(|p| self.extern_type_to_llvm(&p.ty))
                            .collect();
                        // NOTE: Variadic extern functions (with `...` in the source)
                        // are parsed but the variadic marker is not stored in FnDecl.
                        // Therefore we cannot detect variadics from the AST alone.
                        // All extern declares are emitted without `...`.
                        // If you add variadic detection, change the last arg to `...`.
                        let params_str = if param_llvm.is_empty() {
                            "".to_string()
                        } else {
                            param_llvm.join(", ")
                        };
                        self.emitln(&format!("declare {ret_llvm} @{name}({params_str})"));
                    }
                }
                TopDecl::Module(md) => {
                    self.emit_extern_declares(&md.items);
                }
                _ => {}
            }
        }
    }

    // ========================================================================
    // Additive metadata tables (RTTI + contracts)
    //
    // Everything below is a NEW emission surface for the `reflect` and
    // `contracts` stdlib modules. It is strictly ADDITIVE:
    //   * it only READS already-registered state (`type_meta`, `enum_variants`)
    //     and the program AST,
    //   * it only WRITES new globals, new `@xiom_*` function definitions, and
    //     new entries into `self.functions` (never overwriting existing keys),
    //   * it is gated so it emits nothing unless the program actually declares
    //     the corresponding `extern "C"` accessors (only reflect.xi /
    //     contracts.xi do), keeping all other programs identical.
    // ========================================================================

    /// Returns true if any `extern "C"` block in `items` (recursively through
    /// modules) declares a function named `name`.
    fn program_declares_extern(items: &[TopDecl], name: &str) -> bool {
        for item in items {
            match item {
                TopDecl::Extern(eb) => {
                    if eb.functions.iter().any(|f| f.name.name == name) {
                        return true;
                    }
                }
                TopDecl::Module(md) => {
                    if Self::program_declares_extern(&md.items, name) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Recursively collect `(name, requires_count, ensures_count)` for every
    /// function that carries at least one pre/postcondition, in source order.
    fn collect_contract_fns(items: &[TopDecl], out: &mut Vec<(String, usize, usize)>) {
        for item in items {
            match item {
                TopDecl::Fn(fd) => {
                    if !fd.contracts.is_empty() {
                        let mut pre = 0usize;
                        let mut post = 0usize;
                        for c in &fd.contracts {
                            match c {
                                ContractClause::Requires(_, _) => pre += 1,
                                ContractClause::Ensures(_, _) => post += 1,
                            }
                        }
                        let name = if let Some(recv) = &fd.receiver {
                            format!("{}.{}", recv.name, fd.name.name)
                        } else {
                            fd.name.name.clone()
                        };
                        out.push((name, pre, post));
                    }
                }
                TopDecl::Module(md) => {
                    Self::collect_contract_fns(&md.items, out);
                }
                _ => {}
            }
        }
    }

    /// Escape a Rust string for embedding in an LLVM `c"..."` byte string,
    /// matching the convention already used for contract/display strings.
    fn escape_ir_string(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\22")
    }

    /// Emit the additive RTTI + contract metadata tables and their fixed-ABI
    /// accessor functions. See the section header above for the additivity
    /// guarantees. Does nothing unless the program declares the accessors.
    fn emit_metadata_tables(&mut self, program: &Program) {
        let want_reflect = Self::program_declares_extern(&program.items, "xiom_type_count");
        let want_contracts = Self::program_declares_extern(&program.items, "xiom_contract_fn_count");
        if !want_reflect && !want_contracts {
            return;
        }
        self.emitln("; ---- XIOM additive metadata (RTTI / contracts) ----");
        // Shared C-runtime dependency for name lookups (identical duplicate
        // declares are legal in LLVM; this is only emitted in gated programs).
        self.emitln("declare i32 @strcmp(i8*, i8*)");
        if want_reflect {
            self.emit_rtti_table();
        }
        if want_contracts {
            self.emit_contract_table(program);
        }
        self.emitln("");
    }

    /// PART A — read-only RTTI table + accessors for `reflect`.
    fn emit_rtti_table(&mut self) {
        // Collect user types in a deterministic (sorted) order, excluding the
        // compiler's builtin/synthetic types. The type id is the index here.
        let mut names: Vec<String> = self
            .type_meta
            .keys()
            .filter(|n| {
                let n = n.as_str();
                n != "Option" && n != "Result" && n != "Vec" && n != "Tuple" && !n.starts_with("Tuple_")
            })
            .cloned()
            .collect();
        names.sort();
        let n = names.len();
        // Field counts: 0 for enums, otherwise the number of fields.
        let field_counts: Vec<usize> = names
            .iter()
            .map(|name| {
                if self.enum_variants.contains_key(name) {
                    0
                } else {
                    self.type_meta.get(name).map(|m| m.fields.len()).unwrap_or(0)
                }
            })
            .collect();

        // Per-type name string constants.
        for (i, name) in names.iter().enumerate() {
            let escaped = Self::escape_ir_string(name);
            self.emitln(&format!(
                "@.xiom_rtti_name_{i} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"",
                len = name.len() + 1
            ));
        }
        // Fallback name for out-of-range ids.
        self.emitln("@.xiom_rtti_unknown = private unnamed_addr constant [8 x i8] c\"unknown\\00\"");

        // Parallel arrays of name pointers and field counts.
        if n == 0 {
            self.emitln("@.xiom_rtti_names = private unnamed_addr constant [0 x i8*] zeroinitializer");
            self.emitln("@.xiom_rtti_field_counts = private unnamed_addr constant [0 x i64] zeroinitializer");
        } else {
            let name_elems: Vec<String> = names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    format!(
                        "i8* getelementptr inbounds ([{len} x i8], [{len} x i8]* @.xiom_rtti_name_{i}, i64 0, i64 0)",
                        len = name.len() + 1
                    )
                })
                .collect();
            self.emitln(&format!(
                "@.xiom_rtti_names = private unnamed_addr constant [{n} x i8*] [{}]",
                name_elems.join(", ")
            ));
            let fc_elems: Vec<String> = field_counts.iter().map(|c| format!("i64 {c}")).collect();
            self.emitln(&format!(
                "@.xiom_rtti_field_counts = private unnamed_addr constant [{n} x i64] [{}]",
                fc_elems.join(", ")
            ));
        }

        // i64 @xiom_type_count()
        self.functions.insert("xiom_type_count".to_string(), (vec![], "i64".to_string()));
        self.emitln("define i64 @xiom_type_count() {");
        self.emitln("entry:");
        self.emitln(&format!("  ret i64 {n}"));
        self.emitln("}\n");

        // i8* @xiom_type_name(i64 %id) — name or "unknown" if out of range.
        self.functions
            .insert("xiom_type_name".to_string(), (vec!["i64".to_string()], "i8*".to_string()));
        self.emitln("define i8* @xiom_type_name(i64 %id) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %id, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %id, {n}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  %u = getelementptr [8 x i8], [8 x i8]* @.xiom_rtti_unknown, i64 0, i64 0");
        self.emitln("  ret i8* %u");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{n} x i8*], [{n} x i8*]* @.xiom_rtti_names, i64 0, i64 %id"
        ));
        self.emitln("  %v = load i8*, i8** %p");
        self.emitln("  ret i8* %v");
        self.emitln("}\n");

        // i64 @xiom_type_field_count(i64 %id) — 0 if out of range.
        self.functions
            .insert("xiom_type_field_count".to_string(), (vec!["i64".to_string()], "i64".to_string()));
        self.emitln("define i64 @xiom_type_field_count(i64 %id) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %id, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %id, {n}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  ret i64 0");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{n} x i64], [{n} x i64]* @.xiom_rtti_field_counts, i64 0, i64 %id"
        ));
        self.emitln("  %v = load i64, i64* %p");
        self.emitln("  ret i64 %v");
        self.emitln("}\n");

        // i64 @xiom_type_id_by_name(i8* %name) — linear search, -1 if absent.
        self.functions
            .insert("xiom_type_id_by_name".to_string(), (vec!["i8*".to_string()], "i64".to_string()));
        self.emitln("define i64 @xiom_type_id_by_name(i8* %name) {");
        self.emitln("entry:");
        self.emitln("  br label %loop");
        self.emitln("loop:");
        self.emitln("  %i = phi i64 [ 0, %entry ], [ %inext, %cont ]");
        self.emitln(&format!("  %done = icmp sge i64 %i, {n}"));
        self.emitln("  br i1 %done, label %notfound, label %body");
        self.emitln("body:");
        self.emitln(&format!(
            "  %np = getelementptr [{n} x i8*], [{n} x i8*]* @.xiom_rtti_names, i64 0, i64 %i"
        ));
        self.emitln("  %ns = load i8*, i8** %np");
        self.emitln("  %c = call i32 @strcmp(i8* %name, i8* %ns)");
        self.emitln("  %eq = icmp eq i32 %c, 0");
        self.emitln("  br i1 %eq, label %found, label %cont");
        self.emitln("cont:");
        self.emitln("  %inext = add i64 %i, 1");
        self.emitln("  br label %loop");
        self.emitln("found:");
        self.emitln("  ret i64 %i");
        self.emitln("notfound:");
        self.emitln("  ret i64 -1");
        self.emitln("}\n");
    }

    /// PART B — read-only contract metadata table + accessors for `contracts`.
    fn emit_contract_table(&mut self, program: &Program) {
        let mut entries: Vec<(String, usize, usize)> = Vec::new();
        Self::collect_contract_fns(&program.items, &mut entries);
        let m = entries.len();

        for (i, (name, _, _)) in entries.iter().enumerate() {
            let escaped = Self::escape_ir_string(name);
            self.emitln(&format!(
                "@.xiom_contract_name_{i} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"",
                len = name.len() + 1
            ));
        }
        self.emitln("@.xiom_contract_unknown = private unnamed_addr constant [8 x i8] c\"unknown\\00\"");

        if m == 0 {
            self.emitln("@.xiom_contract_names = private unnamed_addr constant [0 x i8*] zeroinitializer");
            self.emitln("@.xiom_contract_pre = private unnamed_addr constant [0 x i64] zeroinitializer");
            self.emitln("@.xiom_contract_post = private unnamed_addr constant [0 x i64] zeroinitializer");
        } else {
            let name_elems: Vec<String> = entries
                .iter()
                .enumerate()
                .map(|(i, (name, _, _))| {
                    format!(
                        "i8* getelementptr inbounds ([{len} x i8], [{len} x i8]* @.xiom_contract_name_{i}, i64 0, i64 0)",
                        len = name.len() + 1
                    )
                })
                .collect();
            self.emitln(&format!(
                "@.xiom_contract_names = private unnamed_addr constant [{m} x i8*] [{}]",
                name_elems.join(", ")
            ));
            let pre_elems: Vec<String> = entries.iter().map(|(_, pre, _)| format!("i64 {pre}")).collect();
            self.emitln(&format!(
                "@.xiom_contract_pre = private unnamed_addr constant [{m} x i64] [{}]",
                pre_elems.join(", ")
            ));
            let post_elems: Vec<String> = entries.iter().map(|(_, _, post)| format!("i64 {post}")).collect();
            self.emitln(&format!(
                "@.xiom_contract_post = private unnamed_addr constant [{m} x i64] [{}]",
                post_elems.join(", ")
            ));
        }

        // i64 @xiom_contract_fn_count()
        self.functions
            .insert("xiom_contract_fn_count".to_string(), (vec![], "i64".to_string()));
        self.emitln("define i64 @xiom_contract_fn_count() {");
        self.emitln("entry:");
        self.emitln(&format!("  ret i64 {m}"));
        self.emitln("}\n");

        // i8* @xiom_contract_fn_name(i64 %idx)
        self.functions
            .insert("xiom_contract_fn_name".to_string(), (vec!["i64".to_string()], "i8*".to_string()));
        self.emitln("define i8* @xiom_contract_fn_name(i64 %idx) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %idx, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %idx, {m}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  %u = getelementptr [8 x i8], [8 x i8]* @.xiom_contract_unknown, i64 0, i64 0");
        self.emitln("  ret i8* %u");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{m} x i8*], [{m} x i8*]* @.xiom_contract_names, i64 0, i64 %idx"
        ));
        self.emitln("  %v = load i8*, i8** %p");
        self.emitln("  ret i8* %v");
        self.emitln("}\n");

        // i64 @xiom_contract_pre_count(i64 %idx)
        self.functions
            .insert("xiom_contract_pre_count".to_string(), (vec!["i64".to_string()], "i64".to_string()));
        self.emitln("define i64 @xiom_contract_pre_count(i64 %idx) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %idx, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %idx, {m}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  ret i64 0");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{m} x i64], [{m} x i64]* @.xiom_contract_pre, i64 0, i64 %idx"
        ));
        self.emitln("  %v = load i64, i64* %p");
        self.emitln("  ret i64 %v");
        self.emitln("}\n");

        // i64 @xiom_contract_post_count(i64 %idx)
        self.functions
            .insert("xiom_contract_post_count".to_string(), (vec!["i64".to_string()], "i64".to_string()));
        self.emitln("define i64 @xiom_contract_post_count(i64 %idx) {");
        self.emitln("entry:");
        self.emitln("  %lo = icmp slt i64 %idx, 0");
        self.emitln(&format!("  %hi = icmp sge i64 %idx, {m}"));
        self.emitln("  %oob = or i1 %lo, %hi");
        self.emitln("  br i1 %oob, label %oob_bb, label %ok_bb");
        self.emitln("oob_bb:");
        self.emitln("  ret i64 0");
        self.emitln("ok_bb:");
        self.emitln(&format!(
            "  %p = getelementptr [{m} x i64], [{m} x i64]* @.xiom_contract_post, i64 0, i64 %idx"
        ));
        self.emitln("  %v = load i64, i64* %p");
        self.emitln("  ret i64 %v");
        self.emitln("}\n");
    }

     fn register_type_layout(&mut self, item: &TopDecl) {
        self.register_type_layout_impl(item, "");
    }

    fn register_type_layout_impl(&mut self, item: &TopDecl, prefix: &str) {
        if let TopDecl::Type(td) = item {
            if td.fields.is_empty() && td.alias.is_some() { return; }
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
                param_types.push(self.llvm_type_for(&recv.name).unwrap_or_else(|_| "i64".to_string()));
            }
            let explicit_params: Vec<String> = fd.params.iter()
                .map(|p| self.llvm_type_for(&Self::type_from_ast(&p.ty)).unwrap_or_else(|_| "i64".to_string()))
                .collect();
            param_types.extend(explicit_params);
            let ret_type = fd.return_type.as_ref()
                .map(|t| self.llvm_type_for(&Self::type_from_ast(t)).unwrap_or_else(|_| "i64".to_string()))
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
            TopDecl::Interface(_) | TopDecl::Enum(_) | TopDecl::Const(_) | TopDecl::Type(_) | TopDecl::Use(_) | TopDecl::Extern(_) => Ok(()),
        }
    }

    fn compile_fn(&mut self, fd: &FnDecl) -> Result<(), String> {
        self.push_scope();
        self.block_counter = 0;
        self.tmp_counter = 0;
        self.fn_ptr_return_types.clear();

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

        // For methods, prepend the self struct parameter
        let self_llvm_ty = fd.receiver.as_ref().map(|r| {
            self.llvm_type_for(&r.name).unwrap_or_else(|_| "i64".to_string())
        });
        let self_offset: usize = if self_llvm_ty.is_some() { 1 } else { 0 };

        let mut params_str: Vec<String> = Vec::new();
        if let Some(ref st) = self_llvm_ty {
            params_str.push(format!("{st} %param_self"));
        }
        let explicit_params: Vec<String> = fd.params.iter()
            .enumerate()
            .map(|(i, p)| {
                let llvm_ty = self.llvm_type_for(&Self::type_from_ast(&p.ty)).unwrap_or_else(|_| "i64".to_string());
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
            let llvm_ty = self.llvm_type_for_fallback(&Self::type_from_ast(&param.ty));
            let alloca = self.fresh_tmp();
            let param_idx = i + self_offset;
            self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
            self.emitln(&format!("  store {llvm_ty} %param{param_idx}, {llvm_ty}* {alloca}"));
            self.add_local(&param.name.name, alloca, &llvm_ty);
            // Track function pointer return types for function pointer parameters
            if let Type::Fn(_, ret) = &param.ty {
                let ret_ty_name = Self::type_from_ast(ret);
                let ret_llvm = self.llvm_type_for_fallback(&ret_ty_name);
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
        } else if !self.current_block_terminated() {
            // A4 fix: the function declares a return type but control reached the
            // end of the body without a terminator — the body ends in a loop, an
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
            Expr::Struct(ident, _, _, _) => Some(ident.name.clone()),
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
                // Type aliases have no fields — nothing to derive or invariant-check
                if td.fields.is_empty() && td.alias.is_some() {
                    return Ok(());
                }
                let bare_name = &td.name.name;
                // Resolve to qualified name using module context
                let type_name = if let Some(ref module) = self.current_module {
                    let qualified = format!("{}.{}", module, bare_name);
                    if self.type_meta.contains_key(&qualified) { qualified } else { bare_name.clone() }
                } else {
                    bare_name.clone()
                };
                let field_names: Vec<String> = td.fields.iter().map(|f| f.name.name.clone()).collect();
                let struct_ty = self.llvm_type_for(&type_name)?;

                for derive in &td.derives {
                    match derive {
                        DeriveTrait::Eq => self.compile_eq_impl(&type_name, &struct_ty, &field_names, &td.fields)?,
                        DeriveTrait::Clone => self.compile_clone_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Display => self.compile_display_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Hash => self.compile_hash_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Ord => self.compile_ord_impl(&type_name, &struct_ty, &field_names, &td.fields)?,
                    }
                }

                // Generate invariant check function if needed (even without derives)
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
                let struct_ty = self.llvm_type_for(&type_name)?;
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
                if last_cmp.is_empty() {
                    last_cmp = cmp;
                } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {cmp}"));
                    last_cmp = and_tmp;
                }
            } else if is_float {
                self.emitln(&format!("  {cmp} = fcmp oeq {field_llvm_ty} {self_val}, {other_val}"));
                let ze = self.fresh_tmp();
                self.emitln(&format!("  {ze} = zext i1 {cmp} to i64"));
                if last_cmp.is_empty() {
                    last_cmp = ze;
                } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {ze}"));
                    last_cmp = and_tmp;
                }
            } else {
                self.emitln(&format!("  {cmp} = icmp eq {field_llvm_ty} {self_val}, {other_val}"));
                let ze = self.fresh_tmp();
                self.emitln(&format!("  {ze} = zext i1 {cmp} to i64"));
                if last_cmp.is_empty() {
                    last_cmp = ze;
                } else {
                    let and_tmp = self.fresh_tmp();
                    self.emitln(&format!("  {and_tmp} = and i64 {last_cmp}, {ze}"));
                    last_cmp = and_tmp;
                }
            }
        }
        if last_cmp.is_empty() {
            self.emitln("  ret i64 1");
        } else {
            self.emitln(&format!("  ret i64 {last_cmp}"));
        }
        self.emitln("}\n");
        // Register the generated function
        self.functions.insert(fn_name, (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    fn compile_clone_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.clone");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
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

    fn compile_display_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.to_str");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], "i8*".to_string()));
        self.emitln(&format!("define i8* @{fn_name}({struct_ty} %self) {{"));
        let self_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));

        // Build format string: "TypeName{ field1: ..., field2: ... }"
        let mut display_parts: Vec<String> = vec![format!("{type_name}{{")];
        for fname in field_names {
            display_parts.push(format!("{fname}: "));
            display_parts.push("%lld ".to_string());
        }
        display_parts.push("}".to_string());
        let fmt_str = display_parts.concat();
        let fmt_label = format!("@.fmt_{fn_name}");
        let escaped = fmt_str.replace('\\', "\\\\").replace('"', "\\22")
            .replace('\n', "\\0A").replace('\t', "\\09");
        self.strings.push(format!(
            "{fmt_label} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"",
            len = fmt_str.len() + 1
        ));

        // Allocate output buffer (256 bytes fixed)
        let buf = self.fresh_tmp();
        self.emitln(&format!("  {buf} = alloca i8, i64 256"));

        // Build sprintf call
        let fmt_ptr = self.fresh_tmp();
        self.emitln(&format!("  {fmt_ptr} = getelementptr [{len} x i8], [{len} x i8]* {fmt_label}, i64 0, i64 0",
            len = fmt_str.len() + 1));

        let mut args = vec![format!("i8* {fmt_ptr}")];
        for (i, _) in field_names.iter().enumerate() {
            let gep = self.fresh_tmp();
            let val = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {val} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
            args.push(format!("{field_llvm_ty} {val}"));
        }
        let buf_ptr = self.fresh_tmp();
        self.emitln(&format!("  {buf_ptr} = getelementptr i8, i8* {buf}, i64 0"));
        self.emitln(&format!("  call i32 (i8*, ...) @printf(i8* {buf_ptr})"));

        // For Phase 1, just return a pointer to the buf (simplified)
        self.emitln(&format!("  ret i8* {buf_ptr}"));
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string()], "i8*".to_string()));
        Ok(())
    }

    fn compile_hash_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.hash");
        if self.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.emitted_fns.insert(fn_name.clone());
        self.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], "i64".to_string()));
        self.emitln(&format!("define i64 @{fn_name}({struct_ty} %self) {{"));
        let self_alloca = self.fresh_tmp();
        self.emitln(&format!("  {self_alloca} = alloca {struct_ty}"));
        self.emitln(&format!("  store {struct_ty} %self, {struct_ty}* {self_alloca}"));

        // Seed: 5381
        self.emitln("  %hash = alloca i64");
        self.emitln("  store i64 5381, i64* %hash");

        for (i, _) in field_names.iter().enumerate() {
            let gep = self.fresh_tmp();
            let val = self.fresh_tmp();
            let loaded_hash = self.fresh_tmp();
            let mul_tmp = self.fresh_tmp();
            let add_tmp = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {val} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
            self.emitln(&format!("  {loaded_hash} = load i64, i64* %hash"));
            self.emitln(&format!("  {mul_tmp} = mul i64 {loaded_hash}, 33"));
            self.emitln(&format!("  {add_tmp} = add i64 {mul_tmp}, {val}"));
            self.emitln(&format!("  store i64 {add_tmp}, i64* %hash"));
        }
        let final_hash = self.fresh_tmp();
        self.emitln(&format!("  {final_hash} = load i64, i64* %hash"));
        self.emitln(&format!("  ret i64 {final_hash}"));
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    fn compile_ord_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String], _fields: &[FieldDecl]) -> Result<(), String> {
        let fn_name = format!("{type_name}.compare");
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

        for i in 0..field_names.len() {
            let self_gep = self.fresh_tmp();
            let self_val = self.fresh_tmp();
            let other_gep = self.fresh_tmp();
            let other_val = self.fresh_tmp();
            let cmp_eq = self.fresh_tmp();
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            let is_float = field_llvm_ty == "double";
            self.emitln(&format!("  {self_gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {self_val} = load {field_llvm_ty}, {field_llvm_ty}* {self_gep}"));
            self.emitln(&format!("  {other_gep} = getelementptr {struct_ty}, {struct_ty}* {other_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {other_val} = load {field_llvm_ty}, {field_llvm_ty}* {other_gep}"));

            let next_field = self.fresh_block("next_field");
            let ret_block = self.fresh_block("ord_ret");
            if field_llvm_ty.starts_with("%struct.") {
                let field_type_name = &field_llvm_ty[8..];
                let compare_fn = format!("{field_type_name}.compare");
                let cmp_result = self.fresh_tmp();
                self.emitln(&format!("  {cmp_result} = call i64 @{compare_fn}({field_llvm_ty} {self_val}, {field_llvm_ty} {other_val})"));
                self.emitln(&format!("  {cmp_eq} = icmp eq i64 {cmp_result}, 0"));
                self.emitln(&format!("  br i1 {cmp_eq}, label %{next_field}, label %{ret_block}"));
                self.emitln(&format!("\n{ret_block}:"));
                self.emitln(&format!("  ret i64 {cmp_result}"));
            } else if is_float {
                self.emitln(&format!("  {cmp_eq} = fcmp oeq {field_llvm_ty} {self_val}, {other_val}"));
                self.emitln(&format!("  br i1 {cmp_eq}, label %{next_field}, label %{ret_block}"));
                self.emitln(&format!("\n{ret_block}:"));
                let fcmp = self.fresh_tmp();
                self.emitln(&format!("  {fcmp} = fcmp olt {field_llvm_ty} {self_val}, {other_val}"));
                let result = self.fresh_tmp();
                self.emitln(&format!("  {result} = select i1 {fcmp}, i64 -1, i64 1"));
                self.emitln(&format!("  ret i64 {result}"));
            } else {
                self.emitln(&format!("  {cmp_eq} = icmp eq {field_llvm_ty} {self_val}, {other_val}"));
                self.emitln(&format!("  br i1 {cmp_eq}, label %{next_field}, label %{ret_block}"));
                self.emitln(&format!("\n{ret_block}:"));
                let cmp_lt = self.fresh_tmp();
                self.emitln(&format!("  {cmp_lt} = icmp slt {field_llvm_ty} {self_val}, {other_val}"));
                let result = self.fresh_tmp();
                self.emitln(&format!("  {result} = select i1 {cmp_lt}, i64 -1, i64 1"));
                self.emitln(&format!("  ret i64 {result}"));
            }
            self.emitln(&format!("\n{next_field}:"));
        }
        self.emitln("  ret i64 0");
        self.emitln("}\n");
        self.functions.insert(fn_name, (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    // ========================================================================
    // Generics: Monomorphisation
    // ========================================================================

    fn monomorphised_fn_name(&self, base_name: &str, concrete_types: &[String]) -> String {
        if concrete_types.is_empty() {
            base_name.to_string()
        } else {
            format!("{}_{}", base_name, concrete_types.join("_"))
        }
    }

    /// After all non-generic functions have been compiled, emit specialized
    /// versions for each tracked generic instantiation.
    /// Uses a worklist pattern: monomorphising one function may trigger new
    /// instantiations (generic chains), which are processed in subsequent passes.
    fn compile_generic_monomorphisations(&mut self) -> Result<(), String> {
        let mut iteration: u32 = 0;
        const MAX_GENERIC_ITERATIONS: u32 = 65536;
        loop {
            iteration += 1;
            if iteration > MAX_GENERIC_ITERATIONS {
                return Err(format!(
                    "generic monomorphisation exceeded {} iterations — possible infinite recursion in generic definitions",
                    MAX_GENERIC_ITERATIONS
                ));
            }
            let instantiations = std::mem::take(&mut self.generic_instantiations);
            if instantiations.is_empty() {
                break;
            }
            for (base_name, concrete_types) in &instantiations {
            // Find the generic function decl
            let fd = match self.generic_fn_decls.iter().find(|f| self.fn_key(f) == *base_name) {
                Some(f) => f.clone(),
                None => continue,
            };
            // Check interface bounds for each generic parameter
            for (gp, concrete_type) in fd.generics.iter().zip(concrete_types.iter()) {
                for bound in &gp.bounds {
                    if let Some(methods) = self.interfaces.get(&bound.name) {
                        for (method_name, _) in methods {
                            let method_key = format!("{}.{}", concrete_type, method_name);
                            // Primitive types implicitly implement the builtin interface
                            // methods (Ord.compare, Eq.eq/ne, Hash.hash, Clone.clone,
                            // comparison ops) via inline codegen — see the primitive
                            // fast-path in compile_expr's method dispatch.
                            let is_builtin_method = matches!(
                                method_name.as_str(),
                                "compare" | "eq" | "ne" | "lt" | "gt" | "le" | "ge" | "hash" | "clone"
                            );
                            if is_builtin_method && Self::is_primitive_type_name(concrete_type) {
                                continue;
                            }
                            if !self.functions.contains_key(&method_key) {
                                return Err(format!(
                                    "type '{}' does not implement '{}': missing method '{}'",
                                    concrete_type, bound.name, method_name
                                ));
                            }
                        }
                    }
                }
            }
            let specialized_name = self.monomorphised_fn_name(base_name, concrete_types);
            // Build type substitution map: generic param name -> concrete type name
            let mut type_map: HashMap<String, String> = HashMap::new();
            for (gp, ct) in fd.generics.iter().zip(concrete_types.iter()) {
                type_map.insert(gp.name.name.clone(), ct.clone());
            }
            // Register concrete tuple types for this monomorphisation
            if let Some(ref ret_ty) = fd.return_type {
                self.ensure_concrete_tuple_type_registered(ret_ty, &type_map);
            }
            for p in &fd.params {
                self.ensure_concrete_tuple_type_registered(&p.ty, &type_map);
            }
            // Register the specialized function signature
            let struct_types: HashSet<String> = self.types.keys().cloned().collect();
            let subst_type = |t: &Type| -> String {
                match t {
                    Type::Named(id, _) => {
                        let raw = if let Some(ct) = type_map.get(&id.name) {
                            ct.clone()
                        } else {
                            id.name.clone()
                        };
                        if struct_types.contains(&raw) {
                            format!("%struct.{raw}")
                        } else {
                            Self::xiom_to_llvm_type(&raw).to_string()
                        }
                    }
                    Type::Tuple(elems) => {
                        let parts: Vec<String> = elems.iter().map(|e| {
                            // Resolve element types: substitute generics, then resolve to LLVM name
                            let xiom_name = match e {
                                Type::Named(id, _) => type_map.get(&id.name).cloned().unwrap_or_else(|| id.name.clone()),
                                _ => Self::type_from_ast(e),
                            };
                            // Strip %struct. prefix if present (element might already be a struct type)
                            if let Some(stripped) = xiom_name.strip_prefix("%struct.") {
                                stripped.to_string()
                            } else {
                                xiom_name
                            }
                        }).collect();
                        format!("%struct.Tuple_{}", parts.join("_"))
                    }
                    _ => Self::xiom_to_llvm_type(&Self::type_from_ast(t)).to_string(),
                }
            };
            let specialized_ret_type = fd.return_type.as_ref()
                .map(|t| subst_type(t))
                .unwrap_or_else(|| "void".to_string());
            let mut specialized_param_types: Vec<String> = Vec::new();
            // Include self/receiver parameter for methods
            let self_llvm_ty = if let Some(ref r) = fd.receiver {
                Some(self.llvm_type_for(&r.name).unwrap_or_else(|_| {
                    // Fallback: try via suffix search across all registered type_meta keys
                    let search = format!(".{}", r.name);
                    for key in self.type_meta.keys() {
                        if key.ends_with(&search) {
                            return format!("%struct.{key}");
                        }
                    }
                    "i64".to_string()
                }))
            } else {
                None
            };
            if let Some(ref st) = self_llvm_ty {
                specialized_param_types.push(st.clone());
            }
            let explicit_param_types: Vec<String> = fd.params.iter()
                .map(|p| subst_type(&p.ty))
                .collect();
            specialized_param_types.extend(explicit_param_types);
            self.functions.insert(specialized_name.clone(), (specialized_param_types.clone(), specialized_ret_type.clone()));

            // Emit the specialized function
            self.push_scope();
            self.block_counter = 0;
            self.tmp_counter = 0;

            self.current_return_type = specialized_ret_type.clone();
            self.current_param_llvm_types = specialized_param_types.clone();
            self.current_fn = Some(specialized_name.clone());

            let self_offset: usize = if self_llvm_ty.is_some() { 1 } else { 0 };
            let mut params_str: Vec<String> = Vec::new();
            if let Some(ref st) = self_llvm_ty {
                params_str.push(format!("{st} %param_self"));
            }
            let explicit_params_str: Vec<String> = fd.params.iter()
                .enumerate()
                .map(|(i, p)| {
                    let llvm_ty = subst_type(&p.ty);
                    format!("{llvm_ty} %param{}", i + self_offset)
                })
                .collect();
            params_str.extend(explicit_params_str);

            self.emitln(&format!("define {specialized_ret_type} @{specialized_name}({}) {{", params_str.join(", ")));
            let entry_block = self.fresh_block("entry");
            self.emitln(&format!("{entry_block}:"));

            // Allocate parameters as locals
            // Allocate self parameter first (for methods)
            if let (Some(st), Some(recv)) = (&self_llvm_ty, &fd.receiver) {
                let self_alloca = self.fresh_tmp();
                self.emitln(&format!("  {self_alloca} = alloca {st}"));
                self.emitln(&format!("  store {st} %param_self, {st}* {self_alloca}"));
                self.add_local("self", self_alloca.clone(), st);
                // Register each struct field as a local (bare name access like `items`)
                let recv_type_name = &recv.name;
                let names_opt = self.types.get(recv_type_name).cloned()
                    .or_else(|| {
                        // Try module-qualified variant
                        if let Some(ref module) = self.current_module {
                            let qualified = format!("{}.{}", module, recv_type_name);
                            self.types.get(&qualified).cloned()
                        } else {
                            // Search all keys
                            self.types.iter()
                                .find(|(k, _)| k.ends_with(&format!(".{recv_type_name}")))
                                .map(|(_, v)| v.clone())
                        }
                    });
                if let Some(names) = names_opt {
                    // Use the found type key for field_llvm_type lookups
                    let type_key = self.types.get(recv_type_name).map(|_| recv_type_name.clone())
                        .or_else(|| {
                            if let Some(ref module) = self.current_module {
                                let q = format!("{}.{}", module, recv_type_name);
                                if self.types.contains_key(&q) { Some(q) } else { None }
                            } else { None }
                        })
                        .or_else(|| {
                            self.types.keys().find(|k| k.ends_with(&format!(".{recv_type_name}"))).cloned()
                        })
                        .unwrap_or_else(|| recv_type_name.clone());
                    for (idx, field_name) in names.iter().enumerate() {
                        let field_llvm_ty = self.field_llvm_type(&type_key, idx);
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {st}, {st}* {self_alloca}, i32 0, i32 {idx}"));
                        self.add_local(field_name, gep, &field_llvm_ty);
                    }
                }
            }
            for (i, param) in fd.params.iter().enumerate() {
                let llvm_ty = subst_type(&param.ty);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                let param_idx = i + self_offset;
                self.emitln(&format!("  store {llvm_ty} %param{param_idx}, {llvm_ty}* {alloca}"));
                self.add_local(&param.name.name, alloca, &llvm_ty);
                // Track params whose original type is a generic parameter being monomorphised
                let xiom_ty = Self::type_from_ast(&param.ty);
                if type_map.contains_key(&xiom_ty) {
                    if let Some(concrete) = type_map.get(&xiom_ty) {
                        self.param_concrete_types.insert(param.name.name.clone(), concrete.clone());
                    }
                }
            }

            // Set type substitution map for method dispatch in body
            self.current_type_map = type_map.clone();

            // Compile body
            if let Some(body) = fd.body.as_ref() {
                self.compile_block(body, fd.return_type.is_some())?;
            }

            // Clear type substitution state
            self.current_type_map.clear();
            self.param_concrete_types.clear();
            if fd.return_type.is_none() {
                self.emitln("  ret void");
            } else if !self.current_block_terminated() {
                // A4 fix (generic monomorphisation path): a value-returning body
                // fell through without a terminator (ends in a loop / if-without-
                // else / statement). Append a safe fallback return so the block is
                // terminated. Terminated bodies are unchanged (no double return).
                let zero = Self::default_const_for(&specialized_ret_type);
                self.emitln(&format!("  ret {specialized_ret_type} {zero}"));
            }
                self.emitln("}\n");
                self.pop_scope();
                self.current_fn = None;
            }
        }
        Ok(())
    }

    // ========================================================================
    // Builtin Runtime Implementations
    // ========================================================================

    /// Emit LLVM IR bodies for compiler-recognized builtin types.
    /// Only emits implementations for types actually used by the program.
    fn compile_builtin_impls(&mut self) {
        if !self.used_builtins.contains("Option") && !self.used_builtins.contains("Result") {
            return;
        }
        self.emitln("; Builtin type implementations\n");
        if self.used_builtins.contains("Option") {
            self.compile_option_impls();
        }
        if self.used_builtins.contains("Result") {
            self.compile_result_impls();
        }
    }

    fn compile_option_impls(&mut self) {
        let opt_ty = "%struct.Option";
        self.emitln(&format!("define i64 @Option.is_some({opt_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {opt_ty}"));
        self.emitln(&format!("  store {opt_ty} %self, {opt_ty}* %val"));
        self.emitln(&format!("  %disc = getelementptr {opt_ty}, {opt_ty}* %val, i32 0, i32 0"));
        self.emitln("  %result = load i64, i64* %disc");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");

        // Option.is_none()
        self.emitln(&format!("define i64 @Option.is_none({opt_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {opt_ty}"));
        self.emitln(&format!("  store {opt_ty} %self, {opt_ty}* %val"));
        self.emitln(&format!("  %disc = getelementptr {opt_ty}, {opt_ty}* %val, i32 0, i32 0"));
        self.emitln("  %is_some = load i64, i64* %disc");
        self.emitln("  %result = xor i64 %is_some, 1");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");

        // Option.unwrap()
        self.emitln(&format!("define i64 @Option.unwrap({opt_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {opt_ty}"));
        self.emitln(&format!("  store {opt_ty} %self, {opt_ty}* %val"));
        self.emitln(&format!("  %disc_gep = getelementptr {opt_ty}, {opt_ty}* %val, i32 0, i32 0"));
        self.emitln("  %is_some = load i64, i64* %disc_gep");
        self.emitln("  %ok = icmp ne i64 %is_some, 0");
        self.emitln("  br i1 %ok, label %unwrap_ok, label %unwrap_fail");
        self.emitln("\nunwrap_fail:");
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln("\nunwrap_ok:");
        self.emitln(&format!("  %val_gep = getelementptr {opt_ty}, {opt_ty}* %val, i32 0, i32 1"));
        self.emitln("  %result = load i64, i64* %val_gep");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");
    }

    fn compile_result_impls(&mut self) {
        // Result struct: { i64 is_ok, i64 value, i64 error }
        let res_ty = "%struct.Result";
        // Result.is_ok()
        self.emitln(&format!("define i64 @Result.is_ok({res_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {res_ty}"));
        self.emitln(&format!("  store {res_ty} %self, {res_ty}* %val"));
        self.emitln(&format!("  %disc = getelementptr {res_ty}, {res_ty}* %val, i32 0, i32 0"));
        self.emitln("  %result = load i64, i64* %disc");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");

        // Result.is_err()
        self.emitln(&format!("define i64 @Result.is_err({res_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {res_ty}"));
        self.emitln(&format!("  store {res_ty} %self, {res_ty}* %val"));
        self.emitln(&format!("  %disc = getelementptr {res_ty}, {res_ty}* %val, i32 0, i32 0"));
        self.emitln("  %is_ok = load i64, i64* %disc");
        self.emitln("  %result = xor i64 %is_ok, 1");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");

        // Result.unwrap()
        self.emitln(&format!("define i64 @Result.unwrap({res_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {res_ty}"));
        self.emitln(&format!("  store {res_ty} %self, {res_ty}* %val"));
        self.emitln(&format!("  %disc_gep = getelementptr {res_ty}, {res_ty}* %val, i32 0, i32 0"));
        self.emitln("  %is_ok = load i64, i64* %disc_gep");
        self.emitln("  %ok = icmp ne i64 %is_ok, 0");
        self.emitln("  br i1 %ok, label %unwrap_ok, label %unwrap_fail");
        self.emitln("\nunwrap_fail:");
        // Print error message and trap
        self.emitln(&format!("  %err_gep = getelementptr {res_ty}, {res_ty}* %val, i32 0, i32 2"));
        self.emitln("  %err_val = load i64, i64* %err_gep");
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln(&format!("\nunwrap_ok:"));
        self.emitln(&format!("  %val_gep = getelementptr {res_ty}, {res_ty}* %val, i32 0, i32 1"));
        self.emitln("  %result = load i64, i64* %val_gep");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");

        // Result.unwrap_err()
        self.emitln(&format!("define i64 @Result.unwrap_err({res_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {res_ty}"));
        self.emitln(&format!("  store {res_ty} %self, {res_ty}* %val"));
        self.emitln(&format!("  %disc_gep = getelementptr {res_ty}, {res_ty}* %val, i32 0, i32 0"));
        self.emitln("  %is_ok = load i64, i64* %disc_gep");
        self.emitln("  %should_err = icmp eq i64 %is_ok, 0");
        self.emitln("  br i1 %should_err, label %unwrap_err_ok, label %unwrap_err_fail");
        self.emitln("\nunwrap_err_fail:");
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln(&format!("\nunwrap_err_ok:"));
        self.emitln(&format!("  %err_gep = getelementptr {res_ty}, {res_ty}* %val, i32 0, i32 2"));
        self.emitln("  %result = load i64, i64* %err_gep");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");
    }

    fn compile_block(&mut self, block: &Block, is_expression: bool) -> Result<Option<String>, String> {
        let mut last_result = None;

        for (idx, item) in block.stmts.iter().enumerate() {
            let is_last = idx == block.stmts.len() - 1;
            match item {
                StmtOrExpr::Stmt(stmt) => {
                    if is_last && is_expression && matches!(stmt, Stmt::Match(..)) {
                        let ret_ty = &self.current_return_type.clone();
                        let result_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {result_alloca} = alloca {ret_ty}"));
                        self.match_result_ptr = Some(result_alloca.clone());
                        self.compile_stmt(stmt)?;
                        self.match_result_ptr = None;
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {ret_ty}, {ret_ty}* {result_alloca}"));
                        if let Some(res_ptr) = self.result_ptr.as_ref() {
                            let ret_ty = &self.current_return_type.clone();
                            self.emitln(&format!("  store {ret_ty} {loaded}, {ret_ty}* {res_ptr}"));
                        }
                        if !self.current_ensures.is_empty() {
                            self.compile_ensures_checks();
                        }
                        let ret_ty = &self.current_return_type.clone();
                        self.emitln(&format!("  ret {ret_ty} {loaded}"));
                        last_result = Some(loaded);
                    } else {
                        self.compile_stmt(stmt)?;
                    }
                }
                StmtOrExpr::Expr(expr) => {
                    let result = self.compile_expr(expr)?;
                    if let Some(ptr) = self.match_result_ptr.clone() {
                        let ret_ty = self.match_result_ty.clone().unwrap_or_else(|| self.current_return_type.clone());
                        let store_val = self.zero_val_for(&result, &ret_ty);
                        self.emitln(&format!("  store {ret_ty} {store_val}, {ret_ty}* {ptr}"));
                    }
                    if is_last && is_expression {
                        // Store result in the result alloca for ensures checks
                        if let Some(res_ptr) = self.result_ptr.as_ref() {
                            let ret_ty = self.current_return_type.clone();
                            let store_val = self.zero_val_for(&result, &ret_ty);
                            self.emitln(&format!("  store {ret_ty} {store_val}, {ret_ty}* {res_ptr}"));
                        }
                        // Check ensures before returning
                        if !self.current_ensures.is_empty() {
                            self.compile_ensures_checks();
                        }
                        let ret_ty = &self.current_return_type.clone();
                        self.emitln(&format!("  ret {ret_ty} {result}"));
                    }
                    last_result = Some(result);
                }
            }
        }
        Ok(last_result)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, _ty, value, _) => {
                let val = self.compile_expr(value)?;
                let llvm_ty = self.infer_llvm_type(value);
                let store_val = self.zero_val_for(&val, &llvm_ty);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {alloca}"));
                self.add_local(&name.name, alloca, &llvm_ty);
                // Check invariants if the value is a struct with invariants
                if self.check_contracts {
                    self.maybe_check_value_invariants(value, &val);
                }
            }
            Stmt::Var(name, _ty, value, _) => {
                let val = self.compile_expr(value)?;
                let llvm_ty = self.infer_llvm_type(value);
                let store_val = self.zero_val_for(&val, &llvm_ty);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {alloca}"));
                self.add_local(&name.name, alloca, &llvm_ty);
                // Check invariants if the value is a struct with invariants
                if self.check_contracts {
                    self.maybe_check_value_invariants(value, &val);
                }
            }
            Stmt::Assign(place, value, _) => {
                let val = self.compile_expr(value)?;
                if let Expr::Ident(ident) = place {
                    if let Some((ptr, llvm_ty)) = self.lookup_local(&ident.name).cloned() {
                        let store_val = self.zero_val_for(&val, &llvm_ty);
                        self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {ptr}"));
                    }
                }
                // Emit invariant check if the assigned place is a struct with invariants
                if let Expr::Field(obj, _, _) = place {
                    if let Expr::Ident(obj_ident) = obj.as_ref() {
                        if let Some((obj_ptr, obj_ty)) = self.lookup_local(&obj_ident.name).cloned() {
                            if obj_ty.starts_with("%struct.") {
                                let type_name = obj_ty[8..].to_string();
                                let has_invariants = self.type_meta.get(&type_name)
                                    .map(|m| !m.invariants.is_empty())
                                    .unwrap_or(false);
                                if has_invariants {
                                    let loaded = self.fresh_tmp();
                                    self.emitln(&format!("  {loaded} = load {obj_ty}, {obj_ty}* {obj_ptr}"));
                                    self.compile_invariant_call(&type_name, &loaded);
                                }
                            }
                        }
                    }
                }
            }
            Stmt::Return(expr, _) => {
                if let Some(e) = expr {
                    let mut val = self.compile_expr(e)?;
                    let mut val_ty = self.infer_llvm_type(e);
                    let ret_ty = self.current_return_type.clone();
                    // Convert value to return type if needed (e.g., i64 to double)
                    if ret_ty == "double" && val_ty == "i64" {
                        let conv = self.fresh_tmp();
                        self.emitln(&format!("  {conv} = sitofp i64 {val} to double"));
                        val = conv;
                        val_ty = "double".to_string();
                    }
                    // Coerce i64 value to struct return type via alloca+load
                    if ret_ty.starts_with("%struct.") && !val_ty.starts_with("%struct.") {
                        let coerced = self.val_to_struct(&val, &val_ty, &ret_ty);
                        val = coerced;
                    }
                    // Store result for ensures checks
                    if let Some(res_ptr) = self.result_ptr.as_ref() {
                        let ret_ty = self.current_return_type.clone();
                        self.emitln(&format!("  store {ret_ty} {val}, {ret_ty}* {res_ptr}"));
                    }
                    // Check ensures before returning
                    if !self.current_ensures.is_empty() {
                        self.compile_ensures_checks();
                    }
                    let ret_ty = self.current_return_type.clone();
                    // Decrement recursion depth
                    let depth_dec = self.fresh_tmp();
                    self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
                    let new_depth_dec = self.fresh_tmp();
                    self.emitln(&format!("  {new_depth_dec} = sub i64 {depth_dec}, 1"));
                    self.emitln(&format!("  store i64 {new_depth_dec}, i64* @xiom_recursion_counter"));
                    self.emitln(&format!("  ret {ret_ty} {val}"));
                } else {
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
            }
            Stmt::Expr(expr, _) => {
                self.compile_expr(expr)?;
            }
            Stmt::If(cond, then_block, elifs, else_block, _) => {
                let cond_raw = self.compile_expr(cond)?;
                let cond_ty = self.infer_llvm_type(cond);
                let cond_val = if cond_ty == "i1" {
                    cond_raw
                } else if cond_ty == "i64" {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = icmp ne i64 {cond_raw}, 0"));
                    tmp
                } else {
                    let tmp1 = self.fresh_tmp();
                    self.emitln(&format!("  {tmp1} = icmp ne {cond_ty} {cond_raw}, 0"));
                    tmp1
                };
                let then_label = self.fresh_block("then");
                let merge_label = self.fresh_block("merge");

                let else_label = if !elifs.is_empty() || else_block.is_some() {
                    self.fresh_block("else")
                } else {
                    merge_label.clone()
                };

                let block_ends_with_ret = |b: &Block| -> bool {
                    b.stmts.last().map_or(false, |s| matches!(s, StmtOrExpr::Stmt(Stmt::Return(..))))
                };

                let mut merge_reachable = false;

                self.emitln(&format!("  br i1 {cond_val}, label %{then_label}, label %{else_label}"));
                self.emitln(&format!("\n{then_label}:"));
                self.compile_block(then_block, false)?;
                if !block_ends_with_ret(then_block) {
                    self.emitln(&format!("  br label %{merge_label}"));
                    merge_reachable = true;
                }

                // Elif chain
                let mut prev_label = if elifs.is_empty() && else_block.is_none() {
                    merge_label.clone()
                } else {
                    else_label.clone()
                };

                for (i, (econd, eblock)) in elifs.iter().enumerate() {
                    self.emitln(&format!("\n{prev_label}:"));
                    let econd_raw = self.compile_expr(econd)?;
                    let econd_ty = self.infer_llvm_type(econd);
                    let econd_val = if econd_ty == "i1" {
                        econd_raw
                    } else if econd_ty == "i64" {
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = icmp ne i64 {econd_raw}, 0"));
                        tmp
                    } else {
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = icmp ne {econd_ty} {econd_raw}, 0"));
                        tmp
                    };
                    let elif_then = self.fresh_block("elif_then");
                    let elif_next = if i + 1 < elifs.len() || else_block.is_some() {
                        self.fresh_block("elif_next")
                    } else {
                        merge_label.clone()
                    };
                    self.emitln(&format!("  br i1 {econd_val}, label %{elif_then}, label %{elif_next}"));
                    self.emitln(&format!("\n{elif_then}:"));
                    self.compile_block(eblock, false)?;
                    if !block_ends_with_ret(eblock) {
                        self.emitln(&format!("  br label %{merge_label}"));
                        merge_reachable = true;
                    }
                    prev_label = elif_next;
                }

                // Else block
                if let Some(eb) = else_block {
                    self.emitln(&format!("\n{prev_label}:"));
                    self.compile_block(eb, false)?;
                    if !block_ends_with_ret(eb) {
                        self.emitln(&format!("  br label %{merge_label}"));
                        merge_reachable = true;
                    }
                } else if elifs.is_empty() && prev_label != merge_label {
                    // No else case for simple if — the else block is just a merge jump
                    self.emitln(&format!("\n{prev_label}:"));
                    self.emitln(&format!("  br label %{merge_label}"));
                    merge_reachable = true;
                } else if elifs.is_empty() {
                    // prev_label == merge_label — skip redundant label emission
                    merge_reachable = true;
                } else if prev_label != merge_label {
                    self.emitln(&format!("\n{prev_label}:"));
                    self.emitln(&format!("  br label %{merge_label}"));
                    merge_reachable = true;
                }

                self.emitln(&format!("\n{merge_label}:"));
                if !merge_reachable {
                    self.emitln("  unreachable");
                }
            }
            Stmt::Match(expr_match, arms, _) => {
                let val = self.compile_expr(expr_match)?;
                let merge_label = self.fresh_block("match_merge");

                // Determine scrutinee type for variant pattern matching
                let scrutinee_type = self.struct_type_from_expr(expr_match);

                // Store scrutinee value in alloca for field extraction
                let mut scrutinee_alloca_info = None;
                if let Some(ref type_name) = scrutinee_type {
                    let struct_ty = format!("%struct.{type_name}");
                    let alloca = self.fresh_tmp();
                    let store_val = self.zero_val_for(&val, &struct_ty);
                    self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                    self.emitln(&format!("  store {struct_ty} {store_val}, {struct_ty}* {alloca}"));
                    scrutinee_alloca_info = Some((alloca, type_name.clone(), struct_ty));
                }

                // Build check block labels and arm labels.
                //
                // The set of arms that receive a runtime check block here MUST
                // stay in lockstep with the emit loop further below. We record
                // that decision exactly once per arm in `arm_is_checked` (using
                // the shared `pattern_needs_check` predicate) so the build and
                // emit loops can never desynchronize. A prior desync between the
                // two loops advanced `check_idx` past `check_labels.len()` and
                // caused an out-of-bounds panic.
                let mut check_labels: Vec<String> = Vec::new();
                let mut arm_labels: Vec<String> = Vec::new();
                let mut arm_is_checked: Vec<bool> = Vec::new();
                let mut wildcard_idx: Option<usize> = None;

                for (i, arm) in arms.iter().enumerate() {
                    let arm_label = self.fresh_block("match_arm");
                    arm_labels.push(arm_label);
                    let checked = self.pattern_needs_check(&arm.pattern, &scrutinee_type);
                    arm_is_checked.push(checked);
                    if checked {
                        check_labels.push(self.fresh_block("match_check"));
                    } else if matches!(&arm.pattern, Pattern::Wildcard(_) | Pattern::Ident(_)) {
                        // Wildcard-like binding arm: acts as the default target.
                        // (Some/None/Ok/Err/Or and non-Int/Bool literal patterns
                        // are intentionally non-checking AND non-default here,
                        // matching the emit loop below.)
                        //
                        // TODO(or-patterns): compile each alternative of
                        // `Pattern::Or` as its own check.
                        wildcard_idx = Some(i);
                    }
                }

                // Branch to the first check block (or straight to the default
                // arm / merge block when there are no checks). All indexing is
                // bounds-guarded.
                if let Some(first_check) = check_labels.first() {
                    self.emitln(&format!("  br label %{first_check}"));
                } else if let Some(wi) = wildcard_idx {
                    let target = arm_labels.get(wi).cloned().unwrap_or_else(|| merge_label.clone());
                    self.emitln(&format!("  br label %{target}"));
                } else {
                    self.emitln(&format!("  br label %{merge_label}"));
                }

                // Emit check blocks.
                //
                // `check_idx` walks `check_labels` in lockstep with the build
                // loop above: it advances by exactly one for every arm whose
                // `arm_is_checked[i]` is `true`, so it can never outrun
                // `check_labels`. Every index into `check_labels`/`arm_labels`
                // is additionally bounds-guarded so that even a future codegen
                // bug degrades to a branch-to-merge instead of a panic — a
                // compiler must never crash.
                let mut check_idx: usize = 0;
                for (i, arm) in arms.iter().enumerate() {
                    let checked = arm_is_checked.get(i).copied().unwrap_or(false);
                    if !checked {
                        // Wildcard-like / non-checking arm: no check block.
                        continue;
                    }

                    // Label for this arm's own check block (bounds-guarded).
                    let this_label = if check_idx < check_labels.len() {
                        check_labels[check_idx].clone()
                    } else {
                        // Safety fallback — unreachable once the build/emit
                        // loops are symmetric. Emit a diagnostic comment and
                        // skip this (impossible) arm rather than panicking.
                        self.emitln(&format!(
                            "  ; codegen: check_idx {} out of range (len {}); skipping",
                            check_idx,
                            check_labels.len()
                        ));
                        check_idx += 1;
                        continue;
                    };

                    // Label to fall through to when this arm's check fails: the
                    // next check block, else the default (wildcard) arm, else
                    // the merge block.
                    let next = if check_idx + 1 < check_labels.len() {
                        check_labels[check_idx + 1].clone()
                    } else if let Some(wi) = wildcard_idx {
                        arm_labels.get(wi).cloned().unwrap_or_else(|| merge_label.clone())
                    } else {
                        merge_label.clone()
                    };

                    // Target block when this arm's check succeeds.
                    let arm_label = arm_labels.get(i).cloned().unwrap_or_else(|| merge_label.clone());

                    self.emitln(&format!("\n{this_label}:"));

                    match &arm.pattern {
                        Pattern::Lit(Literal::Int(n, _)) => {
                            let check = self.fresh_tmp();
                            self.emitln(&format!("  {check} = icmp eq i64 {val}, {n}"));
                            self.emitln(&format!("  br i1 {check}, label %{arm_label}, label %{next}"));
                        }
                        Pattern::Lit(Literal::Bool(b, _)) => {
                            let check = self.fresh_tmp();
                            let bval = if *b { "1" } else { "0" };
                            self.emitln(&format!("  {check} = icmp eq i64 {val}, {bval}"));
                            self.emitln(&format!("  br i1 {check}, label %{arm_label}, label %{next}"));
                        }
                        Pattern::Variant(variant_name, _, _) => {
                            self.emit_variant_discriminant_check(
                                &variant_name.name,
                                &scrutinee_alloca_info,
                                &val,
                                &arm_label,
                                &next,
                            );
                        }
                        Pattern::Ident(ident) => {
                            // Only reachable when this ident names an enum
                            // variant (see `pattern_needs_check`).
                            self.emit_variant_discriminant_check(
                                &ident.name,
                                &scrutinee_alloca_info,
                                &val,
                                &arm_label,
                                &next,
                            );
                        }
                        _ => {
                            // `arm_is_checked` is never true for other patterns;
                            // branch unconditionally so the block stays valid.
                            self.emitln(&format!("  br label %{arm_label}"));
                        }
                    }

                    check_idx += 1;
                }

                // Emit arm bodies
                for (i, arm) in arms.iter().enumerate() {
                    let arm_label = arm_labels.get(i).cloned().unwrap_or_else(|| merge_label.clone());
                    self.emitln(&format!("\n{arm_label}:"));
                    // For variant patterns, extract fields before compiling arm body
                    if let Pattern::Variant(variant_ident, fields, _) = &arm.pattern {
                        if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                            let field_names_opt = self.types.get(type_name).cloned();
                            if let Some(field_names) = field_names_opt {
                                let type_name_clone = type_name.clone();
                                let variants_opt = self.enum_variants.get(type_name).cloned();
                                for (fi, field_ident) in fields.iter().enumerate() {
                                    // First try direct name match
                                    let field_idx_opt = field_names.iter().position(|f| f == &field_ident.name);
                                    // Fallback: use variant field position to find canonical field name
                                    let field_idx_opt = field_idx_opt.or_else(|| {
                                        variants_opt.as_ref().and_then(|variants| {
                                            variants.iter().find(|(vn, _)| vn == &variant_ident.name)
                                                .and_then(|(_, vfields)| vfields.get(fi))
                                                .and_then(|canonical| field_names.iter().position(|f| f == canonical))
                                        })
                                    });
                                    if let Some(field_idx) = field_idx_opt {
                                        let gep = self.fresh_tmp();
                                        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {field_idx}"));
                                        let loaded = self.fresh_tmp();
                                        let field_llvm_ty = self.field_llvm_type(&type_name_clone, field_idx);
                                        self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                        let field_alloca = self.fresh_tmp();
                                        self.emitln(&format!("  {field_alloca} = alloca {field_llvm_ty}"));
                                        self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {field_alloca}"));
                                        self.add_local(&field_ident.name, field_alloca, &field_llvm_ty);
                                    }
                                }
                            }
                        }
                    }
                    // For Ident patterns, bind the matched value to the identifier
                    // (skip for enum variant names, which are handled by the check block)
                    if let Pattern::Ident(ident) = &arm.pattern {
                        let is_variant = scrutinee_type.as_ref().and_then(|tn| {
                            self.enum_variants.get(tn)
                                .map(|vars| vars.iter().any(|(v, _)| v == &ident.name))
                        }).unwrap_or(false);
                        if !is_variant {
                            let match_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {match_alloca} = alloca i64"));
                            self.emitln(&format!("  store i64 {val}, i64* {match_alloca}"));
                            self.add_local(&ident.name, match_alloca, "i64");
                        }
                    }
                    match &arm.body {
                        MatchBody::Block(b) => { self.compile_block(b, false)?; }
                        MatchBody::Expr(e) => {
                            let arm_val = self.compile_expr(e)?;
                            if let Some(ptr) = self.match_result_ptr.clone() {
                                let ret_ty = self.match_result_ty.clone().unwrap_or_else(|| self.current_return_type.clone());
                                let store_val = self.zero_val_for(&arm_val, &ret_ty);
                                self.emitln(&format!("  store {ret_ty} {store_val}, {ret_ty}* {ptr}"));
                            }
                        }
                    }
                    self.emitln(&format!("  br label %{merge_label}"));
                }

                self.emitln(&format!("\n{merge_label}:"));
            }
            Stmt::While(cond, body, _) => {
                let loop_cond = self.fresh_block("while_cond");
                let loop_body = self.fresh_block("while_body");
                let loop_exit = self.fresh_block("while_exit");

                self.emitln(&format!("  br label %{loop_cond}"));
                self.emitln(&format!("\n{loop_cond}:"));
                let cond_raw = self.compile_expr(cond)?;
                let cond_ty = self.infer_llvm_type(cond);
                let cond_val = if cond_ty == "i1" {
                    cond_raw
                } else if cond_ty == "i64" {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = icmp ne i64 {cond_raw}, 0"));
                    tmp
                } else {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = icmp ne {cond_ty} {cond_raw}, 0"));
                    tmp
                };
                self.emitln(&format!("  br i1 {cond_val}, label %{loop_body}, label %{loop_exit}"));
                self.emitln(&format!("\n{loop_body}:"));
                self.loop_stack.push((loop_cond.clone(), loop_exit.clone()));
                self.compile_block(body, false)?;
                self.loop_stack.pop();
                self.emitln(&format!("  br label %{loop_cond}"));
                self.emitln(&format!("\n{loop_exit}:"));
            }
            Stmt::For(_, _, body, _) => {
                // Phase 0: simplified for — just execute body once
                self.compile_block(body, false)?;
            }
            Stmt::Destructure(names, value, _) => {
                let val = self.compile_expr(value)?;
                let llvm_ty = self.infer_llvm_type(value);
                if llvm_ty.starts_with("%struct.") && names.len() > 0 {
                    // Alloca + store the struct value, then GEP to extract each field
                    let alloca_struct = self.fresh_tmp();
                    self.emitln(&format!("  {alloca_struct} = alloca {llvm_ty}"));
                    self.emitln(&format!("  store {llvm_ty} {val}, {llvm_ty}* {alloca_struct}"));
                    let struct_name = &llvm_ty[8..];
                    for (i, name) in names.iter().enumerate() {
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {llvm_ty}, {llvm_ty}* {alloca_struct}, i32 0, i32 {i}"));
                        let field_ty = self.field_llvm_type(struct_name, i);
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {field_ty}, {field_ty}* {gep}"));
                        let alloca = self.fresh_tmp();
                        self.emitln(&format!("  {alloca} = alloca {field_ty}"));
                        self.emitln(&format!("  store {field_ty} {loaded}, {field_ty}* {alloca}"));
                        self.add_local(&name.name, alloca, &field_ty);
                    }
                } else {
                    let store_val = self.zero_val_for(&val, &llvm_ty);
                    for name in names {
                        let alloca = self.fresh_tmp();
                        self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                        self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {alloca}"));
                        self.add_local(&name.name, alloca, &llvm_ty);
                    }
                }
            }
            Stmt::Spawn(body, _) => {
                self.compile_block(body, false)?;
            }
            Stmt::Break(_) => {
                if let Some((_, break_label)) = self.loop_stack.last().cloned() {
                    self.emitln(&format!("  br label %{break_label}"));
                    let dead = self.fresh_block("after_break");
                    self.emitln(&format!("\n{dead}:"));
                }
            }
            Stmt::Continue(_) => {
                if let Some((cont_label, _)) = self.loop_stack.last().cloned() {
                    self.emitln(&format!("  br label %{cont_label}"));
                    let dead = self.fresh_block("after_continue");
                    self.emitln(&format!("\n{dead}:"));
                }
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Ident(ident) => {
                if let Some((ptr, llvm_ty)) = self.lookup_local(&ident.name).cloned() {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                    Ok(tmp)
                } else if let Some(enum_key) = self.enum_variants.iter()
                    .find(|(_, vars)| vars.iter().any(|(v, _)| v == &ident.name))
                    .map(|(ek, _)| ek)
                    .filter(|ek| self.types.contains_key(*ek))
                {
                    if let Some(vars) = self.enum_variants.get(enum_key) {
                        if let Some(var_idx) = vars.iter().position(|(v, _)| v == &ident.name) {
                            let struct_ty = self.llvm_type_for(enum_key)?;
                            let alloca = self.fresh_tmp();
                            self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                            let disc_gep = self.fresh_tmp();
                            self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                            self.emitln(&format!("  store i64 {var_idx}, i64* {disc_gep}"));
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
                            Ok(loaded)
                        } else {
                            Ok("0".to_string())
                        }
                    } else {
                        Ok("0".to_string())
                    }
                } else {
                    Ok("0".to_string())
                }
            }
            Expr::Int(n, _) => {
                Ok(format!("{n}"))
            }
            Expr::Float(f, _) => {
                Ok(format!("{f:.6}"))
            }
            Expr::Bool(b, _) => {
                Ok(if *b { "1".to_string() } else { "0".to_string() })
            }
            Expr::Str(s, _) => {
                let str_id = self.str_counter;
                self.str_counter += 1;
                let label = format!("@.str{str_id}");
                let escaped = s.replace('\\', "\\\\").replace('"', "\\22")
                    .replace('\n', "\\0A").replace('\t', "\\09");
                self.strings.push(format!(
                    "{label} = private unnamed_addr constant [{} x i8] c\"{}\\00\"",
                    s.len() + 1, escaped
                ));
                let tmp = self.fresh_tmp();
                self.emitln(&format!("  {tmp} = getelementptr [{} x i8], [{} x i8]* {label}, i64 0, i64 0", s.len() + 1, s.len() + 1));
                Ok(tmp)
            }
            Expr::Char(c, _) => {
                Ok(format!("{}", *c as u32))
            }
            Expr::Paren(inner, _) => self.compile_expr(inner),
            Expr::Tuple(items, _) => {
                if items.is_empty() {
                    Ok("0".to_string())
                } else {
                    let struct_ty = self.infer_llvm_type(expr);
                    if !struct_ty.starts_with("%struct.") {
                        return Ok("0".to_string());
                    }
                    let alloca = self.fresh_tmp();
                    self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                    for (i, item) in items.iter().enumerate() {
                        let item_val = self.compile_expr(item)?;
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
                        let item_ty = self.infer_llvm_type(item);
                        self.emitln(&format!("  store {item_ty} {item_val}, {item_ty}* {gep}"));
                    }
                    let loaded = self.fresh_tmp();
                    self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
                    Ok(loaded)
                }
            }
            Expr::Unary(op, inner, _) => {
                let val = self.compile_expr(inner)?;
                let tmp = self.fresh_tmp();
                match op {
                    UnaryOp::Neg => {
                        if self.infer_llvm_type(inner) == "double" {
                            self.emitln(&format!("  {tmp} = fneg double {val}"));
                        } else {
                            self.emitln(&format!("  {tmp} = sub i64 0, {val}"));
                        }
                    }
                    UnaryOp::Not => {
                        let val_ty = self.infer_llvm_type(inner);
                        let xor_val = if val_ty == "i1" || val_ty == "i8" {
                            let ext = self.fresh_tmp();
                            self.emitln(&format!("  {ext} = zext {val_ty} {val} to i64"));
                            ext
                        } else {
                            val
                        };
                        self.emitln(&format!("  {tmp} = xor i64 {xor_val}, 1"));
                    }
                    UnaryOp::BitNot => {
                        self.emitln(&format!("  {tmp} = xor i64 {val}, -1"));
                    }
                    UnaryOp::Deref => {
                        let ty = self.infer_llvm_type(inner);
                        self.emitln(&format!("  {tmp} = load {ty}, {ty}* {val}"));
                    }
                    UnaryOp::Ref | UnaryOp::MutRef => return Ok(val),
                }
                Ok(tmp)
            }
            Expr::Binary(left, op, right, _) => {
                let mut l = self.compile_expr(left)?;
                let mut r = self.compile_expr(right)?;
                let tmp = self.fresh_tmp();
                let is_float = self.is_float_expr(left) || self.is_float_expr(right);
                if matches!(op, BinOp::And | BinOp::Or) {
                    let is_or = matches!(op, BinOp::Or);
                    let widen = |s: &mut Self, val: &str, ty: &str| -> String {
                        if ty == "i64" { val.to_string() } else {
                            let ext = s.fresh_tmp();
                            s.emitln(&format!("  {ext} = zext {ty} {val} to i64"));
                            ext
                        }
                    };
                    let lt = self.infer_llvm_type(left);
                    let rt = self.infer_llvm_type(right);
                    let lw = widen(self, &l, &lt);
                    let rw = widen(self, &r, &rt);
                    let op_name = if is_or { "or" } else { "and" };
                    let result = self.fresh_tmp();
                    self.emitln(&format!("  {result} = {op_name} i64 {lw}, {rw}"));
                    return Ok(result);
                }
                // For struct-typed equality/inequality, call derived eq() instead of icmp
                if matches!(op, BinOp::Eq | BinOp::Neq) {
                    let lt = self.infer_llvm_type(left);
                    if lt.starts_with("%struct.") || self.infer_llvm_type(right).starts_with("%struct.") {
                        let struct_name = if lt.starts_with("%struct.") { &lt[8..] } else { &self.infer_llvm_type(right)[8..] };
                        let eq_fn = format!("{}.eq", struct_name);
                        let eq_result = self.fresh_tmp();
                        self.emitln(&format!("  {eq_result} = call i64 @{eq_fn}({lt} {l}, {} {r})", self.infer_llvm_type(right)));
                        if matches!(op, BinOp::Neq) {
                            let negated = self.fresh_tmp();
                            self.emitln(&format!("  {negated} = xor i64 {eq_result}, 1"));
                            return Ok(negated);
                        }
                        return Ok(eq_result);
                    }
                }
                let (ty, inst) = match op {
                    BinOp::Add => (if is_float { "double" } else { "i64" }, if is_float { "fadd" } else { "add" }),
                    BinOp::Sub => (if is_float { "double" } else { "i64" }, if is_float { "fsub" } else { "sub" }),
                    BinOp::Mul => (if is_float { "double" } else { "i64" }, if is_float { "fmul" } else { "mul" }),
                    BinOp::Div => (if is_float { "double" } else { "i64" }, if is_float { "fdiv" } else { "sdiv" }),
                    BinOp::Rem => (if is_float { "double" } else { "i64" }, if is_float { "frem" } else { "srem" }),
                    BinOp::BitXor => ("i64", "xor"),
                    BinOp::BitAnd => ("i64", "and"),
                    BinOp::BitOr => ("i64", "or"),
                    BinOp::Shl => ("i64", "shl"),
                    BinOp::Shr => ("i64", "ashr"),
                    BinOp::Eq => (if is_float { "double" } else { "i64" }, if is_float { "fcmp oeq" } else { "icmp eq" }),
                    BinOp::Neq => (if is_float { "double" } else { "i64" }, if is_float { "fcmp one" } else { "icmp ne" }),
                    BinOp::Lt => (if is_float { "double" } else { "i64" }, if is_float { "fcmp olt" } else { "icmp slt" }),
                    BinOp::Gt => (if is_float { "double" } else { "i64" }, if is_float { "fcmp ogt" } else { "icmp sgt" }),
                    BinOp::Le => (if is_float { "double" } else { "i64" }, if is_float { "fcmp ole" } else { "icmp sle" }),
                    BinOp::Ge => (if is_float { "double" } else { "i64" }, if is_float { "fcmp oge" } else { "icmp sge" }),
                    BinOp::Assign => return Ok(r),
                    _ => unreachable!(),
                };
                // For non-float comparisons, use the actual operand LLVM type
                // (handles pointer types like i8* for string comparisons)
                let ty = if !is_float && inst.starts_with("icmp") {
                    let lt = self.infer_llvm_type(left);
                    let rt = self.infer_llvm_type(right);
                    if lt.contains('*') { lt } else if rt.contains('*') { rt } else { ty.to_string() }
                } else {
                    ty.to_string()
                };
                // Convert literal 0 to null pointer when comparing with pointer types
                if ty.contains('*') {
                    if l == "0" && self.infer_llvm_type(left) == "i64" {
                        let null_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {null_tmp} = inttoptr i64 0 to {ty}"));
                        l = null_tmp;
                    }
                    if r == "0" && self.infer_llvm_type(right) == "i64" {
                        let null_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {null_tmp} = inttoptr i64 0 to {ty}"));
                        r = null_tmp;
                    }
                }
                // Coerce i64 operands to double when in float context (mixed-type expressions)
                if is_float {
                    if self.infer_llvm_type(left) == "i64" {
                        let conv = self.fresh_tmp();
                        self.emitln(&format!("  {conv} = sitofp i64 {l} to double"));
                        l = conv;
                    }
                    if self.infer_llvm_type(right) == "i64" {
                        let conv = self.fresh_tmp();
                        self.emitln(&format!("  {conv} = sitofp i64 {r} to double"));
                        r = conv;
                    }
                }
                let div_cont = if !is_float && matches!(op, BinOp::Div | BinOp::Rem) {
                    let zero_check = self.fresh_tmp();
                    self.emitln(&format!("  {zero_check} = icmp eq i64 {r}, 0"));
                    let trap_block = self.fresh_block("div_zero_trap");
                    let safe_block = self.fresh_block("div_safe");
                    let cont_block = self.fresh_block("div_continue");
                    self.emitln(&format!("  br i1 {zero_check}, label %{trap_block}, label %{safe_block}"));
                    self.emitln(&format!("\n{trap_block}:"));
                    self.emitln("  call void @llvm.trap()");
                    self.emitln("  unreachable");
                    self.emitln(&format!("\n{safe_block}:"));
                    Some(cont_block)
                } else {
                    None
                };
                self.emitln(&format!("  {tmp} = {inst} {ty} {l}, {r}"));
                let result = if inst.starts_with("icmp") || inst.starts_with("fcmp") {
                    let ext = self.fresh_tmp();
                    self.emitln(&format!("  {ext} = zext i1 {tmp} to i64"));
                    ext
                } else {
                    tmp
                };
                if let Some(cont_block) = div_cont {
                    self.emitln(&format!("  br label %{cont_block}"));
                    self.emitln(&format!("\n{cont_block}:"));
                }
                Ok(result)
            }
            Expr::Try(inner, _span) => {
                let val = self.compile_expr(inner)?;
                // Determine if this is Option (2 fields) or Result (3 fields)
                let is_option = match &**inner {
                    Expr::Some(..) | Expr::None(..) => {
                        self.used_builtins.insert("Option".to_string());
                        true
                    }
                    Expr::Ok(..) | Expr::Err(..) => {
                        self.used_builtins.insert("Result".to_string());
                        false
                    }
                    Expr::Ident(id) => {
                        // Check type from locals
                        let mut opt_like = true;
                        if let Some((_, llvm_ty)) = self.lookup_local(&id.name) {
                            if llvm_ty.starts_with("%struct.") {
                                let type_name = &llvm_ty[8..];
                                if let Some(field_names) = self.types.get(type_name) {
                                    opt_like = field_names.len() <= 2;
                                }
                            }
                        }
                        if opt_like { self.used_builtins.insert("Option".to_string()); }
                        else { self.used_builtins.insert("Result".to_string()); }
                        opt_like
                    }
                    Expr::Call(func, _, _) => {
                        // Check return type from function signatures
                        let fn_name = match &**func {
                            Expr::Ident(name) => Some(name.name.clone()),
                            Expr::Field(_, field, _) => Some(field.name.clone()),
                            _ => None,
                        };
                        if let Some(name) = fn_name {
                            // If function is defined and its return type is a struct with ≤2 fields
                            if let Some(field_names) = self.types.get(&name) {
                                let opt_like = field_names.len() <= 2;
                                if opt_like { self.used_builtins.insert("Option".to_string()); }
                                else { self.used_builtins.insert("Result".to_string()); }
                                opt_like
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    _ => {
                        // Default to Result (backward compat)
                        self.used_builtins.insert("Result".to_string());
                        false
                    }
                };
                if is_option {
                    let opt_ty = "%struct.Option";
                    let opt_alloca = self.fresh_tmp();
                    self.emitln(&format!("  {opt_alloca} = alloca {opt_ty}"));
                    self.emitln(&format!("  store {opt_ty} {val}, {opt_ty}* {opt_alloca}"));
                    let tag_gep = self.fresh_tmp();
                    let tag = self.fresh_tmp();
                    self.emitln(&format!("  {tag_gep} = getelementptr {opt_ty}, {opt_ty}* {opt_alloca}, i32 0, i32 0"));
                    self.emitln(&format!("  {tag} = load i64, i64* {tag_gep}"));
                    let some_block = self.fresh_block("try_some");
                    let none_block = self.fresh_block("try_none");
                    self.emitln(&format!("  br i1 {tag}, label %{some_block}, label %{none_block}"));
                    self.emitln(&format!("\n{none_block}:"));
                    let ret_ty = self.current_return_type.clone();
                    let default_val = if ret_ty.starts_with('%') { "zeroinitializer".to_string() } else { "0".to_string() };
                    self.emitln(&format!("  ret {ret_ty} {default_val}"));
                    self.emitln(&format!("\n{some_block}:"));
                    let val_gep = self.fresh_tmp();
                    let some_val = self.fresh_tmp();
                    self.emitln(&format!("  {val_gep} = getelementptr {opt_ty}, {opt_ty}* {opt_alloca}, i32 0, i32 1"));
                    self.emitln(&format!("  {some_val} = load i64, i64* {val_gep}"));
                    Ok(some_val)
                } else {
                    let result_ty = "%struct.Result";
                    let result_alloca = self.fresh_tmp();
                    self.emitln(&format!("  {result_alloca} = alloca {result_ty}"));
                    self.emitln(&format!("  store {result_ty} {val}, {result_ty}* {result_alloca}"));
                    let tag_gep = self.fresh_tmp();
                    let tag = self.fresh_tmp();
                    self.emitln(&format!("  {tag_gep} = getelementptr {result_ty}, {result_ty}* {result_alloca}, i32 0, i32 0"));
                    self.emitln(&format!("  {tag} = load i64, i64* {tag_gep}"));
                    let ok_block = self.fresh_block("try_ok");
                    let err_block = self.fresh_block("try_err");
                    self.emitln(&format!("  br i1 {tag}, label %{ok_block}, label %{err_block}"));
                    self.emitln(&format!("\n{err_block}:"));
                    let err_gep = self.fresh_tmp();
                    let err_val = self.fresh_tmp();
                    self.emitln(&format!("  {err_gep} = getelementptr {result_ty}, {result_ty}* {result_alloca}, i32 0, i32 2"));
                    self.emitln(&format!("  {err_val} = load i64, i64* {err_gep}"));
                    let ret_ty = self.current_return_type.clone();
                    self.emitln(&format!("  ret {ret_ty} {err_val}"));
                    self.emitln(&format!("\n{ok_block}:"));
                    let val_gep = self.fresh_tmp();
                    let ok_val = self.fresh_tmp();
                    self.emitln(&format!("  {val_gep} = getelementptr {result_ty}, {result_ty}* {result_alloca}, i32 0, i32 1"));
                    self.emitln(&format!("  {ok_val} = load i64, i64* {val_gep}"));
                    Ok(ok_val)
                }
            }
            Expr::Imply(left, right, _) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
                let tmp1 = self.fresh_tmp();
                let tmp2 = self.fresh_tmp();
                self.emitln(&format!("  {tmp1} = xor i64 {l}, 1"));
                self.emitln(&format!("  {tmp2} = or i64 {tmp1}, {r}"));
                Ok(tmp2)
            }
            Expr::Is(_, _, _) => Ok("1".to_string()),
            Expr::Field(obj, field, _) => {
                // Simplified field access: if the object is an ident in locals, load the field via GEP
                if let Expr::Ident(obj_ident) = obj.as_ref() {
                    if let Some((ptr, llvm_ty)) = self.lookup_local(&obj_ident.name).cloned() {
                        // Check if it's a struct type
                        if llvm_ty.starts_with("%struct.") {
                            // Find field index
                            let type_name = &llvm_ty[8..];
                            if let Some(field_names) = self.types.get(type_name) {
                                if let Some(field_idx) = field_names.iter().position(|f| f == &field.name) {
                                    let field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                                    let struct_val = self.fresh_tmp();
                                    self.emitln(&format!("  {struct_val} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                                    let struct_alloca = self.fresh_tmp();
                                    self.emitln(&format!("  {struct_alloca} = alloca {llvm_ty}"));
                                    self.emitln(&format!("  store {llvm_ty} {struct_val}, {llvm_ty}* {struct_alloca}"));
                                    let gep = self.fresh_tmp();
                                    let loaded = self.fresh_tmp();
                                    self.emitln(&format!("  {gep} = getelementptr {llvm_ty}, {llvm_ty}* {struct_alloca}, i32 0, i32 {field_idx}"));
                                    self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                    return Ok(loaded);
                                }
                            }
                        }
                    }
                }
                Ok("0".to_string())
            }
            Expr::Call(func, args, _) => {
                // Determine function name and receiver for both direct and method call forms
                let (fn_name_opt, receiver_expr) = match &**func {
                    Expr::Ident(name) => (Some(name.name.clone()), None),
                    Expr::Field(obj, field, _) => (Some(field.name.clone()), Some(obj)),
                    _ => (None, None),
                };
                let fn_name = match fn_name_opt {
                    Some(ref n) => n.clone(),
                    None => return Ok("0".to_string()),
                };
                // Check for contract collection methods
                let is_contract_method = matches!(fn_name.as_str(), "is_sorted" | "all" | "none" | "contains");
                if is_contract_method {
                    let tmp = self.fresh_tmp();
                    if let Some(receiver) = receiver_expr {
                        // Method form: receiver.method(args)
                        let recv_val = self.compile_expr(receiver)?;
                        let recv_llvm_ty = self.infer_llvm_type(receiver);
                        let recv_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {recv_alloca} = alloca {recv_llvm_ty}"));
                        self.emitln(&format!("  store {recv_llvm_ty} {recv_val}, {recv_llvm_ty}* {recv_alloca}"));
                        let ptr = self.fresh_tmp();
                        self.emitln(&format!("  {ptr} = bitcast {recv_llvm_ty}* {recv_alloca} to i8*"));
                        let extra_args: Vec<String> = args.iter()
                            .map(|a| self.compile_expr(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        match fn_name.as_str() {
                            "is_sorted" => {
                                self.emitln(&format!("  {tmp} = call i64 @xiom_is_sorted(i8* {ptr}, i64 0)"));
                            }
                            "all" => {
                                let pred = extra_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @xiom_all(i8* {ptr}, i64 0, i8* {pred})"));
                            }
                            "none" => {
                                let pred = extra_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @xiom_none(i8* {ptr}, i64 0, i8* {pred})"));
                            }
                            "contains" => {
                                let val = extra_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @xiom_contains(i8* {ptr}, i64 {val})"));
                            }
                            _ => unreachable!(),
                        }
                        return Ok(tmp);
                    } else {
                        // Direct form: method(args) — compile all args
                        let compiled_args: Vec<String> = args.iter()
                            .map(|a| self.compile_expr(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        // Convert first argument to i8* pointer via alloca+bitcast
                        let ptr_val = compiled_args.first().cloned().unwrap_or_else(|| "0".to_string());
                        let ptr_ty = if let Some(arg) = args.first() { self.infer_llvm_type(arg) } else { "i64".to_string() };
                        let ptr = if ptr_ty == "i8*" {
                            ptr_val
                        } else {
                            let arg_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {arg_alloca} = alloca {ptr_ty}"));
                            self.emitln(&format!("  store {ptr_ty} {ptr_val}, {ptr_ty}* {arg_alloca}"));
                            let arg_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {arg_ptr} = bitcast {ptr_ty}* {arg_alloca} to i8*"));
                            arg_ptr
                        };
                        match fn_name.as_str() {
                            "is_sorted" => {
                                let len = compiled_args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @xiom_is_sorted(i8* {ptr}, i64 {len})"));
                            }
                            "all" => {
                                let len = compiled_args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                                let pred = compiled_args.get(2).cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @xiom_all(i8* {ptr}, i64 {len}, i8* {pred})"));
                            }
                            "none" => {
                                let len = compiled_args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                                let pred = compiled_args.get(2).cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @xiom_none(i8* {ptr}, i64 {len}, i8* {pred})"));
                            }
                            "contains" => {
                                let val = compiled_args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @xiom_contains(i8* {ptr}, i64 {val})"));
                            }
                            _ => unreachable!(),
                        }
                        return Ok(tmp);
                    }
                }
                // Primitive interface methods (Ord.compare, Eq.eq/ne, comparison ops,
                // Hash.hash, Clone.clone) are emitted inline for scalar receivers, so
                // primitives satisfy Ord/Eq/Hash/Clone bounds without a user method.
                let is_builtin_iface_method = matches!(
                    fn_name.as_str(),
                    "compare" | "eq" | "ne" | "lt" | "gt" | "le" | "ge" | "hash" | "clone"
                );
                if is_builtin_iface_method {
                    if let Some(receiver) = receiver_expr {
                        // Skip static/type-name receivers (e.g. Int.compare(a, b)).
                        let receiver_is_type_name = matches!(&**receiver, Expr::Ident(id)
                            if Self::is_primitive_type_name(&id.name)
                                || self.types.contains_key(&id.name)
                                || self.type_meta.contains_key(&id.name));
                        let recv_llvm_ty = self.infer_llvm_type(receiver);
                        // Only scalar (integer/float) receivers get inline handling;
                        // structs use derived/user impls, pointers (Str) fall through.
                        let is_scalar = !receiver_is_type_name
                            && !recv_llvm_ty.starts_with("%struct.")
                            && recv_llvm_ty != "i8*"
                            && recv_llvm_ty != "void";
                        if is_scalar {
                            let recv_val = self.compile_expr(receiver)?;
                            let is_float = recv_llvm_ty == "double" || recv_llvm_ty == "float";
                            match fn_name.as_str() {
                                "clone" => return Ok(recv_val),
                                "hash" => {
                                    if is_float {
                                        let bits = if recv_llvm_ty == "double" { "i64" } else { "i32" };
                                        let cast = self.fresh_tmp();
                                        self.emitln(&format!("  {cast} = bitcast {recv_llvm_ty} {recv_val} to {bits}"));
                                        if bits == "i64" {
                                            return Ok(cast);
                                        }
                                        let ext = self.fresh_tmp();
                                        self.emitln(&format!("  {ext} = sext i32 {cast} to i64"));
                                        return Ok(ext);
                                    }
                                    if recv_llvm_ty == "i64" {
                                        return Ok(recv_val);
                                    }
                                    let ext = self.fresh_tmp();
                                    self.emitln(&format!("  {ext} = sext {recv_llvm_ty} {recv_val} to i64"));
                                    return Ok(ext);
                                }
                                _ => {}
                            }
                            let arg_val = if let Some(a) = args.first() {
                                self.compile_expr(a)?
                            } else {
                                "0".to_string()
                            };
                            if fn_name == "compare" {
                                let (lt_op, gt_op) = if is_float {
                                    ("fcmp olt", "fcmp ogt")
                                } else {
                                    ("icmp slt", "icmp sgt")
                                };
                                let lt = self.fresh_tmp();
                                let gt = self.fresh_tmp();
                                self.emitln(&format!("  {lt} = {lt_op} {recv_llvm_ty} {recv_val}, {arg_val}"));
                                self.emitln(&format!("  {gt} = {gt_op} {recv_llvm_ty} {recv_val}, {arg_val}"));
                                let s1 = self.fresh_tmp();
                                let res = self.fresh_tmp();
                                self.emitln(&format!("  {s1} = select i1 {gt}, i64 1, i64 0"));
                                self.emitln(&format!("  {res} = select i1 {lt}, i64 -1, i64 {s1}"));
                                return Ok(res);
                            }
                            let op = match (fn_name.as_str(), is_float) {
                                ("eq", false) => "icmp eq",  ("eq", true) => "fcmp oeq",
                                ("ne", false) => "icmp ne",  ("ne", true) => "fcmp one",
                                ("lt", false) => "icmp slt", ("lt", true) => "fcmp olt",
                                ("gt", false) => "icmp sgt", ("gt", true) => "fcmp ogt",
                                ("le", false) => "icmp sle", ("le", true) => "fcmp ole",
                                ("ge", false) => "icmp sge", ("ge", true) => "fcmp oge",
                                _ => unreachable!(),
                            };
                            let cmp = self.fresh_tmp();
                            self.emitln(&format!("  {cmp} = {op} {recv_llvm_ty} {recv_val}, {arg_val}"));
                            let res = self.fresh_tmp();
                            self.emitln(&format!("  {res} = zext i1 {cmp} to i64"));
                            return Ok(res);
                        }
                    }
                }
                // Check for memory allocation/free builtins
                if fn_name == "alloc" {
                    let tmp = self.fresh_tmp();
                    if let Some(size_arg) = args.first() {
                        let size_val = self.compile_expr(size_arg)?;
                        self.emitln(&format!("  {tmp} = call i8* @malloc(i64 {size_val})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i8* @malloc(i64 0)"));
                    }
                    return Ok(tmp);
                }
                if fn_name == "free" {
                    if let Some(ptr_arg) = args.first() {
                        let ptr_val = self.compile_expr(ptr_arg)?;
                        self.emitln(&format!("  call void @free(i8* {ptr_val})"));
                    }
                    return Ok("0".to_string());
                }
                if fn_name == "memcpy" && args.len() >= 3 {
                    let dest_val = self.compile_expr(&args[0])?;
                    let src_val = self.compile_expr(&args[1])?;
                    let size_val = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {dest_val}, i8* {src_val}, i64 {size_val}, i1 false)"));
                    return Ok("0".to_string());
                }
                // Vec.new() — static method on Vec type
                if let Some(receiver) = receiver_expr {
                    if let Expr::Ident(id) = &**receiver {
                        if id.name == "Vec" && fn_name == "new" {
                            let struct_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {struct_alloca} = alloca %struct.Vec"));
                            let data_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {data_ptr} = call i8* @malloc(i64 128)"));
                            // Null check on malloc — trap on OOM
                            let null_check = self.fresh_tmp();
                            let ok_block = self.fresh_block("vec_new_malloc_ok");
                            let trap_block = self.fresh_block("vec_new_malloc_trap");
                            self.emitln(&format!("  {null_check} = icmp eq i8* {data_ptr}, null"));
                            self.emitln(&format!("  br i1 {null_check}, label %{trap_block}, label %{ok_block}"));
                            self.emitln(&format!("\n{trap_block}:"));
                            self.emitln("  call void @llvm.trap()");
                            self.emitln("  unreachable");
                            self.emitln(&format!("\n{ok_block}:"));
                            let data_gep = self.fresh_tmp();
                            self.emitln(&format!("  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 0"));
                            self.emitln(&format!("  store i8* {data_ptr}, i8** {data_gep}"));
                            let len_gep = self.fresh_tmp();
                            self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 1"));
                            self.emitln(&format!("  store i64 0, i64* {len_gep}"));
                            let cap_gep = self.fresh_tmp();
                            self.emitln(&format!("  {cap_gep} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 2"));
                            self.emitln(&format!("  store i64 16, i64* {cap_gep}"));
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load %struct.Vec, %struct.Vec* {struct_alloca}"));
                            return Ok(loaded);
                        }
                    }
                }
                // Vec.push(vec, val) — method call on Vec
                if fn_name == "push" && args.len() >= 1 {
                    if let Some(receiver) = receiver_expr {
                        if self.infer_llvm_type(receiver) != "%struct.Vec" || self.infer_llvm_type(&args[0]) != "i64" {
                            // Not a Vec receiver or non-i64 element type — fall through to general method dispatch
                        } else {
                        let recv_val = self.compile_expr(receiver)?;
                        let val = self.compile_expr(&args[0])?;
                        let vec_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                        self.emitln(&format!("  store %struct.Vec {recv_val}, %struct.Vec* {vec_alloca}"));
                        let len_gep = self.fresh_tmp();
                        let len_val = self.fresh_tmp();
                        self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
                        self.emitln(&format!("  {len_val} = load i64, i64* {len_gep}"));
                        let cap_gep = self.fresh_tmp();
                        let cap_val = self.fresh_tmp();
                        self.emitln(&format!("  {cap_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 2"));
                        self.emitln(&format!("  {cap_val} = load i64, i64* {cap_gep}"));
                        let cap_check = self.fresh_tmp();
                        self.emitln(&format!("  {cap_check} = icmp ult i64 {len_val}, {cap_val}"));
                        let grow_block = self.fresh_block("vec_grow");
                        let store_block = self.fresh_block("vec_store");
                        self.emitln(&format!("  br i1 {cap_check}, label %{store_block}, label %{grow_block}"));
                        self.emitln(&format!("\n{grow_block}:"));
                        let new_cap = self.fresh_tmp();
                        self.emitln(&format!("  {new_cap} = mul i64 {cap_val}, 2"));
                        // Capacity guard: trap if exceeding max (2^20 elements ≈ 8MB)
                        let cap_ok_check = self.fresh_tmp();
                        let cap_ok_cont = self.fresh_block("vec_cap_ok");
                        let cap_trap_block = self.fresh_block("vec_cap_trap");
                        self.emitln(&format!("  {cap_ok_check} = icmp ule i64 {new_cap}, 1048576"));
                        self.emitln(&format!("  br i1 {cap_ok_check}, label %{cap_ok_cont}, label %{cap_trap_block}"));
                        self.emitln(&format!("\n{cap_trap_block}:"));
                        self.emitln("  call void @llvm.trap()");
                        self.emitln("  unreachable");
                        self.emitln(&format!("\n{cap_ok_cont}:"));
                        let new_size = self.fresh_tmp();
                        self.emitln(&format!("  {new_size} = mul i64 {new_cap}, 8"));
                        let grow_data_gep = self.fresh_tmp();
                        let grow_data_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {grow_data_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
                        self.emitln(&format!("  {grow_data_ptr} = load i8*, i8** {grow_data_gep}"));
                        let new_data = self.fresh_tmp();
                        self.emitln(&format!("  {new_data} = call i8* @realloc(i8* {grow_data_ptr}, i64 {new_size})"));
                        // Null check on realloc — trap on OOM
                        let grow_null_check = self.fresh_tmp();
                        let grow_ok_block = self.fresh_block("vec_realloc_ok");
                        let grow_trap_block = self.fresh_block("vec_realloc_trap");
                        self.emitln(&format!("  {grow_null_check} = icmp eq i8* {new_data}, null"));
                        self.emitln(&format!("  br i1 {grow_null_check}, label %{grow_trap_block}, label %{grow_ok_block}"));
                        self.emitln(&format!("\n{grow_trap_block}:"));
                        self.emitln("  call void @llvm.trap()");
                        self.emitln("  unreachable");
                        self.emitln(&format!("\n{grow_ok_block}:"));
                        self.emitln(&format!("  store i8* {new_data}, i8** {grow_data_gep}"));
                        let grow_cap_gep = self.fresh_tmp();
                        self.emitln(&format!("  {grow_cap_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 2"));
                        self.emitln(&format!("  store i64 {new_cap}, i64* {grow_cap_gep}"));
                        self.emitln(&format!("  br label %{store_block}"));
                        self.emitln(&format!("\n{store_block}:"));
                        let store_data_gep = self.fresh_tmp();
                        let store_data_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {store_data_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
                        self.emitln(&format!("  {store_data_ptr} = load i8*, i8** {store_data_gep}"));
                        let offset = self.fresh_tmp();
                        self.emitln(&format!("  {offset} = mul i64 {len_val}, 8"));
                        let dest = self.fresh_tmp();
                        self.emitln(&format!("  {dest} = getelementptr i8, i8* {store_data_ptr}, i64 {offset}"));
                        self.emitln(&format!("  store i64 {val}, i64* {dest}"));
                        let new_len = self.fresh_tmp();
                        self.emitln(&format!("  {new_len} = add i64 {len_val}, 1"));
                        self.emitln(&format!("  store i64 {new_len}, i64* {len_gep}"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load %struct.Vec, %struct.Vec* {vec_alloca}"));
                        return Ok(loaded);
                        }
                    }
                }
                // Vec.len(vec) — method call on Vec
                if fn_name == "len" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        if self.infer_llvm_type(receiver) != "%struct.Vec" {
                            // Not a Vec receiver — fall through to general method dispatch
                        } else {
                        let recv_val = self.compile_expr(receiver)?;
                        let vec_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                        self.emitln(&format!("  store %struct.Vec {recv_val}, %struct.Vec* {vec_alloca}"));
                        let len_gep = self.fresh_tmp();
                        let len_val = self.fresh_tmp();
                        self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
                        self.emitln(&format!("  {len_val} = load i64, i64* {len_gep}"));
                        return Ok(len_val);
                        }
                    }
                }
                // Str.len(s) — method call on Str
                if fn_name == "len" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        if recv_ty == "i8*" || recv_ty == "ptr" {
                            let recv_val = self.compile_expr(receiver)?;
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = call i64 @xiom_str_len(i8* {recv_val})"));
                            return Ok(tmp);
                        }
                    }
                }
                let compiled_args: Vec<String> = args.iter()
                    .map(|a| self.compile_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;
                if fn_name == "io" {
                    if let Some(arg) = compiled_args.first() {
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i32 @puts(i8* {arg})"));
                        return Ok(tmp);
                    }
                    return Ok("0".to_string());
                }
                // Extern runtime functions for file I/O
                if fn_name == "xiom_read_file" {
                    let tmp = self.fresh_tmp();
                    if let Some(path_arg) = args.first() {
                        let path_ptr = self.compile_expr(path_arg)?;
                        self.emitln(&format!("  {tmp} = call i8* @xiom_read_file(i8* {path_ptr})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i8* @xiom_read_file(i8* null)"));
                    }
                    let tmp_int = self.fresh_tmp();
                    self.emitln(&format!("  {tmp_int} = ptrtoint i8* {tmp} to i64"));
                    return Ok(tmp_int);
                }
                if fn_name == "xiom_file_size" {
                    let tmp = self.fresh_tmp();
                    if let Some(path_arg) = args.first() {
                        let path_ptr = self.compile_expr(path_arg)?;
                        self.emitln(&format!("  {tmp} = call i64 @xiom_file_size(i8* {path_ptr})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i64 @xiom_file_size(i8* null)"));
                    }
                    return Ok(tmp);
                }
                if fn_name == "xiom_free" {
                    if let Some(ptr_arg) = args.first() {
                        let ptr_val = self.compile_expr(ptr_arg)?;
                        let ptr_ty = self.infer_llvm_type(ptr_arg);
                        let ptr_ptr = self.val_to_i8ptr(&ptr_val, &ptr_ty);
                        self.emitln(&format!("  call void @xiom_free(i8* {ptr_ptr})"));
                    }
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_char_at" && args.len() >= 2 {
                    let src = self.compile_expr(&args[0])?;
                    let src_ty = self.infer_llvm_type(&args[0]);
                    let pos = self.compile_expr(&args[1])?;
                    let tmp = self.fresh_tmp();
                    let tmp_ext = self.fresh_tmp();
                    let src_ptr = self.val_to_i8ptr(&src, &src_ty);
                    self.emitln(&format!("  {tmp} = call i8 @xiom_char_at(i8* {src_ptr}, i64 {pos})"));
                    self.emitln(&format!("  {tmp_ext} = zext i8 {tmp} to i64"));
                    return Ok(tmp_ext);
                }
                if fn_name == "xiom_str_len" && args.len() >= 1 {
                    let src = self.compile_expr(&args[0])?;
                    let src_ty = self.infer_llvm_type(&args[0]);
                    let tmp = self.fresh_tmp();
                    let src_ptr = self.val_to_i8ptr(&src, &src_ty);
                    self.emitln(&format!("  {tmp} = call i64 @xiom_str_len(i8* {src_ptr})"));
                    return Ok(tmp);
                }
                // v0.9.4 string-based IR emission externs
                if fn_name == "xiom_ir_define_s" && args.len() >= 2 {
                    let name = self.compile_expr(&args[0])?;
                    let ret_type = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @xiom_ir_define_s(i8* {name}, i8* {ret_type})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_param_int" && args.len() >= 1 {
                    let index = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_param_int(i64 {index})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_param_double" && args.len() >= 1 {
                    let index = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_param_double(i64 {index})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_alloca_s" && args.len() >= 1 {
                    let reg = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_alloca_s(i64 {reg})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_store_param" && args.len() >= 2 {
                    let reg = self.compile_expr(&args[0])?;
                    let param = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @xiom_ir_store_param(i64 {reg}, i64 {param})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_load_s" && args.len() >= 2 {
                    let reg = self.compile_expr(&args[0])?;
                    let from_reg = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @xiom_ir_load_s(i64 {reg}, i64 {from_reg})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_add" && args.len() >= 3 {
                    let dst = self.compile_expr(&args[0])?;
                    let left = self.compile_expr(&args[1])?;
                    let right = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @xiom_ir_add(i64 {dst}, i64 {left}, i64 {right})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_fmul" && args.len() >= 3 {
                    let dst = self.compile_expr(&args[0])?;
                    let left = self.compile_expr(&args[1])?;
                    let right = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @xiom_ir_fmul(i64 {dst}, i64 {left}, i64 {right})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_call_fn" && args.len() >= 3 {
                    let dst = self.compile_expr(&args[0])?;
                    let fn_name_str = self.compile_expr(&args[1])?;
                    let ret_type = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @xiom_ir_call_fn(i64 {dst}, i8* {fn_name_str}, i8* {ret_type})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_call_arg_lit" && args.len() >= 2 {
                    let ty = self.compile_expr(&args[0])?;
                    let val = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @xiom_ir_call_arg_lit(i8* {ty}, i8* {val})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_ret_reg" && args.len() >= 1 {
                    let reg = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_ret_reg(i64 {reg})"));
                    return Ok("0".to_string());
                }
                if fn_name == "xiom_ir_ret_lit" && args.len() >= 1 {
                    let val = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_ret_lit(i64 {val})"));
                    return Ok("0".to_string());
                }
                // Check if this is a call to a generic function and track instantiation
                let fn_key = if let Some(receiver) = receiver_expr {
                    if let Some(recv_type) = self.infer_struct_type_name(receiver) {
                        format!("{}.{}", recv_type, fn_name)
                    } else {
                        // Receiver is a module name (not a struct type) — resolve
                        // to module-qualified function name if registered.
                        self.resolve_module_call(receiver, &fn_name)
                    }
                } else {
                    fn_name.clone()
                };
                let is_generic = self.generic_fn_decls.iter().any(|f| self.fn_key(f) == fn_key);
                if is_generic {
                    // Infer concrete types from argument types
                    let mut concrete_types: Vec<String> = Vec::new();
                    // Find the generic function declaration
                    if let Some(fd) = self.generic_fn_decls.iter().find(|f| self.fn_key(f) == fn_key) {
                        for gp in &fd.generics {
                            // Find a function parameter whose type directly uses this generic (not wrapped)
                            let mut inferred = false;
                            for (param, arg_expr) in fd.params.iter().zip(args.iter()) {
                                let param_type = Self::type_from_ast(&param.ty);
                                if param_type == gp.name.name {
                                    let concrete_ty = match arg_expr {
                                        Expr::Int(..) => "Int".to_string(),
                                        Expr::Float(..) => "Float64".to_string(),
                                        Expr::Bool(..) => "Bool".to_string(),
                                        Expr::Str(..) => "Str".to_string(),
                                        Expr::Char(..) => "Char".to_string(),
                                        Expr::Ident(id) => {
                                            if let Some(concrete) = self.param_concrete_types.get(&id.name) {
                                                concrete.clone()
                                            } else if let Some((_, llvm_ty)) = self.lookup_local(&id.name) {
                                                Self::xiom_type_name_from_llvm(llvm_ty)
                                            } else {
                                                gp.name.name.clone()
                                            }
                                        }
                                        _ => "Int".to_string(),
                                    };
                                    concrete_types.push(concrete_ty);
                                    inferred = true;
                                    break;
                                }
                            }
                            if !inferred {
                                // Fallback: use the first argument's outer type
                                if let Some(arg_expr) = args.first() {
                                    let concrete_ty = match arg_expr {
                                        Expr::Int(..) => "Int".to_string(),
                                        Expr::Float(..) => "Float64".to_string(),
                                        Expr::Bool(..) => "Bool".to_string(),
                                        Expr::Str(..) => "Str".to_string(),
                                        Expr::Char(..) => "Char".to_string(),
                                        Expr::Ident(id) => {
                                            if let Some(concrete) = self.param_concrete_types.get(&id.name) {
                                                concrete.clone()
                                            } else if let Some((_, llvm_ty)) = self.lookup_local(&id.name) {
                                                Self::xiom_type_name_from_llvm(llvm_ty)
                                            } else {
                                                gp.name.name.clone()
                                            }
                                        }
                                        _ => "Int".to_string(),
                                    };
                                    concrete_types.push(concrete_ty);
                                }
                            }
                        }
                    }
                    if !concrete_types.is_empty() {
                        let specialized_name = self.monomorphised_fn_name(&fn_key, &concrete_types);
                        // Record this instantiation if not already tracked
                        let already_tracked = self.generic_instantiations.iter()
                            .any(|(f, cts)| f == &fn_key && cts == &concrete_types);
                        if !already_tracked {
                            self.generic_instantiations.push((fn_key.clone(), concrete_types.clone()));
                        }
                        // Call the specialized version
                        let (ret_ty, param_types) = if let Some((pts, rt)) = self.functions.get(&specialized_name) {
                            (rt.clone(), pts.clone())
                        } else {
                            // Not yet registered - use the generic signature with
                            // argument-inferred param types (generic placeholder i64
                            // may be wrong for types like Str → i8*)
                            let inferred_types: Vec<String> = args.iter()
                                .map(|a| self.infer_llvm_type(a))
                                .collect();
                            let generic_ret = self.functions.get(&fn_key)
                                .map(|(_, rt)| rt.clone())
                                .unwrap_or_else(|| "i64".to_string());
                            (generic_ret, inferred_types)
                        };
                        // Include receiver argument only if it's an actual struct instance
                        // AND it's not already in the registered param_types
                        let mut all_args = compiled_args.clone();
                        let mut all_param_types = param_types.clone();
                        if let Some(receiver) = receiver_expr {
                            let is_instance = match receiver.as_ref() {
                                Expr::Ident(ident) => self.lookup_local(&ident.name).is_some(),
                                _ => true,
                            };
                            // Check if param_types already includes a receiver (from monomorphised registration)
                            let has_receiver_in_params = !all_param_types.is_empty() && all_param_types.len() > all_args.len();
                            if is_instance {
                                let recv_val = self.compile_expr(receiver)?;
                                let recv_llvm_ty = self.infer_llvm_type(receiver);
                                if has_receiver_in_params {
                                    // Receiver type already in param_types, just need the value
                                    all_args.insert(0, recv_val);
                                } else {
                                    all_param_types.insert(0, recv_llvm_ty);
                                    all_args.insert(0, recv_val);
                                }
                            } else if has_receiver_in_params {
                                // Type name or module name receiver — remove extra param type
                                all_param_types.remove(0);
                            }
                        }
                        let args_str = all_param_types.iter().zip(all_args.iter())
                            .map(|(ty, arg)| format!("{ty} {arg}"))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let tmp = self.fresh_tmp();
                        if ret_ty == "void" {
                            self.emitln(&format!("  call void @{specialized_name}({args_str})"));
                            Ok(tmp)
                        } else {
                            self.emitln(&format!("  {tmp} = call {ret_ty} @{specialized_name}({args_str})"));
                            Ok(tmp)
                        }
                    } else {
                        Ok("0".to_string())
                    }
                } else {
                    // For method calls, resolve the fully qualified function name
                    let resolved_fn_key = if let Some(receiver) = receiver_expr {
                        let recv_type = self.infer_struct_type_name(receiver);
                        if let Some(rt) = recv_type {
                            format!("{}.{}", rt, fn_name)
                        } else if !self.current_type_map.is_empty() {
                            // Check if receiver is a generic param being monomorphised
                            let obj_var_name = match &**receiver {
                                Expr::Ident(id) => id.name.clone(),
                                _ => String::new(),
                            };
                            if !obj_var_name.is_empty() {
                                if let Some(concrete_type) = self.param_concrete_types.get(&obj_var_name) {
                                    format!("{}.{}", concrete_type, fn_name)
                                } else {
                                    fn_key.clone()
                                }
                            } else {
                                fn_key.clone()
                            }
                        } else {
                            // Receiver is a module name — use module-qualified resolution
                            self.resolve_module_call(receiver, &fn_name)
                        }
                    } else {
                        fn_key.clone()
                    };
                    let args_str = if let Some(receiver) = receiver_expr {
                        // Check if receiver is a real struct instance (local variable)
                        // vs a type name (TrafficLight.xxx()) or module name (pipeline.xxx())
                        let is_instance = match receiver.as_ref() {
                            Expr::Ident(ident) => self.lookup_local(&ident.name).is_some(),
                            _ => true, // complex receiver expressions (e.g. chained calls) are instances
                        };
                        if is_instance {
                            let recv_val = self.compile_expr(receiver)?;
                            let recv_llvm_ty = self.infer_llvm_type(receiver);
                            let rest_str: Vec<String> = args.iter().zip(compiled_args.iter())
                                .map(|(arg_expr, arg_val)| format!("{} {}", self.infer_llvm_type(arg_expr), arg_val))
                                .collect();
                            if rest_str.is_empty() {
                                format!("{recv_llvm_ty} {recv_val}")
                            } else {
                                format!("{recv_llvm_ty} {recv_val}, {}", rest_str.join(", "))
                            }
                        } else {
                            // Type name or module name — no receiver argument
                            args.iter().zip(compiled_args.iter())
                                .map(|(arg_expr, arg_val)| format!("{} {}", self.infer_llvm_type(arg_expr), arg_val))
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    } else {
                        // Use registered param types when available (correct for extern
                        // functions with non-default types like Int32→i32, Float32→float).
                        // Fall back to inferred expression types otherwise.
                        let use_registered = self.functions.get(&resolved_fn_key)
                            .map(|(pts, _)| pts.len() == compiled_args.len())
                            .unwrap_or(false);
                        if use_registered {
                            let pts = &self.functions[&resolved_fn_key].0;
                            pts.iter().zip(compiled_args.iter())
                                .map(|(ty, arg)| format!("{ty} {arg}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        } else {
                            args.iter().zip(compiled_args.iter())
                                .map(|(arg_expr, arg_val)| format!("{} {}", self.infer_llvm_type(arg_expr), arg_val))
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    };
                    let tmp = self.fresh_tmp();
                    let ret_ty = if let Some((_, rt)) = self.functions.get(&resolved_fn_key) {
                        rt.clone()
                    } else {
                        // Fallback: try current-module qualified name
                        let mut found = String::new();
                        if let Some(ref module) = self.current_module {
                            let qualified = format!("{module}.{resolved_fn_key}");
                            if let Some((_, rt)) = self.functions.get(&qualified) {
                                found = rt.clone();
                            }
                        }
                        // Fallback: search for any key ending with .resolved_fn_key
                        if found.is_empty() {
                            let suffix = format!(".{resolved_fn_key}");
                            for (k, (_, rt)) in &self.functions {
                                if k.ends_with(&suffix) && rt.starts_with("%struct.") {
                                    found = rt.clone();
                                    break;
                                }
                            }
                        }
                        if found.is_empty() {
                            "i64".to_string()
                        } else {
                            found
                        }
                    };
                    let callee_is_fn_ptr = receiver_expr.is_none()
                        && self.lookup_local(&fn_name).is_some()
                        && self.functions.get(&resolved_fn_key).is_none()
                        && ret_ty == "i64";
                    if callee_is_fn_ptr {
                        let (alloca_reg, local_llvm_ty) = self.lookup_local(&fn_name).cloned().unwrap();
                        let fn_ptr_loaded = self.fresh_tmp();
                        self.emitln(&format!("  {fn_ptr_loaded} = load {local_llvm_ty}, {local_llvm_ty}* {alloca_reg}"));
                        let param_types: Vec<String> = args.iter().map(|a| self.infer_llvm_type(a)).collect();
                        let actual_ret_ty = if ret_ty == "i64" {
                            self.fn_ptr_return_types.get(&fn_name).cloned().unwrap_or_else(|| "i64".to_string())
                        } else {
                            ret_ty.clone()
                            };
                        let fn_ptr_ty = format!("{actual_ret_ty} ({})*", param_types.join(", "));
                        let fn_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {fn_ptr} = inttoptr {local_llvm_ty} {fn_ptr_loaded} to {fn_ptr_ty}"));
                        if actual_ret_ty == "void" {
                            self.emitln(&format!("  call {fn_ptr_ty} {fn_ptr}({args_str})"));
                        } else {
                            self.emitln(&format!("  {tmp} = call {actual_ret_ty} {fn_ptr}({args_str})"));
                        }
                        Ok(tmp)
                    } else if ret_ty == "void" {
                        self.emitln(&format!("  call void @{resolved_fn_key}({args_str})"));
                        Ok(tmp)
                    } else {
                        self.emitln(&format!("  {tmp} = call {ret_ty} @{resolved_fn_key}({args_str})"));
                        Ok(tmp)
                    }
                }
            }
            Expr::Index(_, _, _) => Ok("0".to_string()),
            Expr::AtPre(inner, _) => {
                // If inner is `self`, resolve to __self_pre (the pre-state snapshot)
                if let Expr::Ident(id) = inner.as_ref() {
                    if id.name == "self" {
                        if let Some((ptr, llvm_ty)) = self.lookup_local("__self_pre").cloned() {
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                            return Ok(tmp);
                        }
                    }
                    // For other @pre expressions e.g. self.field@pre, compile as field access
                    // from self@pre (handled recursively via the self case above)
                }
                self.compile_expr(inner)
            }
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => self.compile_expr(inner),
            Expr::Some(inner, _) => {
                self.used_builtins.insert("Option".to_string());
                let val = self.compile_expr(inner)?;
                let inner_ty = self.infer_llvm_type(inner);
                let store_val = self.val_to_i64(&val, &inner_ty);
                let opt_ty = "%struct.Option";
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {opt_ty}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {opt_ty}, {opt_ty}* {alloca}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 1, i64* {gep0}"));
                let gep1 = self.fresh_tmp();
                self.emitln(&format!("  {gep1} = getelementptr {opt_ty}, {opt_ty}* {alloca}, i32 0, i32 1"));
                self.emitln(&format!("  store i64 {store_val}, i64* {gep1}"));
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {opt_ty}, {opt_ty}* {alloca}"));
                Ok(loaded)
            }
            Expr::None(_) => {
                self.used_builtins.insert("Option".to_string());
                let opt_ty = "%struct.Option";
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {opt_ty}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {opt_ty}, {opt_ty}* {alloca}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 0, i64* {gep0}"));
                let gep1 = self.fresh_tmp();
                self.emitln(&format!("  {gep1} = getelementptr {opt_ty}, {opt_ty}* {alloca}, i32 0, i32 1"));
                self.emitln(&format!("  store i64 0, i64* {gep1}"));
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {opt_ty}, {opt_ty}* {alloca}"));
                Ok(loaded)
            }
            Expr::Ok(inner, _) => {
                self.used_builtins.insert("Result".to_string());
                let val = self.compile_expr(inner)?;
                let inner_ty = self.infer_llvm_type(inner);
                let store_val = self.val_to_i64(&val, &inner_ty);
                let result_ty = "%struct.Result";
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {result_ty}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 1, i64* {gep0}"));
                let gep1 = self.fresh_tmp();
                self.emitln(&format!("  {gep1} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 1"));
                self.emitln(&format!("  store i64 {store_val}, i64* {gep1}"));
                let gep2 = self.fresh_tmp();
                self.emitln(&format!("  {gep2} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 2"));
                self.emitln(&format!("  store i64 0, i64* {gep2}"));
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {result_ty}, {result_ty}* {alloca}"));
                Ok(loaded)
            }
            Expr::Err(inner, _) => {
                self.used_builtins.insert("Result".to_string());
                let val = self.compile_expr(inner)?;
                let inner_ty = self.infer_llvm_type(inner);
                let store_val = self.val_to_i64(&val, &inner_ty);
                let result_ty = "%struct.Result";
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {result_ty}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 0, i64* {gep0}"));
                let gep1 = self.fresh_tmp();
                self.emitln(&format!("  {gep1} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 1"));
                self.emitln(&format!("  store i64 0, i64* {gep1}"));
                let gep2 = self.fresh_tmp();
                self.emitln(&format!("  {gep2} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 2"));
                self.emitln(&format!("  store i64 {store_val}, i64* {gep2}"));
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {result_ty}, {result_ty}* {alloca}"));
                Ok(loaded)
            }
            Expr::Struct(name, fields, _spread, _span) => {
                // If `name` is an enum variant (e.g., `Single`), resolve to parent enum type
                let parent_enum = self.enum_variants.iter()
                    .find(|(_, vars)| vars.iter().any(|(v, _)| v == &name.name))
                    .map(|(ek, _)| ek.clone());
                let struct_ty = if let Some(ref ek) = parent_enum {
                    self.llvm_type_for(ek)?
                } else {
                    self.llvm_type_for_fallback(&name.name)
                };
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                if let Some(ref enum_key) = parent_enum {
                    // Set discriminant (field 0) to the variant index
                    let var_idx = self.enum_variants.get(enum_key)
                        .and_then(|vars| vars.iter().position(|(v, _)| v == &name.name))
                        .unwrap_or(0);
                    let disc_gep = self.fresh_tmp();
                    self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                    self.emitln(&format!("  store i64 {var_idx}, i64* {disc_gep}"));
                    // Map variant fields to their parent enum offsets (after discriminant)
                    // The parent enum stores field names uniquely across all variants,
                    // so we need to look up the actual field index in the parent's field list.
                    let parent_field_names = self.types.get(enum_key).cloned().unwrap_or_default();
                    let variant_fields = self.enum_variants.get(enum_key)
                        .and_then(|vars| vars.iter().find(|(v, _)| v == &name.name))
                        .map(|(_, vf)| vf.clone())
                        .unwrap_or_default();
                    for (i, (_, val)) in fields.iter().enumerate() {
                        let field_val = self.compile_expr(val)?;
                        // Find the actual parent field index for this variant field
                        let field_name = variant_fields.get(i).cloned().unwrap_or_default();
                        let parent_field_idx = parent_field_names.iter()
                            .position(|f| f == &field_name)
                            .unwrap_or(i + 1);
                        let field_llvm_ty = self.field_llvm_type(enum_key, parent_field_idx);
                        let store_val = if field_val == "0" && (field_llvm_ty.ends_with('*') || field_llvm_ty.contains('*')) {
                            "null".to_string()
                        } else if field_llvm_ty.ends_with('*') && field_val.chars().all(|c| c.is_ascii_digit() || c == '-') {
                            let ptr_tmp = self.fresh_tmp();
                            self.emitln(&format!("  {ptr_tmp} = inttoptr i64 {field_val} to {field_llvm_ty}"));
                            ptr_tmp
                        } else { field_val };
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {parent_field_idx}"));
                        self.emitln(&format!("  store {field_llvm_ty} {store_val}, {field_llvm_ty}* {gep}"));
                    }
                } else {
                    for (i, (_, val)) in fields.iter().enumerate() {
                        let field_val = self.compile_expr(val)?;
                        let mut field_llvm_ty = self.field_llvm_type(&name.name, i);
                        // For generic types, field_llvm_type may return "i64" for unresolved type params (like T).
                        // Fall back to the field value expression's inferred LLVM type.
                        if field_llvm_ty == "i64" {
                            let val_ty = self.infer_llvm_type(val);
                            if val_ty != "i64" {
                                field_llvm_ty = val_ty;
                            }
                        }
                        let store_val = if field_val == "0" && (field_llvm_ty.ends_with('*') || field_llvm_ty.starts_with('\"')) {
                            "null".to_string()
                        } else if field_llvm_ty.ends_with('*') && field_val.chars().all(|c| c.is_ascii_digit() || c == '-') {
                            let ptr_tmp = self.fresh_tmp();
                            self.emitln(&format!("  {ptr_tmp} = inttoptr i64 {field_val} to {field_llvm_ty}"));
                            ptr_tmp
                        } else {
                            field_val
                        };
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
                        self.emitln(&format!("  store {field_llvm_ty} {store_val}, {field_llvm_ty}* {gep}"));
                    }
                }
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
                // Emit invariant check after struct creation
                if self.check_contracts {
                    if let Some(meta) = self.type_meta.get(&name.name) {
                        if !meta.invariants.is_empty() {
                            self.compile_invariant_call(&name.name, &loaded);
                        }
                    }
                }
                Ok(loaded)
            }
            Expr::Array(_, _) => Ok("0".to_string()),
            Expr::Closure(_, _, _, _) | Expr::PipeClosure(_, _, _) => Ok("0".to_string()),
            Expr::As(inner, ty, _) => {
                let val = self.compile_expr(inner)?;
                let mut inner_llvm_ty = self.infer_llvm_type(inner);
                if inner_llvm_ty == "i64" {
                    if let Expr::Ident(id) = inner.as_ref() {
                        if let Some(concrete) = self.param_concrete_types.get(&id.name) {
                            let cty = self.llvm_type_for_fallback(concrete);
                            if cty == "double" {
                                inner_llvm_ty = "double".to_string();
                            }
                        }
                    }
                }
                let target_llvm_ty = self.llvm_type_for_fallback(&Self::type_from_ast(ty));
                let tmp = self.fresh_tmp();
                // Bit width of an LLVM integer type name, or None if not an integer type.
                let int_width = |t: &str| -> Option<u32> {
                    match t {
                        "i1" => Some(1),
                        "i8" => Some(8),
                        "i16" => Some(16),
                        "i32" => Some(32),
                        "i64" => Some(64),
                        _ => None,
                    }
                };
                match (inner_llvm_ty.as_str(), target_llvm_ty.as_str()) {
                    ("i64", "double") => {
                        self.emitln(&format!("  {tmp} = sitofp i64 {val} to double"));
                        Ok(tmp)
                    }
                    ("double", "i64") => {
                        self.emitln(&format!("  {tmp} = fptosi double {val} to i64"));
                        Ok(tmp)
                    }
                    (a, b) if a == b => Ok(val),
                    // Integer <-> integer width conversions (e.g. Int<->Char, Int<->Int8/16/32).
                    // Char is i8 and Int is i64, so Int->Char truncs and Char->Int sign-extends.
                    (a, b) if int_width(a).is_some() && int_width(b).is_some() => {
                        let aw = int_width(a).unwrap();
                        let bw = int_width(b).unwrap();
                        if bw < aw {
                            self.emitln(&format!("  {tmp} = trunc {a} {val} to {b}"));
                            Ok(tmp)
                        } else {
                            self.emitln(&format!("  {tmp} = sext {a} {val} to {b}"));
                            Ok(tmp)
                        }
                    }
                    _ => Ok(val),
                }
            }
            Expr::Await(inner, _) => self.compile_expr(inner),
            Expr::Comptime(inner, _) => self.compile_expr(inner),
            Expr::Unsafe(block, _) => {
                for stmt in &block.stmts {
                    match stmt { xiom_ast::StmtOrExpr::Expr(e) => { self.compile_expr(e)?; } _ => {} }
                }
                Ok(String::new())
            }
            Expr::If(cond, then_block, elifs, else_block, _) => {
                self.compile_expr(cond)?;
                for stmt in &then_block.stmts {
                    match stmt { xiom_ast::StmtOrExpr::Expr(e) => { self.compile_expr(e)?; } _ => {} }
                }
                for (_econd, eblock) in elifs {
                    for stmt in &eblock.stmts {
                        match stmt { xiom_ast::StmtOrExpr::Expr(e) => { self.compile_expr(e)?; } _ => {} }
                    }
                }
                if let Some(eb) = else_block {
                    for stmt in &eb.stmts {
                        match stmt { xiom_ast::StmtOrExpr::Expr(e) => { self.compile_expr(e)?; } _ => {} }
                    }
                }
                Ok(String::new())
            }
            Expr::Match(scrutinee, arms, span) => {
                // Compile a match-expression by allocating a result slot, running the
                // statement-form match (whose arm bodies store their value into
                // `match_result_ptr`), then loading the slot as this expression's value.
                // TODO(match-expr-typing): the result LLVM type is inferred from the
                // first arm's tail expression. Arms whose value depends on
                // pattern-bound variables (e.g. enum payloads) may infer an
                // imprecise type; heterogeneous arm types are not yet unified.
                let result_ty = self.infer_match_llvm_type(arms);
                let result_alloca = self.fresh_tmp();
                self.emitln(&format!("  {result_alloca} = alloca {result_ty}"));
                let saved_ptr = self.match_result_ptr.take();
                let saved_ty = self.match_result_ty.take();
                self.match_result_ptr = Some(result_alloca.clone());
                self.match_result_ty = Some(result_ty.clone());
                let stmt = Stmt::Match((**scrutinee).clone(), arms.clone(), *span);
                self.compile_stmt(&stmt)?;
                self.match_result_ptr = saved_ptr;
                self.match_result_ty = saved_ty;
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {result_ty}, {result_ty}* {result_alloca}"));
                Ok(loaded)
            }
        }
    }

    /// Infer the LLVM result type of a match-expression from its arm bodies.
    /// Uses the first arm that yields a tail expression; defaults to `i64`.
    fn infer_match_llvm_type(&self, arms: &[MatchArm]) -> String {
        for arm in arms {
            let ty = match &arm.body {
                MatchBody::Expr(e) => self.infer_llvm_type(e),
                MatchBody::Block(b) => b.stmts.last().and_then(|s| {
                    if let StmtOrExpr::Expr(e) = s { Some(self.infer_llvm_type(e)) } else { None }
                }).unwrap_or_default(),
            };
            if !ty.is_empty() { return ty; }
        }
        "i64".to_string()
    }

    fn infer_struct_type_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(ident) => {
                if let Some((_, llvm_ty)) = self.lookup_local(&ident.name) {
                    if llvm_ty.starts_with("%struct.") {
                        return Some(llvm_ty[8..].to_string());
                    }
                }
                // Check if it's a type name (for static method calls like Rect.new(...))
                if self.types.contains_key(&ident.name) || self.type_meta.contains_key(&ident.name) {
                    return Some(ident.name.clone());
                }
                // Try current module's qualified name first (deterministic)
                if let Some(ref module) = self.current_module {
                    let qualified = format!("{}.{}", module, ident.name);
                    if self.type_meta.contains_key(&qualified) {
                        return Some(qualified);
                    }
                }
                // Fallback: search all qualified keys (last resort)
                for key in self.type_meta.keys() {
                    if key.ends_with(&format!(".{}", ident.name)) {
                        return Some(key.clone());
                    }
                }
                None
            }
            Expr::Struct(ident, _, _, _) => {
                // Try module-qualified name first, then bare name
                if let Some(ref module) = self.current_module {
                    let qualified = format!("{}.{}", module, ident.name);
                    if self.type_meta.contains_key(&qualified) {
                        return Some(qualified);
                    }
                }
                Some(ident.name.clone())
            },
            Expr::Field(obj, _, _) => {
                self.infer_struct_type_name(obj.as_ref())
            }
            Expr::Call(func, _, _) => {
                // Infer type from the return type of a method/function call
                if let Expr::Field(obj, field, _) = func.as_ref() {
                    if let Some(recv_type) = self.infer_struct_type_name(obj.as_ref()) {
                        let fn_key = format!("{}.{}", recv_type, field.name);
                        if let Some((_, ret_ty)) = self.functions.get(&fn_key) {
                            if ret_ty.starts_with("%struct.") {
                                return Some(ret_ty[8..].to_string());
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn infer_llvm_type(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(_, _) | Expr::Bool(_, _) => "i64".to_string(),
            Expr::Float(_, _) => "double".to_string(),
            Expr::Str(_, _) | Expr::Char(_, _) => "i8*".to_string(),
            Expr::Ident(ident) => {
                if let Some((_, llvm_ty)) = self.lookup_local(&ident.name) {
                    if llvm_ty == "double" { return "double".to_string(); }
                    return llvm_ty.clone();
                }
                // If the ident is an enum variant name (e.g., DivByZero), return the parent enum's struct type
                if let Some(enum_key) = self.enum_variants.iter()
                    .find(|(_, vars)| vars.iter().any(|(v, _)| v == &ident.name))
                    .map(|(ek, _)| ek)
                {
                    return format!("%struct.{enum_key}");
                }
                "i64".to_string()
            }
            Expr::Field(obj, field, _) => {
                // Resolve the LLVM type of a struct field access (e.g. r.w where r is Rect{w: Float64, ...})
                if let Expr::Ident(obj_ident) = obj.as_ref() {
                    if let Some((_, llvm_ty)) = self.lookup_local(&obj_ident.name) {
                        if llvm_ty.starts_with("%struct.") {
                            let type_name = &llvm_ty[8..];
                            if let Some(meta) = self.type_meta.get(type_name) {
                                if let Some((_, ty_name)) = meta.fields.iter().find(|(name, _)| name == &field.name) {
                                    return self.llvm_type_for(ty_name).unwrap_or_else(|_| "i64".to_string());
                                }
                            }
                        }
                    }
                }
                "i64".to_string()
            }
            Expr::Call(func, _, _) => {
                // Check for Vec.new() first
                if let Expr::Field(obj, field, _) = func.as_ref() {
                    if let Expr::Ident(id) = obj.as_ref() {
                        if id.name == "Vec" && field.name == "new" {
                            return "%struct.Vec".to_string();
                        }
                    }
                }
                let fn_name = match func.as_ref() {
                    Expr::Ident(name) => Some(name.name.clone()),
                    Expr::Field(obj, field, _) => {
                        // Try to resolve method call: obj.method → Type.method
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
                    if name == "xiom_read_file" { return "i64".to_string(); }
                    if name == "xiom_char_at" || name == "xiom_str_len" { return "i64".to_string(); }
                    if let Some((_, ret_ty)) = self.functions.get(name) {
                        if ret_ty == "double" { return "double".to_string(); }
                        return ret_ty.clone();
                    }
                    // Fallback: try current-module qualified name (e.g., "benchmark.main.make_result")
                    if let Some(ref module) = self.current_module {
                        let qualified = format!("{module}.{name}");
                        if let Some((_, ret_ty)) = self.functions.get(&qualified) {
                            if ret_ty == "double" { return "double".to_string(); }
                            return ret_ty.clone();
                        }
                    }
                    // Fallback: search for any key ending with .name that returns a struct
                    {
                        let suffix = format!(".{name}");
                        for (k, (_, rt)) in &self.functions {
                            if k.ends_with(&suffix) && rt.starts_with("%struct.") {
                                return rt.clone();
                            }
                        }
                    }
                    // Function pointer call — look up tracked return type
                    if self.lookup_local(name).is_some() {
                        if let Some(ret_ty) = self.fn_ptr_return_types.get(name) {
                            if ret_ty == "double" { return "double".to_string(); }
                            return ret_ty.clone();
                        }
                    }
                }
                "i64".to_string()
            }
            Expr::Some(..) | Expr::None(..) => {
                if self.types.contains_key("Option") { "%struct.Option".to_string() } else { "i64".to_string() }
            }
            Expr::Ok(..) | Expr::Err(..) => {
                if self.types.contains_key("Result") { "%struct.Result".to_string() } else { "i64".to_string() }
            }
            Expr::Struct(ident, _, _, _) => self.llvm_type_for(&ident.name).unwrap_or_else(|_| "i64".to_string()),
            Expr::Paren(inner, _) => self.infer_llvm_type(inner),
            Expr::Tuple(items, _) => {
                if items.is_empty() { "void".to_string() } else {
                    let parts: Vec<String> = items.iter().map(|i| {
                        let t = self.infer_llvm_type(i);
                        Self::xiom_type_name_from_llvm(&t)
                    }).collect();
                    let name = format!("Tuple_{}", parts.join("_"));
                    if self.types.contains_key(&name) || self.type_meta.contains_key(&name) {
                        format!("%struct.{name}")
                    } else {
                        "i64".to_string()
                    }
                }
            }
            Expr::Unary(op, inner, _) => {
                match op {
                    UnaryOp::Not | UnaryOp::BitNot => "i64".to_string(),
                    UnaryOp::Neg | UnaryOp::Ref | UnaryOp::MutRef | UnaryOp::Deref => self.infer_llvm_type(inner),
                }
            }
            Expr::Binary(left, op, right, _) => {
                match op {
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => "i64".to_string(),
                    BinOp::And | BinOp::Or | BinOp::Shl | BinOp::Shr | BinOp::BitXor | BinOp::BitAnd | BinOp::BitOr => "i64".to_string(),
                    BinOp::Assign => self.infer_llvm_type(right),
                    _ => {
                        if self.is_float_expr(left) || self.is_float_expr(right) { "double".to_string() } else { "i64".to_string() }
                    }
                }
            }
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => self.infer_llvm_type(inner),
            Expr::As(_, ty, _) => self.llvm_type_for(&Self::type_from_ast(ty)).unwrap_or_else(|_| "i64".to_string()),
            Expr::If(_cond, then_block, _elifs, else_block, _) => {
                // if-expressions return the type of the last expression in each branch
                let then_ty = then_block.stmts.last()
                    .and_then(|s| if let xiom_ast::StmtOrExpr::Expr(e) = s { Some(self.infer_llvm_type(e)) } else { None })
                    .unwrap_or_else(|| "i64".to_string());
                let else_ty = else_block.as_ref().and_then(|b| b.stmts.last()
                    .and_then(|s| if let xiom_ast::StmtOrExpr::Expr(e) = s { Some(self.infer_llvm_type(e)) } else { None }))
                    .unwrap_or_else(|| "i64".to_string());
                if then_ty == "double" || else_ty == "double" { "double".to_string() } else { "i64".to_string() }
            }
            Expr::Match(_scrutinee, arms, _) => self.infer_match_llvm_type(arms),
            _ => "i64".to_string(),
        }
    }

    /// Recursively determine if an expression involves float operations
    fn is_float_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Float(..) => true,
            Expr::Ident(_) => is_float_local(expr, &self.locals),
            Expr::Binary(left, _, right, _) => self.is_float_expr(left) || self.is_float_expr(right),
            Expr::Paren(inner, _) => self.is_float_expr(inner),
            Expr::Tuple(items, _) => items.iter().any(|i| self.is_float_expr(i)),
            Expr::Unary(_, inner, _) => self.is_float_expr(inner),
            Expr::Field(obj, field, _) => {
                if self.is_float_expr(obj) {
                    return true;
                }
                if let Expr::Ident(obj_ident) = obj.as_ref() {
                    if let Some((_, llvm_ty)) = self.lookup_local(&obj_ident.name) {
                        if llvm_ty.starts_with("%struct.") {
                            let type_name = &llvm_ty[8..];
                            if let Some(meta) = self.type_meta.get(type_name) {
                                if let Some((_, ty_name)) = meta.fields.iter().find(|(name, _)| name == &field.name) {
                                    return ty_name == "Float64" || ty_name == "Float32";
                                }
                            }
                        }
                    }
                }
                false
            }
            Expr::As(_, ty, _) => Self::type_from_ast(ty) == "Float64" || Self::type_from_ast(ty) == "Float32",
            Expr::Call(_, _, _) | Expr::If(..) => self.infer_llvm_type(expr) == "double",
            _ => false,
        }
    }
}

/// Check if a local named in an expression is a float type
fn is_float_local(expr: &Expr, locals: &[HashMap<String, (String, String)>]) -> bool {
    if let Expr::Ident(ident) = expr {
        for scope in locals.iter().rev() {
            if let Some((_, llvm_ty)) = scope.get(&ident.name) {
                return llvm_ty == "double";
            }
        }
    }
    false
}


