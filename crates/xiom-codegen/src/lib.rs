// XIOM -- LLVM IR Codegen
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Codegen -- Phase 0: AST to LLVM IR text..
//! Emits human-readable LLVM IR that can be compiled with `llc`.
//! No external dependencies ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â pure string emission.
//! Handles: functions, arithmetic, control flow (if/else/while/match),
//! let/var bindings, structs, function calls.

use xiom_ast::*;
use xiom_ctfe::CtfeEngine;
use std::collections::HashMap;
use std::collections::HashSet;
use std::cell::RefCell;
use std::sync::Arc;
use rayon::prelude::*;

pub mod call;
pub mod llvm_consts;
pub mod coerce;
pub mod context;
pub mod contracts;
pub mod decl;
pub mod emitter;
pub mod enum_ctors;
pub mod expr;
pub mod sandbox;
pub mod stmt;
pub mod vec_abi;

pub use context::{CodegenConfig, TypeContext, FunctionContext, MonoContext, LocalContext, TypeMeta, SyncRegistry};

// ============================================================================
// LLVM IR Emitter (M4.1: decomposed from 86-field god object into 5 sub-contexts)
// ============================================================================

pub struct IrEmitter {
    /// Accumulated LLVM IR text output of the compilation
    pub output: String,
    /// Counter for unique temporary names
    pub tmp_counter: u32,
    /// Counter for unique block labels
    pub block_counter: u32,
    /// Counter for unique string constants
    pub str_counter: u32,
    /// Whether @llvm.trap has been declared
    pub has_llvm_trap_decl: bool,
    /// D2.1 (Unsafe Confinement Phase 3, requirement d): guard-heap nesting
    /// depth. While > 0 (inside an `unsafe` block), allocations route to the
    /// per-thread guard arena (@xiom_guard_alloc) instead of @malloc; on block
    /// exit the arena is discarded wholesale (isolation) and the tail value is
    /// Copy-Out'd to the main heap (requirement i, UAF fix).
    pub guard_heap_depth: u32,
    /// D2.1 (Phase 5): globally-unique counter for emitted unsafe-block
    /// functions (`__unsafe_block_N`). Unlike tmp_counter/block_counter (which
    /// are reset per function by compile_fn), this monotonically increases so
    /// deferred block fn symbols never collide across enclosing functions.
    pub unsafe_block_counter: u32,
    /// D2.1 (Phase 5): 1 while compiling the standalone fn body of an unsafe
    /// block. When set, a `Stmt::Return` inside the block fn emits a
    /// @xiom_trampoline_set_returned() call first, so the call site knows to
    /// return the block's value from the ENCLOSING fn.
    pub in_unsafe_block_fn: bool,
    /// D2.1 (Phase 7): counted number of `#[unsafe_direct]` blocks compiled
    /// (audited cap against config.unsafe_direct_cap).
    pub unsafe_direct_count: u32,

    /// Compilation flags and target configuration
    pub config: CodegenConfig,
    /// Type system registration, interface/enum metadata, function signatures
    pub types: TypeContext,
    /// Per-function compilation state (locals, params, return type, ensures)
    pub fctx: FunctionContext,
    /// Program-wide: true when a non-empty `fn main` exists. Empty-body
    /// `async fn main() { }` placeholders must not shadow the real entry point.
    pub has_non_empty_main: bool,
    /// Monomorphisation worklist and instantiation tracking
    pub mono: MonoContext,
    /// Local variable classification, module-level globals, loop stack
    pub local: LocalContext,
    /// v0.54 Phase B: CTFE engine for compile-time function evaluation.
    /// Populated during register_functions; used by evaluate_const_init.
    pub ctfe: RefCell<CtfeEngine>,
}



impl IrEmitter {
    /// Security review (2026-08-13): maximum CTFE recursion depth for
    /// `evaluate_const_init`. Pathological const expressions bail out
    /// (evaluated at runtime instead) rather than hanging the compiler.
    pub const CONST_EVAL_BUDGET: u32 = 4096;

    pub fn new() -> Self {
        Self {
            output: String::new(),
            tmp_counter: 0,
            block_counter: 0,
            str_counter: 0,
            has_llvm_trap_decl: false,
    guard_heap_depth: 0,
            unsafe_block_counter: 0,
            in_unsafe_block_fn: false,
            unsafe_direct_count: 0,
            config: CodegenConfig::default(),
            types: TypeContext::default(),
            fctx: FunctionContext {
                locals: vec![HashMap::new()],
                ..FunctionContext::default()
            },
            has_non_empty_main: false,
            mono: MonoContext::default(),
            local: LocalContext::default(),
            ctfe: RefCell::new(CtfeEngine::new()),
        }
    }

    pub fn set_target_triple(&mut self, triple: &str) {
        self.config.target_triple = triple.to_string();
    }

    pub fn set_check_contracts(&mut self, enabled: bool) {
        self.config.check_contracts = enabled;
    }
    /// Security review (2026-08-13): release builds strip assert/dbg!/debugger;
    /// `--keep-debug-checks` (or a debug build) retains them.
    pub fn set_strip_debug_checks(&mut self, enabled: bool) {
        self.config.strip_debug_checks = enabled;
    }
    pub fn set_overflow_checks(&mut self, enabled: bool) {
        self.config.overflow_checks = enabled;
    }

    pub fn set_parallel_codegen(&mut self, enabled: bool) {
        self.config.parallel_codegen = enabled;
    }

    pub fn set_debug_symbols(&mut self, enabled: bool) {
        self.config.debug_symbols = enabled;
    }

    /// D2.1 (Phase 7): enable `#[unsafe_direct]` for user code (default: stdlib/
    /// trusted only). Mirrors the --enable-unsafe-direct CLI gate.
    pub fn set_enable_unsafe_direct(&mut self, enabled: bool) {
        self.config.enable_unsafe_direct = enabled;
    }

    /// BUG 25 #2 fix: alias â†’ full dotted use path (from the checker).
    pub fn set_use_alias_paths(&mut self, paths: std::collections::HashMap<String, String>) {
        self.config.use_alias_paths = paths;
    }

    pub fn set_source_file(&mut self, path: String) {
        self.config.source_file = path;
    }

    pub fn set_max_recursion_depth(&mut self, depth: u32) {
        self.config.max_recursion_depth = depth;
    }

    pub fn set_strict_mode(&mut self, strict: bool) {
        self.config.strict_mode = strict;
    }

    pub fn set_hot_reload(&mut self, enabled: bool) {
        self.config.hot_reload = enabled;
    }

    /// djb2 hash of a function name for stable pointer table index (5e.5a).
    pub(crate) fn djb2_hash(name: &str) -> i64 {
        let mut hash: u64 = 5381;
        for b in name.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u64);
        }
        (hash % 1024) as i64
    }

    /// 5e.5c: byte size of an LLVM type for state serialization.
    pub(crate) fn llvm_type_byte_size(llvm_ty: &str, type_meta: &SyncRegistry<String, TypeMeta>) -> usize {
        match llvm_ty {
            "i1" | "i8" => 1,
            "i16" => 2,
            "i32" | "float" => 4,
            "i64" | "double" => 8,
            ty if ty.starts_with("%struct.") => {
                let inner = &ty[8..];
                let meta = type_meta.get(&inner.to_string())
                    .or_else(|| type_meta.entries().into_iter().find(|(k,_)| k.ends_with(&format!(".{inner}"))).map(|(_,v)| v));
                match meta {
                    Some(m) => {
                        let mut total: usize = 0;
                        for (_, field_ty) in &m.fields {
                            if field_ty.contains('[') && !field_ty.starts_with('[') {
                                total += 8; // generic container ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ i64 handle
                            } else if field_ty.starts_with('[') {
                                total += 8; // fixed-size array ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ 8 per element simplified
                            } else {
                                let fllvm = if field_ty.starts_with('%') { field_ty.clone() }
                                    else { format!("%struct.{field_ty}") };
                                total += Self::llvm_type_byte_size(&fllvm, type_meta);
                            }
                        }
                        total
                    }
                    None => 8
                }
            }
            _ => 8,
        }
    }

    /// 5e.5c / 7D.2: emit xiom_hot_save_state() and xiom_hot_restore_state() functions.
    /// These serialize/deserialize all module-level `var` globals to `xiom_hot_state.bin`
    /// with a versioned header containing a layout hash to detect struct changes.
    fn emit_hot_state_functions(&mut self) {
        if self.config.xiom_hot_globals.is_empty() { return; }
        let globals_snapshot = self.config.xiom_hot_globals.clone();

        // Build layout metadata string: "name:type:size;name:type:size;..."
        let layout_metadata: String = globals_snapshot.iter()
            .map(|(sym, ty, sz)| format!("{}:{}:{}", sym, ty, sz))
            .collect::<Vec<_>>()
            .join(";");
        let layout_len = layout_metadata.len();

        // String constants
        self.emitln(&format!("@xiom_hot_state_path = private constant [20 x i8] c\"xiom_hot_state.bin\\00\""));
        self.emitln(&format!("@xiom_hot_wb = private constant [3 x i8] c\"wb\\00\""));
        self.emitln(&format!("@xiom_hot_rb = private constant [3 x i8] c\"rb\\00\""));
        // 7D.2: Layout metadata string (null-terminated)
        self.emitln(&format!("@xiom_hot_layout_meta = private constant [{} x i8] c\"{}\\00\"", layout_len + 1, layout_metadata));
        self.emitln("");

        // --- Save function (7D.2: with layout header) ---
        self.emitln("define void @xiom_hot_save_state() {");
        self.emitln("entry:");
        let f_save = self.fresh_tmp();
        self.emitln(&format!("  {f_save} = call i8* @fopen(i8* getelementptr inbounds ([20 x i8], [20 x i8]* @xiom_hot_state_path, i32 0, i32 0), i8* getelementptr inbounds ([3 x i8], [3 x i8]* @xiom_hot_wb, i32 0, i32 0))"));
        let null_s = self.fresh_tmp();
        self.emitln(&format!("  {null_s} = icmp eq i8* {f_save}, null"));
        let save_body = self.fresh_block("hot_save_body");
        let save_done = self.fresh_block("hot_save_done");
        self.emitln(&format!("  br i1 {null_s}, label %{save_done}, label %{save_body}"));
        self.emitln(&format!("\n{save_body}:"));
        // 7D.2: Write layout metadata first (so host can verify layout on restore)
        let meta_ptr = self.fresh_tmp();
        self.emitln(&format!("  {meta_ptr} = bitcast [{} x i8]* @xiom_hot_layout_meta to i8*", layout_len + 1));
        self.emitln(&format!("  call i64 @fwrite(i8* {meta_ptr}, i64 {}, i64 1, i8* {f_save})", layout_len + 1));
        // Write global variable data
        for (symbol, llvm_ty, byte_sz) in &globals_snapshot {
            let load_tmp = self.fresh_tmp();
            self.emitln(&format!("  {load_tmp} = load {llvm_ty}, {llvm_ty}* @{symbol}"));
            let buf = self.fresh_tmp();
            self.emitln(&format!("  {buf} = alloca {llvm_ty}"));
            self.emitln(&format!("  store {llvm_ty} {load_tmp}, {llvm_ty}* {buf}"));
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = bitcast {llvm_ty}* {buf} to i8*"));
            self.emitln(&format!("  call i64 @fwrite(i8* {bc}, i64 {byte_sz}, i64 1, i8* {f_save})"));
        }
        self.emitln(&format!("  call i32 @fclose(i8* {f_save})"));
        self.emitln(&format!("  br label %{save_done}"));
        self.emitln(&format!("\n{save_done}:"));
        self.emitln("  ret void");
        self.emitln("}\n");

        // --- Restore function (7D.2: with layout verification) ---
        self.emitln("define void @xiom_hot_restore_state() {");
        self.emitln("entry:");
        let f_restore = self.fresh_tmp();
        self.emitln(&format!("  {f_restore} = call i8* @fopen(i8* getelementptr inbounds ([20 x i8], [20 x i8]* @xiom_hot_state_path, i32 0, i32 0), i8* getelementptr inbounds ([3 x i8], [3 x i8]* @xiom_hot_rb, i32 0, i32 0))"));
        let null_r = self.fresh_tmp();
        self.emitln(&format!("  {null_r} = icmp eq i8* {f_restore}, null"));
        let restore_body = self.fresh_block("hot_restore_body");
        let restore_skip = self.fresh_block("hot_restore_skip");
        let restore_done = self.fresh_block("hot_restore_done");
        self.emitln(&format!("  br i1 {null_r}, label %{restore_done}, label %{restore_body}"));
        self.emitln(&format!("\n{restore_body}:"));
        // 7D.2: Read and verify layout metadata before restoring
        // If layout changed, skip restore (avoid corrupting state)
        let meta_buf = self.fresh_tmp();
        self.emitln(&format!("  {meta_buf} = alloca [{} x i8]", layout_len + 1));
        let meta_bc = self.fresh_tmp();
        self.emitln(&format!("  {meta_bc} = bitcast [{} x i8]* {meta_buf} to i8*", layout_len + 1));
        self.emitln(&format!("  call i64 @fread(i8* {meta_bc}, i64 {}, i64 1, i8* {f_restore})", layout_len + 1));
        // Compare layout metadata strings
        let cmp_tmp = self.fresh_tmp();
        self.emitln(&format!("  {cmp_tmp} = call i32 @strncmp(i8* {meta_bc}, i8* getelementptr inbounds ([{} x i8], [{} x i8]* @xiom_hot_layout_meta, i32 0, i32 0), i64 {})", layout_len + 1, layout_len + 1, layout_len));
        let layout_ok = self.fresh_tmp();
        self.emitln(&format!("  {layout_ok} = icmp eq i32 {cmp_tmp}, 0"));
        self.emitln(&format!("  br i1 {layout_ok}, label %{restore_body}_data, label %{restore_skip}"));
        // Data restore
        self.emitln(&format!("\n{restore_body}_data:"));
        for (symbol, llvm_ty, byte_sz) in &globals_snapshot {
            let buf = self.fresh_tmp();
            self.emitln(&format!("  {buf} = alloca {llvm_ty}"));
            let bc = self.fresh_tmp();
            self.emitln(&format!("  {bc} = bitcast {llvm_ty}* {buf} to i8*"));
            self.emitln(&format!("  call i64 @fread(i8* {bc}, i64 {byte_sz}, i64 1, i8* {f_restore})"));
            let val = self.fresh_tmp();
            self.emitln(&format!("  {val} = load {llvm_ty}, {llvm_ty}* {buf}"));
            self.emitln(&format!("  store {llvm_ty} {val}, {llvm_ty}* @{symbol}"));
        }
        self.emitln(&format!("  br label %{restore_skip}"));
        self.emitln(&format!("\n{restore_skip}:"));
        self.emitln(&format!("  call i32 @fclose(i8* {f_restore})"));
        self.emitln(&format!("  br label %{restore_done}"));
        self.emitln(&format!("\n{restore_done}:"));
        self.emitln("  ret void");
        self.emitln("}\n");
    }

    /// Convert literal "0" to "zeroinitializer" for aggregate (struct) types
    fn zero_val_for(&self, val: &str, llvm_ty: &str) -> String {
        if val.is_empty() {
            return Self::default_const_for(llvm_ty);
        }
        if val == "0" {
            if llvm_ty.ends_with('*') {
                return "null".to_string();
            }
            if llvm_ty.starts_with("%struct.") || llvm_ty.starts_with('[') {
                return "zeroinitializer".to_string();
            }
        }
        val.to_string()
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

    /// Produce a valid LLVM *constant* initializer for a module-level `var`
    /// global of type `llvm_ty` from its initializer expression. Only simple
    /// scalar literals (int/bool/float/char, with optional unary negation) are
    /// materialized to their real value ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â these are the initializers that
    /// currently-passing modules depend on (e.g. `_global_state = 12345`).
    /// Anything more complex (enum variants, struct/aggregate values,
    /// constructor calls like `Vec[T]::new()`) is zero-initialized: a valid,
    /// safe default. Such globals are always assigned before first meaningful
    /// read in practice.
    fn global_const_init(value: &Expr, llvm_ty: &str) -> String {
        match value {
            Expr::Int(n, _) => {
                if llvm_ty == "double" || llvm_ty == "float" {
                    format!("{n}.0")
                } else if llvm_ty.starts_with("%struct.") || llvm_ty.ends_with('*') || llvm_ty.starts_with('[') {
                    Self::default_const_for(llvm_ty)
                } else {
                    format!("{n}")
                }
            }
            Expr::Bool(b, _) => {
                if llvm_ty.starts_with('i') {
                    (if *b { "1" } else { "0" }).to_string()
                } else {
                    Self::default_const_for(llvm_ty)
                }
            }
            Expr::Float(f, _) => {
                if llvm_ty == "double" {
                    // BUG 10 fix (2026-08-11): {:.6} truncated literals to 6
                    // decimals; {:.17e} round-trips f64 exactly.
                    format!("{f:.17e}")
                } else if llvm_ty == "float" {
                    // CG-02: LLVM requires float constants to round-trip exactly
                    // through decimalÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢doubleÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢float. Use ryu crate or manual formatting
                    // with enough digits for the exact float32ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢float64ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢decimalÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢float64
                    // roundtrip. 17 significant digits is enough.
                    let f32_val = *f as f32;
                    format!("{:.17e}", f32_val)
                } else {
                    Self::default_const_for(llvm_ty)
                }
            }
            Expr::Char(c, _) => {
                if llvm_ty.starts_with('i') {
                    format!("{}", *c as u32)
                } else {
                    Self::default_const_for(llvm_ty)
                }
            }
            Expr::Unary(UnaryOp::Neg, inner, _) => {
                if let Expr::Int(n, _) = inner.as_ref() {
                    if llvm_ty == "double" || llvm_ty == "float" {
                        format!("-{n}.0")
                    } else if llvm_ty.starts_with('i') {
                        format!("-{n}")
                    } else {
                        Self::default_const_for(llvm_ty)
                    }
                } else {
                    Self::default_const_for(llvm_ty)
                }
            }
            _ => Self::default_const_for(llvm_ty),
        }
    }

    // ========================================================================
    // 5e.7f: Const Evaluation ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â walk & fold const expressions at compile time
    // ========================================================================

    /// Evaluate a const expression to a literal value by recursively resolving
    /// const references and folding arithmetic. Returns `Some(Expr)` on full
    /// evaluation, `None` if the expression cannot be const-evaluated.
    /// `depth` tracks recursion depth for cycle detection (max 128).
    #[allow(dead_code)]
    pub(crate) fn const_eval(
        expr: &Expr,
        constants: &HashMap<String, Expr>,
        depth: u32,
    ) -> Option<Expr> {
        if depth > 128 {
            return None; // cycle detected or too deep
        }
        match expr {
            // Literals return themselves
            Expr::Int(..) | Expr::Float(..) | Expr::Bool(..) | Expr::Char(..) | Expr::Str(..) => {
                Some(expr.clone())
            }
            // Const reference: look up and recurse
            Expr::Ident(ident) => {
                constants.get(&ident.name)
                    .and_then(|val| Self::const_eval(val, constants, depth + 1))
            }
            // Fold arithmetic
            Expr::Binary(lhs, op, rhs, _) => {
                let l = Self::const_eval(lhs, constants, depth + 1)?;
                let r = Self::const_eval(rhs, constants, depth + 1)?;
                Self::const_fold_binary(&l, op, &r)
            }
            // Unary negation
            Expr::Unary(UnaryOp::Neg, inner, _) => {
                let v = Self::const_eval(inner, constants, depth + 1)?;
                match v {
                    Expr::Int(n, _) => Some(Expr::Int((n as i64).wrapping_neg() as u64, Span::new(0, 0))),
                    Expr::Float(f, _) => Some(Expr::Float(-f, Span::new(0, 0))),
                    _ => None,
                }
            }
            // Not-const: calls, casts, field access, index, etc.
            _ => None,
        }
    }

    /// Fold a binary operation on two literal const expressions.
    #[allow(dead_code)]
    fn const_fold_binary(lhs: &Expr, op: &BinOp, rhs: &Expr) -> Option<Expr> {
        let s = Span::new(0, 0);
        match (lhs, rhs) {
            (Expr::Int(a, _), Expr::Int(b, _)) => {
                let a = *a; let b = *b;
                match op {
                    BinOp::Add => Some(Expr::Int(a + b, s)),
                    BinOp::Sub => Some(Expr::Int((a as i64).wrapping_sub(b as i64) as u64, s)),
                    BinOp::Mul => Some(Expr::Int(a * b, s)),
                    BinOp::Div => {
                        if b == 0 { return None; }
                        Some(Expr::Int((a as i64 / b as i64) as u64, s))
                    }
                    BinOp::Rem => {
                        if b == 0 { return None; }
                        Some(Expr::Int((a as i64 % b as i64) as u64, s))
                    }
                    BinOp::Shl => {
                        if b > 63 { return None; }
                        Some(Expr::Int(a << b, s))
                    }
                    BinOp::Shr => {
                        if b > 63 { return None; }
                        Some(Expr::Int((a as i64 >> b) as u64, s))
                    }
                    BinOp::BitAnd => Some(Expr::Int(a & b, s)),
                    BinOp::BitOr => Some(Expr::Int(a | b, s)),
                    BinOp::BitXor => Some(Expr::Int(a ^ b, s)),
                    _ => None,
                }
            }
            (Expr::Float(a, _), Expr::Float(b, _)) => {
                let a = *a; let b = *b;
                match op {
                    BinOp::Add => Some(Expr::Float(a + b, s)),
                    BinOp::Sub => Some(Expr::Float(a - b, s)),
                    BinOp::Mul => Some(Expr::Float(a * b, s)),
                    BinOp::Div => Some(Expr::Float(a / b, s)),
                    _ => None,
                }
            }
            (Expr::Int(a, _), Expr::Float(b, _)) => {
                let a = *a as f64; let b = *b;
                match op {
                    BinOp::Add => Some(Expr::Float(a + b, s)),
                    BinOp::Sub => Some(Expr::Float(a - b, s)),
                    BinOp::Mul => Some(Expr::Float(a * b, s)),
                    BinOp::Div => Some(Expr::Float(a / b, s)),
                    _ => None,
                }
            }
            (Expr::Float(a, _), Expr::Int(b, _)) => {
                let a = *a; let b = *b as f64;
                match op {
                    BinOp::Add => Some(Expr::Float(a + b, s)),
                    BinOp::Sub => Some(Expr::Float(a - b, s)),
                    BinOp::Mul => Some(Expr::Float(a * b, s)),
                    BinOp::Div => Some(Expr::Float(a / b, s)),
                    _ => None,
                }
            }
            (Expr::Bool(a, _), Expr::Bool(b, _)) => {
                let a = *a; let b = *b;
                match op {
                    BinOp::And => Some(Expr::Bool(a && b, s)),
                    BinOp::Or => Some(Expr::Bool(a || b, s)),
                    _ => None,
                }
            }
            _ => None,
        }
    }


    /// 5e.7f: Evaluate all registered constants in-place. Run after
    /// register_functions so cross-references between consts resolve.
    /// v0.54: uses evaluate_const_init (full CTFE with builtins, comparison,
    /// boolean ops, if/else folding) instead of the limited const_eval.
    fn evaluate_all_consts(&mut self) {
        let keys: Vec<String> = self.local.constants.keys().cloned().collect();
        for name in keys {
            if let Some(expr) = self.local.constants.get(&name).cloned() {
                let evaluated = self.evaluate_const_init(&expr);
                self.local.constants.insert(name, evaluated);
            }
        }
    }

    /// Returns `true` when `xiom_type` is a signed integer (Int, Int8, Int16, Int32, Int64).
    fn is_signed_xiom_type(xiom_type: &str) -> bool {
        matches!(xiom_type, "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "Int128")
    }

    /// D1 (2026-08-08): alignment suffix for allocas of 128-bit types.
    /// x86-64 requires 16-byte alignment for i128/fp128 loads/stores; clang
    /// defaults unaligned allocas to 4-byte alignment which faults (#GP) on
    /// Windows x64 for i128/fp128 memory ops. Struct types are aligned only
    /// when they contain an i128/fp128 field (verified: blanket-aligning all
    /// structs, e.g. %struct.Vec, interacts badly with clang frame layout at
    /// -O0/-O1 when an i128 loop-carried local coexists in the same frame).
    fn alloca_align(&self, ty: &str) -> &'static str {
        if ty == "i128" || ty == "fp128" || self.struct_contains_128(&ty) { ", align 16" } else { "" }
    }

    /// D2.1 (Unsafe Confinement Phase 3, requirement d): emit a heap allocation
    /// that routes to the guard arena when inside an `unsafe` block (guard_heap
    /// depth > 0), else to @malloc. The guard arena is discarded wholesale on
    /// block exit, isolating unsafe-block allocations from the main heap.
    fn emit_alloc(&mut self, size_expr: &str) -> String {
        let tmp = self.fresh_tmp();
        if self.guard_heap_depth > 0 {
            self.emitln(&format!("  {tmp} = call i8* @xiom_guard_alloc(i64 {size_expr})"));
        } else {
            self.emitln(&format!("  {tmp} = call i8* @malloc(i64 {size_expr})"));
        }
        tmp
    }

    /// D2.1 (Phase 3, fix): emit a realloc call that routes to the arena-aware
    /// @xiom_guard_realloc when inside a confined block (grows a guard-arena
    /// block by copying to a fresh arena slab), else plain @realloc. Vec growth
    /// inside unsafe blocks MUST NOT use plain realloc â€” it would realloc a
    /// VirtualAlloc slab pointer (invalid) and corrupt/crash the process.
    fn emit_realloc(&mut self, old_ptr: &str, old_size: &str, new_size: &str) -> String {
        let tmp = self.fresh_tmp();
        if self.guard_heap_depth > 0 {
            self.emitln(&format!("  {tmp} = call i8* @xiom_guard_realloc(i8* {old_ptr}, i64 {old_size}, i64 {new_size})"));
        } else {
            self.emitln(&format!("  {tmp} = call i8* @realloc(i8* {old_ptr}, i64 {new_size})"));
        }
        tmp
    }

    /// D1: alignment suffix for stores/loads of 128-bit types.
    fn store_align(&self, ty: &str) -> &'static str {
        if ty == "i128" || ty == "fp128" || self.struct_contains_128(&ty) { ", align 16" } else { "" }
    }

    /// True if the struct type's field list contains an i128/fp128 field.
    /// Fields are stored as (name, xiom_type) tuples; check the type part for
    /// the 128-bit XIOM primitive names.
    fn struct_contains_128(&self, ty: &str) -> bool {
        if !ty.starts_with("%struct.") { return false; }
        let name = ty.trim_start_matches("%struct.").to_string();
        self.types.type_meta.get(&name)
            .map(|meta| meta.fields.iter().any(|(_, ft)| {
                ft == "Int128" || ft == "UInt128" || ft == "Float128"
            }))
            .unwrap_or(false)
    }

    /// Widen a narrow integer value (`i1`/`i8`/`i16`/`i32`) to `i64` so it can
    /// participate in the emitter's i64 integer arithmetic/comparison model.
    /// Consults `self.local.reg_signed` for per-register signedness; falls back
    /// to type-based defaults: zext for i1/i8, sext for i16/i32.
    fn widen_to_i64(&mut self, val: &str, ty: &str) -> String {
        let is_signed = self.local.reg_signed.get(val).copied().unwrap_or_else(|| {
            // Default: sext for i8/i16/i32 (signed types are the common case for
            // function returns and intermediate values). zext only for i1 (Bool).
            // The Ident load path and As expression handler provide per-register
            // overrides via reg_signed for unsigned locals.
            !matches!(ty, "i1")
        });
        self.widen_to_i64_signed(val, ty, is_signed)
    }

    /// Widen with explicit signedness control. `is_signed=true` uses `sext`;
    /// `is_signed=false` uses `zext`.
    fn widen_to_i64_signed(&mut self, val: &str, ty: &str, is_signed: bool) -> String {
        match ty {
            "i1" => {
                // Bool: always zero-extend (false=0, true=1)
                let ext = self.fresh_tmp();
                self.emitln(&format!("  {ext} = zext i1 {val} to i64"));
                ext
            }
            "i8" | "i16" | "i32" => {
                let ext = self.fresh_tmp();
                let op = if is_signed { "sext" } else { "zext" };
                self.emitln(&format!("  {ext} = {op} {ty} {val} to i64"));
                ext
            }
            // A real pointer used in integer arithmetic (e.g. a `&mut Int` param
            // used as a bare Int: `pos + 1`): take its integer address so the
            // `add`/`sub`/... is well-typed. Semantics match address arithmetic.
            t if t.ends_with('*') => {
                let iv = self.fresh_tmp();
                self.emitln(&format!("  {iv} = ptrtoint {t} {val} to i64"));
                iv
            }
            _ => val.to_string(),
        }
    }

    /// Coerce `val` (whose current LLVM type is `from`) to the LLVM type `to`,
    /// emitting the appropriate cast, and return the resulting SSA value. Used at
    /// the value "sink" points ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â call arguments, returns, and stores ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â so a value
    /// always matches the type its context requires (LLVM is strongly typed).
    ///
    /// Handles integer width (zext/sext/trunc), int<->pointer (inttoptr/ptrtoint),
    /// pointer<->pointer (bitcast), int<->double (sitofp/fptosi), and int->struct
    /// (via `val_to_struct`, e.g. a single-field enum like Ordering). No-ops when
    /// the types already match, when `val` is empty/a null literal, or when no
    /// meaningful cast applies.
    /// Produce the final SSA value for a call argument, honoring real-pointer
    /// parameters. When the callee's param LLVM type is a pointer (e.g. `i64*` for a
    /// `*Int` / `&mut Scalar` param) and the argument is an address-of a scalar
    /// lvalue (`&x` / `&mut x`), pass the local's ALLOCA address rather than an
    /// `inttoptr` of its loaded value (which would fabricate a bogus pointer and
    /// crash). A local that already holds a pointer is forwarded as-is. All other
    /// cases fall back to the ordinary `coerce_value` on the precompiled value.

    /// Quick scan: returns true if the expression tree contains any `this` ident.
    /// 5c.29: true when `e` is an `x.unwrap()`-style call. Their i64 ABI
    /// result carries float payloads as RAW BITS (Some(x) boxes via bitcast),
    /// so float-context conversions must bit-reinterpret rather than sitofp.
    fn expr_is_unwrap_call(e: &Expr) -> bool {
        if let Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) = e {
            if let Expr::Field(_, f, _) = func.as_ref() {
                return matches!(f.name.as_str(), "unwrap" | "unwrap_or" | "unwrap_err");
            }
        }
        false
    }

    fn expr_uses_this(expr: &Expr) -> bool {        match expr {
            Expr::Ident(id) => id.name == "this",
            Expr::Paren(e, _) | Expr::Unary(_, e, _) | Expr::Try(e, _)
            | Expr::Ref(e, _) | Expr::MutRef(e, _)
            | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _)
            | Expr::As(e, _, _) => Self::expr_uses_this(e),
            Expr::Binary(a, _, b, _) => Self::expr_uses_this(a) || Self::expr_uses_this(b),
            Expr::Field(obj, _, _) => Self::expr_uses_this(obj),
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => Self::expr_uses_this(func) || args.iter().any(|a| Self::expr_uses_this(a)),
            Expr::Index(arr, idx, _) => Self::expr_uses_this(arr) || Self::expr_uses_this(idx),
            Expr::If(cond, then_b, elifs, else_b, _) => {
                Self::expr_uses_this(cond)
                    || Self::block_uses_this(then_b)
                    || elifs.iter().any(|(c, b)| Self::expr_uses_this(c) || Self::block_uses_this(b))
                    || else_b.as_ref().map_or(false, |b| Self::block_uses_this(b))
            }
            Expr::Match(scrut, arms, _) => {
                Self::expr_uses_this(scrut)
                    || arms.iter().any(|arm| match &arm.body {
                        MatchBody::Block(b) => Self::block_uses_this(b),
                        MatchBody::Expr(e) => Self::expr_uses_this(e),
                    })
            }
            Expr::Array(elems, _) | Expr::Tuple(elems, _) => elems.iter().any(|e| Self::expr_uses_this(e)),
            Expr::Struct(_, fields, base, _) => {
                fields.iter().any(|(_, v)| Self::expr_uses_this(v))
                    || base.as_ref().map_or(false, |b| Self::expr_uses_this(b))
            }
            _ => false,
        }
    }

    fn stmt_uses_this(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e, _) | Stmt::Return(Some(e), _) => Self::expr_uses_this(e),
            Stmt::Let(_, _, init, _) | Stmt::Var(_, _, init, _) => Self::expr_uses_this(init),
            Stmt::Assign(_, rhs, _) => Self::expr_uses_this(rhs),
            Stmt::If(cond, then_b, elifs, else_b, _) => {
                Self::expr_uses_this(cond) || Self::block_uses_this(then_b)
                    || elifs.iter().any(|(c, b)| Self::expr_uses_this(c) || Self::block_uses_this(b))
                    || else_b.as_ref().map_or(false, |b| Self::block_uses_this(b))
            }
            Stmt::While(cond, body, _, _, _) => Self::expr_uses_this(cond) || Self::block_uses_this(body),
            Stmt::Match(scrut, arms, _) => {
                Self::expr_uses_this(scrut)
                    || arms.iter().any(|arm| match &arm.body {
                        MatchBody::Block(b) => Self::block_uses_this(b),
                        MatchBody::Expr(e) => Self::expr_uses_this(e),
                    })
            }
            _ => false,
        }
    }

    fn block_uses_this(block: &Block) -> bool {
        block.stmts.iter().any(|s| match s {
            StmtOrExpr::Stmt(stmt) => Self::stmt_uses_this(stmt),
            StmtOrExpr::Expr(expr) => Self::expr_uses_this(expr),
        })
    }

    /// G-20: does the method body reference receiver STATE ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â either `this`
    /// or a BARE receiver-field ident (e.g. `val` in `fn Counter.inc() ->
    /// Int { return val + 1; }`)? Used by registration + definition to emit
    /// a %param_self slot so the prologue can bind bare fields via GEP.
    /// Param names shadow fields (a param `x` is never receiver state).
    pub(crate) fn body_uses_receiver_state(&self, fd: &FnDecl) -> bool {
        let Some(body) = fd.body.as_ref() else { return false };
        if Self::block_uses_this(body) {
            return true;
        }
        let Some(recv) = fd.receiver.as_ref() else { return false };
        // 5e.3: check BOTH types and type_meta ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â catalog-loaded types (e.g.,
        // benchmark modules) may only be in type_meta, not types.
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
        }).or_else(|| {
            let suffix = format!(".{}", recv.name);
            self.types.type_meta.keys().into_iter().find(|k| k.ends_with(&suffix)).and_then(|k| {
                self.types.type_meta.get(&k).map(|m| {
                    m.fields.iter().map(|(n, _)| n.clone()).collect()
                })
            })
        });
        let Some(fields) = fields else { return false };
        // Names bound ANYWHERE in the fn shadow receiver fields: params,
        // let/var locals, for-binders, match-pattern bindings. A bare ident
        // that is ever locally bound is NOT receiver state (constructors like
        // `HttpRequest.new` commonly use locals named after fields).
        let mut bound: std::collections::HashSet<String> =
            fd.params.iter().map(|p| p.name.name.clone()).collect();
        Self::collect_bound_names(body, &mut bound);
        let candidates: Vec<&str> = fields.iter()
            .map(|f| f.as_str())
            .filter(|f| !bound.contains(**&f))
            .collect();
        if candidates.is_empty() { return false; }
        Self::block_mentions_any_ident(body, &candidates)
    }

    /// 5c.32: Returns true if the block contains any reference to the `self`
    /// identifier (e.g. `self.val`). Used to detect by-value self methods where
    /// the parser stores the receiver but does not inject `self` into fd.params.
    pub(crate) fn block_uses_self_ident(block: &Block) -> bool {
        Self::block_mentions_self(block)
    }

    fn block_mentions_self(block: &Block) -> bool {
        block.stmts.iter().any(|s| match s {
            StmtOrExpr::Stmt(stmt) => Self::stmt_mentions_self(stmt),
            StmtOrExpr::Expr(expr) => Self::expr_mentions_self(expr),
        })
    }

    fn stmt_mentions_self(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(e, _) | Stmt::Return(Some(e), _) => Self::expr_mentions_self(e),
            Stmt::Let(_, _, init, _) | Stmt::Var(_, _, init, _) => Self::expr_mentions_self(init),
            Stmt::Assign(lhs, rhs, _) => Self::expr_mentions_self(lhs) || Self::expr_mentions_self(rhs),
            Stmt::If(cond, then_b, elifs, else_b, _) => {
                Self::expr_mentions_self(cond) || Self::block_mentions_self(then_b)
                    || elifs.iter().any(|(c, b)| Self::expr_mentions_self(c) || Self::block_mentions_self(b))
                    || else_b.as_ref().map_or(false, |b| Self::block_mentions_self(b))
            }
            Stmt::While(cond, body, _, _, _) => Self::expr_mentions_self(cond) || Self::block_mentions_self(body),
            Stmt::For(_, iter, body, _, _) => Self::expr_mentions_self(iter) || Self::block_mentions_self(body),
            Stmt::Match(scrut, arms, _) => {
                Self::expr_mentions_self(scrut)
                    || arms.iter().any(|arm| match &arm.body {
                        MatchBody::Block(b) => Self::block_mentions_self(b),
                        MatchBody::Expr(e) => Self::expr_mentions_self(e),
                    })
            }
            _ => false,
        }
    }

    fn expr_mentions_self(expr: &Expr) -> bool {
        match expr {
            Expr::Ident(id) => id.name == "self",
            Expr::Paren(e, _) | Expr::Unary(_, e, _) | Expr::Try(e, _)
            | Expr::Ref(e, _) | Expr::MutRef(e, _)
            | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _)
            | Expr::As(e, _, _) => Self::expr_mentions_self(e),
            Expr::Field(obj, _, _) => Self::expr_mentions_self(obj),
            Expr::Binary(a, _, b, _) => Self::expr_mentions_self(a) || Self::expr_mentions_self(b),
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => Self::expr_mentions_self(func) || args.iter().any(|a| Self::expr_mentions_self(a)),
            Expr::Index(arr, idx, _) => Self::expr_mentions_self(arr) || Self::expr_mentions_self(idx),
            Expr::If(cond, then_b, elifs, else_b, _) => {
                Self::expr_mentions_self(cond) || Self::block_mentions_self(then_b)
                    || elifs.iter().any(|(c, b)| Self::expr_mentions_self(c) || Self::block_mentions_self(b))
                    || else_b.as_ref().map_or(false, |b| Self::block_mentions_self(b))
            }
            Expr::Match(scrut, arms, _) => {
                Self::expr_mentions_self(scrut)
                    || arms.iter().any(|arm| match &arm.body {
                        MatchBody::Block(b) => Self::block_mentions_self(b),
                        MatchBody::Expr(e) => Self::expr_mentions_self(e),
                    })
            }
            Expr::Array(elems, _) | Expr::Tuple(elems, _) => elems.iter().any(|e| Self::expr_mentions_self(e)),
            Expr::Struct(_, fields, base, _) => {
                fields.iter().any(|(_, v)| Self::expr_mentions_self(v))
                    || base.as_ref().map_or(false, |b| Self::expr_mentions_self(b))
            }
            _ => false,
        }
    }

    /// Collect every name bound by let/var/for/match-patterns in a block
    /// (recursively). Conservative shadow set for body_uses_receiver_state.
    fn collect_bound_names(block: &Block, out: &mut std::collections::HashSet<String>) {
        for item in &block.stmts {
            match item {
                StmtOrExpr::Stmt(stmt) => Self::stmt_collect_bound(stmt, out),
                StmtOrExpr::Expr(e) => Self::expr_collect_bound(e, out),
            }
        }
    }

    fn stmt_collect_bound(stmt: &Stmt, out: &mut std::collections::HashSet<String>) {
        match stmt {
            Stmt::Let(name, _, init, _) | Stmt::Var(name, _, init, _) => {
                out.insert(name.name.clone());
                Self::expr_collect_bound(init, out);
            }
            Stmt::If(_, then_b, elifs, else_b, _) => {
                Self::collect_bound_names(then_b, out);
                for (_, b) in elifs { Self::collect_bound_names(b, out); }
                if let Some(b) = else_b { Self::collect_bound_names(b, out); }
            }
            Stmt::While(_, body, _, _, _) => Self::collect_bound_names(body, out),
            Stmt::For(binder, _, body, _, _) => {
                out.insert(binder.name.clone());
                Self::collect_bound_names(body, out);
            }
            Stmt::Match(_, arms, _) => {
                for arm in arms {
                    Self::pattern_collect_bound(&arm.pattern, out);
                    match &arm.body {
                        MatchBody::Block(b) => Self::collect_bound_names(b, out),
                        MatchBody::Expr(e) => Self::expr_collect_bound(e, out),
                    }
                }
            }
            Stmt::Expr(e, _) | Stmt::Return(Some(e), _) => Self::expr_collect_bound(e, out),
            _ => {}
        }
    }

    fn expr_collect_bound(expr: &Expr, out: &mut std::collections::HashSet<String>) {
        match expr {
            Expr::Unsafe(b, _) => Self::collect_bound_names(b, out),
            Expr::If(_, then_b, elifs, else_b, _) => {
                Self::collect_bound_names(then_b, out);
                for (_, b) in elifs { Self::collect_bound_names(b, out); }
                if let Some(b) = else_b { Self::collect_bound_names(b, out); }
            }
            Expr::Match(_, arms, _) => {
                for arm in arms {
                    Self::pattern_collect_bound(&arm.pattern, out);
                    match &arm.body {
                        MatchBody::Block(b) => Self::collect_bound_names(b, out),
                        MatchBody::Expr(e) => Self::expr_collect_bound(e, out),
                    }
                }
            }
            _ => {}
        }
    }

    fn pattern_collect_bound(pat: &Pattern, out: &mut std::collections::HashSet<String>) {
        match pat {
            Pattern::Ident(id) => { out.insert(id.name.clone()); }
            Pattern::Variant(_, fields, _) => {
                for f in fields { out.insert(f.name.clone()); }
            }
            Pattern::Some(inner, _) | Pattern::Ok(inner, _) | Pattern::Err(inner, _) => {
                Self::pattern_collect_bound(inner, out);
            }
            Pattern::Or(alts, _) => {
                for a in alts { Self::pattern_collect_bound(a, out); }
            }
            Pattern::Struct(_, fields, _) => {
                for (_, sub) in fields { Self::pattern_collect_bound(sub, out); }
            }
            Pattern::Tuple(elements, _) => {
                for e in elements { Self::pattern_collect_bound(e, out); }
            }
            _ => {}
        }
    }

    fn block_mentions_any_ident(block: &Block, names: &[&str]) -> bool {
        block.stmts.iter().any(|s| match s {
            StmtOrExpr::Stmt(stmt) => Self::stmt_mentions_any_ident(stmt, names),
            StmtOrExpr::Expr(expr) => Self::expr_mentions_any_ident(expr, names),
        })
    }

    fn stmt_mentions_any_ident(stmt: &Stmt, names: &[&str]) -> bool {
        match stmt {
            Stmt::Expr(e, _) | Stmt::Return(Some(e), _) => Self::expr_mentions_any_ident(e, names),
            Stmt::Let(_, _, init, _) | Stmt::Var(_, _, init, _) => Self::expr_mentions_any_ident(init, names),
            Stmt::Assign(lhs, rhs, _) => Self::expr_mentions_any_ident(lhs, names) || Self::expr_mentions_any_ident(rhs, names),
            Stmt::If(cond, then_b, elifs, else_b, _) => {
                Self::expr_mentions_any_ident(cond, names) || Self::block_mentions_any_ident(then_b, names)
                    || elifs.iter().any(|(c, b)| Self::expr_mentions_any_ident(c, names) || Self::block_mentions_any_ident(b, names))
                    || else_b.as_ref().map_or(false, |b| Self::block_mentions_any_ident(b, names))
            }
            Stmt::While(cond, body, _, _, _) => Self::expr_mentions_any_ident(cond, names) || Self::block_mentions_any_ident(body, names),
            Stmt::For(_, iter, body, _, _) => Self::expr_mentions_any_ident(iter, names) || Self::block_mentions_any_ident(body, names),
            Stmt::Match(scrut, arms, _) => {
                Self::expr_mentions_any_ident(scrut, names)
                    || arms.iter().any(|arm| match &arm.body {
                        MatchBody::Block(b) => Self::block_mentions_any_ident(b, names),
                        MatchBody::Expr(e) => Self::expr_mentions_any_ident(e, names),
                    })
            }
            _ => false,
        }
    }

    fn expr_mentions_any_ident(expr: &Expr, names: &[&str]) -> bool {
        match expr {
            Expr::Ident(id) => names.contains(&id.name.as_str()),
            Expr::Paren(e, _) | Expr::Unary(_, e, _) | Expr::Try(e, _)
            | Expr::Ref(e, _) | Expr::MutRef(e, _)
            | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _)
            | Expr::As(e, _, _) => Self::expr_mentions_any_ident(e, names),
            // obj.FIELD: the field NAME is not a bare ident ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â only scan the object.
            Expr::Field(obj, _, _) => Self::expr_mentions_any_ident(obj, names),
            Expr::Binary(a, _, b, _) => Self::expr_mentions_any_ident(a, names) || Self::expr_mentions_any_ident(b, names),
            Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => Self::expr_mentions_any_ident(func, names) || args.iter().any(|a| Self::expr_mentions_any_ident(a, names)),
            Expr::Index(arr, idx, _) => Self::expr_mentions_any_ident(arr, names) || Self::expr_mentions_any_ident(idx, names),
            Expr::Unsafe(block, _) => Self::block_mentions_any_ident(block, names),
            Expr::If(cond, then_b, elifs, else_b, _) => {
                Self::expr_mentions_any_ident(cond, names) || Self::block_mentions_any_ident(then_b, names)
                    || elifs.iter().any(|(c, b)| Self::expr_mentions_any_ident(c, names) || Self::block_mentions_any_ident(b, names))
                    || else_b.as_ref().map_or(false, |b| Self::block_mentions_any_ident(b, names))
            }
            Expr::Match(scrut, arms, _) => {
                Self::expr_mentions_any_ident(scrut, names)
                    || arms.iter().any(|arm| match &arm.body {
                        MatchBody::Block(b) => Self::block_mentions_any_ident(b, names),
                        MatchBody::Expr(e) => Self::expr_mentions_any_ident(e, names),
                    })
            }
            Expr::Array(elems, _) | Expr::Tuple(elems, _) => elems.iter().any(|e| Self::expr_mentions_any_ident(e, names)),
            Expr::Struct(_, fields, base, _) => {
                fields.iter().any(|(_, v)| Self::expr_mentions_any_ident(v, names))
                    || base.as_ref().map_or(false, |b| Self::expr_mentions_any_ident(b, names))
            }
            _ => false,
        }
    }

    fn is_primitive_type_name(type_name: &str) -> bool {
        matches!(
            type_name,
            "Bool" | "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "Int128"
                | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64" | "UInt128"
                | "Float32" | "Float64" | "Float128" | "Char" | "Str"
        )
    }

    fn xiom_to_llvm_type(xiom_ty: &str) -> &'static str {
        match xiom_ty {
            "Int8" | "UInt8" => "i8",
            "Char" => "i32",
            "Bool" => "i64",
            "Int16" | "UInt16" => "i16",
            "Int32" | "UInt32" => "i32",
            "Int" | "Int64" | "UInt" | "UInt64" => "i64",
            "Int128" | "UInt128" => "i128",
            "Float32" => "float",
            "Float64" => "double",
            "Float128" => "fp128",
            "Str" => "i8*",
            "()" => "void",
            // v0.56/P2-4: Never type (!) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â bottom type, never returns a value.
            "!" => "void",
            "Unit" => "i64", // Zero-sized type ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â stored as i64 in Result/Option
            // Known container type names ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â these are struct types resolved
            // via llvm_type_for/type_meta, not primitives. Silent i64 fallback.
            "Vec" | "Map" | "Set" | "Option" | "Result" => "i64",
            // M16: Silent i64 defaults for types that are expected to be
            // unresolved during generic-compilation passes.
            _ => {
                // Underscore placeholder ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â wildcard/inferred type that should
                // never produce a diagnostic. Silently default to i64.
                if xiom_ty == "_" {
                    return "i64";
                }
                // Generic type parameters: T, K, V, E, A, B, etc.
                if xiom_ty.len() == 1 && xiom_ty.chars().next().map_or(false, |c| c.is_uppercase()) {
                    return "i64";
                }
                // Self-receiver placeholder in type_meta field lists.
                if xiom_ty == "Self" {
                    return "i64";
                }
                // Bracket-preserving type names (Vec[T], Map[K,V], etc.)
                // stored by type_from_ast_with_args ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â strip to base and recurse.
                if let Some(_stripped) = xiom_ty.strip_prefix("Vec[")
                    .or_else(|| xiom_ty.strip_prefix("Map["))
                    .or_else(|| xiom_ty.strip_prefix("Set["))
                    .or_else(|| xiom_ty.strip_prefix("Option["))
                    .or_else(|| xiom_ty.strip_prefix("Result["))
                    .and_then(|rest| rest.strip_suffix(']'))
                {
                    let base_name = match xiom_ty {
                        t if t.starts_with("Vec[") => "Vec",
                        t if t.starts_with("Map[") => "Map",
                        t if t.starts_with("Set[") => "Set",
                        t if t.starts_with("Option[") => "Option",
                        t if t.starts_with("Result[") => "Result",
                        _ => xiom_ty,
                    };
                    return Self::xiom_to_llvm_type(base_name);
                }
                // Well-known marker/zero-sized types ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â silently return i64 without
                // warning to keep diagnostics clean for stdlib-internal types.
                if matches!(xiom_ty, "PhantomData" | "MaybeUninit" | "ManuallyDrop"
                    | "Unpin" | "PhantomPinned" | "UnsafeCell" | "Cell" | "RefCell") {
                    return "i64";
                }
                eprintln!("xiom: warning: unknown type '{}' ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â defaulting to i64. This may produce incorrect code.", xiom_ty);
                "i64"
            }
        }
    }

    /// Map an AST Type to its LLVM type string, handling pointer types (`*T` -> `<T>*`),
    /// ref types (`&T` -> `<T>*`), and named/builtin types.
    fn extern_type_to_llvm(&self, ty: &Type) -> String {
        match ty {
            Type::Ptr(inner) | Type::Ref(inner) | Type::MutRef(inner) => {
                let inner_llvm = self.extern_type_to_llvm(inner);
                // LLVM has no `void*`; a pointer to unit/void is represented as i8*.
                if inner_llvm == "void" {
                    "i8*".to_string()
                } else {
                    format!("{}*", inner_llvm)
                }
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

    pub(crate) fn xiom_type_name_from_llvm(llvm_ty: &str) -> String {
        // Check pointer types before stripping `*` ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â `i8*` is Str, not Int8.
        if llvm_ty == "i8*" { return "Str".to_string(); }
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
            _ => base.to_string(),
        }
    }

    /// 5c.36: Resolve field index by name, with tuple numeric/underscore fallback.
    /// Accepts both `0` and `_0` for tuple field names.
    pub(crate) fn resolve_field_index(field_names: &[String], field_name: &str) -> Option<usize> {
        if let Some(idx) = field_names.iter().position(|f| f == field_name) {
            return Some(idx);
        }
        // Numeric name ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ try with underscore prefix (legacy _N format)
        if field_name.chars().all(|c| c.is_ascii_digit()) {
            let alt = format!("_{field_name}");
            if let Some(idx) = field_names.iter().position(|f| f == &alt) { return Some(idx); }
        }
        // Underscore-prefixed name ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ try bare numeric (new N format)
        if field_name.starts_with('_') && field_name[1..].chars().all(|c| c.is_ascii_digit()) {
            let bare = &field_name[1..];
            if let Some(idx) = field_names.iter().position(|f| f == bare) { return Some(idx); }
        }
        None
    }

    /// Extract the name of each type argument from a Type AST node.
    /// For `Option[T]` returns `["T"]`, for `Map[K, V]` returns `["K", "V"]`.
    fn extract_type_arg_names(ty: &Type) -> Vec<String> {
        match ty {
            Type::Named(_, args) => args.iter().map(|a| Self::type_from_ast(a)).collect(),
            Type::Option(inner) => vec![Self::type_from_ast(inner)],
            Type::Result(ok, err) => vec![Self::type_from_ast(ok), Self::type_from_ast(err)],
            Type::Vec(inner) => vec![Self::type_from_ast(inner)],
            Type::Map(k, v) => vec![Self::type_from_ast(k), Self::type_from_ast(v)],
            Type::Set(inner) => vec![Self::type_from_ast(inner)],
            // Unwrap Ref/MutRef/Ptr to find type args nested inside.
            Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) => Self::extract_type_arg_names(inner),
            // Array: extract const-generic size ident + element type args.
            Type::Array(size_expr, elem) => {
                let mut names = vec![Self::type_from_ast(elem)];
                if let Expr::Ident(id) = size_expr.as_ref() {
                    names.insert(0, id.name.clone());
                }
                names
            }
            // Slice: extract element type args.
            Type::Slice(elem) => vec![Self::type_from_ast(elem)],
            _ => vec![],
        }
    }

    /// Returns `true` when `name` is NOT a known primitive/scalar/container ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â
    /// i.e., it is a user-defined named struct that needs concrete monomorphisation
    /// inside Result/Option generic types (B-001).
    fn is_struct_type_name(name: &str) -> bool {
        // Rust-like primitives and well-known container names.
        const NON_STRUCT: &[&str] = &[
            "Int", "Str", "Bool", "Char", "Float", "Double",
            "UInt8", "Int8", "Int16", "UInt16", "Int32", "UInt32",
            "UInt64", "Int64", "Float32", "Float64", "String",
            "void", "()", "Unit", "Option", "Result", "Vec", "Map", "Set",
            "Self", "CallTrace", "CallFrame",
            "i1", "i8", "i16", "i32", "i64", "float", "double",
        ];
        if NON_STRUCT.contains(&name) { return false; }
        if name.starts_with('*') || name.starts_with('[') { return false; }
        if name.contains("__") { return false; } // already a concrete monomorph
        true
    }

    /// Like `is_struct_type_name` but also checks the type registry so
    /// single-letter names that ARE registered structs (e.g. type J = {...})
    /// are correctly identified, not mistaken for generic parameters.
    fn is_struct_type_in_registry(&self, name: &str) -> bool {
        if !Self::is_struct_type_name(name) { return false; }
        // Single-char names might be generic params (T, K, V, E) but could
        // also be real structs (type J = {...}). Check the type registry.
        if name.len() == 1 && name.chars().next().map_or(false, |c| c.is_uppercase()) {
            return self.resolve_type_key(name) != name
                || self.types.type_meta.contains_key(&name.to_string())
                || self.types.types.contains_key(&name.to_string());
        }
        true
    }

    fn type_from_ast(ty: &Type) -> String {
        match ty {
            Type::Named(ident, _) => ident.name.clone(),
            // `&T` is always passed by-value at the ABI (unchanged).
            Type::Ref(inner) => Self::type_from_ast(inner),
            // `&mut` is always a real pointer (`*Inner`) so mutations propagate
            // to the caller. Scalars and structs both get pointer types.
            Type::MutRef(inner) => {
                format!("*{}", Self::type_from_ast(inner))
            }
            // `*T` raw pointer: encode with a leading `*` so `llvm_type_for` lowers
            // it to a real LLVM pointer (`*Int` -> `i64*`, `*UInt8` -> `i8*`).
            Type::Ptr(inner) => format!("*{}", Self::type_from_ast(inner)),
            Type::Option(_) => "Option".to_string(),
            Type::Result(_, _) => "Result".to_string(),
            Type::Vec(_) => "Vec".to_string(),
            Type::Map(_, _) => "Map".to_string(),
            Type::Set(_) => "Set".to_string(),
            Type::Tuple(types) => {
                let parts: Vec<String> = types.iter().map(Self::type_from_ast).collect();
                format!("Tuple__{}", parts.join("__"))
            }
            Type::Array(size_expr, elem) => {
                let elem_name = Self::type_from_ast(elem);
                match size_expr.as_ref() {
                    Expr::Int(n, _) => format!("[{n} x {elem_name}]"),
                    Expr::Ident(id) => format!("[{} x {elem_name}]", id.name),
                    _ => elem_name,
                }
            }
            Type::Slice(elem) => Self::type_from_ast(elem),
            // v0.56/P2-4: Never type (!) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â bottom type, never produces a value.
            Type::Never => "!".to_string(),
            Type::AnonStruct(fields) => {
                let parts: Vec<String> = fields.iter()
                    .map(|f| format!("{}_{}", f.name.name, Self::type_from_ast(&f.ty)))
                    .collect();
                format!("_Anon__{}", parts.join("__"))
            }
            _ => "Int".to_string(),
        }
    }

    /// Like `type_from_ast`, but preserves generic type arguments for Vec, Map, Set.
    /// Used for type_meta field registration so we can resolve element types at
    /// Vec index time (5c.21 Vec-of-struct fix).
    fn type_from_ast_with_args(ty: &Type) -> String {
        match ty {
            Type::Vec(inner) => format!("Vec[{}]", Self::type_from_ast_with_args(inner)),
            Type::Map(k, v) => format!("Map[{},{}]", Self::type_from_ast_with_args(k), Self::type_from_ast_with_args(v)),
            Type::Set(inner) => format!("Set[{}]", Self::type_from_ast_with_args(inner)),
            other => Self::type_from_ast(other),
        }
    }

    /// 5c.30: FULL type string including Option/Result payload args
    /// ("Result[Vec[Int], Str]"). Used ONLY for fn_return_xiom ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the field
    /// registration keeps type_from_ast_with_args so Option/Result struct
    /// fields keep their by-value layout.
    fn type_string_full(ty: &Type) -> String {
        match ty {
            Type::Option(inner) => format!("Option[{}]", Self::type_string_full(inner)),
            Type::Result(ok, err) => format!("Result[{}, {}]", Self::type_string_full(ok), Self::type_string_full(err)),
            Type::Vec(inner) => format!("Vec[{}]", Self::type_string_full(inner)),
            Type::Map(k, v) => format!("Map[{},{}]", Self::type_string_full(k), Self::type_string_full(v)),
            Type::Set(inner) => format!("Set[{}]", Self::type_string_full(inner)),
            other => Self::type_from_ast(other),
        }
    }

    /// 5c.33: Register anonymous struct types encountered in AST type
    /// annotations so `llvm_type_for` can resolve them and field access
    /// (Expr::Field) can GEP into the correct struct layout.
    pub(crate) fn register_anon_struct_from_ast(&mut self, ty: &Type) {
        match ty {
            Type::AnonStruct(fields) => {
                let parts: Vec<String> = fields.iter()
                    .map(|f| format!("{}_{}", f.name.name, Self::type_from_ast(&f.ty)))
                    .collect();
                let anon_name = format!("_Anon__{}", parts.join("__"));
                if !self.types.types.contains_key(&anon_name) {
                    let field_names: Vec<String> = fields.iter()
                        .map(|f| f.name.name.clone())
                        .collect();
                    let field_meta: Vec<(String, String)> = fields.iter()
                        .map(|f| (f.name.name.clone(), Self::type_from_ast_with_args(&f.ty)))
                        .collect();
                    self.types.types.insert(anon_name.clone(), field_names);
                    self.types.type_meta.or_insert_with(anon_name, || TypeMeta {
                        fields: field_meta,
                        invariants: vec![],
                        derives: vec![],
                    });
                }
            }
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
            _ => {}
        }
    }

    // ========================================================================
    // B-001: Concrete monomorphised Result/Option type helpers
    // ========================================================================

    /// Resolve a short type name to the key it actually lives under in type_meta
    /// (handles module-qualified names like "tests.ecosystem.test_json.JsonValue").
    fn resolve_type_key(&self, short_name: &str) -> String {
        // Current module's qualified name first (mirrors llvm_type_for):
        // deterministic when the module defines the type, e.g. "Big" â†’
        // "probe_big2.Big".
        if let Some(ref module) = self.local.current_module {
            let qualified = format!("{}.{}", module, short_name);
            if self.types.type_meta.contains_key(&qualified) || self.types.types.contains_key(&qualified) {
                return qualified;
            }
        }
        // Exact match next
        if self.types.type_meta.contains_key(&short_name.to_string()) || self.types.types.contains_key(&short_name.to_string()) {
            return short_name.to_string();
        }
        // Module-qualified suffix match. Skip GENERATED aggregate keys
        // (`Tuple__...`, `Option__...`, `Result__...`, `_Anon__...`): their
        // names end with `.Type` too (e.g. `Tuple__probe_big2.Big__probe_big2.
        // Big` ends with `.Big`), which would resolve an element type to the
        // aggregate itself and nest the tuple name into itself (BUG 1
        // fallout â€” docs/COMPILER_BUGS.md).
        let is_generated = |k: &str| {
            k.contains("Tuple__")
                || k.starts_with("_Anon__")
                || k.starts_with("Option__")
                || k.starts_with("Result__")
        };
        let suffix = format!(".{}", short_name);
        for key in self.types.type_meta.keys() {
            if key.ends_with(&suffix) && !is_generated(key.as_str()) {
                return key.clone();
            }
        }
        for key in self.types.types.keys() {
            if key.ends_with(&suffix) && !is_generated(key.as_str()) {
                return key.clone();
            }
        }
        short_name.to_string()
    }

    /// Create a concrete `Option__T` type in type_meta, duplicating the field
    /// layout of the base `Option` type but substituting the value field with
    /// the full struct type `T` so it is not truncated to 8 bytes.
    fn ensure_concrete_option(&mut self, inner_type_name: &str) {
        let concrete_name = format!("Option__{}", inner_type_name);
        if self.types.type_meta.contains_key(&concrete_name) { return; }
        // Use the fully-qualified type key so the emission loop resolves correctly.
        let resolved = self.resolve_type_key(inner_type_name);
        let fields = vec![
            ("discriminant".to_string(), "Int".to_string()),
            ("value".to_string(), resolved.clone()),
        ];
        let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
        self.types.types.insert(concrete_name.clone(), field_names);
        self.types.type_meta.insert(concrete_name, TypeMeta {
            fields,
            derives: vec![],
            invariants: vec![],
        });
    }

    /// Create a concrete `Result__Ok__Err` type in type_meta, duplicating the
    /// field layout of the base `Result` type but substituting value/error fields
    /// with the full struct types so neither payload is truncated.
    fn ensure_concrete_result(&mut self, ok_type_name: &str, err_type_name: &str) {
        let concrete_name = format!("Result__{}__{}", ok_type_name, err_type_name);
        if self.types.type_meta.contains_key(&concrete_name) { return; }
        let resolved_ok = self.resolve_type_key(ok_type_name);
        let resolved_err = self.resolve_type_key(err_type_name);
        let fields = vec![
            ("discriminant".to_string(), "Int".to_string()),
            ("value".to_string(), resolved_ok),
            ("error".to_string(), resolved_err),
        ];
        let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
        self.types.types.insert(concrete_name.clone(), field_names);
        self.types.type_meta.insert(concrete_name, TypeMeta {
            fields,
            derives: vec![],
            invariants: vec![],
        });
    }

    /// Pre-register all concrete Option/Result monomorphs for every struct type
    /// known after `register_type_layout` completes. Called BEFORE the emission
    /// loop so every function body sees the correct LLVM type definition.
    #[allow(dead_code)]
    fn pre_register_concrete_types(&mut self) {
        let struct_names: Vec<String> = self.types.type_meta.keys().into_iter()
     .filter(|k| Self::is_struct_type_name(k)
                // Skip already-concrete monomorphs (they contain "__")
                && !k.contains("__"))
            .collect();
        // Option__T for every struct T
        for t in &struct_names {
            self.ensure_concrete_option(t);
        }
        // Result__A__B for every pair of struct types
        for a in &struct_names {
            for b in &struct_names {
                self.ensure_concrete_result(a, b);
            }
        }
    }

    /// B-001: For a `Result[T, E]` or `Option[T]` AST type, return the concrete
    /// monomorphised name (`Result__T__E` / `Option__T`) if at least one inner type
    /// is a user-defined struct. Otherwise returns the base name (`Result`/`Option`).
    /// This is used by `compile_fn` to select the correct LLVM struct layout.
    /// BUG 1 fix (2026-08-10): LLVM type for a function parameter, routing
    /// tuple types through `concrete_type_for` so their element names are
    /// module-qualified (`Tuple__probe_tuple.Pair__probe_tuple.Pair`).
    /// `type_from_ast` produces bare names (`Tuple__Pair__Pair`) that never
    /// match the expression-level tuple type used at call sites, causing
    /// clang IR rejection (see docs/COMPILER_BUGS.md BUG 1).
    ///
    /// Also fixes the "struct `&T` param mutation lost" bug: STRUCT-typed
    /// `&T` params now pass the ADDRESS (`%struct.X*`) instead of a by-value
    /// struct copy, so callee mutations (e.g. `_trim(&result)`'s
    /// `digits.pop()`) write through to the caller's variable. Scalar `&T`
    /// params already carried the address (as i64); plain struct params and
    /// generic container params (`&Vec[T]`, `&Slice[T]`, `&Map`, `&Set`) keep
    /// the existing by-value ABI to avoid changing their established layout
    /// (see coerce_arg_for_param).
    fn param_llvm_type(&mut self, ty: &Type) -> String {
        match ty {
            Type::Tuple(_) => {
                let concrete = self.concrete_type_for(ty);
                self.llvm_type_for(&concrete).unwrap_or_else(|_| "i64".to_string())
            }
            Type::Ref(inner) => {
                let inner_llvm = self.llvm_type_for(&Self::type_from_ast(inner)).unwrap_or_else(|_| "i64".to_string());
                if inner_llvm.starts_with("%struct.") {
                    // BUG 23 #8 fix: &Vec[T]/&Slice[..]/&Map[..]/&Set[..] params
                    // must pass the POINTER, like &mut Vec[T] already did. The
                    // previous exclusion passed these BY VALUE, so a catalog fn
                    // taking &Vec[T] mutated its own copy â€” push/realloc in the
                    // callee never reached the caller's Vec (silent no-op).
                    format!("{inner_llvm}*")
                } else {
                    inner_llvm
                }
            }
            _ => self.llvm_type_for(&Self::type_from_ast(ty)).unwrap_or_else(|_| "i64".to_string()),
        }
    }

    fn concrete_type_for(&mut self, ty: &Type) -> String {
        match ty {
            Type::Option(inner) => {
                let inner_name = Self::type_from_ast(inner);
                // M18: Enums excluded from concrete types until enum layout
                // supports per-variant field types (shared field names cause
                // type mismatches between variants).
                let resolved = self.resolve_type_key(&inner_name);
                let is_enum = self.types.enum_variants.contains_key(&resolved);
                if self.is_struct_type_in_registry(&inner_name) && !is_enum {
                    let concrete = format!("Option__{}", inner_name);
                    if !self.types.type_meta.contains_key(&concrete) {
                        self.ensure_concrete_option(&inner_name);
                    }
                    concrete
                } else {
                    "Option".to_string()
                }
            }
            Type::Result(ok, err) => {
                let ok_name = Self::type_from_ast(ok);
                let err_name = Self::type_from_ast(err);
                let ok_resolved = self.resolve_type_key(&ok_name);
                let err_resolved = self.resolve_type_key(&err_name);
                let ok_struct = self.is_struct_type_in_registry(&ok_name)
                    && !self.types.enum_variants.contains_key(&ok_resolved);
                let err_struct = self.is_struct_type_in_registry(&err_name)
                    && !self.types.enum_variants.contains_key(&err_resolved);
                if ok_struct || err_struct {
                    let concrete = format!("Result__{}__{}", ok_name, err_name);
                    if !self.types.type_meta.contains_key(&concrete) {
                        self.ensure_concrete_result(&ok_name, &err_name);
                    }
                    concrete
                } else {
                    "Result".to_string()
                }
            }
            Type::Tuple(types) => {
                // BUG 1 fix (2026-08-10): module-qualify element names so the
                // tuple type key matches the expression-level registration.
                // The body path (Expr::Tuple in expr.rs) resolves struct
                // elements through `infer_llvm_type` and produces fully-
                // qualified keys (e.g. `Tuple__probe_tuple.Pair__probe_tuple.
                // Pair`), while `type_from_ast` produces BARE names
                // (`Tuple__Pair__Pair`). Using bare names here made the fn
                // signature/return type a DIFFERENT type than the one used by
                // the body's alloca/GEP â€” clang rejected the IR ("Cannot
                // allocate unsized type") or the tuple was stored truncated
                // to i64s (tuple-of-struct codegen bug, docs/COMPILER_BUGS.md
                // BUG 1).
                let parts: Vec<String> = types.iter()
                    .map(|t| self.resolve_type_key(&Self::type_from_ast(t)))
                    .collect();
                let name = format!("Tuple__{}", parts.join("__"));
                if !self.types.type_meta.contains_key(&name) && !types.is_empty() {
                    let fields: Vec<(String, String)> = types.iter().enumerate()
                        .map(|(i, t)| (format!("_{i}"), self.resolve_type_key(&Self::type_from_ast(t))))
                        .collect();
                    let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                    self.types.type_meta.insert(name.clone(), crate::context::TypeMeta {
                        fields,
                        derives: vec![],
                        invariants: vec![],
                    });
                    self.types.types.insert(name.clone(), field_names);
                }
                name
            }
            other => Self::type_from_ast(other),
        }
    }

    /// Resolve a Vec field's element type from its container expression.
    /// For `h.entries[i].name`, the container `h.entries` has field type
    /// `Vec[HttpHeader]` in type_meta. This extracts `HttpHeader` (fully qualified).
    /// 5c.30: For an Option[X]/Result[X, E] type string, return X (the
    /// success payload), respecting nested brackets ("Result[Vec[Int], Str]"
    /// ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ "Vec[Int]").
    fn option_result_payload(s: &str) -> Option<String> {
        let open = s.find('[')?;
        let head = &s[..open];
        if head != "Option" && head != "Result" {
            return None;
        }
        let inner = &s[open + 1..s.rfind(']')?];
        let mut depth = 0i32;
        let mut end = inner.len();
        for (i, c) in inner.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => depth -= 1,
                ',' if depth == 0 => { end = i; break; }
                _ => {}
            }
        }
        Some(inner[..end].trim().to_string())
    }

    /// BUG 29 (Map.keys on module globals): split a generic type string into
    /// (base, top-level args). "Map[Str, Bool]" -> ("Map", ["Str", "Bool"]);
    /// "Vec[Int]" -> ("Vec", ["Int"]). Depth-aware so nested args
    /// ("Map[Str, Vec[Int]]") split only on TOP-LEVEL commas.
    fn parse_generic_type_string(s: &str) -> (String, Vec<String>) {
        let Some(open) = s.find('[') else {
            return (s.to_string(), Vec::new());
        };
        let Some(close) = s.rfind(']') else {
            return (s.to_string(), Vec::new());
        };
        let base = s[..open].trim().to_string();
        let inner = &s[open + 1..close];
        let mut args: Vec<String> = Vec::new();
        let mut depth = 0i32;
        let mut current = String::new();
        for c in inner.chars() {
            match c {
                '[' => { depth += 1; current.push(c); }
                ']' => { depth -= 1; current.push(c); }
                ',' if depth == 0 => {
                    args.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(c),
            }
        }
        if !current.trim().is_empty() {
            args.push(current.trim().to_string());
        }
        (base, args)
    }

    /// 5d: Extract the ERROR payload type of `Result[T, E]` (the second
    /// generic argument). Returns None for Option or non-generic types.
    fn option_result_err_payload(s: &str) -> Option<String> {
        let open = s.find('[')?;
        if &s[..open] != "Result" {
            return None;
        }
        let inner = &s[open + 1..s.rfind(']')?];
        let mut depth = 0i32;
        for (i, c) in inner.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => depth -= 1,
                ',' if depth == 0 => {
                    return Some(inner[i + 1..].trim().to_string());
                }
                _ => {}
            }
        }
        None
    }

    /// 5c.30: Resolve the declared XIOM return type of a call expression's
    /// callee (exact key, then unique `.name` suffix match).
    fn callee_return_xiom(&self, func: &Expr) -> Option<String> {
        // BUG 29 (BUG 27 #12): for a module-qualified callee (`crypto.
        // aes_encrypt_gcm`) resolve the RECEIVER prefix first — the bare-leaf
        // suffix match is ambiguous when two modules export the same fn name
        // with different return types (cipher.aes_encrypt_gcm -> Vec[UInt8]
        // vs crypto.aes_encrypt_gcm -> Result[Tuple__Vec__Vec, Str]), and the
        // ambiguity return None silently dropped the Option/Result payload
        // tracking — `pair.1` then compiled as Str.len(inttoptr 0) -> garbage
        // lengths -> AES-GCM smoke heap corruption.
        if let Expr::Field(recv, f, _) = func {
            let leaf = f.name.clone();
            let receiver_name = match recv.as_ref() {
                Expr::Ident(id) => id.name.clone(),
                Expr::Field(_, rf, _) => rf.name.clone(),
                _ => String::new(),
            };
            if !receiver_name.is_empty() {
                if let Some(rt) = self.types.fn_return_xiom.get(&format!("{receiver_name}.{leaf}")) {
                    return Some(rt.clone());
                }
                // Also try the full dotted receiver (e.g. "xiom.crypto.…").
                let dotted = {
                    let mut segs: Vec<String> = Vec::new();
                    let mut cur = recv.as_ref();
                    loop {
                        match cur {
                            Expr::Ident(id) => { segs.insert(0, id.name.clone()); break; }
                            Expr::Field(base, fname, _) => { segs.insert(0, fname.name.clone()); cur = base; }
                            _ => break,
                        }
                    }
                    segs.join(".")
                };
                if !dotted.is_empty() {
                    if let Some(rt) = self.types.fn_return_xiom.get(&format!("{dotted}.{leaf}")) {
                        return Some(rt.clone());
                    }
                }
            }
            return self.callee_return_xiom_suffix(&leaf);
        }
        let leaf = match func {
            Expr::Ident(id) => id.name.clone(),
            _ => return None,
        };
        if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() {
            let hits: Vec<(String, String)> = self.types.fn_return_xiom.entries().into_iter()
                .filter(|(k, _)| k.contains(&leaf)).collect();
            eprintln!("[retxiom] leaf={leaf} keys: {hits:?}");
        }
        if let Some(rt) = self.types.fn_return_xiom.get(&leaf) {
            return Some(rt.clone());
        }
        self.callee_return_xiom_suffix(&leaf)
    }

    /// Unique `.leaf` suffix match for a bare callee; None when multiple
    /// modules export the same leaf with DIFFERENT return types.
    fn callee_return_xiom_suffix(&self, leaf: &str) -> Option<String> {
        let suffix = format!(".{leaf}");
        let mut found: Option<String> = None;
        for (k, v) in self.types.fn_return_xiom.entries() {
            if k.ends_with(&suffix) {
                match &found {
                    None => found = Some(v),
                    Some(prev) if *prev == v => {}
                    _ => return None, // ambiguous with different types
                }
            }
        }
        found
    }

    /// 5c.30: Track boxed-struct payload flow for a `let`/`var` binding.
    /// - `x.pop()` / `x.get(i)` on a Vec-of-struct container ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ the Option's
    ///   payload is a boxed struct pointer (record in local_opt_payload).
    /// - fn calls returning Option[X]/Result[X, E] ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ record X.
    /// - `opt.unwrap()` where opt is such an Option ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ classify the binding:
    ///   Vec[T] payloads are container HANDLES, struct payloads are boxes.
    fn track_boxed_payload_binding(&mut self, name: &str, value: &Expr) {
        self.local.local_opt_payload.remove(name);
        self.local.local_boxed_struct.remove(name);
        self.local.local_vec_handle.remove(name);
        self.local.local_err_payload.remove(name);
        self.local.local_opt_payload_xiom.remove(name);

        // BUG 22 #4 fix: track the SCALAR XIOM payload type of Some/Ok/Err
        // bindings (`var o = Some(5.0)` â†’ "Float64"). Some(5.0) stores the
        // double BITS in the i64 payload slot; match extraction needs to
        // know to bitcast back (float payloads read as raw i64 otherwise).
        if let Some(payload_xiom) = self.ctor_payload_xiom(value) {
            self.local.local_opt_payload_xiom.insert(name.to_string(), payload_xiom);
        }

        // M18: Detect struct payload types from inline `Some(..)` and `Ok(..)`
        // constructors. When `var opt = Some(Ok(77))` has no type annotation,
        // the codegen still needs to know that the Option payload is a boxed
        // Result struct so that subsequent `match opt { Some(r) => ... }` can
        // load `r` as %struct.Result rather than as a raw i64 pointer.
        // Without this, nested match guards on extracted payloads see stale
        // zero values because the intermediate matches have no scrutinee alloca.
        if let Some(payload_type) = Self::struct_ctor_type_name(value) {
            let stored = self.types.types.keys().into_iter()
    .find(|k| k.ends_with(&format!(".{payload_type}")) || k.as_str() == payload_type)
                .unwrap_or_else(|| payload_type.clone());
            self.local.local_opt_payload.insert(name.to_string(), stored);
            return;
        }
        if let Some(err_type) = Self::err_ctor_type_name(value) {
            self.local.local_err_payload.insert(name.to_string(), err_type);
            return;
        }

        if let Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) = value {
            if let Expr::Field(recv, method, _) = func.as_ref() {
                match method.name.as_str() {
                    // Only the INLINE builtins that box struct payloads.
                    "pop" | "get" | "remove" => {
                        if let Some(elem_ty) = self.resolve_vec_elem_type(recv) {
                            self.local.local_opt_payload.insert(name.to_string(), elem_ty);
                            return;
                        }
                    }
                    "unwrap" | "unwrap_or" => {
                        // BUG 29 (repro_opt_vec): resolve the payload through a
                        // LOCAL's tracked type when the receiver is an Ident
                        // (`var v = o.unwrap()`), OR through the receiver CALL's
                        // declared return type when chained
                        // (`var v1 = captures(...).unwrap()` — the receiver is
                        // a Call, not an Ident, so no local payload was tracked).
                        let payload_opt: Option<String> = if let Expr::Ident(opt_id) = recv.as_ref() {
                            self.local.local_opt_payload.get(&opt_id.name).cloned()
                        } else {
                            // Chained receiver: `captures(...).unwrap()` — the
                            // receiver is a CALL; resolve the callee's declared
                            // return type directly.
                            let inner_callee: &Expr = match recv.as_ref() {
                                Expr::Call(f, _, _) | Expr::GenericCall(f, _, _, _) => f.as_ref(),
                                other => other,
                            };
                            self.callee_return_xiom(inner_callee)
                                .and_then(|ret| Self::option_result_payload(&ret))
                                .map(|p| {
                                    if p.contains('[') {
                                        p
                                    } else {
                                        self.types.types.keys().into_iter()
                                            .find(|k| k.ends_with(&format!(".{p}")) || k.as_str() == p)
                                            .unwrap_or(p)
                                    }
                                })
                        };
                        if let Some(t) = payload_opt {
                            if let Some(elem) = t.strip_prefix("Vec[").and_then(|s| s.strip_suffix(']')) {
                                self.local.local_vec_handle.insert(name.to_string(), elem.to_string());
                            } else {
                                self.local.local_boxed_struct.insert(name.to_string(), t);
                            }
                            return;
                        }
                    }
                    _ => {}
                }
            }
            // General call returning Option[X]/Result[X, E]: record X so a
            // later `.unwrap()` binding can be classified.
            if let Some(ret) = self.callee_return_xiom(func) {
                if let Some(payload) = Self::option_result_payload(&ret) {
                    // Struct payload names are stored qualified when possible.
                    let stored = if payload.contains('[') {
                        payload
                    } else {
                        self.types.types.keys().into_iter()
    .find(|k| k.ends_with(&format!(".{payload}")) || k.as_str() == payload)
                            .unwrap_or(payload)
                    };
                    self.local.local_opt_payload.insert(name.to_string(), stored);
                }
                // 5d: record the Result ERROR payload for unwrap_err/match Err(e).
                if let Some(err_payload) = Self::option_result_err_payload(&ret) {
                    self.local.local_err_payload.insert(name.to_string(), err_payload);
                }
            }
        }
    }

    /// BUG 22 #4 fix: infer the SCALAR XIOM payload type of a Some/Ok/Err
    /// constructor expression (Float64 for Some(5.0), Int for Some(1),
    /// Str for Some("x"), Idents resolve via the registered local type,
    /// nested ctors recurse). Struct payloads return None (boxed path).
    fn ctor_payload_xiom(&self, value: &Expr) -> Option<String> {
        let inner = match value {
            Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _) => e.as_ref(),
            _ => return None,
        };
        match inner {
            Expr::Float(..) => Some("Float64".to_string()),
            Expr::Int(..) => Some("Int".to_string()),
            Expr::Bool(..) => Some("Bool".to_string()),
            Expr::Str(..) => Some("Str".to_string()),
            Expr::Char(..) => Some("Char".to_string()),
            Expr::Ident(id) => self.local.local_xiom_types.get(&id.name).cloned(),
            Expr::Some(..) | Expr::Ok(..) | Expr::Err(..) => self.ctor_payload_xiom(inner),
            Expr::Paren(e, _) => self.ctor_payload_xiom(e),
            // `Some(Vec[Int].new())` â€” a Vec-ctor payload ("Vec[Int]").
            Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
                if let Expr::Field(obj, method, _) = func.as_ref() {
                    if method.name == "new" {
                        if let Expr::Index(base, idx, _) = obj.as_ref() {
                            if let Expr::Ident(b) = base.as_ref() {
                                if b.name == "Vec" {
                                    let rendered = Self::type_arg_to_name(&Expr::Index(base.clone(), idx.clone(), b.span));
                                    if rendered.starts_with("Vec[") {
                                        return Some(rendered);
                                    }
                                }
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// M18: Determine the XIOM struct type name for the payload of a `Some(..)`
    /// or `Ok(..)` constructor expression. Returns the type name if the inner
    /// expression is a struct-producing expression, otherwise `None`.
    /// E.g. `Some(Ok(77))` Ã¢â‚¬' `Some("Result")` because the Some payload is
    /// an `Ok(77)` which produces a `Result` struct.
    fn struct_ctor_type_name(expr: &Expr) -> Option<String> {
        match expr {
            // Some(inner): the payload type is the type of `inner`
            // E.g. Some(Ok(77)) ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ payload is Result struct
            //      Some(42)     ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ payload is Int (not a struct) ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ None
            Expr::Some(inner, _) => Self::inner_payload_type(inner),
            // Ok(inner): the payload type is whatever `inner` produces
            Expr::Ok(inner, _) => Self::inner_payload_type(inner),
            _ => None,
        }
    }

    /// M18: Given the inner expression of a Some/Ok constructor, determine
    /// if it produces a struct type. Returns the XIOM type name or None.
    fn inner_payload_type(inner: &Expr) -> Option<String> {
        match inner {
            // Any Ok/Err constructor always produces a Result struct
            Expr::Ok(..) | Expr::Err(..) => Some("Result".to_string()),
            // Some produces an Option ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â recurse if the inner payload is a struct
            Expr::Some(sub, _) => Self::inner_payload_type(sub),
            // Named struct literal `TypeName { field: val; }`
            Expr::Struct(name, _, _, _) if name.name != "_" => Some(name.name.clone()),
            _ => None,
        }
    }

    /// M18: Determine the XIOM error type name for an `Err(..)` constructor.
    fn err_ctor_type_name(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Err(inner, _) => Self::inner_payload_type(inner),
            _ => None,
        }
    }

    /// 5c.30: If `expr` is a `Vec[T].new()` / `Vec[T].with_capacity(..)` call,
    /// return the element type name `T` (from the explicit type argument).
    fn vec_ctor_elem_type(expr: &Expr) -> Option<String> {
        if let Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) = expr {
            if let Expr::Field(obj, method, _) = func.as_ref() {
                if matches!(method.name.as_str(), "new" | "with_capacity") {
                    if let Expr::Index(base, idx, _) = obj.as_ref() {
                        if let Expr::Ident(b) = base.as_ref() {
                            if b.name == "Vec" {
                                match idx.as_ref() {
                                    Expr::Ident(t) => return Some(t.name.clone()),
                                    Expr::Tuple(elems, _) => {
                                        // Tuple element types (e.g. Vec[(Str, Str)]):
                                        // build the full "Tuple__Str__Str" name so
                                        // resolve_vec_elem_type can find the registered
                                        // tuple struct and load full elements (not just
                                        // the first field).
                                        let mut parts: Vec<String> = Vec::new();
                                        for e in elems.iter() {
                                            if let Expr::Ident(t) = e {
                                                parts.push(t.name.clone());
                                            } else {
                                                parts.clear();
                                                break;
                                            }
                                        }
                                        if !parts.is_empty() && parts.len() == elems.len() {
                                            return Some(format!("Tuple__{}", parts.join("__")));
                                        }
                                    }
                                    // BUG 23 #2 fix: NESTED generic arg
                                    // (`Vec[Vec[Int]].new()`) â€” the type arg is itself
                                    // an Index expression; render it to "Vec[Int]" so
                                    // the element type and size resolve correctly.
                                    _ => {
                                        let span = b.span;
                                        let rendered = Self::type_arg_to_name(&Expr::Index(base.clone(), idx.clone(), span));
                                        if rendered != "Int" {
                                            return Some(rendered);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Array literals: infer element type from first element.
        // e.g. [1.5, 2.5] ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ Float64, [1, 2, 3] ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ Int
        if let Expr::Array(elems, _) = expr {
            if let Some(first) = elems.first() {
                match first {
                    Expr::Float(..) => return Some("Float64".to_string()),
                    Expr::Int(..) => return Some("Int".to_string()),
                    Expr::Bool(..) => return Some("Bool".to_string()),
                    Expr::Str(..) => return Some("Str".to_string()),
                    _ => {}
                }
            }
        }
        None
    }

    /// Extract the element type name from a `Vec[T]` type annotation.
    pub fn vec_elem_from_type_annotation(ty: &Type) -> Option<String> {
        // Unwrap reference wrappers (`&Vec[T]`/`&mut Vec[T]` params) â€” must
        // match the types.rs copy (BUG 12/17: without this, `v[0]` on a
        // `&Vec[Float64]` param loads i64 + sitofp the bit pattern, and
        // Vec[Str] elements lose the strcmp equality lowering).
        let inner = match ty {
            Type::Ref(t) | Type::MutRef(t) | Type::Ptr(t) => t.as_ref(),
            other => other,
        };
        match inner {
            Type::Vec(inner) => Some(Self::type_from_ast(inner)),
            Type::Named(ident, type_args) if ident.name == "Vec" => {
                type_args.first().map(|t| Self::type_from_ast(t))
            }
            _ => None,
        }
    }

    /// Extract the inner type parameter from an Option[T] or Result[T, E] annotation.
    /// Returns T for Option[T]; returns T (the value type) for Result[T, E].
    pub fn option_type_param(ty: &Type, container: &str) -> Option<String> {
        match ty {
            Type::Option(inner) if container == "Option" => Some(Self::type_from_ast(inner)),
            Type::Result(t, _) if container == "Option" => Some(Self::type_from_ast(t)),
            Type::Result(_, e) if container == "Result" => Some(Self::type_from_ast(e)),
            Type::Named(ident, type_args) if ident.name == container => {
                type_args.first().map(|t| Self::type_from_ast(t))
            }
            _ => None,
        }
    }

    /// Extract the error type parameter from a Result[T, E] annotation.
    pub fn result_err_type_param(ty: &Type) -> Option<String> {
        match ty {
            Type::Result(_, e) => Some(Self::type_from_ast(e)),
            Type::Named(ident, type_args) if ident.name == "Result" => {
                type_args.get(1).map(|t| Self::type_from_ast(t))
            }
            _ => None,
        }
    }

    /// 5c.29: If `container` is a struct-field access whose declared type is a
    /// float container (Vec[Float32] / Vec[Float64]), return the float LLVM
    /// type. Float Vec elements are stored as RAW BITS (val_to_i64 bitcast),
    /// so Index loads must bit-reinterpret instead of numerically converting.
    fn vec_elem_float_type(&self, container: &Expr) -> Option<&'static str> {
        // 5c.30: local Vec bindings (`var v = Vec[Float32].new()`) and
        // container-handle bindings.
        if let Expr::Ident(id) = container {
            if let Some(elem) = self.local.local_vec_elem.get(&id.name)
                .or_else(|| self.local.local_vec_handle.get(&id.name))
            {
                return match elem.as_str() {
                    "Float32" => Some("float"),
                    "Float64" | "Float" => Some("double"),
                    _ => None,
                };
            }
        }
        // BUG 23 #2: `fm[0][1]` â€” the INNER index's container is the outer
        // index result; its element type is the inner of the OUTER Vec's
        // registered element, stripped recursively ("Vec[Vec[Float64]]" â†’
        // "Vec[Float64]" â†’ "Float64" â†’ double).
        if let Expr::Index(base, _, _) = container {
            if let Expr::Ident(b) = base.as_ref() {
                if let Some(outer) = self.local.local_vec_elem.get(&b.name)
                    .or_else(|| self.local.local_vec_handle.get(&b.name))
                {
                    let mut inner = outer.as_str();
                    loop {
                        if let Some(rest) = inner.strip_prefix("Vec[").and_then(|s| s.strip_suffix(']')) {
                            inner = rest;
                        } else {
                            break;
                        }
                    }
                    return match inner {
                        "Float32" => Some("float"),
                        "Float64" | "Float" => Some("double"),
                        _ => None,
                    };
                }
            }
        }
        if let Expr::Field(base, field_expr, _) = container {
            let base_ty = self.infer_struct_type_name(base)?;
            for key in self.types.type_meta.keys() {
                if key.ends_with(&base_ty) || key == base_ty {
                    if let Some(meta) = self.types.type_meta.get(&key) {
                        for (fname, ftype) in &meta.fields {
                            if fname == &field_expr.name {
                                return match ftype.as_str() {
                                    "Vec[Float32]" => Some("float"),
                                    "Vec[Float64]" | "Vec[Float]" => Some("double"),
                                    _ => None,
                                };
                            }
                        }
                    }
                    break;
                }
            }
        }
        None
    }

    pub(crate) fn should_store_back_method(&self, fn_key: &str) -> bool {
        self.types.by_value_self_methods.contains(fn_key)
    }

    fn resolve_vec_elem_type(&self, container: &Expr) -> Option<String> {
        // 5c.30: local Vec bindings (`var v = Vec[Point2D].new()`): the elem
        // type was recorded at the let/var binding. Only struct element types
        // are returned (primitives use the scalar elem_load path).
        if let Expr::Ident(id) = container {
            let elem = self.local.local_vec_elem.get(&id.name)
                .or_else(|| self.local.local_vec_handle.get(&id.name))?;
            if matches!(elem.as_str(), "Int" | "Bool" | "Str" | "Float64" | "Float32" | "UInt8" | "Int8" | "Int16" | "Int32" | "UInt16" | "UInt32" | "Char" | "Float") {
                return None;
            }
            // BUG 23 #2 fix: NESTED Vec elements (`Vec[Vec[T]]`, `Vec[Vec[Int]]`)
            // have no dedicated struct key â€” the element IS the generic %struct.Vec.
            // Return the generic-args name so the index site can load it as a Vec.
            if elem.starts_with("Vec[") {
                return Some(elem.clone());
            }
            return self.types.types.keys().into_iter()
    .find(|k| k.ends_with(&format!(".{}", elem)) || k.as_str() == elem);
        }
        if let Expr::Field(base, field_expr, _) = container {
            let base_ty = self.infer_struct_type_name(base)?;
            for key in self.types.type_meta.keys() {
                if key.ends_with(&base_ty) || key == base_ty {
                    if let Some(meta) = self.types.type_meta.get(&key) {
                        for (fname, ftype) in &meta.fields {
                            if fname == &field_expr.name {
                                if let Some(inner) = ftype.strip_prefix("Vec[") {
                                    if let Some(bare_name) = inner.strip_suffix(']') {
                                        // Only return if this is a known struct type
                                        // (not a primitive like Int, Str, Bool, etc.)
                                        if let Some(qualified) = self.types.types.keys().into_iter()
    .find(|k| k.ends_with(&format!(".{}", bare_name)) || k.as_str() == bare_name)
                                        {
                                            return Some(qualified);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
        None
    }

    /// M33: Given an Expr (typically Expr::Index), resolve the Vec element
    /// struct type if the index is into a Vec with struct elements. Used when
    /// field access on a vec index needs to dereference boxed struct pointers.
    /// Returns the XIOM type name of the element struct, or None.
    pub(crate) fn resolve_vec_elem_type_for_index(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Index(container, _, _) => self.resolve_vec_elem_type(container),
            _ => None,
        }
    }

    /// FIELD-I64: When obj_val is an i64 from a Vec index of a struct element
    /// (stored inline via memcpy or as val_to_i64 heap pointer), resolve field
    /// access via inttoptr+GEP on a known struct type. Returns None if no
    fn llvm_type_for(&self, type_name: &str) -> Result<String, String> {
        // Parse array types like [N x ElementType] ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â used for fixed-size stack arrays.
        if type_name.starts_with('[') {
            if let Some(rest) = type_name.strip_prefix('[') {
                if let Some(x_pos) = rest.find(" x ") {
                    let n_str = rest[..x_pos].trim();
                    // BUG 24 fix: strip the CLOSING bracket from the element
                    // name â€” "[10 x Int]" produced elem "Int]" (unknown type â†’
                    // i64 degradation â†’ corrupted fixed-array locals like
                    // bigint's `var digits: [10]Int`).
                    let elem_name = rest[x_pos + 3..].trim_end_matches(']').trim();
                    let elem_llvm = self.llvm_type_for(elem_name)
                        .unwrap_or_else(|_| Self::xiom_to_llvm_type(elem_name).to_string());
                    // Literal integer size (e.g. [4 x i64]).
                    if let Ok(n) = n_str.parse::<u64>() {
                        return Ok(format!("[{n} x {elem_llvm}]"));
                    }
                    // Const-ident size: resolve from `self.local.constants` (module-level
                    // `const N: Int = 32;` declared before the type is used).
                    if let Some(cval) = self.local.constants.get(n_str) {
                        if let Expr::Int(n, _) = cval {
                            let n = *n as u64;
                            return Ok(format!("[{n} x {elem_llvm}]"));
                        }
                    }
                    // If the size is an ident we can't resolve (e.g. a const-generic
                    // param N), fall through and let the rest of llvm_type_for attempt
                    // to resolve it as a struct name or builtin ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â the caller will get
                    // an error if the type is genuinely unresolvable.
                }
            }
        }
        // Real-pointer encoding: a leading `*` (from `type_from_ast` for `*T` /
        // `&mut Scalar`) lowers to an LLVM pointer to the inner type. `*Int`->`i64*`,
        // `*Float32`->`float*`, `*UInt8`->`i8*`, `*Str`->`i8**`. A pointer to a
        // void/unit inner is represented as `i8*` (LLVM has no `void*`).
        if let Some(inner) = type_name.strip_prefix('*') {
            let inner_llvm = self
                .llvm_type_for(inner)
                .unwrap_or_else(|_| Self::xiom_to_llvm_type(inner).to_string());
            if inner_llvm == "void" {
                return Ok("i8*".to_string());
            }
            return Ok(format!("{inner_llvm}*"));
        }
        // 5c.37: Strip generic type arguments (Vec[Int] ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ Vec) before lookup.
        // type_from_ast_with_args preserves them for element type resolution,
        // but llvm_type_for needs the base struct name.
        let clean_name = if let Some(bracket) = type_name.find('[') {
            &type_name[..bracket]
        } else {
            type_name
        };
        // Try current module's qualified name first (e.g., "types.Person")
        if let Some(ref module) = self.local.current_module {
            let qualified = format!("{}.{}", module, clean_name);
            if self.types.types.contains_key(&qualified) || self.types.type_meta.contains_key(&qualified) {
                return Ok(format!("%struct.{qualified}"));
            }
        }
        // Try exact match
        if self.types.types.contains_key(&clean_name.to_string()) || self.types.type_meta.contains_key(&clean_name.to_string()) {
            return Ok(format!("%struct.{clean_name}"));
        }
        // Search for any module-qualified variant ending with .clean_name.
        // BUG 16-family fix (2026-08-11): skip GENERATED aggregate keys
        // (Tuple__/Option__/Result__/_Anon__) â€” their names end with .Type
        // too, so a bare `Big` could resolve to the TUPLE key depending on
        // HashMap iteration order (must mirror the types.rs copy).
        for (key, _) in self.types.type_meta.entries() {
            if key.ends_with(&format!(".{clean_name}"))
                && !key.contains("Tuple__")
                && !key.starts_with("Option__")
                && !key.starts_with("Result__")
                && !key.starts_with("_Anon__")
            {
                return Ok(format!("%struct.{key}"));
            }
        }
        // Check builtin types first (match known xiom type names, NOT the default i64 fallback)
        let builtin = Self::xiom_to_llvm_type(clean_name);
        match type_name {
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
            | "Bool" | "Float32" | "Float64" | "Str" | "Char" | "()" | "!" => return Ok(builtin.to_string()),
            // Generic type parameters (single uppercase letters: T, K, V, E, etc.)
            // silently default to i64 ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â these are expected when monomorphisation
            // hasn't substituted them yet (e.g. in type_meta field lists).
            name if name.len() == 1 && name.chars().next().map_or(false, |c| c.is_uppercase()) => {
                return Ok("i64".to_string());
            }
            // "Self" in type_meta field lists is a placeholder ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â silently default to i64.
            "Self" => { return Ok("i64".to_string()); }
            _ => {}
        }
        // If type_name is an enum variant (e.g., "Image"), find its parent enum type
        for (enum_key, variants) in self.types.enum_variants.entries() {
            if variants.iter().any(|(v, _)| v == type_name) {
                return Ok(format!("%struct.{enum_key}"));
            }
        }
        // If type_name is itself an enum TYPE name (e.g. "Ordering"), it is lowered
        // to a struct `%struct.Name = { i64, ... }`. Enums are registered in
        // `enum_variants` (keyed by enum name) but not in `types`/`type_meta`, so
        // without this an enum-typed function return/param would resolve to the
        // `i64` fallback while `infer_llvm_type` resolves it to `%struct.Name`,
        // producing store/return/arg type mismatches. Match exact, module-qualified,
        // then suffix ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â mirroring the struct lookup above.
        if self.types.enum_variants.contains_key(&type_name.to_string()) {
            return Ok(format!("%struct.{type_name}"));
        }
        if let Some(ref module) = self.local.current_module {
            let qualified = format!("{}.{}", module, type_name);
            if self.types.enum_variants.contains_key(&qualified) {
                return Ok(format!("%struct.{qualified}"));
            }
        }
        for enum_key in self.types.enum_variants.keys() {
            if enum_key.ends_with(&format!(".{type_name}")) {
                return Ok(format!("%struct.{enum_key}"));
            }
        }
        match type_name {
            "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64"
            | "Bool" | "Float32" | "Float64" | "Str" | "Char" | "()" => Ok(builtin.to_string()),
            _ => {
                // 5e.2 G-34: function-pointer types: "fn(Int) -> Int"
                // ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ "i64 (i64)*". Parse the signature and lower each part.
                if type_name.starts_with("fn(") {
                    if let Some(sig) = type_name.strip_prefix("fn(") {
                        if let Some(arrow_pos) = sig.find(") -> ") {
                            let params_str = &sig[..arrow_pos];
                            let ret_str = &sig[arrow_pos + 5..];
                            let param_llvm: Vec<String> = if params_str.is_empty() {
                                Vec::new()
                            } else {
                                params_str.split(',')
                                    .map(|p| p.trim())
                                    .filter(|p| !p.is_empty())
                                    .map(|p| self.llvm_type_for(p).unwrap_or_else(|_| "i64".to_string()))
                                    .collect()
                            };
                            let ret_llvm = self.llvm_type_for(ret_str.trim()).unwrap_or_else(|_| "i64".to_string());
                            return Ok(format!("{ret_llvm} ({})*", param_llvm.join(", ")));
                        }
                    }
                }
                // M36: Before giving up, check if this is a type alias (e.g.,
                // `type MyInt8 = Int8`, `type MyResult = Result[Int, Str]`).
                // Follow alias chains with cycle detection.
                {
                    let mut resolved = type_name.to_string();
                    let mut visited = std::collections::HashSet::new();
                    while let Some(target) = self.types.type_aliases.get(&resolved) {
                        if !visited.insert(resolved.clone()) {
                            break; // cycle detected
                        }
                        resolved = target.clone();
                    }
                    if resolved != type_name {
                        return self.llvm_type_for(&resolved);
                    }
                }
                // Final fallback: use xiom_to_llvm_type which maps unknown types
                // to i64 with a warning. This prevents compilation failures for
                // PhantomData, GenericParam, and other marker/forward-declared types.
                Ok(Self::xiom_to_llvm_type(type_name).to_string())
            }
        }
    }

    /// Resolve type name to LLVM type, with suffix-search fallback for module-qualified types.
    /// Use this when the exact type registration is uncertain (e.g., type aliases from other modules).
    fn llvm_type_for_fallback(&self, type_name: &str) -> String {
        match self.llvm_type_for(type_name) {
            Ok(t) => t,
            Err(_) => {
                // Try suffix search across type_meta and types
                let search = format!(".{}", type_name);
                for key in self.types.type_meta.keys() {
                    if key.ends_with(&search) {
                        return format!("%struct.{key}");
                    }
                }
                for key in self.types.types.keys() {
                    if key.ends_with(&search) {
                        return format!("%struct.{key}");
                    }
                }
                // Also check generic_type_names ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â generic types may not
                // be in type_meta/types with bare names but ARE registered
                // as structs (e.g. Cell[T], Map[K,V]).
                for key in self.types.generic_type_names.iter() {
                    if key.ends_with(&search) || key == type_name {
                        if let Ok(t) = self.llvm_type_for(key) {
                            return t;
                        }
                        return format!("%struct.{key}");
                    }
                }
                "i64".to_string()
            }
        }
    }

    pub fn block_contains_unsafe(block: &Block) -> bool {
        // Walk the block looking for Expr::Unsafe
        for stmt in &block.stmts {
            if Self::stmt_or_expr_contains_unsafe(stmt) { return true; }
        }
        false
    }

    /// 5c.30: Byte size of a struct's LLVM layout. Nested BY-VALUE struct
    /// fields contribute their own size ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â the old `field_count ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â 8` math
    /// undercounted (JsonEntry { key: Str, value: JsonValue } is 24 bytes,
    /// not 16), truncating Vec elements on push/index.
    fn struct_byte_size(&self, type_name: &str) -> i64 {
        self.struct_byte_size_depth(type_name, 0)
    }

    fn struct_byte_size_depth(&self, type_name: &str, depth: u32) -> i64 {
        if depth > 8 {
            return 8;
        }
        // BUG 23 #2 fix: NESTED Vec[...] element names have no type_meta entry
        // (the element IS the generic %struct.Vec â€” 4 Ã— i64 = 32 bytes). Without
        // this, Vec[Vec[T]] allocated its elements at 8 bytes each, truncating
        // every inner Vec to its data pointer and corrupting m[i][j] reads.
        if type_name.starts_with("Vec[") && type_name.ends_with(']') {
            return 32;
        }
        let meta = self.types.type_meta.get(&type_name.to_string())
            .or_else(|| {
                self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{type_name}")))
                    .map(|(_, v)| v)
            });
        let Some(meta) = meta else {
            return 8
        };
        let mut total = 0i64;
        for (_, fty) in meta.fields.iter() {
            // Generic type parameters (T, V, K) that represent unresolved
            // container types are stored as i64 handles (pointers to boxed
            // structs). Concrete generic types like Vec[Int] are full inline
            // structs whose size is computed via recursive lookup.
            // Distinguish: single-char uppercase = type param (i64 handle);
            // named types with brackets (Vec[Int]) = concrete types (full struct).
            let is_type_param = fty.len() == 1
                && fty.chars().next().map_or(false, |c| c.is_ascii_uppercase());
            if is_type_param {
                total += 8;
                continue;
            }
            if fty.starts_with('[') {
                if let Some(x_pos) = fty.find(" x ") {
                    if let Ok(n) = fty[1..x_pos].trim().parse::<i64>() {
                        total += n * 8;
                        continue;
                    }
                }
                total += 8;
                continue;
            }
            let llvm = self.llvm_type_for(fty).unwrap_or_else(|_| "i64".to_string());
            if llvm.starts_with("%struct.") && !llvm.ends_with('*') {
                let inner = llvm[8..].to_string();
                total += self.struct_byte_size_depth(&inner, depth + 1);
            } else {
                total += 8;
            }
        }
        if total == 0 { 8 } else { total }
    }

    /// v0.54: Compute alignment of a type in bytes. For built-in types,
    /// returns the natural alignment; for structs, returns max field alignment.
    pub(crate) fn align_of_type(&self, type_name: &str) -> u64 {
        match type_name {
            "Int" | "Int64" | "UInt64" => 8,
            "Int32" | "UInt32" | "Float32" => 4,
            "Int16" | "UInt16" => 2,
            "Int8" | "UInt8" | "Bool" => 1,
            "Float64" | "Str" => 8,
            "Char" => 4,
            _ => {
                // For struct types, alignment = max field alignment
                if let Some(meta) = self.types.type_meta.get(&type_name.to_string()) {
                    let mut max_align = 1u64;
                    for (_, fty) in meta.fields.iter() {
                        let fa = self.align_of_type(fty);
                        if fa > max_align { max_align = fa; }
                    }
                    max_align
                } else {
                    // Try qualified lookup
                    if let Some(meta) = self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{type_name}")))
                        .map(|(_, v)| v)
                    {
                        let mut max_align = 1u64;
                        for (_, fty) in meta.fields.iter() {
                            let fa = self.align_of_type(fty);
                            if fa > max_align { max_align = fa; }
                        }
                        return max_align;
                    }
                    8 // default pointer alignment
                }
            }
        }
    }

    /// v0.54: Compute a stable numeric type ID from the type name.
    /// Uses FNV-1a hash for deterministic cross-platform results.
    pub(crate) fn type_id_of(type_name: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in type_name.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    /// v0.54: Compute byte offset of a named field within a struct type.
    /// Returns 0 for the first field, sizeof(field0) for the second, etc.
    pub(crate) fn field_offset_of(&self, type_name: &str, field_name: &str) -> u64 {
        let meta = self.types.type_meta.get(&type_name.to_string())
            .or_else(|| {
                self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{type_name}")))
                    .map(|(_, v)| v)
            });
        let Some(meta) = meta else { return 0 };
        let mut offset = 0u64;
        for (fname, fty) in meta.fields.iter() {
            if fname == field_name { return offset; }
            // Estimate field size: for simple types use alignment as size
            offset += self.size_of_type(fty);
        }
        0 // field not found
    }

    /// Helper: estimate size of a type name in bytes.
    fn size_of_type(&self, type_name: &str) -> u64 {
        match type_name {
            "Int" | "Int64" | "UInt64" | "Float64" => 8,
            "Int32" | "UInt32" | "Float32" | "Char" => 4,
            "Int16" | "UInt16" => 2,
            "Int8" | "UInt8" | "Bool" => 1,
            "Str" => 8,
            _ => {
                if let Some(_meta) = self.types.type_meta.get(&type_name.to_string()) {
                    self.struct_byte_size(type_name) as u64
                } else {
                    8
                }
            }
        }
    }

    fn field_llvm_type(&self, struct_name: &str, field_idx: usize) -> String {
        let meta = self.types.type_meta.get(&struct_name.to_string())
            .or_else(|| {
                // Try current module's qualified name first (deterministic)
                if let Some(ref module) = self.local.current_module {
                    let qualified = format!("{}.{}", module, struct_name);
                    self.types.type_meta.get(&qualified)
            } else {
                    None
                }
            })
            .or_else(|| {
                // Fallback: search all qualified keys
                self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{struct_name}")))
                    .map(|(_, v)| v)
            });
        if let Some(meta) = meta {
            if let Some((_, ty_name)) = meta.fields.get(field_idx) {
                let result = if ty_name.contains('[') {
                    if let Some(bracket) = ty_name.find('[') {
                        self.llvm_type_for(&ty_name[..bracket])
                            .unwrap_or_else(|_| "i64".to_string())
                    } else { "i64".to_string() }
                } else {
                    let t = self.llvm_type_for(ty_name).unwrap_or_else(|_| "i64".to_string());
                    t
                };
                return result;
            }
        }
        "i64".to_string()
    }

    /// 5e.1 G-18: byte size of a struct from type_meta field list.
    /// Sums LLVM type widths: i8=1, i16=2, i32=4, i64=8, etc.
    /// Returns 0 for unknown types. Used for C FFI malloc/offsetof.
    pub(crate) fn sizeof_struct(&self, type_name: &str) -> usize {
        let suffix = format!(".{type_name}");
        let meta = self.types.type_meta.get(&type_name.to_string())
            .or_else(|| self.types.type_meta.entries().into_iter().find(|(k, _)| k.ends_with(&suffix)).map(|(_, v)| v));
        let Some(meta) = meta else { return 0 };
        meta.fields.iter().map(|(_, ty_name)| {
            if ty_name.contains('[') { return 8; }
            let llvm_ty = self.llvm_type_for(ty_name).unwrap_or_else(|_| ty_name.clone());
            match llvm_ty.as_str() {
                "i1" | "i8" => 1, "i16" => 2, "i32" | "float" => 4,
                "i64" | "double" => 8, _ => 8,
            }
        }).sum()
    }

    /// Returns `true` if `name` is the name of a variant of the enum currently
    /// being matched on. Used to decide whether a bare `Pattern::Ident` should
    /// be compiled as a runtime discriminant check (rather than a variable
    /// binding / wildcard).
    fn ident_is_enum_variant(&self, scrutinee_type: &Option<String>, name: &str) -> bool {
        if let Some(type_name) = scrutinee_type {
            // 5c.29: qualified variant idents (`SqliteValue.Null`) carry the
            // enum qualifier ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â compare the LEAF segment. Also tolerate
            // qualified/unqualified enum keys.
            let leaf = name.rsplit('.').next().unwrap_or(name);
            let variants = self.types.enum_variants.get(&type_name.to_string())
                .or_else(|| {
                    self.types.enum_variants.entries().into_iter()
    .find(|(k, _)| {
                            k.ends_with(&format!(".{type_name}"))
                                || type_name.ends_with(&format!(".{}", k.as_str()))
                        })
                        .map(|(_, v)| v)
                });
            if let Some(variants) = variants {
                return variants.iter().any(|(v, _)| v.as_str() == leaf || v.as_str() == name);
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
            Pattern::Lit(Literal::Int(..)) | Pattern::Lit(Literal::Float(..)) | Pattern::Lit(Literal::Bool(..))
            | Pattern::Lit(Literal::Str(..)) | Pattern::Lit(Literal::Char(..)) => true,
            Pattern::Variant(..) => true,
            Pattern::Some(..) | Pattern::None(..) | Pattern::Ok(..) | Pattern::Err(..) => true,
            Pattern::Ident(ident) => self.ident_is_enum_variant(scrutinee_type, &ident.name),
            Pattern::Or(alternatives, _) => alternatives.iter().any(|a| self.pattern_needs_check(a, scrutinee_type)),
            Pattern::Struct(..) => true,
            Pattern::Tuple(..) => true,
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
            // 5c.29: qualified variant patterns (`SqliteValue.Integer(v)`)
            // carry the enum qualifier in the name ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â compare against the LEAF
            // segment. Also tolerate qualified/unqualified enum keys.
            let leaf = variant_name.rsplit('.').next().unwrap_or(variant_name);
            let variants = self.types.enum_variants.get(&type_name.to_string())
                .or_else(|| {
                    self.types.enum_variants.entries().into_iter()
    .find(|(k, _)| {
                            k.ends_with(&format!(".{type_name}"))
                                || type_name.ends_with(&format!(".{}", k.as_str()))
                        })
                        .map(|(_, v)| v)
                });
            let variant_idx = variants
                .and_then(|vs| vs.iter().position(|(vn, _)| vn == leaf || vn == variant_name))
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
        if !self.types.types.contains_key(&"Option".to_string()) {
            self.types.types.insert("Option".to_string(), vec!["discriminant".to_string(), "value".to_string()]);
            self.types.type_meta.insert("Option".to_string(), TypeMeta {
                fields: vec![("discriminant".to_string(), "Int".to_string()), ("value".to_string(), "Int".to_string())],
                derives: vec![],
                invariants: vec![],
            });
        }
        if !self.types.types.contains_key(&"Result".to_string()) {
            self.types.types.insert("Result".to_string(), vec!["discriminant".to_string(), "value".to_string(), "error".to_string()]);
            self.types.type_meta.insert("Result".to_string(), TypeMeta {
                fields: vec![
                    ("discriminant".to_string(), "Int".to_string()),
                    ("value".to_string(), "Int".to_string()),
                    ("error".to_string(), "Int".to_string()),
                ],
                derives: vec![],
                invariants: vec![],
            });
        }
        if !self.types.enum_variants.contains_key(&"Result".to_string()) {
            self.types.enum_variants.insert("Result".to_string(), vec![
                ("Err".to_string(), vec!["error".to_string()]),
                ("Ok".to_string(), vec!["value".to_string()]),
            ]);
        }
        // Register Vec type for runtime operations ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ensure 4 fields
        // (data, len, cap, elem_size). The elem_size field tracks the
        // element byte width so narrow types (UInt8ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢1, Int16ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢2, etc.)
        // work correctly in push/pop/get/index operations.
        // Vec may be registered under "Vec" or "xiom.collections.Vec"
        // or may not be in type_meta at all (generic type skipped).
        {
            let fields = vec!["data".to_string(), "len".to_string(), "cap".to_string(), "elem_size".to_string()];
            let full_fields: Vec<(String, String)> = vec![
                ("data".to_string(), "*UInt8".to_string()),
                ("len".to_string(), "Int".to_string()),
                ("cap".to_string(), "Int".to_string()),
                ("elem_size".to_string(), "Int".to_string()),
            ];
            self.types.types.insert("Vec".to_string(), fields);
            // Ensure type_meta has a Vec entry so the struct is emitted
            self.types.type_meta.or_insert_with("Vec".to_string(), || TypeMeta {
                fields: full_fields.clone(),
                derives: Vec::new(),
                invariants: Vec::new(),
            });
            for key in &["xiom.collections.Vec".to_string()] {
                let meta_opt = self.types.type_meta.get(key);
                if let Some(mut meta) = meta_opt {
                    if meta.fields.len() < 4 {
                        meta.fields.push(("elem_size".to_string(), "Int".to_string()));
                    }
                    self.types.type_meta.insert(key.clone(), meta);
                }
            }
            // Map[K, V]: register with keys and values fields (both are %struct.Vec).
            // Without this, field access like `entries.keys` falls through
            // to function-pointer resolution (producing undefined @Map.keys).
            let map_fields: Vec<String> = vec!["keys".to_string(), "values".to_string()];
            let map_full_fields: Vec<(String, String)> = vec![
                ("keys".to_string(), "Vec".to_string()),
                ("values".to_string(), "Vec".to_string()),
            ];
            self.types.types.insert("Map".to_string(), map_fields);
            self.types.type_meta.or_insert_with("Map".to_string(), || TypeMeta {
                fields: map_full_fields,
                derives: Vec::new(),
                invariants: Vec::new(),
            });
        }

        // 5e.3: Register Layout as a builtin type so its struct definition
        // ({i64, i64}) is emitted even when alloc.xi is not compiled directly.
        // The Layout.new constructor is inlined in expr.rs; this ensures the
        // type definition exists for the emitted insertvalue instructions.
        if !self.types.type_meta.contains_key(&"xiom.alloc.Layout".to_string()) {
            self.types.type_meta.insert("xiom.alloc.Layout".to_string(), TypeMeta {
                fields: vec![
                    ("size".to_string(), "Int".to_string()),
                    ("align".to_string(), "Int".to_string()),
                ],
                derives: vec![],
                invariants: vec![],
            });
            self.types.types.insert("Layout".to_string(), vec!["size".to_string(), "align".to_string()]);
        }

        // 5e.3: Register RcInner as a builtin type so size_of[RcInner[T]]()
        // resolves inside monomorphised generic bodies (e.g. Rc.new_Int).
        // RcInner has 3 i64-wide fields: strong, weak, value = 24 bytes.
        if !self.types.type_meta.contains_key(&"xiom.rc.RcInner".to_string()) {
            self.types.type_meta.insert("xiom.rc.RcInner".to_string(), TypeMeta {
                fields: vec![
                    ("strong".to_string(), "Int".to_string()),
                    ("weak".to_string(), "Int".to_string()),
                    ("value".to_string(), "Int".to_string()),
                ],
                derives: vec![],
                invariants: vec![],
            });
            self.types.types.insert("RcInner".to_string(), vec![
                "strong".to_string(), "weak".to_string(), "value".to_string(),
            ]);
        }

        // Register type structures
        for item in &program.items {
            self.register_type_layout(item);
        }

        // B-001: Concrete Result/Option types are created on-demand by
        // `concrete_type_for` during `register_functions` and `compile_fn`.
        // Pre-registration (O(n^2)) is NOT used because it creates excessive
        // types that interfere with module-qualified field resolution in the
        // prologue. Each function that returns Result/Option with struct args
        // triggers just the concrete types it needs via `ensure_concrete_*`.
        // Pre-registration is kept as dead code for reference only.
        // self.pre_register_concrete_types();

        // Register function signatures
        for item in &program.items {
            self.register_functions(item);
        }

        // 5c.36: Pre-register expression-level tuple types from function bodies
        // so their struct definitions are emitted at module level.
        for item in &program.items {
            self.register_expr_tuple_types(item);
        }

        // 5e.7f: Const evaluation pass ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â fold const expressions after all
        // constants are registered so cross-references resolve correctly.
        self.evaluate_all_consts();

        // Scan interface implementations: for each interface, find all concrete
        // types that implement all its methods (BUG-007 interface dispatch).
        self.scan_interface_impls();

        // Emit module header
        self.emitln("; XIOM v0.50.0 LLVM IR");
        self.emitln("; Auto-generated by xiom\n");
        self.emitln(&format!("target triple = \"{}\"", self.config.target_triple));
        // D1 (2026-08-08): explicit datalayout so clang aligns i128/fp128 to
        // 16 bytes (required for native Int128/UInt128/Float128 on x86-64).
        // Without it, clang defaults i128 alignment to 8 on MSVC, producing
        // misaligned struct fields and #GP faults on 16-byte loads/stores.
        self.emitln("target datalayout = \"e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128\"");
        self.emitln("");
        // v0.55: Declare C runtime threading/channel functions (always available).
        // Deferred: emit these AFTER user extern blocks so xiom_thread_spawn
        // doesn't conflict with user declarations (e.g. thread.xi).
        let spawn_declare = String::from("declare i64 @xiom_thread_spawn(ptr, ptr)");
        self.emitln("declare ptr @xiom_channel_create()");
        // Runtime arg seeding (env.args()) ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â declared here so the @main entry
        // can call it regardless of whether io.xi's extern block is loaded.
        self.emitln("declare void @xiom_set_args(i32, i8**)");
        self.emitln("declare i64 @xiom_channel_send(ptr, i64)");
        self.emitln("declare i64 @xiom_channel_recv(ptr)");
        self.emitln("declare i64 @xiom_channel_try_recv(ptr, ptr)");
        // v0.56: Thread pool functions
        self.emitln("declare void @xiom_threadpool_init(i64)");
        self.emitln("declare void @xiom_threadpool_spawn(ptr, ptr)");
        self.emitln("");

        // Emit builtin struct types FIRST so user types can reference them.
        // Vec is emitted from type_meta (with elem_size if registered via
        // the code below) ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â NOT hardcoded so narrow-type Vecs get correct layout.
        self.emitln("");

        // Emit struct type definitions using actual field types from type_meta
        for (name, meta) in self.types.type_meta.entries() {
            let struct_ref = format!("%struct.{name}");
            let field_types: Vec<String> = meta.fields.iter()
                .map(|(_, ty_name)| {
                    let t = self.llvm_type_for(ty_name).unwrap_or_else(|_| "i64".to_string());
                    if t == struct_ref { format!("{t}*") } else { t }
                })
                .collect();
            self.emitln(&format!("%struct.{name} = type {{ {} }}", field_types.join(", ")));
        }
        if !self.types.types.len() == 0 {
            self.emitln("");
        }

        // Emit mutable module-level `var` globals (real LLVM globals read via
        // `load` and written via `store`). Registered during register_functions;
        // deduped by symbol so the defining module and an injected external copy
        // never emit the same global twice.
        if !self.local.module_global_defs.is_empty() {
            for (symbol, llvm_ty, init) in &self.local.module_global_defs.clone() {
                self.emitln(&format!("@{symbol} = internal global {llvm_ty} {init}"));
            }
            self.emitln("");
        }

        // BUG 3 fix: globals whose initializer is a runtime expression get a
        // @llvm.global_ctors entry (startup initializer). The ctor function
        // bodies are emitted at module end, after all functions are compiled.
        if !self.local.global_runtime_inits.is_empty() {
            let n = self.local.global_runtime_inits.len();
            let entries: Vec<String> = (0..n)
                .map(|i| format!("{{ i32, ptr, ptr }} {{ i32 65535, ptr @__xiom_ginit_{i}, ptr null }}"))
                .collect();
            self.emitln(&format!(
                "@llvm.global_ctors = appending global [{n} x {{ i32, ptr, ptr }}] [{}]",
                entries.join(", ")
            ));
            self.emitln("");
        }

        // 5e.5c: emit hot reload state save/restore functions
        if self.config.hot_reload && !self.config.xiom_hot_globals.is_empty() {
            self.emit_hot_state_functions();
        }

        self.emit_builtin_declares();

        // R1: Emit DWARF debug metadata for .xi source-level debugging
        self.emit_debug_metadata();

        // Emit declares for user-defined extern "C" functions
        // (skips names already in self.mono.already_declared, e.g. malloc)
        // Pre-seed the metadata-accessor names when their tables will be DEFINED
        // below (reflect/contracts), so the user extern block's `declare` for them
        // is skipped ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â otherwise the same symbol is both declared and defined,
        // which clang rejects as an invalid redefinition.
        if Self::program_declares_extern(&program.items, "xiom_type_count") {
            for nm in ["xiom_type_count", "xiom_type_name", "xiom_type_field_count", "xiom_type_id_by_name"] {
                self.mono.already_declared.insert(nm.to_string());
            }
        }
        if Self::program_declares_extern(&program.items, "xiom_contract_fn_count") {
            for nm in ["xiom_contract_fn_count", "xiom_contract_fn_name", "xiom_contract_pre_count", "xiom_contract_post_count"] {
                self.mono.already_declared.insert(nm.to_string());
            }
        }
        self.emit_extern_declares(&program.items);
        // Emit deferred spawn declare ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â only if user didn't already declare it
        // in an extern block (thread.xi has its own *UInt8-based declaration).
        if !self.local.spawn_declared {
            self.emitln(&format!("{spawn_declare}"));
        }

        // Additive metadata emission: RTTI for the `reflect` stdlib module and a
        // contract table for the `contracts` stdlib module. This is a NEW,
        // self-contained step appended alongside the runtime `declare`s above.
        // It NEVER alters any existing lowering path ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â it only reads
        // already-registered type metadata (`self.types.type_meta` / `self.types.enum_variants`)
        // and the program AST (function contract clauses), then emits new globals
        // and `@xiom_*` function definitions. Emission is gated on the presence of
        // the matching `extern "C"` declarations (added only in reflect.xi /
        // contracts.xi), so every other program is byte-for-byte unaffected.
        self.emit_metadata_tables(program);

        // Emit derive implementations for types with derive clauses
        self.compile_derive_impls(&program.items)?;

        // Define all non-generic function bodies, tracking generic instantiations.
        // I2: When --parallel-codegen is enabled, compile independent functions
        // in parallel using rayon. Each function gets its own output buffer; we
        // merge them in declaration order after all tasks complete.
        // BUG 22 #11: pre-assign symbols for ALL fns first (sequential path)
        // so definitions and call sites agree regardless of emit order.
        self.preassign_fn_symbols(&program.items);
        if self.config.parallel_codegen {
            self.has_non_empty_main = Self::program_has_non_empty_main(&program.items);
            self.compile_functions_parallel(&program.items)?;
        } else {
            self.has_non_empty_main = Self::program_has_non_empty_main(&program.items);
            for item in &program.items {
                self.compile_top_decl(item)?;
            }
        }

        // Emit monomorphised generic function bodies
        self.compile_generic_monomorphisations()?;

        // Emit builtin runtime implementations (Option, Result, alloc, etc.)
        self.compile_builtin_impls();

        // Emit string constants collected during compilation
        let strings_before_ginits = self.fctx.strings.len();
        for s in &self.fctx.strings.clone() {
            self.emitln(&s);
        }
        if !self.fctx.strings.is_empty() {
            self.emitln("");
        }

        // Safety net: stub any called-but-undefined function symbol. Such symbols
        // only arise from erased-generic dead-code method bodies (e.g. a
        // `data.len()` inside a monomorphised-away `BinaryHeap.push` where `self`
        // is opaque), which would otherwise make clang reject the whole module
        // with "use of undefined value '@name'". On any well-formed program (all
        // callees resolved) this pass emits nothing, so it is a strict no-op on
        // the existing test gate. A stub returns a typed default, so it can never
        // manufacture a *correct* live result â€” only unblock linking.
        self.emit_undefined_symbol_stubs();

        // BUG 3 fix: emit the @llvm.global_ctors initializer bodies for
        // module-level `var` globals with runtime initializer expressions.
        if !self.local.global_runtime_inits.is_empty() {
            let inits = self.local.global_runtime_inits.clone();
            for (i, (symbol, llvm_ty, init_expr)) in inits.iter().enumerate() {
                // Minimal function context so compile_expr can resolve calls
                // (fn_key uses current_fn for the caller-module fallback).
                self.push_scope();
                self.block_counter = 0;
                self.tmp_counter = 0;
                self.local.signed_locals.clear();
                self.local.local_xiom_types.clear();
                self.local.reg_signed.clear();
                self.local.ptr_locals.clear();
                self.local.bool_locals.clear();
                self.fctx.current_fn = Some(format!("__xiom_ginit_{i}"));
                self.fctx.current_receiver = None;
                self.fctx.current_ensures.clear();
                self.fctx.current_return_type = "void".to_string();
                self.emitln(&format!("\ndefine internal void @__xiom_ginit_{i}() {{"));
                self.emitln("entry:");
                let (val, val_ty) = self.compile_expr(init_expr)?;
                let store_val = self.coerce_value(&val, &val_ty, llvm_ty);
                self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* @{symbol}"));
                self.emitln("  ret void");
                self.emitln("}");
                self.pop_scope();
                self.fctx.current_fn = None;
                self.fctx.current_ensures.clear();
            }
            self.emitln("");
            // BUG 29 (m36_r01/r14): ginit bodies compile AFTER the string
            // constants were dumped above, so strings collected while
            // compiling a global initializer (`var s: Str = "...";`) were
            // never emitted — clang rejected "use of undefined value
            // '@.str0'". Re-emit ONLY the strings added since the first
            // dump (re-emitting all would redefine every constant).
            for s in &self.fctx.strings.clone()[strings_before_ginits..] {
                self.emitln(s);
            }
            if self.fctx.strings.len() > strings_before_ginits {
                self.emitln("");
            }
        }

        // BUG 27 (Map.new in module-global inits): the @llvm.global_ctors
        // bodies compile AFTER the first monomorphisation pass, so a generic
        // constructor called from a global initializer (contracts.xi's
        // `_coverage = Map[Str, Bool].new()`) queues its instantiation too
        // late — the body was never emitted and the call referenced an
        // undefined symbol. Drain the queue again (the loop is idempotent:
        // it stops when the worklist is empty).
        self.compile_generic_monomorphisations()?;

        // Deferred tuple/anon struct type definitions discovered during function
        // body compilation. Emitted at module end (top level) — LLVM named types
        // support forward references, so this is valid even though the types may
        // already be used above.
        if !self.local.pending_module_type_defs.is_empty() {
            self.emitln("");
            let pending = self.local.pending_module_type_defs.clone();
            for def in &pending {
                self.emitln(def);
            }
        }

        Ok(self.output.clone())
    }

    // ========================================================================
    // I2: Parallel Codegen ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â rayon-based per-function IR emission
    // ========================================================================

    /// Walk the program tree to collect all non-generic function declarations
    /// with their module prefix and positional index (for output ordering).
    /// Static fn_key equivalent for parallel symbol pre-assignment. Mirrors
    /// IrEmitter::fn_key for receiver methods and free functions using the
    /// module prefix (no emitter state needed).
    fn fn_key_with_prefix(fd: &FnDecl, module_prefix: &str) -> String {
        if let Some(recv_name) = &fd.receiver {
            let recv_type = if module_prefix.is_empty() {
                recv_name.name.clone()
            } else {
                format!("{}.{}", module_prefix, recv_name.name)
            };
            let bare_method = fd.name.name.rsplit('.').next().unwrap_or(&fd.name.name);
            format!("{recv_type}.{bare_method}")
        } else {
            fd.name.name.clone()
        }
    }

    /// Returns true when the program contains a `fn main` with a non-empty body.
    fn program_has_non_empty_main(items: &[TopDecl]) -> bool {        for item in items {
            match item {
                TopDecl::Fn(fd) => {
                    if fd.name.name == "main"
                        && fd.body.as_ref().map_or(false, |b| !b.stmts.is_empty()) {
                        return true;
                    }
                }
                TopDecl::Module(md) => {
                    if Self::program_has_non_empty_main(&md.items) { return true; }
                }
                _ => {}
            }
        }
        false
    }

    fn collect_functions_to_compile<'a>(
        items: &'a [TopDecl],
        module_prefix: &str,
        start_idx: &mut usize,
    ) -> Vec<(usize, String, &'a FnDecl)> {
        let mut result = Vec::new();
        for item in items {
            match item {
                TopDecl::Fn(fd) => {
                    if fd.generics.is_empty() && fd.body.is_some()
                        // Empty-body `fn main() { }` is a placeholder (e.g. async
                        // main). Only emit it when NO non-empty main exists ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â a
                        // real entry point must not be shadowed. A standalone
                        // empty main IS emitted (JIT/shared-lib need @main).
                        && !(fd.name.name == "main"
                            && fd.body.as_ref().map_or(false, |b| b.stmts.is_empty())
                            && Self::program_has_non_empty_main(items))
                    {
                        let recv_is_generic = fd.receiver.as_ref()
                            .map(|_r| false) // simplified: type check done by caller
                            .unwrap_or(false);
                        if !recv_is_generic {
                            let prefix = if module_prefix.is_empty() {
                                String::new()
                            } else {
                                format!("{}.", module_prefix)
                            };
                            let idx = *start_idx;
                            *start_idx += 1;
                            result.push((idx, prefix, fd));
                        }
                    }
                }
                TopDecl::Module(md) => {
                    let new_prefix = if module_prefix.is_empty() {
                        md.name.name.clone()
                    } else {
                        format!("{}.{}", module_prefix, md.name.name)
                    };
                    let sub = Self::collect_functions_to_compile(&md.items, &new_prefix, start_idx);
                    result.extend(sub);
                }
                _ => {}
            }
        }
        result
    }

    /// Compile function bodies in parallel using rayon. Each function gets its
    /// own output buffer; we merge them in declaration order after all tasks
    /// complete. This maps naturally to XIOM's `spawn` + `Channel[T]` pattern
    /// for the selfhost compiler.
    fn compile_functions_parallel(&mut self, items: &[TopDecl]) -> Result<(), String> {
        // Step 1: Collect all functions to compile
        let mut idx = 0;
        let functions = Self::collect_functions_to_compile(items, "", &mut idx);

        if functions.is_empty() {
            return Ok(());
        }

        // Step 1.5: Pre-assign unique symbols across ALL functions BEFORE the
        // parallel loop. Each parallel emitter starts with an empty emitted_fns,
        // so fn_symbol's collision check never fired ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â env.args and io.args both
        // compiled to `@args`, the later one overwriting the earlier and
        // self-recursing (env.args() ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ @args ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ @args ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ infinite recursion).
        let mut seen_bare: HashSet<String> = HashSet::new();
        let mut assignments: Vec<(usize, String, HashSet<String>)> = Vec::new();
        for (idx, prefix, fd) in &functions {
            let bare = Self::fn_key_with_prefix(fd, prefix);
            let snapshot = seen_bare.clone();
            let sym = if seen_bare.contains(&bare) {
                if prefix.is_empty() {
                    bare.clone()
                } else {
                    format!("{}.{}", prefix, bare)
                }
            } else {
                bare.clone()
            };
            seen_bare.insert(bare);
            assignments.push((*idx, sym, snapshot));
        }

        // Step 2: Compile each function in parallel
        let type_ctx = Arc::new(self.types.clone());
        let cfg = Arc::new(self.config.clone());
        let ctfe_snapshot = Arc::new(self.ctfe.borrow().clone());
        let local_constants = Arc::new(self.local.constants.clone());
        let outputs: Vec<_> = functions
            .par_iter()
            .zip(assignments)
            .map(|((idx, prefix, fd), (_aidx, sym, snapshot))| {
                let mut emitter = IrEmitter::new();
                emitter.types = (*type_ctx).clone();
                emitter.config = (*cfg).clone();
                *emitter.ctfe.borrow_mut() = (*ctfe_snapshot).clone();
                emitter.local.constants = (*local_constants).clone();
                emitter.local.current_module = if prefix.is_empty() { None } else { Some(prefix.clone()) };
                // I2 fix: Prevent global string name collisions across parallel
                // emitters by offsetting each emitter's str_counter into a unique
                // range (each function gets 1000 string slots).
                emitter.str_counter = (*idx as u32) * 1000;
                emitter.tmp_counter = (*idx as u32) * 10000;
                emitter.block_counter = (*idx as u32) * 10000;
                // I2 fix: the unsafe-block counter must ALSO be unique per
                // function â€” otherwise every function with an `unsafe { }`
                // emits `%struct.__unsafe_ctx_0` / `__unsafe_block_0` and the
                // merged module has a redefinition of the ctx type (clang:
                // "redefinition of type %struct.__unsafe_ctx_0"). Each function
                // gets 1000 unsafe-block slots.
                emitter.unsafe_block_counter = (*idx as u32) * 1000;

                // Use the pre-assigned symbol; seed emitted_fns with the names
                // seen BEFORE this function so fn_symbol qualifies identically.
                emitter.mono.emitted_fns = snapshot;
                let fn_name = sym;
                emitter.mono.emitted_fns.insert(fn_name.clone());
                if emitter.config.hot_reload && fd.is_pub {
                    emitter.config.pub_functions.insert(fn_name);
                }

                match emitter.compile_fn(fd) {
                    Ok(()) => {
                        let out = emitter.output.clone();
                        let gens = emitter.mono.generic_instantiations.clone();
                        let used = emitter.types.used_builtins.clone();
                        let strs = emitter.fctx.strings.clone();
                        (*idx, out, gens, used, strs)
                    },
                    Err(e) => (*idx, format!("; ERROR compiling {}: {}\n", fd.name.name, e), Vec::new(), HashSet::new(), Vec::new()),
                }
            })
            .collect::<Vec<_>>();

        // Step 3: Merge outputs in declaration order
        let mut sorted: Vec<_> = outputs.into_iter().collect();
        sorted.sort_by_key(|(idx, _, _, _, _)| *idx);

        for (_idx, output, generics, builtins, strings) in sorted {
            self.output.push_str(&output);
            for g in generics {
                self.mono.generic_instantiations.push(g);
            }
            for b in builtins {
                self.types.used_builtins.insert(b);
            }
            for s in strings {
                self.fctx.strings.push(s);
            }
        }

        Ok(())
    }

    // Contract runtime checks ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ see contracts.rs


    /// Infer the struct type name from an expression (if it produces a struct value).
    fn struct_type_from_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            // M18: Inlined Option/Result constructors ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â return the struct type
            // so match compilation creates scrutinee alloca for inner value checks.
            Expr::Some(..) | Expr::None(..) => {
                if self.types.types.contains_key(&"Option".to_string()) { Some("Option".to_string()) }
                else { None }
            }
            Expr::Ok(..) | Expr::Err(..) => {
                if self.types.types.contains_key(&"Result".to_string()) { Some("Result".to_string()) }
                else { None }
            }
            Expr::Field(obj, field, _) => {
                // 5c.29: Enum variant literal `EnumType.Variant` (e.g.
                // `match DistanceMetric.Cosine { ... }`): the scrutinee type is
                // the enum itself. Without this, bare-ident match arms were
                // treated as bindings and dispatch fell through to the last arm.
                if let Expr::Ident(base_id) = obj.as_ref() {
                    for (ek, vars) in self.types.enum_variants.entries() {
                        if (ek == base_id.name || ek.ends_with(&format!(".{}", base_id.name)))
                            && vars.iter().any(|(v, _)| v == &field.name)
                        {
                            return Some(ek.clone());
                        }
                    }
                }
                // Field access: resolve the base struct, then look up
                // the field's declared type for accurate match dispatch.
                // e.g. `match a.state { ... }` where `a: &Agent` and
                // `state: AgentState` should use `AgentState` as the
                // scrutinee type, not `Agent`.
                if let Some(base_type) = self.infer_struct_type_name(obj.as_ref()) {
                    for key in self.types.type_meta.keys() {
                        if key.ends_with(&base_type) || key == base_type {
                            if let Some(meta) = self.types.type_meta.get(&key) {
                                for (fname, ftype) in &meta.fields {
                                    if fname == &field.name {
                                        let clean = ftype.trim_start_matches('*');
                                        // 5c.31: Skip primitive types (Int, Bool, etc.)
                                        // ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â `%struct.Int` is not a valid LLVM type.
                                        // Primitive field matches use plain values.
                                        if Self::is_primitive_type_name(clean) {
                                            return None;
                                        }
                                        if self.types.type_meta.contains_key(&clean.to_string()) {
                                            return Some(clean.to_string());
                                        }
                                        for mk in self.types.type_meta.keys() {
                                            if mk.ends_with(&format!(".{}", clean)) {
                                                return Some(mk.clone());
                                            }
                                        }
                                        return Some(clean.to_string());
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
                None
            }
            Expr::Struct(ident, _, _, _) => Some(ident.name.clone()),
            Expr::Ident(ident) => {
                // `this` keyword remaps to `self` in method bodies (same as compile_expr).
                let lookup_name = if ident.name == "this" { "self" } else { ident.name.as_str() };
                if let Some((_, llvm_ty)) = self.lookup_local(lookup_name) {
                    if llvm_ty.starts_with("%struct.") {
                        let raw = &llvm_ty[8..]; // strip "%struct."
                        let clean = raw.trim_end_matches('*'); // strip pointer suffix
                        return Some(clean.to_string());
                    }
                }
                None
            }
            Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
                let fn_name = match &**func {
                    Expr::Ident(name) => Some(name.name.clone()),
                    Expr::Field(obj, field, _) => {
                        let bare = field.name.clone();
                        if let Some(recv_type) = self.infer_struct_type_name(obj) {
                            let qualified = format!("{}.{}", recv_type, field.name);
                            if self.types.functions.contains_key(&qualified) {
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
                    if self.types.type_meta.contains_key(name) {
                        return Some(name.clone());
                    }
                    // Also check the return type from the function registry
                    if let Some((_, ret_ty)) = self.types.functions.get(name) {
                        if ret_ty.starts_with("%struct.") {
                            return Some(ret_ty[8..].to_string());
                        }
                    }
                }
                None
            }
            Expr::Tuple(_, _) => None,
            _ => None,
        }
    }


    /// Compile a function pointer call from a Vec index: `tests[i]()`.
    /// The `container[index]` expression yields a function pointer (stored as i64
    /// in the Vec's data buffer). Load it, inttoptr, and call.
    fn compile_index_fn_ptr_call(&mut self, container: &Expr, index: &Expr, args: &[Expr]) -> Result<(String, String), String> {
        // Compile the container[index] expression to get the element value.
        let idx_expr = Expr::Index(Box::new(container.clone()), Box::new(index.clone()), xiom_ast::Span { line: 0, col: 0 });
        let (elem_val, elem_ty) = self.compile_expr(&idx_expr)?;
        // Convert the element to i64 (it may already be i64 from Vec indexing).
        let i64_val = self.val_to_i64(&elem_val, &elem_ty);
        // Build the function pointer type from args.
        let compiled_args: Vec<(String, String)> = args.iter()
            .map(|a| self.compile_expr(a).map(|(v, t)| (v, t)))
            .collect::<Result<Vec<_>, _>>()?;
        let args_str = compiled_args.iter()
            .map(|(v, t)| format!("{t} {v}"))
            .collect::<Vec<_>>().join(", ");
        let param_types: Vec<String> = args.iter()
            .map(|a| self.infer_llvm_type(a))
            .collect();
        let fn_ptr_ty = format!("i64 ({})*", param_types.join(", "));
        let fn_ptr = self.fresh_tmp();
        self.emitln(&format!("  {fn_ptr} = inttoptr i64 {i64_val} to {fn_ptr_ty}"));
        let tmp = self.fresh_tmp();
        self.emitln(&format!("  {tmp} = call i64 {fn_ptr}({args_str})"));
        Ok((tmp, "i64".to_string()))
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
                // Type aliases have no fields ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â nothing to derive or invariant-check
                if td.fields.is_empty() && td.alias.is_some() {
                    return Ok(());
                }
                let bare_name = &td.name.name;
                // Resolve to qualified name using module context
                let type_name = if let Some(ref module) = self.local.current_module {
                    let qualified = format!("{}.{}", module, bare_name);
                    if self.types.type_meta.contains_key(&qualified) { qualified } else { bare_name.clone() }
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
                        DeriveTrait::Debug => self.compile_debug_impl(&type_name, &struct_ty, &field_names)?,
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
                let type_name = if let Some(ref module) = self.local.current_module {
                    let qualified = format!("{}.{}", module, bare_name);
                    if self.types.type_meta.contains_key(&qualified) { qualified } else { bare_name.clone() }
                } else {
                    bare_name.clone()
                };
                if !self.types.types.contains_key(&type_name) {
                    self.types.types.insert(type_name.clone(), vec!["discriminant".to_string()]);
                    self.types.type_meta.insert(type_name.clone(), TypeMeta {
                        fields: vec![("discriminant".to_string(), "Int".to_string())],
                        derives: ed.derives.clone(),
                        invariants: Vec::new(),
                    });
                }
                let struct_ty = self.llvm_type_for(&type_name)?;
                let field_names: Vec<String> = vec!["discriminant".to_string()];

                for derive in &ed.derives {
                    match derive {
                        DeriveTrait::Eq => self.compile_enum_eq_impl(&type_name, &struct_ty, ed)?,
                        DeriveTrait::Clone => self.compile_clone_impl(&type_name, &struct_ty, &field_names)?,
                        DeriveTrait::Hash => self.compile_enum_hash_impl(&type_name, &struct_ty, ed)?,
                        DeriveTrait::Ord => self.compile_enum_ord_impl(&type_name, &struct_ty, ed)?,
                        DeriveTrait::Display => self.compile_enum_display_impl(&type_name, &struct_ty, ed)?,
                        DeriveTrait::Debug => self.compile_enum_debug_impl(&type_name, &struct_ty, ed)?,
                    }
                }
            }
            TopDecl::Module(md) => {
                let saved_module = self.local.current_module.clone();
                self.local.current_module = Some(if let Some(ref prev) = saved_module {
                    format!("{}.{}", prev, md.name.name)
                } else {
                    md.name.name.clone()
                });
                for sub in &md.items {
                    self.compile_derive_for_item(sub)?;
                }
                self.local.current_module = saved_module;
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_eq_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String], _fields: &[FieldDecl]) -> Result<(), String> {
        let fn_name = format!("{type_name}.eq");
        if self.mono.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
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
        self.types.functions.insert(fn_name, (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    fn compile_clone_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.clone");
        if self.mono.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], struct_ty.to_string()));
        // G-13: a by-value struct return copies EVERY field ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â including enum
        // payload slots that aren't in field_names (the old field-by-field
        // loop copied only ["discriminant"] for enums, dropping payloads).
        // Shallow-copy semantics match derived struct clone (heap handles
        // shared), now uniform across structs and enums.
        let _ = field_names;
        self.emitln(&format!("define {struct_ty} @{fn_name}({struct_ty} %self) {{"));
        self.emitln(&format!("  ret {struct_ty} %self"));
        self.emitln("}\n");
        self.types.functions.insert(fn_name, (vec![struct_ty.to_string()], struct_ty.to_string()));
        Ok(())
    }

    fn compile_display_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.to_str");
        if self.mono.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], "i8*".to_string()));
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
        self.fctx.strings.push(format!(
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
        self.types.functions.insert(fn_name, (vec![struct_ty.to_string()], "i8*".to_string()));
        Ok(())
    }

    /// 8B/M9: Debug derive for structs ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â delegates to Display for now
    fn compile_debug_impl(&mut self, type_name: &str, struct_ty: &str, _field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.fmt");
        if self.mono.emitted_fns.contains(&fn_name) { return Ok(()); }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), "i8*".to_string()], "i8*".to_string()));

        // Debug.fmt delegates to Display.to_str
        self.emitln(&format!("define i8* @{fn_name}({struct_ty} %self, i8* %_f) {{"));
        self.emitln("entry:");
        let ptr = self.fresh_tmp();
        self.emitln(&format!("  {ptr} = call i8* @{type_name}.to_str({struct_ty} %self)"));
        self.emitln(&format!("  ret i8* {ptr}"));
        self.emitln("}\n");
        Ok(())
    }

    fn compile_hash_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String]) -> Result<(), String> {
        let fn_name = format!("{type_name}.hash");
        if self.mono.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], "i64".to_string()));
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
            // Coerce the field value to i64 before mixing into the hash accumulator
            // (a Str field is i8*, a Char field is i8, etc.) ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â avoids `add i64, i8*`.
            let val_i64 = self.val_to_i64(&val, &field_llvm_ty);
            self.emitln(&format!("  {loaded_hash} = load i64, i64* %hash"));
            self.emitln(&format!("  {mul_tmp} = mul i64 {loaded_hash}, 33"));
            self.emitln(&format!("  {add_tmp} = add i64 {mul_tmp}, {val_i64}"));
            self.emitln(&format!("  store i64 {add_tmp}, i64* %hash"));
        }
        let final_hash = self.fresh_tmp();
        self.emitln(&format!("  {final_hash} = load i64, i64* %hash"));
        self.emitln(&format!("  ret i64 {final_hash}"));
        self.emitln("}\n");
        self.types.functions.insert(fn_name, (vec![struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    fn compile_ord_impl(&mut self, type_name: &str, struct_ty: &str, field_names: &[String], _fields: &[FieldDecl]) -> Result<(), String> {
        let fn_name = format!("{type_name}.compare");
        if self.mono.emitted_fns.contains(&fn_name) {
            return Ok(());
        }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
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
                // Integer/pointer field comparison. Coerce both operands to i64
                // (a Str field is i8* ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ ptrtoint; a Char field is i8 ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ zext) so the
                // icmp is well-typed. Note: comparing Str by pointer identity is a
                // derive limitation, but it is at least valid IR.
                let cmp_ty = "i64";
                let self_i = self.val_to_i64(&self_val, &field_llvm_ty);
                let other_i = self.val_to_i64(&other_val, &field_llvm_ty);
                self.emitln(&format!("  {cmp_eq} = icmp eq {cmp_ty} {self_i}, {other_i}"));
                self.emitln(&format!("  br i1 {cmp_eq}, label %{next_field}, label %{ret_block}"));
                self.emitln(&format!("\n{ret_block}:"));
                let cmp_lt = self.fresh_tmp();
                self.emitln(&format!("  {cmp_lt} = icmp slt {cmp_ty} {self_i}, {other_i}"));
                let result = self.fresh_tmp();
                self.emitln(&format!("  {result} = select i1 {cmp_lt}, i64 -1, i64 1"));
                self.emitln(&format!("  ret i64 {result}"));
            }
            self.emitln(&format!("\n{next_field}:"));
        }
        self.emitln("  ret i64 0");
        self.emitln("}\n");
        self.types.functions.insert(fn_name, (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));
        Ok(())
    }

    // ========================================================================
    // 5e.7d: Enum-aware derive implementations (deep compare payloads)
    // ========================================================================

    /// Emit derive[Eq] for enums: compares discriminant + variant-specific payload fields.
    fn compile_enum_eq_impl(&mut self, type_name: &str, struct_ty: &str, ed: &xiom_ast::EnumDecl) -> Result<(), String> {
        let fn_name = format!("{type_name}.eq");
        if self.mono.emitted_fns.contains(&fn_name) { return Ok(()); }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));

        // Pre-build variant comparison blocks
        struct VariantEq { block: String, body: Vec<String> }
        let mut variants: Vec<VariantEq> = Vec::new();
        for (vi, variant) in ed.variants.iter().enumerate() {
            let blk = self.fresh_block(&format!("eq_v{vi}"));
            let mut body = Vec::new();
            if variant.fields.is_empty() {
                body.push("  ret i64 1".to_string());
            } else {
                let mut fidx: usize = 1;
                for f in &variant.fields {
                    let fty_name = Self::type_from_ast(&f.ty);
                    // Use the ACTUAL struct field LLVM type (floats stored as i64 in structs)
                    let actual_llvm = self.field_llvm_type(type_name, fidx);
                    let sv = self.fresh_tmp();
                    let ov = self.fresh_tmp();
                    body.push(format!("  {sv} = extractvalue {struct_ty} %self, {fidx}"));
                    body.push(format!("  {ov} = extractvalue {struct_ty} %other, {fidx}"));

                    // Determine how to compare based on the logical type, not the struct storage
                    if fty_name == "Str" || fty_name == "xiom.Str" {
                        // Str stored as i64 handle in struct ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â inttoptr before strcmp
                        let sp = self.fresh_tmp();
                        let op = self.fresh_tmp();
                        body.push(format!("  {sp} = inttoptr i64 {sv} to i8*"));
                        body.push(format!("  {op} = inttoptr i64 {ov} to i8*"));
                        let cr = self.fresh_tmp();
                        let eq = self.fresh_tmp();
                        let ze = self.fresh_tmp();
                        body.push(format!("  {cr} = call i32 @strcmp(i8* {sp}, i8* {op})"));
                        body.push(format!("  {eq} = icmp eq i32 {cr}, 0"));
                        body.push(format!("  {ze} = zext i1 {eq} to i64"));
                        body.push(format!("  ret i64 {ze}"));
                    } else if fty_name.starts_with("Vec[") || fty_name == "Vec" {
                        // 7d: Vec payload ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â delegate to Vec.eq() for deep comparison
                        let vp = self.fresh_tmp();
                        body.push(format!("  {vp} = inttoptr i64 {sv} to %struct.Vec*"));
                        let vl = self.fresh_tmp();
                        body.push(format!("  {vl} = load %struct.Vec, %struct.Vec* {vp}"));
                        let op = self.fresh_tmp();
                        body.push(format!("  {op} = inttoptr i64 {ov} to %struct.Vec*"));
                        let ol = self.fresh_tmp();
                        body.push(format!("  {ol} = load %struct.Vec, %struct.Vec* {op}"));
                        let r = self.fresh_tmp();
                        body.push(format!("  {r} = call i64 @Vec.eq(%struct.Vec {vl}, %struct.Vec {ol})"));
                        body.push(format!("  ret i64 {r}"));
                    } else if fty_name.starts_with("Option[") || fty_name == "Option" {
                        // Option payload ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â delegate to Option.eq()
                        let vp = self.fresh_tmp();
                        body.push(format!("  {vp} = inttoptr i64 {sv} to %struct.Option*"));
                        let vl = self.fresh_tmp();
                        body.push(format!("  {vl} = load %struct.Option, %struct.Option* {vp}"));
                        let op = self.fresh_tmp();
                        body.push(format!("  {op} = inttoptr i64 {ov} to %struct.Option*"));
                        let ol = self.fresh_tmp();
                        body.push(format!("  {ol} = load %struct.Option, %struct.Option* {op}"));
                        let r = self.fresh_tmp();
                        body.push(format!("  {r} = call i64 @Option.eq(%struct.Option {vl}, %struct.Option {ol})"));
                        body.push(format!("  ret i64 {r}"));
                    } else if fty_name.starts_with("Result[") || fty_name == "Result" {
                        let vp = self.fresh_tmp();
                        body.push(format!("  {vp} = inttoptr i64 {sv} to %struct.Result*"));
                        let vl = self.fresh_tmp();
                        body.push(format!("  {vl} = load %struct.Result, %struct.Result* {vp}"));
                        let op = self.fresh_tmp();
                        body.push(format!("  {op} = inttoptr i64 {ov} to %struct.Result*"));
                        let ol = self.fresh_tmp();
                        body.push(format!("  {ol} = load %struct.Result, %struct.Result* {op}"));
                        let r = self.fresh_tmp();
                        body.push(format!("  {r} = call i64 @Result.eq(%struct.Result {vl}, %struct.Result {ol})"));
                        body.push(format!("  ret i64 {r}"));
                    } else if actual_llvm == "i8*" {
                        // Raw i8* pointer field ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â compare via strcmp
                        let cr = self.fresh_tmp();
                        let eq = self.fresh_tmp();
                        let ze = self.fresh_tmp();
                        body.push(format!("  {cr} = call i32 @strcmp(i8* {sv}, i8* {ov})"));
                        body.push(format!("  {eq} = icmp eq i32 {cr}, 0"));
                        body.push(format!("  {ze} = zext i1 {eq} to i64"));
                        body.push(format!("  ret i64 {ze}"));
                    } else if fty_name == "Float64" || fty_name == "Float32" {
                        // Float stored as i64 in struct ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â bitcast before fcmp
                        let sbc = self.fresh_tmp();
                        let obc = self.fresh_tmp();
                        body.push(format!("  {sbc} = bitcast i64 {sv} to double"));
                        body.push(format!("  {obc} = bitcast i64 {ov} to double"));
                        let cmp = self.fresh_tmp();
                        let ze = self.fresh_tmp();
                        body.push(format!("  {cmp} = fcmp oeq double {sbc}, {obc}"));
                        body.push(format!("  {ze} = zext i1 {cmp} to i64"));
                        body.push(format!("  ret i64 {ze}"));
                    } else if actual_llvm == "double" || actual_llvm == "float" {
                        let cmp = self.fresh_tmp();
                        let ze = self.fresh_tmp();
                        body.push(format!("  {cmp} = fcmp oeq {actual_llvm} {sv}, {ov}"));
                        body.push(format!("  {ze} = zext i1 {cmp} to i64"));
                        body.push(format!("  ret i64 {ze}"));
                    } else if actual_llvm.starts_with("%struct.") {
                        let inner = &actual_llvm[8..];
                        let r = self.fresh_tmp();
                        body.push(format!("  {r} = call i64 @{inner}.eq({actual_llvm} {sv}, {actual_llvm} {ov})"));
                        body.push(format!("  ret i64 {r}"));
                    } else {
                        // i64/Int/Bool
                        let cmp = self.fresh_tmp();
                        let ze = self.fresh_tmp();
                        body.push(format!("  {cmp} = icmp eq {actual_llvm} {sv}, {ov}"));
                        body.push(format!("  {ze} = zext i1 {cmp} to i64"));
                        body.push(format!("  ret i64 {ze}"));
                    }
                    fidx += 1;
                }
            }
            variants.push(VariantEq { block: blk, body });
        }
        let default_blk = self.fresh_block("eq_default");

        // Emit function
        self.emitln(&format!("define i64 @{fn_name}({struct_ty} %self, {struct_ty} %other) {{"));
        self.emitln("entry:");
        let d1 = self.fresh_tmp();
        let d2 = self.fresh_tmp();
        self.emitln(&format!("  {d1} = extractvalue {struct_ty} %self, 0"));
        self.emitln(&format!("  {d2} = extractvalue {struct_ty} %other, 0"));
        let dc = self.fresh_tmp();
        self.emitln(&format!("  {dc} = icmp eq i64 {d1}, {d2}"));
        let ret_false = self.fresh_block("eq_false");
        let switch_blk = self.fresh_block("eq_switch");
        self.emitln(&format!("  br i1 {dc}, label %{switch_blk}, label %{ret_false}"));
        // Switch block with switch instruction
        self.emitln(&format!("\n{switch_blk}:"));
        let mut case_strs = Vec::new();
        for (vi, v) in variants.iter().enumerate() {
            case_strs.push(format!("i64 {vi}, label %{}", v.block));
        }
        self.emitln(&format!("  switch i64 {d1}, label %{default_blk} [ {} ]", case_strs.join(" ")));
        // Variant blocks
        for v in &variants {
            self.emitln(&format!("\n{}:", v.block));
            for line in &v.body {
                self.emitln(line);
            }
        }
        // Default + false
        self.emitln(&format!("\n{default_blk}:"));
        self.emitln("  ret i64 0");
        self.emitln(&format!("\n{ret_false}:"));
        self.emitln("  ret i64 0");
        self.emitln("}\n");
        Ok(())
    }

    /// Emit derive[Hash] for enums: hashes discriminant + variant-specific payload fields.
    /// 5e.7d: For heap types (Str, Vec, Option, Result, structs), delegates to their
    /// content-based .hash() methods instead of hashing raw pointer values.
    fn compile_enum_hash_impl(&mut self, type_name: &str, struct_ty: &str, ed: &xiom_ast::EnumDecl) -> Result<(), String> {
        let fn_name = format!("{type_name}.hash");
        if self.mono.emitted_fns.contains(&fn_name) { return Ok(()); }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], "i64".to_string()));

        // Build variant type info for heap-aware hashing
        let mut variant_field_types: Vec<Vec<(String, String)>> = Vec::new();
        for variant in &ed.variants {
            let ftypes: Vec<(String, String)> = variant.fields.iter()
                .map(|f| (f.name.name.clone(), Self::type_from_ast(&f.ty)))
                .collect();
            variant_field_types.push(ftypes);
        }

        self.emitln(&format!("define i64 @{fn_name}({struct_ty} %self) {{"));
        self.emitln("entry:");
        let d = self.fresh_tmp();
        self.emitln(&format!("  {d} = extractvalue {struct_ty} %self, 0"));
        let mut h = self.fresh_tmp();
        self.emitln(&format!("  {h} = mul i64 {d}, 31"));
        // Hash payload slots (indices 1..N) using content-aware hashing
        let field_count = self.types.types.get(&type_name.to_string()).map(|f| f.len()).unwrap_or(1);
        for fi in 1..field_count {
            let fv = self.fresh_tmp();
            self.emitln(&format!("  {fv} = extractvalue {struct_ty} %self, {fi}"));
            // Check if any variant has a heap type at this field index
            let ftype_name = variant_field_types.iter()
                .filter_map(|v| v.get(fi - 1))
                .map(|(_, tn)| tn.as_str())
                .next();
            let hash_val = match ftype_name {
                Some("Str") | Some("xiom.Str") => {
                    // Hash string content: djb2 over bytes (simple inline, no call needed)
                    let ptr = self.fresh_tmp();
                    let null_check = self.fresh_tmp();
                    self.emitln(&format!("  {ptr} = inttoptr i64 {fv} to i8*"));
                    self.emitln(&format!("  {null_check} = icmp eq i8* {ptr}, null"));
                    let hash_blk = self.fresh_block("str_hash");
                    let skip_blk = self.fresh_block("str_hash_skip");
                    self.emitln(&format!("  br i1 {null_check}, label %{skip_blk}, label %{hash_blk}"));
                    self.emitln(&format!("\n{hash_blk}:"));
                    let sh = self.fresh_tmp();
                    self.emitln(&format!("  {sh} = call i64 @xiom_str_hash(i8* {ptr})"));
                    self.emitln(&format!("  br label %{skip_blk}"));
                    self.emitln(&format!("\n{skip_blk}:"));
                    let phi = self.fresh_tmp();
                    self.emitln(&format!("  {phi} = phi i64 [ {sh}, %{hash_blk} ], [ 0, %entry ]"));
                    phi
                }
                Some(n) if n.starts_with("Vec") || n.starts_with("Vec[") || n == "Vec" => {
                    // Delegate to Vec.hash()
                    let vp = self.fresh_tmp();
                    self.emitln(&format!("  {vp} = inttoptr i64 {fv} to %struct.Vec*"));
                    let vl = self.fresh_tmp();
                    self.emitln(&format!("  {vl} = load %struct.Vec, %struct.Vec* {vp}"));
                    let vh = self.fresh_tmp();
                    self.emitln(&format!("  {vh} = call i64 @Vec.hash(%struct.Vec {vl})"));
                    vh
                }
                _ => {
                    // For i64 values and other types, use raw value with multiplier
                    let add = self.fresh_tmp();
                    self.emitln(&format!("  {add} = mul i64 {fv}, 33"));
                    add
                }
            };
            let add = self.fresh_tmp();
            self.emitln(&format!("  {add} = add i64 {h}, {hash_val}"));
            h = add;
        }
        self.emitln(&format!("  ret i64 {h}"));
        self.emitln("}\n");
        Ok(())
    }

    /// Emit derive[Ord] for enums: compares discriminant, then payload fields lexicographically.
    /// 5e.7d: Now compares payload values when discriminants match, not just discriminants.
    fn compile_enum_ord_impl(&mut self, type_name: &str, struct_ty: &str, ed: &xiom_ast::EnumDecl) -> Result<(), String> {
        let fn_name = format!("{type_name}.compare");
        if self.mono.emitted_fns.contains(&fn_name) { return Ok(()); }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), struct_ty.to_string()], "i64".to_string()));

        // Build variant payload field indices for same-discriminant comparison
        struct VariantOrd { field_indices: Vec<usize>, block: String }
        let mut variants: Vec<VariantOrd> = Vec::new();
        for (vi, variant) in ed.variants.iter().enumerate() {
            let blk = self.fresh_block(&format!("ord_v{vi}"));
            let mut indices = Vec::new();
            let mut fidx: usize = 1;
            for _ in &variant.fields {
                indices.push(fidx);
                fidx += 1;
            }
            variants.push(VariantOrd { field_indices: indices, block: blk });
        }
        let default_blk = self.fresh_block("ord_default");

        self.emitln(&format!("define i64 @{fn_name}({struct_ty} %self, {struct_ty} %other) {{"));
        self.emitln("entry:");
        let d1 = self.fresh_tmp();
        let d2 = self.fresh_tmp();
        self.emitln(&format!("  {d1} = extractvalue {struct_ty} %self, 0"));
        self.emitln(&format!("  {d2} = extractvalue {struct_ty} %other, 0"));
        // First compare discriminants
        let lt = self.fresh_tmp();
        let gt = self.fresh_tmp();
        let eq = self.fresh_tmp();
        self.emitln(&format!("  {lt} = icmp slt i64 {d1}, {d2}"));
        self.emitln(&format!("  {gt} = icmp sgt i64 {d1}, {d2}"));
        self.emitln(&format!("  {eq} = icmp eq i64 {d1}, {d2}"));
        // If discriminants differ, return -1/1; if equal, switch to variant payload comparison
        let ret_simple = self.fresh_block("ord_discrim");
        let switch_blk = self.fresh_block("ord_switch");
        self.emitln(&format!("  br i1 {eq}, label %{switch_blk}, label %{ret_simple}"));
        // Simple discriminant-only result
        self.emitln(&format!("\n{ret_simple}:"));
        let r1 = self.fresh_tmp();
        let r2 = self.fresh_tmp();
        self.emitln(&format!("  {r1} = select i1 {lt}, i64 -1, i64 0"));
        self.emitln(&format!("  {r2} = select i1 {gt}, i64 1, i64 {r1}"));
        self.emitln(&format!("  ret i64 {r2}"));
        // Switch to per-variant payload comparison
        self.emitln(&format!("\n{switch_blk}:"));
        let mut case_strs = Vec::new();
        for (vi, v) in variants.iter().enumerate() {
            case_strs.push(format!("i64 {vi}, label %{}", v.block));
        }
        self.emitln(&format!("  switch i64 {d1}, label %{default_blk} [ {} ]", case_strs.join(" ")));
        // Emit per-variant comparison blocks
        for v in &variants {
            self.emitln(&format!("\n{}:", v.block));
            if v.field_indices.is_empty() {
                self.emitln("  ret i64 0"); // same variant, no payload ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ equal
            } else {
                for &fi in &v.field_indices {
                    let sv = self.fresh_tmp();
                    let ov = self.fresh_tmp();
                    self.emitln(&format!("  {sv} = extractvalue {struct_ty} %self, {fi}"));
                    self.emitln(&format!("  {ov} = extractvalue {struct_ty} %other, {fi}"));
                    let cmp = self.fresh_tmp();
                    let lt_chk = self.fresh_tmp();
                    let next_blk = self.fresh_block("ord_next");
                    self.emitln(&format!("  {cmp} = icmp eq i64 {sv}, {ov}"));
                    self.emitln(&format!("  {lt_chk} = icmp slt i64 {sv}, {ov}"));
                    let r = self.fresh_tmp();
                    self.emitln(&format!("  {r} = select i1 {lt_chk}, i64 -1, i64 1"));
                    self.emitln(&format!("  br i1 {cmp}, label %{next_blk}, label %{default_blk}_return"));
                    self.emitln(&format!("\n{next_blk}:"));
                }
                self.emitln("  ret i64 0"); // all payload fields equal
            }
        }
        // Return block for payload comparison result
        self.emitln(&format!("\n{default_blk}_return:"));
        self.emitln(&format!("  ret i64 {r}", r = if variants.iter().any(|v| !v.field_indices.is_empty()) { "r" } else { "0" }));
        self.emitln(&format!("\n{default_blk}:"));
        self.emitln("  ret i64 0");
        self.emitln("}\n");
        Ok(())
    }

    /// Emit derive[Display] for enums: shows variant name + payload values.
    /// 5e.7d: Now includes payload values in display output for single-field variants.
    fn compile_enum_display_impl(&mut self, type_name: &str, struct_ty: &str, ed: &xiom_ast::EnumDecl) -> Result<(), String> {
        let fn_name = format!("{type_name}.to_str");
        if self.mono.emitted_fns.contains(&fn_name) { return Ok(()); }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string()], "i8*".to_string()));

        // Emit string constants for each variant display string
        for variant in &ed.variants {
            let vname = &variant.name.name;
            let display_str = if variant.fields.len() == 1 {
                format!("{}(%lld)", vname)
            } else {
                vname.clone()
            };
            self.emitln(&format!("@.str.{type_name}_{vname} = private constant [{} x i8] c\"{display_str}\\00\"", display_str.len() + 1));
        }
        // Format buffer for sprintf
        self.emitln(&format!("@.fmt_buf_{type_name} = private global [256 x i8] zeroinitializer"));

        let default_blk = self.fresh_block("disp_default");
        let mut case_strs = Vec::new();
        let mut variant_blocks: Vec<(String, Vec<String>)> = Vec::new();
        for (vi, variant) in ed.variants.iter().enumerate() {
            let vblk = self.fresh_block(&format!("disp_v{vi}"));
            let vname = &variant.name.name;
            let mut body = Vec::new();
            if variant.fields.len() == 1 {
                // Show variant(value) via sprintf
                let fmt_ptr = self.fresh_tmp();
                body.push(format!("  {fmt_ptr} = getelementptr inbounds [{} x i8], [{} x i8]* @.str.{type_name}_{vname}, i32 0, i32 0",
                    vname.len() + 7, vname.len() + 7)); // +7 for "(%lld)\0"
                let buf_ptr = self.fresh_tmp();
                body.push(format!("  {buf_ptr} = getelementptr inbounds [256 x i8], [256 x i8]* @.fmt_buf_{type_name}, i32 0, i32 0"));
                let pv = self.fresh_tmp();
                body.push(format!("  {pv} = extractvalue {struct_ty} %self, 1"));
                body.push(format!("  call i32 (i8*, ...) @sprintf(i8* {buf_ptr}, i8* {fmt_ptr}, i64 {pv})"));
                body.push(format!("  ret i8* {buf_ptr}"));
            } else {
                let ptr = self.fresh_tmp();
                body.push(format!("  {ptr} = getelementptr inbounds [{} x i8], [{} x i8]* @.str.{type_name}_{vname}, i32 0, i32 0",
                    vname.len() + 1, vname.len() + 1));
                body.push(format!("  ret i8* {ptr}"));
            }
            case_strs.push(format!("i64 {vi}, label %{vblk}"));
            variant_blocks.push((vblk, body));
        }

        self.emitln(&format!("define i8* @{fn_name}({struct_ty} %self) {{"));
        self.emitln("entry:");
        let d = self.fresh_tmp();
        self.emitln(&format!("  {d} = extractvalue {struct_ty} %self, 0"));
        self.emitln(&format!("  switch i64 {d}, label %{default_blk} [ {} ]", case_strs.join(" ")));
        for (blk, body) in &variant_blocks {
            self.emitln(&format!("\n{blk}:"));
            for line in body {
                self.emitln(line);
            }
        }
        self.emitln(&format!("\n{default_blk}:"));
        self.emitln("  ret i8* null");
        self.emitln("}\n");
        Ok(())
    }

    /// 8B/M9: Debug derive for enums ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â delegates to Display.to_str
    fn compile_enum_debug_impl(&mut self, type_name: &str, struct_ty: &str, _ed: &xiom_ast::EnumDecl) -> Result<(), String> {
        let fn_name = format!("{type_name}.fmt");
        if self.mono.emitted_fns.contains(&fn_name) { return Ok(()); }
        self.mono.emitted_fns.insert(fn_name.clone());
        self.types.functions.insert(fn_name.clone(), (vec![struct_ty.to_string(), "i8*".to_string()], "i8*".to_string()));
        self.emitln(&format!("define i8* @{fn_name}({struct_ty} %self, i8* %_f) {{"));
        self.emitln("entry:");
        let ptr = self.fresh_tmp();
        self.emitln(&format!("  {ptr} = call i8* @{type_name}.to_str({struct_ty} %self)"));
        self.emitln(&format!("  ret i8* {ptr}"));
        self.emitln("}\n");
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
        const MAX_GENERIC_ITERATIONS: u32 = 65536;        loop {
            iteration += 1;
            if iteration > MAX_GENERIC_ITERATIONS {
                return Err(format!(
                    "generic monomorphisation exceeded {} iterations ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â possible infinite recursion in generic definitions",
                    MAX_GENERIC_ITERATIONS
                ));
            }
            let instantiations = std::mem::take(&mut self.mono.generic_instantiations);
            if instantiations.is_empty() {
                break;
            }
            for (base_name, concrete_types) in &instantiations {
            // Find the generic function decl
            let fd = match self.mono.generic_fn_decls.iter().find(|(k, _)| k == base_name) {
                Some((_, f)) => f.clone(),
                None => {
                    continue;
                }
            };
            // Emit each unique specialization at most once. Without this, a
            // generic that (transitively) instantiates itself re-queues the same
            // specialization on every worklist pass, never draining the queue ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â
            // producing the "exceeded 65536 iterations" error (or a hang).
            // `insert` returns false when the key is already present.
            let mono_key = self.monomorphised_fn_name(base_name, concrete_types);
            if !self.mono.mono_emitted.insert(mono_key) {
                continue;
            }
            // Check interface bounds for each generic parameter
            for (gp, concrete_type) in fd.generics.iter().zip(concrete_types.iter()) {
                for bound in &gp.bounds {
                    if let Some(methods) = self.types.interfaces.get(&bound.name) {
                        for (method_name, _) in methods {
                            let method_key = format!("{}.{}", concrete_type, method_name);
                            // Primitive types implicitly implement the builtin interface
                            // methods (Ord.compare, Eq.eq/ne, Hash.hash, Clone.clone,
                            // comparison ops) via inline codegen ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â see the primitive
                            // fast-path in compile_expr's method dispatch.
                            let is_builtin_method = matches!(
                                method_name.as_str(),
                                "compare" | "eq" | "ne" | "lt" | "gt" | "le" | "ge" | "hash" | "clone"
                            );
                            if is_builtin_method && Self::is_primitive_type_name(concrete_type) {
                                continue;
                            }
                            if !self.types.functions.contains_key(&method_key) {
                                // 3c: impls inside catalog-loaded modules register
                                // MODULE-QUALIFIED (`core.Float64.add`) and are
                                // populated LAZILY on first call. Accept a method
                                // when (a) a module-qualified impl key exists, or
                                // (b) the impl fn exists in the injected program
                                // (generic_fn_decls / pending FnDecls) and will
                                // register when monomorphised.
                                let qualified = self.types.functions.keys()
                                    .iter().any(|k| k.ends_with(&format!(".{method_key}")) || k == &method_key);
                                let pending = self.mono.generic_fn_decls.iter()
                                    .any(|(k, _)| k == &method_key || k.ends_with(&format!(".{method_key}")));
                                if !qualified && !pending {
                                    return Err(format!(
                                        "type '{}' does not implement '{}': missing method '{}'",
                                        concrete_type, bound.name, method_name
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            let specialized_name = self.monomorphised_fn_name(base_name, concrete_types);
            // Build type substitution map: generic param name -> concrete type name
            let mut type_map: HashMap<String, String> = HashMap::new();
            let const_map: HashMap<String, i64> = self.mono.const_value_map.get(&specialized_name).cloned().unwrap_or_default();
            for (gp, ct) in fd.generics.iter().zip(concrete_types.iter()) {
                if gp.is_const { continue; } // const params use const_map, not type_map
                type_map.insert(gp.name.name.clone(), ct.clone());
            }
            // Interface-typed params: map interface names to concrete types.
            // For `fn f(r: Reporter)` called with `GoodReporter`, map
            // "Reporter" ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ "GoodReporter" so subst_type can resolve params.
            if fd.generics.is_empty() && !concrete_types.is_empty() {
                let mut ct_idx = 0;
                for param in &fd.params {
                    let param_name = Self::type_from_ast(&param.ty);
                    if self.types.interfaces.contains_key(&param_name) {
                        if ct_idx < concrete_types.len() {
                            type_map.insert(param_name.clone(), concrete_types[ct_idx].clone());
                            ct_idx += 1;
                        }
                    }
                }
            }
            // Register concrete tuple types for this monomorphisation
            if let Some(ref ret_ty) = fd.return_type {
                self.ensure_concrete_tuple_type_registered(ret_ty, &type_map);
            }
            for p in &fd.params {
                self.ensure_concrete_tuple_type_registered(&p.ty, &type_map);
            }
            // Register the specialized function signature
            let struct_types: HashSet<String> = self.types.types.keys().into_iter().collect();
            let subst_type = |t: &Type| -> String {
                match t {
                    // Result[T, E] / Option[T] must resolve to the CONCRETE
                    // monomorphised struct (e.g. %struct.Result__Entity__Str)
                    // when the inner types are user structs; otherwise the
                    // generic %struct.Result / %struct.Option layout.
                    Type::Result(ok, err) => {
                        let ok_name = Self::type_from_ast(&Self::substitute_type(ok, ok, &type_map));
                        let err_name = Self::type_from_ast(&Self::substitute_type(err, err, &type_map));
                        let concrete = format!("Result__{}__{}", ok_name, err_name);
                        let key_matches = struct_types.iter()
                            .any(|k| k.as_str() == concrete || k.ends_with(&format!(".{concrete}")));
                        if key_matches {
                            let qualified = struct_types.iter()
                                .find(|k| k.as_str() == concrete || k.ends_with(&format!(".{concrete}")))
                                .cloned().unwrap_or(concrete);
                            format!("%struct.{qualified}")
                        } else {
                            "%struct.Result".to_string()
                        }
                    }
                    Type::Option(inner) => {
                        let inner_name = Self::type_from_ast(&Self::substitute_type(inner, inner, &type_map));
                        let concrete = format!("Option__{}", inner_name);
                        let key_matches = struct_types.iter()
                            .any(|k| k.as_str() == concrete || k.ends_with(&format!(".{concrete}")));
                        if key_matches {
                            let qualified = struct_types.iter()
                                .find(|k| k.as_str() == concrete || k.ends_with(&format!(".{concrete}")))
                                .cloned().unwrap_or(concrete);
                            format!("%struct.{qualified}")
                        } else {
                            "%struct.Option".to_string()
                        }
                    }
                    Type::Named(id, args) => {
                        // Parameterized generic types like Result[T, E] or
                        // Option[Entity] must resolve to the CONCRETE struct
                        // (e.g. %struct.Result__Entity__Str), not the generic
                        // %struct.Result ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â callers pass the concrete layout.
                        if !args.is_empty() {
                            // Resolve each type arg to its XIOM type NAME (e.g. "Str",
                            // "Entity") ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â the concrete struct name joins these with
                            // "__" (Result__Entity__Str), NOT the LLVM type (i8*).
                            let resolved: Vec<String> = args.iter()
                                .map(|a| {
                                    let subst_a = Self::substitute_type(a, a, &type_map);
                                    let name = Self::type_from_ast(&subst_a);
                                    let clean = name.trim_start_matches('*').to_string();
                                    clean
                                })
                                .collect();
                                // Build the concrete monomorphised struct name, matching
                                // the naming used by register_mapped_type / concretisation
                                // (e.g. Result__Entity__Str).
                                let base = id.name.clone();
                                if !resolved.is_empty() {
                                    let concrete_name = format!("{base}__{}", resolved.join("__"));
                                    let key_matches = struct_types.iter()
                                        .any(|k| k.as_str() == concrete_name || k.ends_with(&format!(".{concrete_name}")));
                                    if key_matches {
                                        let qualified = struct_types.iter()
                                            .find(|k| k.as_str() == concrete_name || k.ends_with(&format!(".{concrete_name}")))
                                            .cloned().unwrap_or(concrete_name);
                                        return format!("%struct.{qualified}");
                                    }
                                }
                            }
                        let raw = if let Some(ct) = type_map.get(&id.name) {
                            ct.clone()
                        } else {
                            id.name.clone()
                        };
                        // 5c.35: Check both bare and module-qualified struct names.
                        // `struct_types` may contain "module.Type" while `raw` is "Type".
                        let is_struct = struct_types.contains(&raw)
                            || struct_types.iter().any(|k| k.ends_with(&format!(".{}", raw)));
                        if is_struct {
                            let qualified = struct_types.iter()
                                .find(|k| **k == raw || k.ends_with(&format!(".{}", raw)))
                                .cloned()
                                .unwrap_or(raw);
                            format!("%struct.{qualified}")
                        } else {
                            Self::xiom_to_llvm_type(&raw).to_string()
                        }
                    }
                    // Pointer / mutable-scalar-ref types: substitute the inner
                    // generic, then lower to a REAL pointer (e.g. `mem.swap[Int]`'s
                    // `&mut T` -> `i64*`). Uses the captured `struct_types` set for
                    // struct detection so the closure stays `self`-free. Without this,
                    // such params defaulted to `i64` and `from_mut(x)` fed an `i64*`
                    // address into an `i64` slot ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ miscompiled swap.
                    Type::Ptr(inner) | Type::MutRef(inner) => {
                        let subst = Self::substitute_type(t, inner, &type_map);
                        let name = Self::type_from_ast(&subst);
                        if let Some(inner_name) = name.strip_prefix('*') {
                            let inner_llvm = if struct_types.contains(inner_name) {
                                format!("%struct.{inner_name}")
                            } else {
                                Self::xiom_to_llvm_type(inner_name).to_string()
                            };
                            if inner_llvm == "void" { "i8*".to_string() } else { format!("{inner_llvm}*") }
                        } else {
                            if struct_types.contains(&name) {
                                format!("%struct.{name}*")
                            } else {
                                format!("{}*", Self::xiom_to_llvm_type(&name))
                            }
                        }
                    }
                    Type::Ref(inner) => {
                        // Unwrap Ref to reach Array/Slice handlers directly.
                        let inner_subst = Self::substitute_type(inner, inner, &type_map);
                        // Arrays/Slices: produce proper pointer types with const-size resolution.
let inner_llvm = match &inner_subst {
                            Type::Array(size_expr, elem) => {
                                let subst_elem = Self::substitute_type(t, elem, &type_map);
                                let elem_name = Self::type_from_ast(&subst_elem);
                                let elem_ty = if struct_types.contains(&elem_name) {
                                    format!("%struct.{elem_name}")
                                } else {
                                    Self::xiom_to_llvm_type(&elem_name).to_string()
                                };
                                let _size_val: u64 = match size_expr.as_ref() {
                                    Expr::Int(n, _) => *n as u64,
                                    Expr::Ident(id) => const_map.get(&id.name).copied().unwrap_or(0) as u64,
                                    _ => 0,
                                };
                                // For ref params, produce a plain pointer (i64*) instead
                                // of typed array pointer ([5 x i64]*) for ABI compat.
                                // The const N is used in the body via const_map.
                                format!("{elem_ty}*")
                            }
                            Type::Slice(elem) => {
                                // &Slice[T] must produce %struct.Vec (not just a
                                // data pointer) so the function body can access
                                // .len() and [i] on the Slice parameter. Without
                                // this, is_sorted/contains receive i64* and read
                                // the first element as the length, producing
                                // incorrect results (stdlib_exec_core_runs).
                                let _subst_elem = Self::substitute_type(t, elem, &type_map);
                                "%struct.Vec".to_string()
                            }
                            _ => {
                                let name = Self::type_from_ast(&inner_subst);
                                if let Some(inner_name) = name.strip_prefix('*') {
                                    let base = if struct_types.contains(inner_name) {
                                        format!("%struct.{inner_name}")
                                    } else {
                                        Self::xiom_to_llvm_type(inner_name).to_string()
                                    };
                                    if base == "void" { "i8*".to_string() } else { format!("{base}*") }
                                } else if struct_types.contains(&name) {
                                    format!("%struct.{name}")
                                } else {
                                    Self::xiom_to_llvm_type(&name).to_string()
                                }
                            }
                        };
                        // Arrays from Ref/Ptr/MutRef always become pointers.
                        if inner_llvm.starts_with('[') { format!("{inner_llvm}*") }
                        else { inner_llvm }
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
                        format!("%struct.Tuple__{}", parts.join("__"))
                    }
                    Type::Array(size_expr, elem) => {
                        // Substitute generic params in element type; resolve const size.
                        let subst_elem = Self::substitute_type(t, elem, &type_map);
                        let elem_name = Self::type_from_ast(&subst_elem);
                        let elem_llvm = if struct_types.contains(&elem_name) {
                            format!("%struct.{elem_name}")
                        } else {
                            Self::xiom_to_llvm_type(&elem_name).to_string()
                        };
                        let size_val: u64 = match size_expr.as_ref() {
                            Expr::Int(n, _) => *n as u64,
                            Expr::Ident(id) => const_map.get(&id.name).copied().unwrap_or(0) as u64,
                            _ => 0,
                        };
                        if size_val == 0 { elem_llvm } else { format!("[{size_val} x {elem_llvm}]") }
                    }
                    _ => {
                        let base_name = Self::type_from_ast(t);
                        if struct_types.contains(&base_name) {
                            format!("%struct.{base_name}")
                        } else {
                            Self::xiom_to_llvm_type(&base_name).to_string()
                        }
                    },
                }
            };
            let specialized_ret_type = fd.return_type.as_ref()
                .map(|t| subst_type(t))
                .unwrap_or_else(|| "void".to_string());
            let mut specialized_param_types: Vec<String> = Vec::new();
            // Include self/receiver parameter for methods. A receiver-qualified fn
            // with NO `self` param is a static constructor ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â no receiver argument.
            let has_self_param = fd.params.iter().any(|p| p.name.name == "self");
            let is_mut_self = fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self);
            // 5c.34: Detect by-value self usage in generic method bodies
            // (same fix as compile_fn for non-generic methods).
            let body_uses_self = !has_self_param
                && fd.receiver.is_some()
                && fd.body.as_ref().map_or(false, |b| IrEmitter::block_uses_self_ident(b));
            // BUG 29 (Map.keys on module globals): THIS-BASED generic method
            // bodies — `fn Map.keys[K, V]() -> Vec[K] { ... keys.len() ... }`
            // references BARE FIELDS (no `self`/`this` ident). The non-generic
            // path binds the receiver for these via body_uses_receiver_state;
            // the generic monomorphisation only handled self-ident bodies, so
            // the emitted fn took NO receiver and bare `keys` resolved to a
            // fn-pointer (@Map.keys) → "use of undefined value '@Map.keys'".
            // Detect the same this-based shape here so the receiver pointer +
            // field locals are bound.
            let is_this_based = !has_self_param
                && !body_uses_self
                && fd.receiver.is_some()
                && self.body_uses_receiver_state(&fd);
            let self_llvm_ty = if let (true, Some(r)) = (has_self_param, fd.receiver.as_ref()) {
                let base = self.llvm_type_for(&r.name).unwrap_or_else(|_| {
                    let search = format!(".{}", r.name);
                    for key in self.types.type_meta.keys() {
                        if key.ends_with(&search) { return format!("%struct.{key}"); }
                    }
                    "i64".to_string()
                });
                Some(if is_mut_self && base.starts_with('%') { format!("{base}*") } else { base })
            } else if body_uses_self || is_this_based {
                // 5c.34: By-value self in generic method — pass pointer so
                // mutations propagate to caller (matches compile_fn behavior).
                // This-based: bind the receiver by pointer (bare field reads
                // GEP through it).
                let base = fd.receiver.as_ref().and_then(|r| {
                    self.llvm_type_for(&r.name).ok().or_else(|| {
                        let search = format!(".{}", r.name);
                        self.types.type_meta.keys().into_iter().find(|k| k.ends_with(&search))
                            .map(|k| format!("%struct.{k}"))
                    })
                }).unwrap_or_else(|| "i64".to_string());
                Some(if base.starts_with('%') { format!("{base}*") } else { base })
            } else {
                None
            };
            if let Some(ref st) = self_llvm_ty {
                specialized_param_types.push(st.clone());
            }
            let explicit_param_types: Vec<String> = fd.params.iter()
                .filter(|p| !(self_llvm_ty.is_some() && p.name.name == "self"))
                .map(|p| subst_type(&p.ty))
                .collect();
            specialized_param_types.extend(explicit_param_types);
            self.types.functions.insert(specialized_name.clone(), (specialized_param_types.clone(), specialized_ret_type.clone()));

            // Emit the specialized function
            self.push_scope();
            self.block_counter = 0;
            self.tmp_counter = 0;

            self.fctx.current_return_type = specialized_ret_type.clone();
            self.fctx.current_param_llvm_types = specialized_param_types.clone();
            self.fctx.current_fn = Some(specialized_name.clone());

            let self_offset: usize = if self_llvm_ty.is_some() { 1 } else { 0 };
            let mut params_str: Vec<String> = Vec::new();
            if let Some(ref st) = self_llvm_ty {
                params_str.push(format!("{st} %param_self"));
            }
            let explicit_params_str: Vec<String> = fd.params.iter()
                .filter(|p| !(self_offset == 1 && p.name.name == "self"))
                .enumerate()
                .map(|(i, p)| {
                    let llvm_ty = subst_type(&p.ty);
                    format!("{llvm_ty} %param{}", i + self_offset)
                })
                .collect();
            params_str.extend(explicit_params_str);

            self.flush_deferred_types();
            self.emitln(&format!("define {specialized_ret_type} @{specialized_name}({}) {{", params_str.join(", ")));
            let entry_block = self.fresh_block("entry");
            self.emitln(&format!("{entry_block}:"));

            // Allocate parameters as locals
            // Allocate self parameter first (for methods)
            if let (Some(st), Some(recv)) = (&self_llvm_ty, &fd.receiver) {
                let is_ptr = st.ends_with('*');
                let self_alloca = self.fresh_tmp();
                self.emitln(&format!("  {self_alloca} = alloca {st}"));
                self.emitln(&format!("  store {st} %param_self, {st}* {self_alloca}"));
                if is_ptr {
                    let loaded_ptr = self.fresh_tmp();
                    let struct_ty = st.trim_end_matches('*');
                    self.emitln(&format!("  {loaded_ptr} = load {st}, {st}* {self_alloca}"));
                    self.add_local("self", loaded_ptr.clone(), struct_ty);
                    // Register fields via GEP on the loaded pointer
                    let recv_type_name = &recv.name;
                    let names_opt = self.types.types.get(&recv_type_name.to_string())
                        .or_else(|| {
                            if let Some(ref module) = self.local.current_module {
                                let qualified = format!("{}.{}", module, recv_type_name);
                                self.types.types.get(&qualified)
                            } else {
                                self.types.types.keys().into_iter().find(|k| k.ends_with(&format!(".{recv_type_name}"))).and_then(|k|self.types.types.get(&k))
                            }
                        });
                    if let Some(names) = names_opt {
                        let type_key = self.types.types.get(&recv_type_name.to_string()).map(|_| recv_type_name.clone())
                            .or_else(|| {
                                if let Some(ref module) = self.local.current_module {
                                    let q = format!("{}.{}", module, recv_type_name);
                                    if self.types.types.contains_key(&q) { Some(q) } else { None }
                                } else { None }
                            })
                            .or_else(|| self.types.types.keys().into_iter().find(|k| k.ends_with(&format!(".{recv_type_name}"))))
                            .unwrap_or_else(|| recv_type_name.clone());
                        for (idx, field_name) in names.iter().enumerate() {
                            let field_llvm_ty = self.field_llvm_type(&type_key, idx);
                            let gep = self.fresh_tmp();
                            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {loaded_ptr}, i32 0, i32 {idx}"));
                            self.add_local(field_name, gep, &field_llvm_ty);
                        }
                    }
                } else {
                    self.add_local("self", self_alloca.clone(), st);
                    // Register each struct field as a local (bare name access like `items`)
                    let recv_type_name = &recv.name;
                    let names_opt = self.types.types.get(&recv_type_name.to_string())
                        .or_else(|| {
                            if let Some(ref module) = self.local.current_module {
                                let qualified = format!("{}.{}", module, recv_type_name);
                                self.types.types.get(&qualified)
                            } else {
                                self.types.types.entries().into_iter().find(|(k, _)| k.ends_with(&format!(".{recv_type_name}"))).map(|(_, v)| v.clone())
                            }
                        });
                    if let Some(names) = names_opt {
                        let type_key = self.types.types.get(&recv_type_name.to_string()).map(|_| recv_type_name.clone())
                            .or_else(|| {
                                if let Some(ref module) = self.local.current_module {
                                    let q = format!("{}.{}", module, recv_type_name);
                                    if self.types.types.contains_key(&q) { Some(q) } else { None }
                                } else { None }
                            })
                            .or_else(|| self.types.types.keys().into_iter().find(|k| k.ends_with(&format!(".{recv_type_name}"))))
                            .unwrap_or_else(|| recv_type_name.clone());
                        for (idx, field_name) in names.iter().enumerate() {
                            let field_llvm_ty = self.field_llvm_type(&type_key, idx);
                            let gep = self.fresh_tmp();
                            self.emitln(&format!("  {gep} = getelementptr {st}, {st}* {self_alloca}, i32 0, i32 {idx}"));
                            self.add_local(field_name, gep, &field_llvm_ty);
                        }
                    }
                }
            }
            let mut emitted_param_idx = self_offset;
            for param in fd.params.iter() {
                // Skip the duplicate `self` param (see the note in compile_fn):
                // the receiver already bound the struct `self`; it is also filtered
                // from the signature, so `match self` uses the real struct receiver.
                if self_offset == 1 && param.name.name == "self" {
                    continue;
                }
                let llvm_ty = subst_type(&param.ty);
                let alloca = self.fresh_tmp();
                let param_idx = emitted_param_idx;
                emitted_param_idx += 1;
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                self.emitln(&format!("  store {llvm_ty} %param{param_idx}, {llvm_ty}* {alloca}"));
                self.add_local(&param.name.name, alloca, &llvm_ty);
                // Track params whose original type is a generic parameter being monomorphised
                let xiom_ty = Self::type_from_ast(&param.ty);
                if type_map.contains_key(&xiom_ty) {
                    if let Some(concrete) = type_map.get(&xiom_ty) {
                        self.mono.param_concrete_types.insert(param.name.name.clone(), concrete.clone());
                    }
                }
                // Mirror compile_fn's ref-param tracking: plain `&T` params carry
                // the ADDRESS as i64 (deref must inttoptr+load), &mut T / *T are
                // real pointers. Without this, eq/compare inside generic bodies
                // (e.g. array.contains's `arr[i].eq(x)`) compares the element
                // against the address instead of the value.
                self.local.param_locals.insert(param.name.name.clone());
                if matches!(&param.ty, Type::Ref(_)) {
                    self.local.ref_params.insert(param.name.name.clone());
                }
            }

            // Set type substitution map for method dispatch in body
            self.mono.current_const_map = const_map.clone();
            self.mono.current_type_map = type_map.clone();
            // D2.1 (Phase 6): honor #[unsafe_no_retry] on generic fns too.
            self.fctx.unsafe_allow_retry = !fd.attributes.iter().any(|a| a.name.name == "unsafe_no_retry");
            // D2.1 (Phase 7): honor #[unsafe_direct] on generic fns too.
            let has_direct = fd.attributes.iter().any(|a| a.name.name == "unsafe_direct");
            let is_stdlib = self.config.source_file.contains("stdlib")
                || self.config.source_file.contains("selfhost");
            self.fctx.unsafe_direct = has_direct && (is_stdlib || self.config.enable_unsafe_direct);

            // P0-2: Clear deferred cleanup stack at function start
            self.clear_deferred_cleanups();

            // Compile body
            if let Some(body) = fd.body.as_ref() {
                self.compile_block(body, fd.return_type.is_some())?;
            }
            // P0-2: Clear deferred cleanup stack after function body is done
            self.clear_deferred_cleanups();

            // Clear type substitution state
            self.mono.current_type_map.clear();
            self.mono.param_concrete_types.clear();
            if fd.return_type.is_none() {
                self.compile_deferred_cleanups()?;
                self.emitln("  ret void");
            } else if !self.current_block_terminated() {
                self.compile_deferred_cleanups()?;
                // A4 fix (generic monomorphisation path): a value-returning body
                // fell through without a terminator (ends in a loop / if-without-
                // else / statement). Append a safe fallback return so the block is
                // terminated. Terminated bodies are unchanged (no double return).
                let zero = Self::default_const_for(&specialized_ret_type);
                self.emitln(&format!("  ret {specialized_ret_type} {zero}"));
            }
                self.emitln("}\n");
                self.pop_scope();
                self.fctx.current_fn = None;
                self.fctx.current_receiver = None;
                // D2.1 (Phase 5): flush deferred unsafe-block functions emitted
                // from THIS generic specialization's body. (compile_fn flushes
                // after each non-generic fn; the monomorphisation path must too,
                // or __unsafe_block_N defs from generic bodies are dropped.)
                self.flush_deferred_closures();
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
        if !self.types.used_builtins.contains("Option") && !self.types.used_builtins.contains("Result") {
            return;
        }
        self.emitln("; Builtin type implementations\n");
        if self.types.used_builtins.contains("Option") {
            self.compile_option_impls();
        }
        if self.types.used_builtins.contains("Result") {
            self.compile_result_impls();
        }
    }

    fn compile_option_impls(&mut self) {
        let opt_ty = "%struct.Option";
        // Base Option methods
        self.emitln(&format!("define i64 @Option.is_some({opt_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {opt_ty}"));
        self.emitln(&format!("  store {opt_ty} %self, {opt_ty}* %val"));
        self.emitln(&format!("  %disc = getelementptr {opt_ty}, {opt_ty}* %val, i32 0, i32 0"));
        self.emitln("  %result = load i64, i64* %disc");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");

        self.emitln(&format!("define i64 @Option.is_none({opt_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {opt_ty}"));
        self.emitln(&format!("  store {opt_ty} %self, {opt_ty}* %val"));
        self.emitln(&format!("  %disc = getelementptr {opt_ty}, {opt_ty}* %val, i32 0, i32 0"));
        self.emitln("  %is_some = load i64, i64* %disc");
        self.emitln("  %result = xor i64 %is_some, 1");
        self.emitln("  ret i64 %result");
        self.emitln("}\n");

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

        // Option.len()
        self.emitln(&format!("define i64 @Option.len({opt_ty} %self) {{"));
        self.emitln("entry:");
        self.emitln(&format!("  %val = alloca {opt_ty}"));
        self.emitln(&format!("  store {opt_ty} %self, {opt_ty}* %val"));
        self.emitln(&format!("  %disc_gep = getelementptr {opt_ty}, {opt_ty}* %val, i32 0, i32 0"));
        self.emitln("  %is_some = load i64, i64* %disc_gep");
        self.emitln("  %ok = icmp ne i64 %is_some, 0");
        self.emitln("  br i1 %ok, label %len_some, label %len_zero");
        self.emitln("\nlen_zero:");
        self.emitln("  ret i64 0");
        self.emitln("\nlen_some:");
        self.emitln(&format!("  %val_gep = getelementptr {opt_ty}, {opt_ty}* %val, i32 0, i32 1"));
        self.emitln("  %payload = load i64, i64* %val_gep");
        self.emitln("  %str_ptr = inttoptr i64 %payload to i8*");
        self.emitln("  %len_result = call i64 @xiom_str_len(i8* %str_ptr)");
        self.emitln("  ret i64 %len_result");
        self.emitln("}\n");

        // B-001: Emit builtins for concrete Option__T types so `.is_some`,
        // `.is_none`, and `.unwrap` resolve without falling through to
        // the monomorphisation stub (which returns 0).
        let concrete_opts: Vec<String> = self.types.type_meta.keys().into_iter()
     .filter(|k| k.starts_with("Option__"))
            .collect();
        for name in &concrete_opts {
            let cty = format!("%struct.{name}");
            let field_ty_1 = self.types.type_meta.get(name)
                .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                .unwrap_or_else(|| "Int".to_string());
            let llvm1 = if field_ty_1 == "Int" { "i64".to_string() }
                else { format!("%struct.{field_ty_1}") };

            // is_some
            self.emitln(&format!("define i64 @{name}.is_some({cty} %self) {{"));
            self.emitln("entry:");
            self.emitln(&format!("  %val = alloca {cty}"));
            self.emitln(&format!("  store {cty} %self, {cty}* %val"));
            self.emitln(&format!("  %disc = getelementptr {cty}, {cty}* %val, i32 0, i32 0"));
            self.emitln("  %result = load i64, i64* %disc");
            self.emitln("  ret i64 %result");
            self.emitln("}\n");

            // is_none
            self.emitln(&format!("define i64 @{name}.is_none({cty} %self) {{"));
            self.emitln("entry:");
            self.emitln(&format!("  %val = alloca {cty}"));
            self.emitln(&format!("  store {cty} %self, {cty}* %val"));
            self.emitln(&format!("  %disc = getelementptr {cty}, {cty}* %val, i32 0, i32 0"));
            self.emitln("  %is_some = load i64, i64* %disc");
            self.emitln("  %result = xor i64 %is_some, 1");
            self.emitln("  ret i64 %result");
            self.emitln("}\n");

            // unwrap
            self.emitln(&format!("define {llvm1} @{name}.unwrap({cty} %self) {{"));
            self.emitln("entry:");
            self.emitln(&format!("  %val = alloca {cty}"));
            self.emitln(&format!("  store {cty} %self, {cty}* %val"));
            self.emitln(&format!("  %disc_gep = getelementptr {cty}, {cty}* %val, i32 0, i32 0"));
            self.emitln("  %is_some = load i64, i64* %disc_gep");
            self.emitln("  %ok = icmp ne i64 %is_some, 0");
            self.emitln("  br i1 %ok, label %unwrap_ok, label %unwrap_fail");
            self.emitln("\nunwrap_fail:");
            self.emitln("  call void @llvm.trap()");
            self.emitln("  unreachable");
            self.emitln("\nunwrap_ok:");
            self.emitln(&format!("  %val_gep = getelementptr {cty}, {cty}* %val, i32 0, i32 1"));
            self.emitln(&format!("  %result = load {llvm1}, {llvm1}* %val_gep"));
            self.emitln(&format!("  ret {llvm1} %result"));
            self.emitln("}\n");
        }
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
                        let ret_ty = &self.fctx.current_return_type.clone();
                        let result_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {result_alloca} = alloca {ret_ty}"));
                        // 5c.37: Seed with default so paths without an explicit store
                        // (e.g. guard fall-through) don't load garbage.
                        let seed = Self::default_const_for(ret_ty);
                        self.emitln(&format!("  store {ret_ty} {seed}, {ret_ty}* {result_alloca}"));
                        self.fctx.match_result_ptr = Some(result_alloca.clone());
                        self.compile_stmt(stmt)?;
                        self.fctx.match_result_ptr = None;
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {ret_ty}, {ret_ty}* {result_alloca}"));
                        if let Some(res_ptr) = self.fctx.result_ptr.as_ref() {
                            let ret_ty = &self.fctx.current_return_type.clone();
                            self.emitln(&format!("  store {ret_ty} {loaded}, {ret_ty}* {res_ptr}"));
                        }
                        if !self.fctx.current_ensures.is_empty() {
                            self.compile_ensures_checks();
                        }
                        let ret_ty = &self.fctx.current_return_type.clone();
                        // P0-2: Emit deferred cleanups before return
                        self.compile_deferred_cleanups()?;
                        // R5: Decrement recursion depth before tail-return
                        let depth_dec = self.fresh_tmp();
                        self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
                        let new_depth = self.fresh_tmp();
                        self.emitln(&format!("  {new_depth} = sub i64 {depth_dec}, 1"));
                        self.emitln(&format!("  store i64 {new_depth}, i64* @xiom_recursion_counter"));
                        self.emitln(&format!("  ret {ret_ty} {loaded}"));
                        last_result = Some(loaded);
                    } else if is_last && is_expression && matches!(stmt, Stmt::If(..)) {
                        // A tail `if`-expression used as the function's implicit
                        // return value: `fn f() -> T { if c { a } else { b } }`.
                        // Reuse the proven tail-Match mechanism ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â allocate a result
                        // slot, redirect each branch's tail expression to store into
                        // it (via `match_result_ptr`), then load + `ret`. Previously
                        // such a body fell through to the A4 fallback and returned
                        // `0`/default, silently discarding the branch values.
                        let ret_ty = self.fctx.current_return_type.clone();
                        let result_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {result_alloca} = alloca {ret_ty}"));
                        // Seed a default so an else-less path can't load garbage.
                        let seed = Self::default_const_for(&ret_ty);
                        self.emitln(&format!("  store {ret_ty} {seed}, {ret_ty}* {result_alloca}"));
                        let saved_ptr = self.fctx.match_result_ptr.take();
                        let saved_ty = self.fctx.match_result_ty.take();
                        self.fctx.match_result_ptr = Some(result_alloca.clone());
                        self.fctx.match_result_ty = Some(ret_ty.clone());
                        self.compile_stmt(stmt)?;
                        self.fctx.match_result_ptr = saved_ptr;
                        self.fctx.match_result_ty = saved_ty;
                        // Emit the load + ret only if the merge block is live (not
                        // `unreachable` from all-branches-returned).
                        if !self.current_block_terminated() {
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load {ret_ty}, {ret_ty}* {result_alloca}"));
                            if let Some(res_ptr) = self.fctx.result_ptr.as_ref() {
                                self.emitln(&format!("  store {ret_ty} {loaded}, {ret_ty}* {res_ptr}"));
                            }
                            if !self.fctx.current_ensures.is_empty() {
                                self.compile_ensures_checks();
                            }
                            // P0-2: Emit deferred cleanups before return
                            self.compile_deferred_cleanups()?;
                            // R5: Decrement recursion depth before tail-return
                            let depth_dec = self.fresh_tmp();
                            self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
                            let new_depth = self.fresh_tmp();
                            self.emitln(&format!("  {new_depth} = sub i64 {depth_dec}, 1"));
                            self.emitln(&format!("  store i64 {new_depth}, i64* @xiom_recursion_counter"));
                            self.emitln(&format!("  ret {ret_ty} {loaded}"));
                            last_result = Some(loaded);
                        }
                    } else {
                        self.compile_stmt(stmt)?;
                    }
                }
                StmtOrExpr::Expr(expr) => {
                    let (result, result_ty) = self.compile_expr(expr)?;
                    if let Some(ptr) = self.fctx.match_result_ptr.clone() {
                        let ret_ty = self.fctx.match_result_ty.clone().unwrap_or_else(|| self.fctx.current_return_type.clone());
                        // Coerce the arm's value to the match result type. An arm
                        // whose body is (e.g.) a bare enum-variant identifier can
                        // compile to a raw i64 discriminant; wrap it into the
                        // result struct so `store volatile %struct.X i64` is never emitted.
                        // `result_ty` is the value's real LLVM type from compile_expr.
                        let from_ty = result_ty.clone();
                        let store_val = self.coerce_value(&result, &from_ty, &ret_ty);
                        self.emitln(&format!("  store {ret_ty} {store_val}, {ret_ty}* {ptr}"));
                    }
                    if is_last && is_expression {
                        if self.current_block_terminated() {
                            // The tail expression already emitted a terminator
                            // (e.g. `unsafe { return X() }`, or a tail if/match that
                            // returns on every path). Do NOT emit a second ret.
                        } else {
                            let ret_ty = self.fctx.current_return_type.clone();
                            // Coerce the tail value's REAL type to the declared
                            // return type (struct->i64 extracts field 0 / empty
                            // struct -> 0; scalar->struct widens), then guard against
                            // an empty operand. Prevents `ret i64 %s` where %s is a
                            // struct (e.g. an empty GlobalAlloc value).
                            let coerced = self.coerce_value(&result, &result_ty, &ret_ty);
                            let ret_val = self.zero_val_for(&coerced, &ret_ty);
                            // Store result in the result alloca for ensures checks
                            if let Some(res_ptr) = self.fctx.result_ptr.as_ref() {
                                self.emitln(&format!("  store {ret_ty} {ret_val}, {ret_ty}* {res_ptr}"));
                            }
                            // Check ensures before returning
                            if !self.fctx.current_ensures.is_empty() {
                                self.compile_ensures_checks();
                            }
                            // P0-2: Emit deferred cleanups before return
                            self.compile_deferred_cleanups()?;
                            // R5: Decrement recursion depth before tail-return
                            let depth_dec = self.fresh_tmp();
                            self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
                            let new_depth = self.fresh_tmp();
                            self.emitln(&format!("  {new_depth} = sub i64 {depth_dec}, 1"));
                            self.emitln(&format!("  store i64 {new_depth}, i64* @xiom_recursion_counter"));
                            self.emitln(&format!("  ret {ret_ty} {ret_val}"));
                        }
                    }
                    last_result = Some(result);
                }
            }
        }
        Ok(last_result)
    }



    /// Compile the statements in an if-expression arm block, taking the last
    /// expression and storing it into `result_alloca`.
    fn compile_if_arm_value(&mut self, block: &Block, result_alloca: &str, result_ty: &str) -> Result<(), String> {
        let n = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i + 1 == n;
            match stmt {
                StmtOrExpr::Stmt(s) => {
                    self.compile_stmt(s)?;
                    if matches!(s, Stmt::Return(..)) { break; }
                }
                StmtOrExpr::Expr(e) => {
                    if is_last {
                        let (val, val_ty) = self.compile_expr(e)?;
                        let store_val = self.coerce_value(&val, &val_ty, result_ty);
                        self.emitln(&format!("  store {result_ty} {store_val}, {result_ty}* {result_alloca}"));
                    } else {
                        self.compile_expr(e)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Compile a statement that does not produce a value (emit-only).
    /// Collects types from ALL arms and picks the widest (struct > i64 > narrower)
    /// so the result alloca is large enough for every arm.  `coerce_value` handles
    /// the actual per-arm conversion during the store.
    fn infer_match_llvm_type(&self, arms: &[MatchArm], scrutinee_llvm_ty: &str) -> String {
        // Extract the struct type name from the scrutinee (e.g. %struct.Result__Regex__Str ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ Result__Regex__Str)
        let scrutinee_struct_name = if scrutinee_llvm_ty.starts_with("%struct.") {
            Some(&scrutinee_llvm_ty[8..])
        } else { None };
        let mut types: Vec<String> = Vec::new();
        for arm in arms {
            let ty = match &arm.body {
                MatchBody::Expr(e) => {
                    let t = self.infer_llvm_type(e);
                    // If inference returns i64 and the expression is an Ident
                    // that's a pattern binding (Ok(r) => r), resolve from the
                    // scrutinee struct's field type instead.
                    if t == "i64" || t == "i8*" || t.is_empty() {
                        if let Expr::Ident(ident) = e {
                            if let Some(payload_ty) = self.infer_pattern_binding_type(
                                &arm.pattern, &ident.name, scrutinee_struct_name) {
                                payload_ty
                            } else if t.is_empty() { continue; } else { t }
                        } else if t.is_empty() { continue; } else { t }
                    } else if t.is_empty() { continue; } else { t }
                }
                MatchBody::Block(b) => {
                    let t = b.stmts.last().and_then(|s| {
                        if let StmtOrExpr::Expr(e) = s { Some(self.infer_llvm_type(e)) } else { None }
                    }).unwrap_or_default();
                    if t.is_empty() { continue; }
                    t
                }
            };
            types.push(ty);
        }
        if types.is_empty() {
            return "i64".to_string();
        }
        // Prefer a struct type (wider alloca).
        let has_struct = types.iter().any(|t| t.starts_with("%struct."));
        if has_struct {
            return types.iter().find(|t| t.starts_with("%struct.")).cloned().unwrap_or_else(|| types[0].clone());
        }
        // All-pointer arms: use i8* as the common pointer type.
        if types.iter().all(|t| t.ends_with('*')) {
            return "i8*".to_string();
        }
        // Otherwise, use i64 (widest integer-like type).
        types[0].clone()
    }

    /// For patterns like Ok(r) or Some(v), resolve the bound ident's type from
    /// the scrutinee struct's field list.
    fn infer_pattern_binding_type(&self, pattern: &Pattern, ident_name: &str, scrutinee_struct_name: Option<&str>) -> Option<String> {
        let scrutinee_name = scrutinee_struct_name?;
        let (inner, field_idx) = match pattern {
            Pattern::Some(inner, _) | Pattern::Ok(inner, _) => (inner.as_ref(), 1),
            Pattern::Err(inner, _) => (inner.as_ref(), 2),
            _ => return None,
        };
        if let Pattern::Ident(id) = inner {
            if id.name == ident_name {
                return Some(self.field_llvm_type(scrutinee_name, field_idx));
            }
        }
        None
    }

    /// Given a parent expression `obj` (e.g. `LogLevel`, `xiom.log.LogLevel`) and a
    /// candidate `variant` name, return the registered enum key if `obj` names an
    /// enum type that has that variant. Used to compile qualified enum-variant
    /// paths like `xiom.log.LogLevel.Warn`.
    fn resolve_enum_for_variant(&self, obj: &Expr, variant: &str) -> Option<String> {
        // Extract the trailing type-name segment of `obj` (the last Field/Ident).
        let type_seg = match obj {
            Expr::Ident(id) => Some(id.name.clone()),
            Expr::Field(_, f, _) => Some(f.name.clone()),
            _ => None,
        }?;
        // Exact enum key.
        if let Some(vars) = self.types.enum_variants.get(&type_seg) {
            if vars.iter().any(|(v, _)| v == variant) {
                return Some(type_seg);
            }
        }
        // Module-qualified enum key ending in `.type_seg` (e.g. `xiom.log.LogLevel`).
        for (enum_key, vars) in self.types.enum_variants.entries() {
            if enum_key.ends_with(&format!(".{type_seg}"))
                && vars.iter().any(|(v, _)| v == variant)
            {
                return Some(enum_key.clone());
            }
        }
        // NOTE 4 fix: module-qualified VARIANT form â€” `bigfloat.Down` (obj is
        // the MODULE, the variant's parent enum is a type the module exports).
        // Prefer an enum key whose module prefix matches; otherwise accept the
        // first enum carrying the variant (deterministic registry order).
        for (enum_key, vars) in self.types.enum_variants.entries() {
            if vars.iter().any(|(v, _)| v == variant) {
                let parent_mod = enum_key.rsplitn(2, '.').nth(1).unwrap_or("");
                if parent_mod == type_seg {
                    return Some(enum_key.clone());
                }
            }
        }
        for (enum_key, vars) in self.types.enum_variants.entries() {
            if vars.iter().any(|(v, _)| v == variant) {
                return Some(enum_key.clone());
            }
        }
        None
    }


    fn infer_struct_type_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(ident) => {
                // `this` is registered as the `self` local inside methods (5c.19).
                let lookup_name = if ident.name == "this" { "self" } else { ident.name.as_str() };
                if let Some((_, llvm_ty)) = self.lookup_local(lookup_name) {
                    if llvm_ty.starts_with("%struct.") {
                        let raw = &llvm_ty[8..]; // strip "%struct."
                        let clean = raw.trim_end_matches('*'); // strip pointer suffix
                        return Some(clean.to_string());
                    }
                }
                // Check module-level globals (var _exec: Executor = ...)
                if let Some((_, global_ty)) = self.local.module_globals.get(&ident.name) {
                    if global_ty.starts_with("%struct.") {
                        let raw = &global_ty[8..];
                        let clean = raw.trim_end_matches('*');
                        return Some(clean.to_string());
                    }
                }
                // Check if it's a type name (for static method calls like Rect.new(...))
                if self.types.types.contains_key(&ident.name) || self.types.type_meta.contains_key(&ident.name) {
                    return Some(ident.name.clone());
                }
                // Try current module's qualified name first (deterministic)
                if let Some(ref module) = self.local.current_module {
                    let qualified = format!("{}.{}", module, ident.name);
                    if self.types.type_meta.contains_key(&qualified) {
                        return Some(qualified);
                    }
                }
                // Fallback: search all qualified keys (last resort)
                for key in self.types.type_meta.keys() {
                    if key.ends_with(&format!(".{}", ident.name)) {
                        return Some(key.clone());
                    }
                }
                // Fallback: search generic_type_names ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â generic types may not
                // be in type_meta (injection chain can block Type while allowing
                // its methods), but they ARE registered as structs (e.g. Map[K,V]).
                for key in self.types.generic_type_names.iter() {
                    if key.ends_with(&format!(".{}", ident.name)) || key == &ident.name {
                        return Some(key.clone());
                    }
                }
                None
            }
            Expr::Struct(ident, _, _, _) => {
                // Try module-qualified name first, then bare name
                if let Some(ref module) = self.local.current_module {
                    let qualified = format!("{}.{}", module, ident.name);
                    if self.types.type_meta.contains_key(&qualified) {
                        return Some(qualified);
                    }
                }
                Some(ident.name.clone())
            },
            // 5c.29: Vec-of-struct element access (`tree.nodes[idx]`): the
            // struct type is the container's element type. Needed so nested
            // receivers like `tree.nodes[idx].keys` resolve their base type.
            Expr::Index(container, _, _) => {
                // Type-parameterized STATIC receiver (`Map[Str, Bool].new()`,
                // `Option[Int].unwrap()`): the base is a KNOWN TYPE name and
                // the index is a type-argument expression. Resolve to the
                // BASE type so the fn_key becomes "Map.new" — previously the
                // bare-key fallback hijacked another module's generic `new`
                // (stub body returning 0 → runtime crash in module-global
                // initializers like core/contracts.xi's `_coverage`).
                let base_is_type = match container.as_ref() {
                    Expr::Ident(id) => {
                        self.types.types.contains_key(&id.name)
                            || self.types.type_meta.contains_key(&id.name)
                            || self.types.generic_type_names.iter().any(|k| k == &id.name || k.ends_with(&format!(".{}", id.name)))
                    }
                    _ => false,
                };
                if base_is_type {
                    if let Expr::Ident(id) = container.as_ref() {
                        return Some(id.name.clone());
                    }
                }
                self.resolve_vec_elem_type(container)
            }
            Expr::Field(obj, field, _) => {
                // `module.Type` path (e.g. `alloc.Layout`): if the base is not an
                // instance value and the leaf names a known type, resolve to that
                // type so `alloc.Layout.new(..)` dispatches to `Layout.new`.
                if !self.receiver_is_instance(obj.as_ref()) {
                    if self.types.types.contains_key(&field.name) || self.types.type_meta.contains_key(&field.name) {
                        return Some(field.name.clone());
                    }
                    for key in self.types.type_meta.keys() {
                        if key.ends_with(&format!(".{}", field.name)) {
                            return Some(key.clone());
                        }
                    }
                    // Fallback: search generic_type_names for generic types
                    // whose Type declaration may not be in type_meta
                    for key in self.types.generic_type_names.iter() {
                        if key.ends_with(&format!(".{}", field.name)) || key == &field.name {
                            return Some(key.clone());
                        }
                    }
                }
                // Instance field access (e.g. `row.values.push(..)`):
                // resolve the base struct, then look up the field type so the
                // method receiver resolves to `Vec` rather than `SqliteRow`.
                if let Some(base_struct) = self.infer_struct_type_name(obj.as_ref()) {
                    // Try module-qualified type lookup first
                    for key in self.types.type_meta.keys() {
                        if key.ends_with(&base_struct) || key == base_struct {
                            if let Some(meta) = self.types.type_meta.get(&key) {
                                for (fname, ftype) in &meta.fields {
                                    if fname == &field.name {
                                        // Strip leading `*` from pointer types (e.g. `*SqliteRow`).
                                        let clean = ftype.trim_start_matches('*');
                                        if self.types.type_meta.contains_key(&clean.to_string())
                                            || self.types.types.contains_key(&clean.to_string())
                                            || clean == "Vec" || clean == "Option"
                                            || clean == "Result" || clean == "Map"
                                            || clean == "Set" || clean == "Str" {
                                            return Some(clean.to_string());
                                        }
                                        // Try suffix-match for module-qualified types
                                        for mk in self.types.type_meta.keys() {
                                            if mk.ends_with(&format!(".{}", clean)) {
                                                return Some(mk.clone());
                                            }
                                        }
                                        return None; // field type is not a known struct
                                    }
                                }
                            }
                            break;
                        }
                    }
                    return Some(base_struct.clone());
                }
                self.infer_struct_type_name(obj.as_ref())
            }
            Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
                // Infer type from the return type of a method/function call
                let fn_key = if let Expr::Field(obj, field, _) = func.as_ref() {
                    // Try module-qualified resolution first (e.g. iter.range ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ xiom.iter.range)
                    if let Some(recv_type) = self.infer_struct_type_name(obj.as_ref()) {
                        format!("{}.{}", recv_type, field.name)
                    } else {
                        let resolved = self.resolve_module_call(obj.as_ref(), &field.name);
                        if resolved != field.name { resolved } else { field.name.clone() }
                    }
                } else if let Expr::Ident(id) = func.as_ref() {
                    id.name.clone()
                } else {
                    return None;
                };
                if let Some((_, ret_ty)) = self.types.functions.get(&fn_key) {
                    if ret_ty.starts_with("%struct.") {
                        return Some(ret_ty[8..].to_string());
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn infer_llvm_type(&self, expr: &Expr) -> String {
        // (kept below)
        self.infer_llvm_type_impl(expr)
    }

    /// 5c.38: Infer the LLVM element type for Vec/Slice index operations.
    /// For struct elements like `Vec[Item]`, returns `%struct.Item`.
    /// For scalar elements like `Vec[Int]`, returns `i64`.
    fn infer_vec_elem_llvm_type(&self, expr: &Expr) -> String {
        // M33: Resolve via the Vec element type infrastructure first,
        // which handles compound expressions (field access through &mut).
        if let Some(elem_name) = self.resolve_vec_elem_type(expr) {
            return self.llvm_type_for(&elem_name).unwrap_or_else(|_| "i64".to_string());
        }
        if let Expr::Ident(id) = expr {
            if let Some(ref elem_xiom) = self.local.local_vec_elem.get(&id.name) {
                return self.llvm_type_for(elem_xiom).unwrap_or_else(|_| "i64".to_string());
            }
            // Try checking the local's XIOM type for concrete type info
            if let Some(xiom_ty) = self.local.local_xiom_types.get(&id.name) {
                // Type might be "Vec[Item]" ÃƒÂ¢Ã¢â€šÂ¬Ã¢â‚¬Â extract element
                if let Some(bracket) = xiom_ty.find('[') {
                    let elem_name = &xiom_ty[bracket+1..xiom_ty.len()-1];
                    return self.llvm_type_for(elem_name).unwrap_or_else(|_| "i64".to_string());
                }
            }
        }
        "i64".to_string()
    }

    /// Parse field LLVM types from a tuple struct name.
    /// `%struct.Tuple_Float32_Float32` ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ `["float", "float"]`
    /// `%struct.Tuple_Int_Float64` ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ `["i64", "double"]`
    fn parse_struct_field_types(&self, struct_ty: &str) -> Vec<String> {
        let name = struct_ty.trim_start_matches("%struct.");
        // BUG 1 fix (2026-08-10): prefer the registered type_meta layout. The
        // legacy split-based parser below only understands PRIMITIVE element
        // names, so a tuple containing structs (e.g. `Tuple__probe_tuple.Pair__
        // probe_tuple.Pair`) parsed every element as i64 â€” the tuple literal
        // stores then wrote only the first i64 of each struct into the slot
        // (garbage Vec pointers at runtime, docs/COMPILER_BUGS.md BUG 1).
        if let Some(meta) = self.types.type_meta.get(&name.to_string())
            .or_else(|| {
                self.local.current_module.as_ref()
                    .and_then(|m| self.types.type_meta.get(&format!("{m}.{name}")))
            })
            .or_else(|| {
                self.types.type_meta.entries().into_iter()
                    .find(|(k, _)| k.ends_with(&format!(".{name}")))
                    .map(|(_, v)| v)
            }) {
            return meta.fields.iter()
                .map(|(_, t)| self.llvm_type_for(t).unwrap_or_else(|_| "i64".to_string()))
                .collect();
        }
        // Fallback: legacy split parser for unregistered primitive tuples
        // (format Tuple_Type1_Type2_... or Type1_Type2).
        let parts: Vec<&str> = name.split('_').collect();
        let mut types = Vec::new();
        for part in parts {
            if part == "Tuple" { continue; }
            // Map XIOM type names to LLVM types
            let llvm = match part {
                "Int" | "Bool" | "Int32" | "UInt32" | "UInt64" | "Int64" | "Int8" | "UInt8" | "Int16" | "UInt16" => "i64",
                "Float64" => "double",
                "Float32" => "float",
                "Char" => "i32",
                "Str" => "i8*",
                _ => "i64", // default for unknown/custom types
            };
            types.push(llvm.to_string());
        }
        types
    }

    /// Emit a private constant C string and return an `i8*` register pointing at it.
    /// Uses byte length (not char count) so multi-byte UTF-8 is sized correctly.
    fn intern_cstring(&mut self, s: &str) -> String {
        let str_id = self.str_counter;
        self.str_counter += 1;
        let label = format!("@.str{str_id}");
        let escaped = s.replace('\\', "\\\\").replace('"', "\\22")
            .replace('\n', "\\0A").replace('\t', "\\09");
        let n = s.len() + 1;
        self.fctx.strings.push(format!(
            "{label} = private unnamed_addr constant [{n} x i8] c\"{escaped}\\00\""
        ));
        let tmp = self.fresh_tmp();
        self.emitln(&format!("  {tmp} = getelementptr [{n} x i8], [{n} x i8]* {label}, i64 0, i64 0"));
        tmp
    }

    fn field_xiom_type(&self, struct_name: &str, field_idx: usize) -> Option<String> {
        let meta = self.types.type_meta.get(&struct_name.to_string())
            .or_else(|| {
                self.local.current_module.as_ref()
                    .and_then(|m| self.types.type_meta.get(&format!("{m}.{struct_name}")))
            })
            .or_else(|| {
                self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{struct_name}")))
                    .map(|(_, v)| v)
            })?;
        meta.fields.get(field_idx).map(|(_, t)| t.clone())
    }


    /// Extract the element type from an LLVM array type like `[64 x i64]` ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ `i64`.
    fn extract_array_elem_ty(array_ty: &str) -> String {
        if let Some(rest) = array_ty.strip_prefix('[') {
            if let Some(x_pos) = rest.find(" x ") {
                let elem = rest[x_pos + 3..].trim();
                let elem_stripped = elem.strip_suffix(']').unwrap_or(elem);
                return elem_stripped.trim().to_string();
            }
        }
        "i64".to_string()
    }

    /// True when `expr` refers to a raw-pointer local/param (`*T`, tracked in
    /// `ptr_locals`), so `expr[i]` must inttoptr-and-byte-access rather than use
    /// the Str/Vec index paths.
    fn is_ptr_local_expr(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Ident(id) if self.local.ptr_locals.contains(&id.name))
    }

    /// Best-effort check whether an expression is Bool-typed (for `.to_str()`
    /// formatting). Recognizes bool literals, comparisons/logical ops, and locals
    /// previously recorded as Bool.
    fn expr_is_bool(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Bool(..) => true,
            Expr::Ident(id) => self.local.bool_locals.contains(&id.name),
            Expr::Paren(e, _) => self.expr_is_bool(e),
            Expr::Unary(UnaryOp::Not, _, _) => true,
            Expr::Binary(_, op, _, _) => matches!(
                op,
                BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge
                    | BinOp::And | BinOp::Or
            ),
            _ => false,
        }
    }

    fn infer_llvm_type_impl(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(_, _) | Expr::Bool(_, _) => "i64".to_string(),
            Expr::Float(_, _) => "double".to_string(),
            Expr::Str(_, _) => "i8*".to_string(),
            Expr::Char(_, _) => "i32".to_string(),
            Expr::Ident(ident) => {
                if let Some((_, llvm_ty)) = self.lookup_local(&ident.name) {
                    if llvm_ty == "double" { return "double".to_string(); }
                    return llvm_ty.clone();
                }
                // If the ident is an enum variant name (e.g., DivByZero), return the parent enum's struct type
                if let Some(enum_key) = self.types.enum_variants.entries().into_iter()
    .find(|(_, vars)| vars.iter().any(|(v, _)| v == &ident.name))
                    .map(|(ek, _)| ek)
                {
                    return format!("%struct.{enum_key}");
                }
                "i64".to_string()
            }
            Expr::Field(obj, field, _) => {
                // Resolve the LLVM type of a struct field access.
                // Handle both simple (obj.x) and compound (a.b[idx].x) bases.
                // The base may be a POINTER to the struct (struct `&T`/`&mut T`
                // params lower to `%struct.X*`) â€” strip the trailing `*` so the
                // field resolves against the pointee layout.
                let obj_ty_raw = self.infer_llvm_type(obj);
                let obj_ty = obj_ty_raw.trim_end_matches('*').to_string();
                if obj_ty.starts_with("%struct.") {
                    let type_name = &obj_ty[8..]; // strip "%struct."
                    if let Some(meta) = self.types.type_meta.get(&type_name.to_string())
                        .or_else(|| {
                            let suffix = format!(".{type_name}");
                            self.types.type_meta.keys().into_iter()
    .find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                .and_then(|k|self.types.type_meta.get(&k))
                        })
                    {
                        if let Some((_, ty_name)) = meta.fields.iter().find(|(name, _)| name == &field.name) {
                            // Generic container fields ("Vec[Int]", "Map[K,V]")
                            // resolve by their base container name ("Vec" â†’
                            // %struct.Vec), mirroring field_llvm_type.
                            let base = ty_name.split('[').next().unwrap_or(ty_name);
                            return self.llvm_type_for(base).unwrap_or_else(|_| "i64".to_string());
                        }
                    }
                }
                // Fallback: try by looking up the base's ident (legacy path)
                if let Expr::Ident(obj_ident) = obj.as_ref() {
                    if let Some((_, llvm_ty)) = self.lookup_local(&obj_ident.name) {
                        let base = llvm_ty.trim_end_matches('*').to_string();
                        if base.starts_with("%struct.") {
                            let type_name = &base[8..];
                            if let Some(meta) = self.types.type_meta.get(&type_name.to_string()) {
                                if let Some((_, ty_name)) = meta.fields.iter().find(|(name, _)| name == &field.name) {
                                    let base = ty_name.split('[').next().unwrap_or(ty_name);
                                    return self.llvm_type_for(base).unwrap_or_else(|_| "i64".to_string());
                                }
                            }
                        }
                    }
                }
                "i64".to_string()
            }
            Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
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
                        // Try to resolve method call: obj.method ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¾ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ Type.method
                        let bare = field.name.clone();
                        if let Some(recv_type) = self.infer_struct_type_name(obj) {
                            let qualified = format!("{}.{}", recv_type, field.name);
                            if self.types.functions.contains_key(&qualified) {
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
                    if let Some((_, ret_ty)) = self.types.functions.get(name) {
                        if ret_ty == "double" { return "double".to_string(); }
                        return ret_ty.clone();
                    }
                    // Fallback: try current-module qualified name (e.g., "benchmark.main.make_result")
                    if let Some(ref module) = self.local.current_module {
                        let qualified = format!("{module}.{name}");
                        if let Some((_, ret_ty)) = self.types.functions.get(&qualified) {
                            if ret_ty == "double" { return "double".to_string(); }
                            return ret_ty.clone();
                        }
                    }
                    // Fallback: search for any key ending with .name that returns a struct
                    {
                        let suffix = format!(".{name}");
                        for (k, (_, rt)) in self.types.functions.entries() {
                            if k.ends_with(&suffix) && rt.starts_with("%struct.") {
                                return rt.clone();
                            }
                        }
                    }
                    // Function pointer call ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â¦ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â look up tracked return type
                    if self.lookup_local(name).is_some() {
                        if let Some(ret_ty) = self.types.fn_ptr_return_types.get(name) {
                            if ret_ty == "double" { return "double".to_string(); }
                            return ret_ty.clone();
                        }
                    }
                }
                "i64".to_string()
            }
            Expr::Some(..) | Expr::None(..) => {
                // B-001: Use the function's return type when available so
                // concrete monomorphised types (Option__Point) infer correctly.
                if self.fctx.current_return_type.starts_with("%struct.") {
                    self.fctx.current_return_type.clone()
                } else if self.types.types.contains_key(&"Option".to_string()) {
                    "%struct.Option".to_string()
                } else {
                    "i64".to_string()
                }
            }
            Expr::Ok(..) | Expr::Err(..) => {
                if self.fctx.current_return_type.starts_with("%struct.") {
                    self.fctx.current_return_type.clone()
                } else if self.types.types.contains_key(&"Result".to_string()) {
                    "%struct.Result".to_string()
                } else {
                    "i64".to_string()
                }
            }
            Expr::Struct(ident, _, _, _) => self.llvm_type_for(&ident.name).unwrap_or_else(|_| "i64".to_string()),
            Expr::Paren(inner, _) => self.infer_llvm_type(inner),
            Expr::Tuple(items, _) => {
                if items.is_empty() { "void".to_string() } else {
                    let parts: Vec<String> = items.iter().map(|i| {
                        let t = self.infer_llvm_type(i);
                        Self::xiom_type_name_from_llvm(&t)
                    }).collect();
                    let name = format!("Tuple__{}", parts.join("__"));
                    if self.types.types.contains_key(&name) || self.types.type_meta.contains_key(&name) {
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
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => {
                let inner_ty = self.infer_llvm_type(inner);
                format!("{inner_ty}*")
            }
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
            Expr::Match(_scrutinee, arms, _) => self.infer_match_llvm_type(arms, "i64"),
            Expr::Index(container, _, _) => {
                // For indexed Vec elements, return the element's struct type.
                if let Some(elem_type_name) = self.resolve_vec_elem_type(container) {
                    return format!("%struct.{elem_type_name}");
                }
                // M33: For scalar elements (Int, Float, etc.), return i64,
                // not the container's type. Previously this returned "%struct.Vec"
                // which caused `id(arr[0])` to be monomorphised as id_Vec instead
                // of id_Int, leading to inttoptr+load of the element value as a
                // Vec pointer ÃƒÂ¢Ã¢â‚¬Â Ã¢â‚¬â„¢ ACCESS_VIOLATION.
                let cont_ty = self.infer_llvm_type(container);
                if cont_ty == "%struct.Vec" || cont_ty.contains("struct.Vec") {
                    return "i64".to_string();
                }
                "i64".to_string()
            }
            _ => "i64".to_string(),
        }
    }

    /// Recursively determine if an expression involves float operations
    fn is_float_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Float(..) => true,
            Expr::Ident(_) => is_float_local(expr, &self.fctx.locals),
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
                            if let Some(meta) = self.types.type_meta.get(&type_name.to_string()) {
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
            Expr::Call(_, _, _) | Expr::GenericCall(_, _, _, _) | Expr::If(..) => {
                let ty = self.infer_llvm_type(expr);
                ty == "double" || ty == "float"
            },
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

