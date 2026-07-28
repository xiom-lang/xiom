// XIOM Codegen — IrEmitter sub-contexts (M4.1: god object decomposition)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_ast::*;
use std::collections::{HashMap, HashSet};

// ============================================================================
// CodegenConfig — Compilation flags and target configuration
// ============================================================================

#[derive(Clone)]
pub struct CodegenConfig {
    /// LLVM target triple (default: auto-detected from host OS/arch)
    pub target_triple: String,
    /// Whether to emit contract runtime checks
    pub check_contracts: bool,
    /// Strict mode: error on unknown types defaulting to i64
    pub strict_mode: bool,
    /// Maximum allowed recursion depth
    pub max_recursion_depth: u32,
    /// Hot reload mode: pub fn calls go through @xiom_hot_get_ptr thunks
    pub hot_reload: bool,
    /// M18: Enable integer overflow checks (llvm.sadd/sub/mul.with.overflow + trap)
    pub overflow_checks: bool,
    /// Set of pub function keys (for hot reload thunk dispatch)
    pub pub_functions: HashSet<String>,
    /// Globals to save/restore across hot reload: (symbol, llvm_type, byte_size)
    pub xiom_hot_globals: Vec<(String, String, usize)>,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        // Auto-detect host target triple at runtime so Linux/macOS builds
        // get the correct default without manual --target-triple overrides.
        let host_triple = if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc".to_string()
        } else if cfg!(target_os = "linux") {
            "x86_64-unknown-linux-gnu".to_string()
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin".to_string()
        } else {
            "x86_64-unknown-linux-gnu".to_string() // fallback
        };
        Self {
            target_triple: host_triple,
            check_contracts: true,
            strict_mode: false,
            max_recursion_depth: 2000,
            hot_reload: false,
            overflow_checks: false, // M18: opt-in, OFF by default
            pub_functions: HashSet::new(),
            xiom_hot_globals: Vec::new(),
        }
    }
}

// ============================================================================
// TypeContext — Type system registration and interface/enum metadata
// ============================================================================

#[derive(Clone, Default)]
pub struct TypeContext {
    /// Known type structures: name -> field names
    pub types: HashMap<String, Vec<String>>,
    /// Full type metadata: name -> TypeMeta
    pub type_meta: HashMap<String, TypeMeta>,
    /// Names of types declared with generic params
    pub generic_type_names: HashSet<String>,
    /// Known function signatures: name -> (param_llvm_types, return_llvm_type_or_empty)
    pub functions: HashMap<String, (Vec<String>, String)>,
    /// Declared XIOM return type per function key
    pub fn_return_xiom: HashMap<String, String>,
    /// Maps function pointer parameter names to their LLVM return types
    pub fn_ptr_return_types: HashMap<String, String>,
    /// Interface registry: name -> vec of (method_name, param_type_names)
    pub interfaces: HashMap<String, Vec<(String, Vec<String>)>>,
    /// Concrete types that implement each interface
    pub interface_impls: HashMap<String, HashSet<String>>,
    /// Enum variants registry: name -> vec of (variant_name, field_names)
    pub enum_variants: HashMap<String, Vec<(String, Vec<String>)>>,
    /// Per-variant payload field TYPE names
    pub enum_variant_field_types: HashMap<String, Vec<(String, Vec<String>)>>,
    /// Builtin types whose impls have been referenced
    pub used_builtins: HashSet<String>,
    /// M36: Type alias map — alias name → resolved XIOM type name
    /// (e.g., "MyResult" → "Result[Int, Str]", "MyInt8" → "Int8").
    /// Populated during type registration from `type T = Underlying;` declarations.
    pub type_aliases: HashMap<String, String>,
}

// ============================================================================
// FunctionContext — Per-function compilation state
// ============================================================================

#[derive(Clone, Default)]
pub struct FunctionContext {
    /// Current function name (for labels)
    pub current_fn: Option<String>,
    /// Return type of current function (empty = void)
    pub current_return_type: String,
    /// String constants to emit at the top
    pub strings: Vec<String>,
    /// Current function's param LLVM types (index -> type)
    pub current_param_llvm_types: Vec<String>,
    /// Current function's ensures clauses
    pub current_ensures: Vec<Expr>,
    /// Current method's receiver TYPE NAME
    pub current_receiver: Option<String>,
    /// Local variables: name -> (alloca_register, llvm_type) — scope stack
    pub locals: Vec<HashMap<String, (String, String)>>,
    /// Pre-state value of self (for self@pre in ensures)
    pub self_pre_value: Option<String>,
    /// Alloca for the result value in ensures expressions
    pub result_ptr: Option<String>,
    /// Alloca for match result in expression position
    pub match_result_ptr: Option<String>,
    /// LLVM type used when storing an arm body into match_result_ptr
    pub match_result_ty: Option<String>,
}

// ============================================================================
// MonoContext — Monomorphisation state
// ============================================================================

#[derive(Clone, Default)]
pub struct MonoContext {
    /// Generic function ASTs stored for later monomorphisation
    pub generic_fn_decls: Vec<(String, FnDecl)>,
    /// Tracked generic instantiations: (fn_original_name, vec![concrete_type_names])
    pub generic_instantiations: Vec<(String, Vec<String>)>,
    /// Const-generic value map: monomorphised_fn_name -> {const_param_name -> value}
    pub const_value_map: HashMap<String, HashMap<String, i64>>,
    /// Specialized monomorphised function names already emitted
    pub mono_emitted: HashSet<String>,
    /// Current type substitution map for monomorphisation
    pub current_type_map: HashMap<String, String>,
    /// Current const-generic value map during monomorphised body compilation
    pub current_const_map: HashMap<String, i64>,
    /// Maps variable name to concrete type for generic params in monomorphised functions
    pub param_concrete_types: HashMap<String, String>,
    /// Tracks emitted function names to avoid duplicate definitions
    pub emitted_fns: HashSet<String>,
    /// Set of function names already declared via `declare`
    pub already_declared: HashSet<String>,
}

// ============================================================================
// LocalContext — Local variable tracking and module-level state
// ============================================================================

#[derive(Clone, Default)]
pub struct LocalContext {
    /// Locals whose declared XIOM type is Bool
    pub bool_locals: HashSet<String>,
    /// Locals whose declared XIOM type is a raw pointer (*T)
    pub ptr_locals: HashSet<String>,
    /// Locals bound from Expr::Array literals (for indexing dispatch)
    pub array_locals: HashSet<String>,
    /// Temporary register values that originated from Expr::Array literals
    pub array_value_regs: HashSet<String>,
    /// LLVM element type for local array bindings
    pub local_array_elem: HashMap<String, String>,
    /// Fixed-size array-local bindings (var name -> N elements)
    pub local_array_sizes: HashMap<String, i64>,
    /// Local Vec bindings' declared element type name
    pub local_vec_elem: HashMap<String, String>,
    /// Option locals whose payload is a heap-boxed STRUCT pointer
    pub local_opt_payload: HashMap<String, String>,
    /// i64 locals holding a heap-boxed struct pointer
    pub local_boxed_struct: HashMap<String, String>,
    /// Locals holding an i64 CONTAINER HANDLE (pointer to boxed Vec header)
    pub local_vec_handle: HashMap<String, String>,
    /// ERROR payload type of locals holding Result[T, E] values
    pub local_err_payload: HashMap<String, String>,
    /// Stack of active loop labels: (continue_label, break_label)
    pub loop_stack: Vec<(String, String)>,
    /// Module/global const values
    pub constants: HashMap<String, Expr>,
    /// Mutable module-level var globals: name -> (llvm_symbol, llvm_type)
    pub module_globals: HashMap<String, (String, String)>,
    /// Ordered list of module-global definitions to emit
    pub module_global_defs: Vec<(String, String, String)>,
    /// Current module prefix for scoped type resolution
    pub current_module: Option<String>,
    /// Struct type definitions created during compilation
    pub deferred_struct_types: Vec<(String, String)>,
    /// Scrutinee info for match arm field extraction
    pub scrutinee_info: Option<(String, String)>,
    /// M20-A1: Deferred closure function definitions (emitted after current fn)
    pub deferred_closure_defs: Vec<String>,
    /// M20-A1: Deferred env struct type definitions (emitted before fn body)
    pub deferred_pre_body_defs: Vec<String>,
    /// M20-A1: Set of local variable names that hold closure values.
    /// Used by the call dispatch to detect closure calls vs regular function calls.
    pub closure_locals: HashSet<String>,
    /// M17: Set of local variable names whose declared XIOM type is a signed integer
    /// (Int, Int8, Int16, Int32, Int64). Used by widen_to_i64 to select sext vs zext.
    pub signed_locals: HashSet<String>,
    /// M17: XIOM type name for each local. Maps local name → XIOM type string
    /// (e.g. "x" → "Int8", "y" → "UInt16"). Populated from declared type annotations.
    pub local_xiom_types: HashMap<String, String>,
    /// M17: Tracks which SSA register names hold signed integer values.
    /// Populated when values are created with known XIOM type (Ident loads,
    /// As expressions, literals). Consulted by widen_to_i64 to select sext/zext.
    pub reg_signed: HashMap<String, bool>,
}

// ============================================================================
// TypeMeta (re-exported from lib.rs context)
// ============================================================================

#[derive(Clone)]
#[allow(dead_code)]
pub struct TypeMeta {
    pub fields: Vec<(String, String)>,
    pub derives: Vec<DeriveTrait>,
    pub invariants: Vec<Expr>,
}
