// XIOM Codegen -- IrEmitter sub-contexts (M4.1: god object decomposition)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_ast::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::hash::Hash;

// ============================================================================
// v0.54: SyncRegistry -- Thread-safe HashMap wrapper for parallel compilation.
//
// Wraps an Arc<RwLock<HashMap<K,V>>> and provides a HashMap-like API where
// .get() returns Option<V> (cloned). Multiple readers can access concurrently;
// writers acquire an exclusive lock. Enables parallel type-checking and
// codegen in v0.55 without invasive call-site changes.
// ============================================================================

pub struct SyncRegistry<K: Eq + Hash, V> {
    inner: Arc<RwLock<HashMap<K, V>>>,
}

impl<K: Eq + Hash, V> Clone for SyncRegistry<K, V> {
    fn clone(&self) -> Self { Self { inner: Arc::clone(&self.inner) } }
}

impl<K: Eq + Hash, V> Default for SyncRegistry<K, V> {
    fn default() -> Self { Self { inner: Arc::new(RwLock::new(HashMap::new())) } }
}

impl<K: Eq + Hash + Clone, V: Clone> SyncRegistry<K, V> {
    /// Thread-safe read access -- clones the value.
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.read().unwrap().get(key).cloned()
    }

    /// Thread-safe write access -- inserts a value, returns the old value if any.
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.write().unwrap().insert(key, value)
    }

    /// Thread-safe contains check.
    pub fn contains_key(&self, key: &K) -> bool {
        self.inner.read().unwrap().contains_key(key)
    }

    /// Thread-safe get-or-insert: if key exists, returns the existing value;
    /// otherwise inserts the value produced by `f` and returns it.
    pub fn or_insert_with(&self, key: K, f: impl FnOnce() -> V) -> V {
        let mut map = self.inner.write().unwrap();
        if let Some(v) = map.get(&key) {
            return v.clone();
        }
        let v = f();
        map.insert(key, v.clone());
        v
    }

    /// Returns a snapshot of all keys (for iteration patterns).
    pub fn keys(&self) -> Vec<K> {
        self.inner.read().unwrap().keys().cloned().collect()
    }

    /// Returns a snapshot of all key-value pairs (for iteration patterns).
    pub fn entries(&self) -> Vec<(K, V)> {
        self.inner.read().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Returns the number of entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}

// ============================================================================
// CodegenConfig -- Compilation flags and target configuration
// ============================================================================

#[derive(Clone)]
pub struct CodegenConfig {
    /// LLVM target triple (default: auto-detected from host OS/arch)
    pub target_triple: String,
    /// Whether to emit contract runtime checks
    pub check_contracts: bool,
    /// Security review (2026-08-13): strip `assert`/`dbg!`/`debugger;` from
    /// release builds (contracts are governed by check_contracts). Keeps
    /// attack surface and debug output out of shipped binaries.
    pub strip_debug_checks: bool,
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
    /// I2: Enable parallel codegen (rayon-based per-function IR emission)
    pub parallel_codegen: bool,
    /// R1: Enable DWARF debug info emission from .xi source
    pub debug_symbols: bool,
    /// R1: Source file path for DWARF DIFile metadata
    pub source_file: String,
    /// D2.1 (Phase 7): `#[unsafe_direct]` -- trusted escape hatch. When true,
    /// user code may tag unsafe blocks `#[unsafe_direct]` (bypass confinement:
    /// no trampoline/arena/guard page, runs as today's plain unsafe block).
    /// Restricted to stdlib/trusted packages by default; `--enable-unsafe-direct`
    /// grants it to user code. The compiler reports the number of direct blocks.
    pub enable_unsafe_direct: bool,
    /// D2.1 (Phase 7): counted cap of `#[unsafe_direct]` blocks allowed.
    pub unsafe_direct_cap: u32,
    /// BUG 25 #2 fix: `use X.Y.f as alias;` -- alias name -> the FULL dotted
    /// use path (recorded by the checker; the driver strips UseDecls before
    /// codegen). The codegen resolves each path to its registered fn key at
    /// preassign time so bare calls through the alias work.
    pub use_alias_paths: HashMap<String, String>,
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
            strip_debug_checks: false,
            overflow_checks: true, // v0.56: ON by default (AI-safe systems compiler)
            pub_functions: HashSet::new(),
            xiom_hot_globals: Vec::new(),
            parallel_codegen: false,
            debug_symbols: false,
            source_file: "unknown.xi".to_string(),
            enable_unsafe_direct: false,
            unsafe_direct_cap: 64,
            use_alias_paths: HashMap::new(),
        }
    }
}

// ============================================================================
// TypeContext -- Type system registration and interface/enum metadata
// ============================================================================

#[derive(Clone, Default)]
pub struct TypeContext {
    /// Known type structures: name -> field names
    pub types: SyncRegistry<String, Vec<String>>,
    /// Full type metadata: name -> TypeMeta
    pub type_meta: SyncRegistry<String, TypeMeta>,
    /// M65 R7 (2026-09-10): concrete type names whose `%struct.X = type`
    /// definition was already emitted by the type-decl pass. Used to avoid
    /// re-emitting them when a body-time creation is appended at module end.
    pub emitted_type_defs: HashSet<String>,
    /// Names of types declared with generic params
    pub generic_type_names: HashSet<String>,
    /// BUG 52 (2026-08-18): GENERIC type decls' field types WITH their type
    /// args ("Vec[K]", "Vec[V]") keyed by bare type name ("Map"). The builtin
    /// Map/Set registrations pre-empt type_meta with bare "Vec" field types,
    /// so the stdlib's generic-arg field types are lost -- this map keeps them
    /// so mono'd method bodies can substitute the concrete args ("V"->"MyVal")
    /// for struct/enum Vec-element reads/writes.
    pub generic_type_field_types: HashMap<String, Vec<(String, String)>>,
    /// M65 Part 2 (2026-09-10): GENERIC type decls' parameter NAMES in
    /// declaration order ("Map" -> ["K", "V"]), keyed by bare and qualified
    /// type name. Needed to substitute a field's declared generic args
    /// ("Vec[V]") when a concrete instantiation's full type string is known
    /// ("Map[Str, JsonValue]") -- e.g. resolving `entries.values[i]` to the
    /// JsonValue element type so the read takes the struct-load path.
    pub generic_type_params: HashMap<String, Vec<String>>,
    /// Known function signatures: name -> (param_llvm_types, return_llvm_type_or_empty)
    pub functions: SyncRegistry<String, (Vec<String>, String)>,
    /// Declared XIOM return type per function key
    pub fn_return_xiom: SyncRegistry<String, String>,
    /// Maps function pointer parameter names to their LLVM return types
    pub fn_ptr_return_types: SyncRegistry<String, String>,
    /// Interface registry: name -> vec of (method_name, param_type_names)
    pub interfaces: SyncRegistry<String, Vec<(String, Vec<String>)>>,
    /// Concrete types that implement each interface
    pub interface_impls: SyncRegistry<String, HashSet<String>>,
    /// Method keys that take `self` by value (not `&self`) -- need store_back
    pub by_value_self_methods: HashSet<String>,
    /// Enum variants registry: name -> vec of (variant_name, field_names)
    pub enum_variants: SyncRegistry<String, Vec<(String, Vec<String>)>>,
    /// Per-variant payload field TYPE names
    pub enum_variant_field_types: SyncRegistry<String, Vec<(String, Vec<String>)>>,
    /// Builtin types whose impls have been referenced
    pub used_builtins: HashSet<String>,
    /// M36: Type alias map -- alias name -> resolved XIOM type name
    pub type_aliases: SyncRegistry<String, String>,
    /// M19: Default method bodies from interfaces, keyed by "Interface.method".
    pub interface_defaults: SyncRegistry<String, FnDecl>,
}

// ============================================================================
// FunctionContext -- Per-function compilation state
// ============================================================================

#[derive(Clone, Default)]
pub struct FunctionContext {
    /// Current function name (for labels)
    pub current_fn: Option<String>,
    /// Return type of current function (empty = void)
    pub current_return_type: String,
    /// BUG 55 (2026-08-18): inside a confined-unsafe BLOCK fn,
    /// current_return_type is the block's i64 ABI -- this holds the
    /// ENCLOSING fn's declared return type so Some/None/Ok/Err ctors
    /// build the CONCRETE container (Option__Rc), not the generic
    /// %struct.Option (whose i64 payload slot corrupted the concrete
    /// inline-struct field -- the Option/Result payload family root).
    pub enclosing_return_type: Option<String>,
    /// String constants to emit at the top
    pub strings: Vec<String>,
    /// Current function's param LLVM types (index -> type)
    pub current_param_llvm_types: Vec<String>,
    /// Current function's ensures clauses
    pub current_ensures: Vec<Expr>,
    /// Current method's receiver TYPE NAME
    pub current_receiver: Option<String>,
    /// Local variables: name -> (alloca_register, llvm_type) -- scope stack
    pub locals: Vec<HashMap<String, (String, String)>>,
    /// Pre-state value of self (for self@pre in ensures)
    pub self_pre_value: Option<String>,
    /// Alloca for the result value in ensures expressions
    pub result_ptr: Option<String>,
    /// LLVM type of the result alloca (for per-check `result` scope rebinding)
    pub result_llvm_ty: Option<String>,
    /// Alloca for match result in expression position
    pub match_result_ptr: Option<String>,
    /// LLVM type used when storing an arm body into match_result_ptr
    pub match_result_ty: Option<String>,
    /// v0.56/P2-4: Whether the current function returns `!` (Never type).
    /// When true, fallthrough/return paths emit `unreachable` instead of `ret`.
    pub is_never_return: bool,
    /// D2.1 (Phase 6): whether the current function's unsafe blocks should
    /// retry a transient fault once. Set from `#[unsafe_no_retry]` on the fn
    /// (deterministic faults shouldn't retry). Default: true (retry once).
    pub unsafe_allow_retry: bool,
    /// D2.1 (Phase 7): whether the current function's unsafe blocks run
    /// `#[unsafe_direct]` -- trusted, no trampoline/arena/guard page (plain
    /// unsafe). Restricted to stdlib/trusted, or user code with
    /// --enable-unsafe-direct.
    pub unsafe_direct: bool,
}

// ============================================================================
// MonoContext -- Monomorphisation state
// ============================================================================

#[derive(Clone, Default)]
pub struct MonoContext {
    /// Generic function ASTs stored for later monomorphisation
    pub generic_fn_decls: Vec<(String, FnDecl)>,
    /// B-007: fn key -> (param index, declared RETURN XIOM type) for every
    /// fn-typed (closure) PARAM. Populated at declaration registration for
    /// EVERY fn (generic or not) -- the direct call path needs it to wrap raw
    /// fn-REFERENCE args into closure envs with a forwarding THUNK (the
    /// thunk's signature needs the return type; the erased signature can't
    /// tell a fn-typed param from a plain Int).
    pub fn_typed_params: HashMap<String, Vec<(usize, String)>>,
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
    /// Bare leaf name -> leaf-qualified key for INJECTED stdlib free fns
    /// (e.g. "args" -> "env.args"). Injected decls are leaf-qualified so the
    /// driver merge (which drops TopDecl::Module) keeps module context; bare
    /// internal calls inside stdlib bodies (env.args_os -> args()) are
    /// rewritten to the qualified key so the emitted symbol matches the
    /// definition. Keep-first: a user-defined bare fn wins over injection.
    pub bare_fn_aliases: HashMap<String, String>,
    /// BUG 25 #2 fix: `use X.Y.f as alias;` -- alias name -> the registered fn
    /// key. The checker binds the alias for type checking, but the codegen's
    /// bare-call resolution had no alias table, so `af(-4.0)` through an
    /// alias resolved to the wrong symbol (wrong returns). Populated from the
    /// program's UseDecls; consulted by the bare-call fn-key resolution.
    pub use_alias_map: HashMap<String, String>,
    /// BUG 22 #11: PRE-ASSIGNED fn key -> emitted LLVM symbol for every
    /// non-generic fn with a body (walked once before any body compiles, in
    /// program order, using fn_symbol's dedup rule: the first same-key fn
    /// emits the bare symbol, later ones qualify). Definitions AND call
    /// sites consult this map so a call compiled before its def can never
    /// emit a qualified symbol the def went bare on (zero-param stub ->
    /// garbage). Keys cover the bare key, the leaf-qualified alias, and the
    /// module-qualified call key.
    pub fn_symbol_map: HashMap<String, String>,
}

// ============================================================================
// LocalContext -- Local variable tracking and module-level state
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
    /// round-15: XIOM element NAME for local array bindings ("UInt8" for
    /// `[200 as UInt8, ...]` -- the LLVM map only knows the width and would
    /// degrade signedness to "Int8"). Consumed by resolve_local_xiom_type so
    /// generic-arg inference monomorphises &[N]UInt8 bodies (zext reads).
    pub local_array_elem_xiom: HashMap<String, String>,
    /// Fixed-size array-local bindings (var name -> N elements)
    pub local_array_sizes: HashMap<String, i64>,
    /// BUG 22 #6: while/for nesting depth -- bindings compiled at depth > 0
    /// hoist their alloca to the fn entry (loop-body allocas do not dominate
    /// later blocks, which broke unsafe-block ctx captures referencing them).
    pub loop_depth: u32,
    /// BUG 22 #6: (alloca_reg, llvm_ty) pairs hoisted from loop bodies --
    /// spliced into the fn's entry block at fn end.
    pub hoisted_allocas: Vec<(String, String)>,
    /// Local Vec bindings' declared element type name
    pub local_vec_elem: HashMap<String, String>,
    /// Option locals whose payload is a heap-boxed STRUCT pointer
    pub local_opt_payload: HashMap<String, String>,
    /// BUG 22 #4 fix: the SCALAR XIOM payload type of a Some/Ok/Err binding
    /// (`var o = Some(5.0)` -> "Float64"). Some(5.0) stores the DOUBLE BITS in
    /// the i64 payload slot; the match extraction must bitcast back. Struct
    /// payloads stay in local_opt_payload (boxed); scalars land here.
    pub local_opt_payload_xiom: HashMap<String, String>,
    /// i64 locals holding a heap-boxed struct pointer
    pub local_boxed_struct: HashMap<String, String>,
    /// Locals holding an i64 CONTAINER HANDLE (pointer to boxed Vec header)
    pub local_vec_handle: HashMap<String, String>,
    /// ERROR payload type of locals holding Result[T, E] values
    pub local_err_payload: HashMap<String, String>,
    /// Stack of active loop labels: (optional_label, continue_label, break_label)
    pub loop_stack: Vec<(Option<String>, String, String)>,
    /// v0.56/P0-2: Deferred blocks to execute at scope exit (LIFO order)
    pub defer_stack: Vec<Block>,
    /// Module/global const values
    pub constants: HashMap<String, Expr>,
    /// Cycle detection stack for const evaluation -- tracks which constants
    /// are currently being resolved. Prevents infinite recursion on cycles
    /// like `const A = B; const B = A;`.
    pub const_eval_stack: RefCell<HashSet<String>>,
    /// Current const-evaluation recursion depth (security review 2026-08-13):
    /// a hostile or pathological const expression must not be able to hang
    /// the compiler via unbounded CTFE recursion. `evaluate_const_init` bails
    /// out (returns the expression unevaluated, falling back to runtime
    /// evaluation) once the depth exceeds CONST_EVAL_BUDGET.
    pub const_eval_depth: std::cell::Cell<u32>,
    /// Mutable module-level var globals: name -> (llvm_symbol, llvm_type)
    pub module_globals: HashMap<String, (String, String)>,
    /// BUG 29 (Map.keys on module globals): name -> XIOM type string
    /// ("Map[Str, Bool]", "Vec[Int]", ...) recorded at global registration so
    /// generic METHOD calls on globals (`_coverage.keys()`) can infer their
    /// concrete type args instead of defaulting to Int.
    pub global_xiom_types: HashMap<String, String>,
    /// Ordered list of module-global definitions to emit
    pub module_global_defs: Vec<(String, String, String)>,
    /// Module-level `var` globals whose initializer is a RUNTIME expression
    /// (fn call, etc.) -- cannot be a compile-time constant. The global is
    /// emitted zero-initialized and a @llvm.global_ctors entry runs the
    /// initializer at startup (BUG 3 fix).
    pub global_runtime_inits: Vec<(String, String, Expr)>,
    /// Current module prefix for scoped type resolution
    pub current_module: Option<String>,
    /// Struct type definitions created during compilation
    pub deferred_struct_types: Vec<(String, String)>,
    /// Tuple/anon struct type definition LINES discovered during function-body
    /// compilation. Flushed at the very end of the module (LLVM permits forward
    /// references to named types), so they never appear inline inside a function.
    pub pending_module_type_defs: Vec<String>,
    /// Scrutinee info for match arm field extraction
    pub scrutinee_info: Option<(String, String)>,
    /// M20-A1: Deferred closure function definitions (emitted after current fn)
    pub deferred_closure_defs: Vec<String>,
    /// M20-A1: Deferred env struct type definitions (emitted before fn body)
    pub deferred_pre_body_defs: Vec<String>,
    /// M20-A1: Set of local variable names that hold closure values.
    /// Used by the call dispatch to detect closure calls vs regular function calls.
    pub closure_locals: HashSet<String>,
    /// B-007: closure/fn-typed local -> declared RETURN XIOM type name
    /// ("Option[Int]", "Int", ...). The M20-A1 closure call path must use the
    /// real return type for the fn-pointer signature and the call -- struct
    /// returns (%struct.Option) are BY VALUE; hardcoding `call i64` +
    /// inttoptr turned a by-value struct return into a pointer deref
    /// (0xC0000005 in Option.and_then's closure call).
    pub fn_local_returns: HashMap<String, String>,
    /// M17: Set of local variable names whose declared XIOM type is a signed integer
    /// (Int, Int8, Int16, Int32, Int64). Used by widen_to_i64 to select sext vs zext.
    pub signed_locals: HashSet<String>,
    /// M17: XIOM type name for each local. Maps local name -> XIOM type string
    /// (e.g. "x" -> "Int8", "y" -> "UInt16"). Populated from declared type annotations.
    pub local_xiom_types: HashMap<String, String>,
    /// Names of the CURRENT function's parameters. Used to distinguish by-value
    /// `&T` params (ABI passes the VALUE -- `*r` is a no-op) from local variables
    /// that HOLD an address (`var r = &x` -- `*r` must deref).
    pub param_locals: HashSet<String>,
    /// Params declared with a plain `&T` reference type (address carried as i64).
    /// `&mut T` / `*T` params are real pointers (i64*) and are NOT listed here.
    pub ref_params: HashSet<String>,
    /// BUG 44: LOCALS bound from `&expr` or annotated `&T` (`var p = &s;`,
    /// `var p: &Str = ...`). They hold an ADDRESS (as i64 or a real pointer
    /// for Str pointees) -- deref (`*p`) must load through, and auto-coercion
    /// to the pointee value (&Str -> Str) must deref instead of treating the
    /// address as a byte value. Mirrors ref_params for non-param bindings.
    pub ref_locals: HashSet<String>,
    /// M17: Tracks which SSA register names hold signed integer values.
    /// Populated when values are created with known XIOM type (Ident loads,
    /// As expressions, literals). Consulted by widen_to_i64 to select sext/zext.
    pub reg_signed: HashMap<String, bool>,
    /// BUG 38: true while compiling the LEFT side of an `=>` Imply
    /// (contract ensures). The BUG 29 bare `is Some/Ok/Err` scrutinee-name
    /// payload rebind fires ONLY here -- in if/while conditions it poisoned
    /// the subsequent `match` on the same value (the scrutinee read as an
    /// i64 payload, so Some(v) arms bound 0 and skipped the disc check).
    pub in_imply_lhs: bool,
    /// v0.55: Whether @xiom_thread_spawn has been declared in this module
    pub spawn_declared: bool,
    /// v0.55: Counter for unique spawn function names
    pub spawn_counter: u32,
    /// R1: Counter for DWARF debug info metadata node numbering
    pub di_node_counter: u32,
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
