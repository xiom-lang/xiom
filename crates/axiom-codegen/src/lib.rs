//! AXIOM Codegen — Phase 0: AST → LLVM IR text.
//! Emits human-readable LLVM IR that can be compiled with `llc`.
//! No external dependencies — pure string emission.
//! Handles: functions, arithmetic, control flow (if/else/while/match),
//! let/var bindings, structs, function calls.

use axiom_ast::*;
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
        }
    }

    pub fn set_target_triple(&mut self, triple: &str) {
        self.target_triple = triple.to_string();
    }

    pub fn set_check_contracts(&mut self, enabled: bool) {
        self.check_contracts = enabled;
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

    fn axiom_to_llvm_type(axiom_ty: &str) -> &'static str {
        match axiom_ty {
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

    fn type_from_ast(ty: &Type) -> String {
        match ty {
            Type::Named(ident, _) => ident.name.clone(),
            Type::Ref(inner) => Self::type_from_ast(inner),
            Type::MutRef(inner) => Self::type_from_ast(inner),
            Type::Option(_) => "Option".to_string(),
            Type::Result(_, _) => "Result".to_string(),
            _ => "Int".to_string(),
        }
    }

    fn llvm_type_for(&self, type_name: &str) -> String {
        if self.types.contains_key(type_name) || self.type_meta.contains_key(type_name) {
            format!("%struct.{type_name}")
        } else {
            Self::axiom_to_llvm_type(type_name).to_string()
        }
    }

    fn field_llvm_type(&self, struct_name: &str, field_idx: usize) -> String {
        if let Some(meta) = self.type_meta.get(struct_name) {
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
        self.emitln("; AXIOM Phase 1 — LLVM IR");
        self.emitln("; Auto-generated by axiomc\n");
        self.emitln(&format!("target triple = \"{}\"", self.target_triple));
        self.emitln("");

        // Emit struct type definitions using actual field types from type_meta
        for (name, meta) in &self.type_meta.clone() {
            let field_types: Vec<String> = meta.fields.iter()
                .map(|(_, ty_name)| self.llvm_type_for(ty_name))
                .collect();
            self.emitln(&format!("%struct.{name} = type {{ {} }}", field_types.join(", ")));
        }
        if !self.types.is_empty() {
            self.emitln("");
        }

        // Emit Vec struct type for runtime operations
        self.emitln("%struct.Vec = type { i8*, i64, i64 }");
        self.emitln("");

        // Declare external C functions + LLVM intrinsics
        self.emitln("declare i32 @printf(i8*, ...)");
        self.emitln("declare i32 @puts(i8*)");
        self.emitln("declare void @llvm.trap()");
        self.emitln("declare i8* @malloc(i64)");
        self.emitln("declare void @free(i8*)");
        self.emitln("declare void @llvm.memcpy.p0i8.p0i8.i64(i8*, i8*, i64, i1)");
        self.emitln("declare i64 @axiom_is_sorted(i8*, i64)");
        self.emitln("declare i64 @axiom_all(i8*, i64, i8*)");
        self.emitln("declare i64 @axiom_none(i8*, i64, i8*)");
        self.emitln("declare i64 @axiom_contains(i8*, i64)");
        self.emitln("declare i8* @axiom_read_file(i8*)");
        self.emitln("declare i64 @axiom_file_size(i8*)");
        self.emitln("declare void @axiom_free(i8*)");
        self.emitln("declare i8 @axiom_char_at(i8*, i64)");
        self.emitln("declare i64 @axiom_str_len(i8*)");
        // String interning
        self.emitln("declare i64 @axiom_intern(i8*, i64, i64)");
        self.emitln("declare i8* @axiom_lookup(i64)");
        // IR emission
        self.emitln("declare i64 @axiom_ir_open(i8*)");
        self.emitln("declare void @axiom_ir_close()");
        self.emitln("declare void @axiom_ir_header()");
        self.emitln("declare void @axiom_ir_define(i64, i64)");
        self.emitln("declare void @axiom_ir_param(i64, i64)");
        self.emitln("declare void @axiom_ir_entry()");
        self.emitln("declare void @axiom_ir_alloca(i64, i64)");
        self.emitln("declare void @axiom_ir_store(i64, i64, i64)");
        self.emitln("declare void @axiom_ir_load(i64, i64, i64)");
        self.emitln("declare void @axiom_ir_binop(i8*, i64, i64, i64, i64)");
        self.emitln("declare void @axiom_ir_call(i64, i64, i64)");
        self.emitln("declare void @axiom_ir_call_arg(i64, i64)");
        self.emitln("declare void @axiom_ir_call_lit(i8*)");
        self.emitln("declare void @axiom_ir_call_end()");
        self.emitln("declare void @axiom_ir_ret(i64, i64)");
        self.emitln("declare void @axiom_ir_ret_void()");
        self.emitln("declare void @axiom_ir_endfn()");
        self.emitln("declare void @axiom_ir_raw(i8*)");
        self.emitln("declare void @axiom_ir_emit_program(i64)");
        // v0.9.4 string-based IR emission
        self.emitln("declare void @axiom_ir_define_s(i8*, i8*)");
        self.emitln("declare void @axiom_ir_param_int(i64)");
        self.emitln("declare void @axiom_ir_param_double(i64)");
        self.emitln("declare void @axiom_ir_alloca_s(i64)");
        self.emitln("declare void @axiom_ir_store_param(i64, i64)");
        self.emitln("declare void @axiom_ir_load_s(i64, i64)");
        self.emitln("declare void @axiom_ir_add(i64, i64, i64)");
        self.emitln("declare void @axiom_ir_fmul(i64, i64, i64)");
        self.emitln("declare void @axiom_ir_call_fn(i64, i8*, i8*)");
        self.emitln("declare void @axiom_ir_call_arg_lit(i8*, i8*)");
        self.emitln("declare void @axiom_ir_ret_reg(i64)");
        self.emitln("declare void @axiom_ir_ret_lit(i64)");
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
        if let TopDecl::Type(td) = item {
            let fields: Vec<String> = td.fields.iter()
                .map(|f| f.name.name.clone())
                .collect();
            self.types.insert(td.name.name.clone(), fields);
            let full_fields: Vec<(String, String)> = td.fields.iter()
                .map(|f| (f.name.name.clone(), Self::type_from_ast(&f.ty)))
                .collect();
            self.type_meta.insert(td.name.name.clone(), TypeMeta {
                fields: full_fields,
                derives: td.derives.clone(),
                invariants: td.invariants.clone(),
            });
        }
        if let TopDecl::Enum(ed) = item {
            let enum_name = ed.name.name.clone();
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
            for sub in &md.items {
                self.register_type_layout(sub);
            }
        }
    }

    fn register_functions(&mut self, item: &TopDecl) {
        if let TopDecl::Fn(fd) = item {
            let param_types: Vec<String> = fd.params.iter()
                .map(|p| self.llvm_type_for(&Self::type_from_ast(&p.ty)))
                .collect();
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
            for sub in &md.items {
                self.register_functions(sub);
            }
        }
    }

    fn fn_key(&self, fd: &FnDecl) -> String {
        if let Some(recv_name) = &fd.receiver {
            format!("{}.{}", recv_name.name, fd.name.name)
        } else {
            fd.name.name.clone()
        }
    }

    fn compile_top_decl(&mut self, item: &TopDecl) -> Result<(), String> {
        match item {
            TopDecl::Fn(fd) => {
                // Skip generic functions — they will be monomorphised later
                if fd.generics.is_empty() {
                    if fd.body.is_some() {
                        self.compile_fn(fd)?;
                    }
                }
                Ok(())
            }
            TopDecl::Module(md) => {
                for sub in &md.items {
                    self.compile_top_decl(sub)?;
                }
                Ok(())
            }
            TopDecl::Interface(_) | TopDecl::Enum(_) | TopDecl::Const(_) | TopDecl::Type(_) | TopDecl::Use(_) => Ok(()),
        }
    }

    fn compile_fn(&mut self, fd: &FnDecl) -> Result<(), String> {
        self.push_scope();
        self.block_counter = 0;
        self.tmp_counter = 0;

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

        let params_str: Vec<String> = fd.params.iter()
            .enumerate()
            .map(|(i, p)| {
                let llvm_ty = self.llvm_type_for(&Self::type_from_ast(&p.ty));
                format!("{llvm_ty} %param{i}")
            })
            .collect();

        self.emitln(&format!("define {ret_llvm} @{name}({}) {{", params_str.join(", ")));

        // Entry block
        let entry_block = self.fresh_block("entry");
        self.emitln(&format!("{entry_block}:"));

        // Allocate parameters as locals
        for (i, param) in fd.params.iter().enumerate() {
            let llvm_ty = self.llvm_type_for(&Self::type_from_ast(&param.ty));
            let alloca = self.fresh_tmp();
            self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
            self.emitln(&format!("  store {llvm_ty} %param{i}, {llvm_ty}* {alloca}"));
            self.add_local(&param.name.name, alloca, &llvm_ty);
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
                    Expr::Field(_, field, _) => Some(field.name.clone()),
                    _ => None,
                };
                if let Some(name) = fn_name {
                    // Check if the known return type is a struct
                    if self.type_meta.contains_key(&name) {
                        return Some(name);
                    }
                }
                None
            }
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
        if ty == "double" {
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = bitcast double {val} to i64"));
            bc
        } else if ty == "i8*" || ty.contains('*') {
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = ptrtoint {ty} {val} to i64"));
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
                let type_name = &td.name.name;
                let field_names: Vec<String> = td.fields.iter().map(|f| f.name.name.clone()).collect();
                let struct_ty = format!("%struct.{type_name}");

                for derive in &td.derives {
                    match derive {
                        DeriveTrait::Eq => self.compile_eq_impl(type_name, &struct_ty, &field_names, &td.fields)?,
                        DeriveTrait::Clone => self.compile_clone_impl(type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Display => self.compile_display_impl(type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Hash => self.compile_hash_impl(type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Ord => self.compile_ord_impl(type_name, &struct_ty, &field_names, &td.fields)?,
                    }
                }

                // Generate invariant check function if needed (even without derives)
                if !td.invariants.is_empty() {
                    self.compile_invariant_check(type_name)?;
                }
            }
            TopDecl::Enum(ed) => {
                if ed.derives.is_empty() {
                    return Ok(());
                }
                let type_name = &ed.name.name;
                if !self.types.contains_key(type_name) {
                    self.types.insert(type_name.clone(), vec!["discriminant".to_string()]);
                    self.type_meta.insert(type_name.clone(), TypeMeta {
                        fields: vec![("discriminant".to_string(), "Int".to_string())],
                        derives: ed.derives.clone(),
                        invariants: Vec::new(),
                    });
                }
                let struct_ty = format!("%struct.{type_name}");
                let field_names: Vec<String> = vec!["discriminant".to_string()];

                for derive in &ed.derives {
                    match derive {
                        DeriveTrait::Eq => self.compile_eq_impl(type_name, &struct_ty, &field_names, &[])?,
                        DeriveTrait::Clone => self.compile_clone_impl(type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Hash => self.compile_hash_impl(type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Ord => self.compile_ord_impl(type_name, &struct_ty, &field_names, &[])?,
                        _ => {}
                    }
                }
            }
            TopDecl::Module(md) => {
                for sub in &md.items {
                    self.compile_derive_for_item(sub)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_eq_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String], fields: &[FieldDecl]) -> Result<(), String> {
        let fn_name = format!("{type_name}.eq");
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
            // Check if field is float
            let is_float = fields.get(i).map(|f| matches!(&f.ty, Type::Named(id, _) if id.name == "Float64" || id.name == "Float32")).unwrap_or(false);
            if is_float {
                self.emitln(&format!("  {cmp} = fcmp oeq double {self_val}, {other_val}"));
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

    fn compile_ord_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String], fields: &[FieldDecl]) -> Result<(), String> {
        let fn_name = format!("{type_name}.compare");
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
            let is_float = fields.get(i).map(|f| matches!(&f.ty, Type::Named(id, _) if id.name == "Float64" || id.name == "Float32")).unwrap_or(false);
            self.emitln(&format!("  {self_gep} = getelementptr {struct_ty}, {struct_ty}* {self_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {self_val} = load {field_llvm_ty}, {field_llvm_ty}* {self_gep}"));
            self.emitln(&format!("  {other_gep} = getelementptr {struct_ty}, {struct_ty}* {other_alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  {other_val} = load {field_llvm_ty}, {field_llvm_ty}* {other_gep}"));

            self.emitln(&format!("  {cmp_eq} = icmp eq {field_llvm_ty} {self_val}, {other_val}"));
            let next_field = self.fresh_block("next_field");
            let ret_block = self.fresh_block("ord_ret");
            self.emitln(&format!("  br i1 {cmp_eq}, label %{next_field}, label %{ret_block}"));
            self.emitln(&format!("\n{ret_block}:"));
            if is_float {
                let fself = self.fresh_tmp();
                let fother = self.fresh_tmp();
                let fcmp = self.fresh_tmp();
                self.emitln(&format!("  {fself} = sitofp i64 {self_val} to double"));
                self.emitln(&format!("  {fother} = sitofp i64 {other_val} to double"));
                self.emitln(&format!("  {fcmp} = fcmp olt double {fself}, {fother}"));
                let result = self.fresh_tmp();
                self.emitln(&format!("  {result} = select i1 {fcmp}, i64 -1, i64 1"));
                self.emitln(&format!("  ret i64 {result}"));
            } else {
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
        loop {
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
                            Self::axiom_to_llvm_type(&raw).to_string()
                        }
                    }
                    _ => Self::axiom_to_llvm_type(&Self::type_from_ast(t)).to_string(),
                }
            };
            let specialized_ret_type = fd.return_type.as_ref()
                .map(|t| subst_type(t))
                .unwrap_or_else(|| "void".to_string());
            let specialized_param_types: Vec<String> = fd.params.iter()
                .map(|p| subst_type(&p.ty))
                .collect();
            self.functions.insert(specialized_name.clone(), (specialized_param_types.clone(), specialized_ret_type.clone()));

            // Emit the specialized function
            self.push_scope();
            self.block_counter = 0;
            self.tmp_counter = 0;

            self.current_return_type = specialized_ret_type.clone();
            self.current_param_llvm_types = specialized_param_types.clone();
            self.current_fn = Some(specialized_name.clone());

            let params_str: Vec<String> = fd.params.iter()
                .enumerate()
                .map(|(i, p)| {
                    let llvm_ty = subst_type(&p.ty);
                    format!("{llvm_ty} %param{i}")
                })
                .collect();

            self.emitln(&format!("define {specialized_ret_type} @{specialized_name}({}) {{", params_str.join(", ")));
            let entry_block = self.fresh_block("entry");
            self.emitln(&format!("{entry_block}:"));

            // Allocate parameters as locals
            for (i, param) in fd.params.iter().enumerate() {
                let llvm_ty = subst_type(&param.ty);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                self.emitln(&format!("  store {llvm_ty} %param{i}, {llvm_ty}* {alloca}"));
                self.add_local(&param.name.name, alloca, &llvm_ty);
                // Track params whose original type is a generic parameter being monomorphised
                let axiom_ty = Self::type_from_ast(&param.ty);
                if type_map.contains_key(&axiom_ty) {
                    if let Some(concrete) = type_map.get(&axiom_ty) {
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
                        let result_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {result_alloca} = alloca i64"));
                        self.match_result_ptr = Some(result_alloca.clone());
                        self.compile_stmt(stmt)?;
                        self.match_result_ptr = None;
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load i64, i64* {result_alloca}"));
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
                    if let Some(ref ptr) = self.match_result_ptr {
                        self.emitln(&format!("  store i64 {result}, i64* {ptr}"));
                    }
                    if is_last && is_expression {
                        // Store result in the result alloca for ensures checks
                        if let Some(res_ptr) = self.result_ptr.as_ref() {
                            let ret_ty = &self.current_return_type.clone();
                            self.emitln(&format!("  store {ret_ty} {result}, {ret_ty}* {res_ptr}"));
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
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                self.emitln(&format!("  store {llvm_ty} {val}, {llvm_ty}* {alloca}"));
                self.add_local(&name.name, alloca, &llvm_ty);
                // Check invariants if the value is a struct with invariants
                if self.check_contracts {
                    self.maybe_check_value_invariants(value, &val);
                }
            }
            Stmt::Var(name, _ty, value, _) => {
                let val = self.compile_expr(value)?;
                let llvm_ty = self.infer_llvm_type(value);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                self.emitln(&format!("  store {llvm_ty} {val}, {llvm_ty}* {alloca}"));
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
                        self.emitln(&format!("  store {llvm_ty} {val}, {llvm_ty}* {ptr}"));
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
                    let val = self.compile_expr(e)?;
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
                    self.emitln(&format!("  ret {ret_ty} {val}"));
                } else {
                    if !self.current_ensures.is_empty() {
                        self.compile_ensures_checks();
                    }
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
                    self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                    self.emitln(&format!("  store {struct_ty} {val}, {struct_ty}* {alloca}"));
                    scrutinee_alloca_info = Some((alloca, type_name.clone(), struct_ty));
                }

                // Build check block labels and arm labels
                let mut check_labels: Vec<String> = Vec::new();
                let mut arm_labels: Vec<String> = Vec::new();
                let mut wildcard_idx: Option<usize> = None;

                for (i, arm) in arms.iter().enumerate() {
                    let arm_label = self.fresh_block("match_arm");
                    arm_labels.push(arm_label);
                    match &arm.pattern {
                        Pattern::Lit(Literal::Int(..)) | Pattern::Lit(Literal::Bool(..)) => {
                            check_labels.push(self.fresh_block("match_check"));
                        }
                        Pattern::Variant(..) => {
                            check_labels.push(self.fresh_block("match_check"));
                        }
                        Pattern::Wildcard(_) | Pattern::Ident(_) => { wildcard_idx = Some(i); }
                        _ => {}
                    }
                }

                // Branch to first check block
                if check_labels.is_empty() && wildcard_idx.is_some() {
                    self.emitln(&format!("  br label %{}", arm_labels[wildcard_idx.unwrap()]));
                } else if !check_labels.is_empty() {
                    self.emitln(&format!("  br label %{}", check_labels[0]));
                } else {
                    self.emitln(&format!("  br label %{merge_label}"));
                }

                // Emit check blocks
                let mut check_idx = 0;
                for (i, arm) in arms.iter().enumerate() {
                    match &arm.pattern {
                        Pattern::Lit(Literal::Int(n, _)) => {
                            self.emitln(&format!("\n{}:", check_labels[check_idx]));
                            let check = self.fresh_tmp();
                            self.emitln(&format!("  {check} = icmp eq i64 {val}, {n}"));
                            let next = if check_idx + 1 < check_labels.len() {
                                check_labels[check_idx + 1].clone()
                            } else if let Some(wi) = wildcard_idx {
                                arm_labels[wi].clone()
                            } else {
                                merge_label.clone()
                            };
                            self.emitln(&format!("  br i1 {check}, label %{}, label %{next}", arm_labels[i]));
                            check_idx += 1;
                        }
                        Pattern::Lit(Literal::Bool(b, _)) => {
                            self.emitln(&format!("\n{}:", check_labels[check_idx]));
                            let check = self.fresh_tmp();
                            let bval = if *b { "1" } else { "0" };
                            self.emitln(&format!("  {check} = icmp eq i64 {val}, {bval}"));
                            let next = if check_idx + 1 < check_labels.len() {
                                check_labels[check_idx + 1].clone()
                            } else if let Some(wi) = wildcard_idx {
                                arm_labels[wi].clone()
                            } else {
                                merge_label.clone()
                            };
                            self.emitln(&format!("  br i1 {check}, label %{}, label %{next}", arm_labels[i]));
                            check_idx += 1;
                        }
                        Pattern::Variant(variant_name, _, _) => {
                            self.emitln(&format!("\n{}:", check_labels[check_idx]));
                            // Load discriminant (field 0) from the scrutinee struct
                            if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                                // Find the variant index in the enum
                                let variant_idx = self.enum_variants.get(type_name)
                                    .and_then(|variants| variants.iter().position(|(vn, _)| vn == &variant_name.name))
                                    .unwrap_or(0);
                                let disc_gep = self.fresh_tmp();
                                let disc_val = self.fresh_tmp();
                                self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                                self.emitln(&format!("  {disc_val} = load i64, i64* {disc_gep}"));
                                let check = self.fresh_tmp();
                                self.emitln(&format!("  {check} = icmp eq i64 {disc_val}, {variant_idx}"));
                                let next = if check_idx + 1 < check_labels.len() {
                                    check_labels[check_idx + 1].clone()
                                } else if let Some(wi) = wildcard_idx {
                                    arm_labels[wi].clone()
                                } else {
                                    merge_label.clone()
                                };
                                self.emitln(&format!("  br i1 {check}, label %{}, label %{next}", arm_labels[i]));
                            } else {
                                // If no struct type info, fall back to literal comparison on val
                                let variant_idx = 0;
                                let check = self.fresh_tmp();
                                self.emitln(&format!("  {check} = icmp eq i64 {val}, {variant_idx}"));
                                let next = if check_idx + 1 < check_labels.len() {
                                    check_labels[check_idx + 1].clone()
                                } else if let Some(wi) = wildcard_idx {
                                    arm_labels[wi].clone()
                                } else {
                                    merge_label.clone()
                                };
                                self.emitln(&format!("  br i1 {check}, label %{}, label %{next}", arm_labels[i]));
                            }
                            check_idx += 1;
                        }
                        Pattern::Wildcard(_) => {}
                        _ => {}
                    }
                }

                // Emit arm bodies
                for (i, arm) in arms.iter().enumerate() {
                    self.emitln(&format!("\n{}:", arm_labels[i]));
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
                    if let Pattern::Ident(ident) = &arm.pattern {
                        let match_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {match_alloca} = alloca i64"));
                        self.emitln(&format!("  store i64 {val}, i64* {match_alloca}"));
                        self.add_local(&ident.name, match_alloca, "i64");
                    }
                    match &arm.body {
                        MatchBody::Block(b) => { self.compile_block(b, false)?; }
                        MatchBody::Expr(e) => {
                            let arm_val = self.compile_expr(e)?;
                            if let Some(ref ptr) = self.match_result_ptr {
                                self.emitln(&format!("  store i64 {arm_val}, i64* {ptr}"));
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
                self.compile_block(body, false)?;
                self.emitln(&format!("  br label %{loop_cond}"));
                self.emitln(&format!("\n{loop_exit}:"));
            }
            Stmt::For(_, _, body, _) => {
                // Phase 0: simplified for — just execute body once
                self.compile_block(body, false)?;
            }
            Stmt::Spawn(body, _) => {
                self.compile_block(body, false)?;
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
            Expr::Unary(op, inner, _) => {
                let val = self.compile_expr(inner)?;
                let tmp = self.fresh_tmp();
                match op {
                    UnaryOp::Neg => {
                        if let Expr::Float(..) = **inner {
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
                    UnaryOp::Ref | UnaryOp::MutRef => return Ok(val),
                }
                Ok(tmp)
            }
            Expr::Binary(left, op, right, _) => {
                let l = self.compile_expr(left)?;
                let r = self.compile_expr(right)?;
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
                let (ty, inst) = match op {
                    BinOp::Add => (if is_float { "double" } else { "i64" }, if is_float { "fadd" } else { "add" }),
                    BinOp::Sub => (if is_float { "double" } else { "i64" }, if is_float { "fsub" } else { "sub" }),
                    BinOp::Mul => (if is_float { "double" } else { "i64" }, if is_float { "fmul" } else { "mul" }),
                    BinOp::Div => (if is_float { "double" } else { "i64" }, if is_float { "fdiv" } else { "sdiv" }),
                    BinOp::Rem => (if is_float { "double" } else { "i64" }, if is_float { "frem" } else { "srem" }),
                    BinOp::Eq => (if is_float { "double" } else { "i64" }, if is_float { "fcmp oeq" } else { "icmp eq" }),
                    BinOp::Neq => (if is_float { "double" } else { "i64" }, if is_float { "fcmp one" } else { "icmp ne" }),
                    BinOp::Lt => (if is_float { "double" } else { "i64" }, if is_float { "fcmp olt" } else { "icmp slt" }),
                    BinOp::Gt => (if is_float { "double" } else { "i64" }, if is_float { "fcmp ogt" } else { "icmp sgt" }),
                    BinOp::Le => (if is_float { "double" } else { "i64" }, if is_float { "fcmp ole" } else { "icmp sle" }),
                    BinOp::Ge => (if is_float { "double" } else { "i64" }, if is_float { "fcmp oge" } else { "icmp sge" }),
                    BinOp::Assign => return Ok(r),
                    _ => unreachable!(),
                };
                self.emitln(&format!("  {tmp} = {inst} {ty} {l}, {r}"));
                let result = if inst.starts_with("icmp") || inst.starts_with("fcmp") {
                    let ext = self.fresh_tmp();
                    self.emitln(&format!("  {ext} = zext i1 {tmp} to i64"));
                    ext
                } else {
                    tmp
                };
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
                    self.emitln(&format!("  ret {ret_ty} 0"));
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
                                self.emitln(&format!("  {tmp} = call i64 @axiom_is_sorted(i8* {ptr}, i64 0)"));
                            }
                            "all" => {
                                let pred = extra_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @axiom_all(i8* {ptr}, i64 0, i8* {pred})"));
                            }
                            "none" => {
                                let pred = extra_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @axiom_none(i8* {ptr}, i64 0, i8* {pred})"));
                            }
                            "contains" => {
                                let val = extra_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @axiom_contains(i8* {ptr}, i64 {val})"));
                            }
                            _ => unreachable!(),
                        }
                        return Ok(tmp);
                    } else {
                        // Direct form: method(args) — compile all args
                        let compiled_args: Vec<String> = args.iter()
                            .map(|a| self.compile_expr(a))
                            .collect::<Result<Vec<_>, _>>()?;
                        match fn_name.as_str() {
                            "is_sorted" => {
                                let ptr = compiled_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                let len = compiled_args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @axiom_is_sorted(i8* {ptr}, i64 {len})"));
                            }
                            "all" => {
                                let ptr = compiled_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                let len = compiled_args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                                let pred = compiled_args.get(2).cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @axiom_all(i8* {ptr}, i64 {len}, i8* {pred})"));
                            }
                            "none" => {
                                let ptr = compiled_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                let len = compiled_args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                                let pred = compiled_args.get(2).cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @axiom_none(i8* {ptr}, i64 {len}, i8* {pred})"));
                            }
                            "contains" => {
                                let ptr = compiled_args.first().cloned().unwrap_or_else(|| "0".to_string());
                                let val = compiled_args.get(1).cloned().unwrap_or_else(|| "0".to_string());
                                self.emitln(&format!("  {tmp} = call i64 @axiom_contains(i8* {ptr}, i64 {val})"));
                            }
                            _ => unreachable!(),
                        }
                        return Ok(tmp);
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
                        let recv_val = self.compile_expr(receiver)?;
                        let val = self.compile_expr(&args[0])?;
                        let vec_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                        self.emitln(&format!("  store %struct.Vec {recv_val}, %struct.Vec* {vec_alloca}"));
                        let len_gep = self.fresh_tmp();
                        let len_val = self.fresh_tmp();
                        self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
                        self.emitln(&format!("  {len_val} = load i64, i64* {len_gep}"));
                        let data_gep = self.fresh_tmp();
                        let data_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
                        self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                        let offset = self.fresh_tmp();
                        self.emitln(&format!("  {offset} = mul i64 {len_val}, 8"));
                        let dest = self.fresh_tmp();
                        self.emitln(&format!("  {dest} = getelementptr i8, i8* {data_ptr}, i64 {offset}"));
                        self.emitln(&format!("  store i64 {val}, i64* {dest}"));
                        let new_len = self.fresh_tmp();
                        self.emitln(&format!("  {new_len} = add i64 {len_val}, 1"));
                        self.emitln(&format!("  store i64 {new_len}, i64* {len_gep}"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load %struct.Vec, %struct.Vec* {vec_alloca}"));
                        return Ok(loaded);
                    }
                }
                // Vec.len(vec) — method call on Vec
                if fn_name == "len" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
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
                if fn_name == "axiom_read_file" {
                    let tmp = self.fresh_tmp();
                    if let Some(path_arg) = args.first() {
                        let path_ptr = self.compile_expr(path_arg)?;
                        self.emitln(&format!("  {tmp} = call i8* @axiom_read_file(i8* {path_ptr})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i8* @axiom_read_file(i8* null)"));
                    }
                    let tmp_int = self.fresh_tmp();
                    self.emitln(&format!("  {tmp_int} = ptrtoint i8* {tmp} to i64"));
                    return Ok(tmp_int);
                }
                if fn_name == "axiom_file_size" {
                    let tmp = self.fresh_tmp();
                    if let Some(path_arg) = args.first() {
                        let path_ptr = self.compile_expr(path_arg)?;
                        self.emitln(&format!("  {tmp} = call i64 @axiom_file_size(i8* {path_ptr})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i64 @axiom_file_size(i8* null)"));
                    }
                    return Ok(tmp);
                }
                if fn_name == "axiom_free" {
                    if let Some(ptr_arg) = args.first() {
                        let ptr_val = self.compile_expr(ptr_arg)?;
                        let ptr_ty = self.infer_llvm_type(ptr_arg);
                        let ptr_ptr = self.val_to_i8ptr(&ptr_val, &ptr_ty);
                        self.emitln(&format!("  call void @axiom_free(i8* {ptr_ptr})"));
                    }
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_char_at" && args.len() >= 2 {
                    let src = self.compile_expr(&args[0])?;
                    let src_ty = self.infer_llvm_type(&args[0]);
                    let pos = self.compile_expr(&args[1])?;
                    let tmp = self.fresh_tmp();
                    let tmp_ext = self.fresh_tmp();
                    let src_ptr = self.val_to_i8ptr(&src, &src_ty);
                    self.emitln(&format!("  {tmp} = call i8 @axiom_char_at(i8* {src_ptr}, i64 {pos})"));
                    self.emitln(&format!("  {tmp_ext} = zext i8 {tmp} to i64"));
                    return Ok(tmp_ext);
                }
                if fn_name == "axiom_str_len" && args.len() >= 1 {
                    let src = self.compile_expr(&args[0])?;
                    let src_ty = self.infer_llvm_type(&args[0]);
                    let tmp = self.fresh_tmp();
                    let src_ptr = self.val_to_i8ptr(&src, &src_ty);
                    self.emitln(&format!("  {tmp} = call i64 @axiom_str_len(i8* {src_ptr})"));
                    return Ok(tmp);
                }
                // v0.9.4 string-based IR emission externs
                if fn_name == "axiom_ir_define_s" && args.len() >= 2 {
                    let name = self.compile_expr(&args[0])?;
                    let ret_type = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @axiom_ir_define_s(i8* {name}, i8* {ret_type})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_param_int" && args.len() >= 1 {
                    let index = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @axiom_ir_param_int(i64 {index})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_param_double" && args.len() >= 1 {
                    let index = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @axiom_ir_param_double(i64 {index})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_alloca_s" && args.len() >= 1 {
                    let reg = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @axiom_ir_alloca_s(i64 {reg})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_store_param" && args.len() >= 2 {
                    let reg = self.compile_expr(&args[0])?;
                    let param = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @axiom_ir_store_param(i64 {reg}, i64 {param})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_load_s" && args.len() >= 2 {
                    let reg = self.compile_expr(&args[0])?;
                    let from_reg = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @axiom_ir_load_s(i64 {reg}, i64 {from_reg})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_add" && args.len() >= 3 {
                    let dst = self.compile_expr(&args[0])?;
                    let left = self.compile_expr(&args[1])?;
                    let right = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @axiom_ir_add(i64 {dst}, i64 {left}, i64 {right})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_fmul" && args.len() >= 3 {
                    let dst = self.compile_expr(&args[0])?;
                    let left = self.compile_expr(&args[1])?;
                    let right = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @axiom_ir_fmul(i64 {dst}, i64 {left}, i64 {right})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_call_fn" && args.len() >= 3 {
                    let dst = self.compile_expr(&args[0])?;
                    let fn_name_str = self.compile_expr(&args[1])?;
                    let ret_type = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @axiom_ir_call_fn(i64 {dst}, i8* {fn_name_str}, i8* {ret_type})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_call_arg_lit" && args.len() >= 2 {
                    let ty = self.compile_expr(&args[0])?;
                    let val = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @axiom_ir_call_arg_lit(i8* {ty}, i8* {val})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_ret_reg" && args.len() >= 1 {
                    let reg = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @axiom_ir_ret_reg(i64 {reg})"));
                    return Ok("0".to_string());
                }
                if fn_name == "axiom_ir_ret_lit" && args.len() >= 1 {
                    let val = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @axiom_ir_ret_lit(i64 {val})"));
                    return Ok("0".to_string());
                }
                // Check if this is a call to a generic function and track instantiation
                let fn_key = fn_name.clone();
                let is_generic = self.generic_fn_decls.iter().any(|f| self.fn_key(f) == fn_key);
                if is_generic {
                    // Infer concrete types from argument types
                    let mut concrete_types: Vec<String> = Vec::new();
                    // Find the generic function declaration
                    if let Some(fd) = self.generic_fn_decls.iter().find(|f| self.fn_key(f) == fn_key) {
                        for (gp, arg_expr) in fd.generics.iter().zip(args.iter()) {
                            let concrete_ty = match arg_expr {
                                Expr::Int(..) => "Int".to_string(),
                                Expr::Float(..) => "Float64".to_string(),
                                Expr::Bool(..) => "Bool".to_string(),
                                Expr::Str(..) => "Str".to_string(),
                                Expr::Char(..) => "Char".to_string(),
                                Expr::Ident(id) => {
                                    // Check param_concrete_types first (AXIOM type name during monomorphisation)
                                    if let Some(concrete) = self.param_concrete_types.get(&id.name) {
                                        concrete.clone()
                                    } else if let Some((_, llvm_ty)) = self.lookup_local(&id.name) {
                                        llvm_ty.clone()
                                    } else {
                                        gp.name.name.clone()
                                    }
                                }
                                _ => "Int".to_string(),
                            };
                            concrete_types.push(concrete_ty);
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
                            // Not yet registered - use the generic signature
                            if let Some((pts, rt)) = self.functions.get(&fn_key) {
                                (rt.clone(), pts.clone())
                            } else {
                                ("i64".to_string(), vec!["i64".to_string(); args.len()])
                            }
                        };
                        let args_str = param_types.iter().zip(compiled_args.iter())
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
                            fn_key.clone()
                        }
                    } else {
                        fn_key.clone()
                    };
                    let (ret_ty, param_types) = if let Some((pts, rt)) = self.functions.get(&resolved_fn_key) {
                        (rt.clone(), pts.clone())
                    } else {
                        ("i64".to_string(), vec!["i64".to_string(); args.len()])
                    };
                    let args_str = if let Some(receiver) = receiver_expr {
                        if param_types.len() > compiled_args.len() {
                            // Method call: prepend receiver value, skip first param type when zipping
                            let recv_val = self.compile_expr(receiver)?;
                            let recv_llvm_ty = self.infer_llvm_type(receiver);
                            let rest_str: Vec<String> = param_types[1..].iter().zip(compiled_args.iter())
                                .map(|(ty, arg)| format!("{ty} {arg}"))
                                .collect();
                            format!("{recv_llvm_ty} {recv_val}, {}", rest_str.join(", "))
                        } else {
                            param_types.iter().zip(compiled_args.iter())
                                .map(|(ty, arg)| format!("{ty} {arg}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    } else {
                        param_types.iter().zip(compiled_args.iter())
                            .map(|(ty, arg)| format!("{ty} {arg}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    let tmp = self.fresh_tmp();
                    if ret_ty == "void" {
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
            Expr::Struct(name, fields, _) => {
                let struct_ty = format!("%struct.{}", name.name);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                for (i, (_, val)) in fields.iter().enumerate() {
                    let field_val = self.compile_expr(val)?;
                    let field_llvm_ty = self.field_llvm_type(&name.name, i);
                    let store_val = if field_val == "0" && (field_llvm_ty.ends_with('*') || field_llvm_ty.starts_with('\"')) {
                        "null".to_string()
                    } else {
                        field_val
                    };
                    let gep = self.fresh_tmp();
                    self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
                    self.emitln(&format!("  store {field_llvm_ty} {store_val}, {field_llvm_ty}* {gep}"));
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
            Expr::Closure(_, _, _) | Expr::PipeClosure(_, _, _) => Ok("0".to_string()),
            Expr::Await(inner, _) => self.compile_expr(inner),
            Expr::Comptime(inner, _) => self.compile_expr(inner),
        }
    }

    fn infer_struct_type_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(ident) => {
                if let Some((_, llvm_ty)) = self.lookup_local(&ident.name) {
                    if llvm_ty.starts_with("%struct.") {
                        return Some(llvm_ty[8..].to_string());
                    }
                }
                None
            }
            Expr::Struct(ident, _, _) => Some(ident.name.clone()),
            Expr::Field(obj, _, _) => {
                self.infer_struct_type_name(obj.as_ref())
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
                    Expr::Field(_, field, _) => Some(field.name.clone()),
                    _ => None,
                };
                if let Some(ref name) = fn_name {
                    if name == "axiom_read_file" { return "i64".to_string(); }
                    if name == "axiom_char_at" || name == "axiom_str_len" { return "i64".to_string(); }
                    if let Some((_, ret_ty)) = self.functions.get(name) {
                        if ret_ty == "double" { return "double".to_string(); }
                        return ret_ty.clone();
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
            Expr::Struct(ident, _, _) => {
                if self.types.contains_key(&ident.name) || self.type_meta.contains_key(&ident.name) {
                    format!("%struct.{}", ident.name)
                } else {
                    "i64".to_string()
                }
            }
            Expr::Paren(inner, _) => self.infer_llvm_type(inner),
            Expr::Unary(op, inner, _) => {
                match op {
                    UnaryOp::Not => "i64".to_string(),
                    UnaryOp::Neg | UnaryOp::Ref | UnaryOp::MutRef => self.infer_llvm_type(inner),
                }
            }
            Expr::Binary(left, op, right, _) => {
                match op {
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => "i64".to_string(),
                    BinOp::And | BinOp::Or => "i64".to_string(),
                    BinOp::Assign => self.infer_llvm_type(right),
                    _ => {
                        if self.is_float_expr(left) || self.is_float_expr(right) { "double".to_string() } else { "i64".to_string() }
                    }
                }
            }
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


