// XIOM -- LLVM IR Codegen
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Codegen -- Phase 0: AST to LLVM IR text..
//! Emits human-readable LLVM IR that can be compiled with `llc`.
//! No external dependencies ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â pure string emission.
//! Handles: functions, arithmetic, control flow (if/else/while/match),
//! let/var bindings, structs, function calls.

use xiom_ast::*;
use std::collections::HashMap;
use std::collections::HashSet;

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

pub use context::{CodegenConfig, TypeContext, FunctionContext, MonoContext, LocalContext, TypeMeta};

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

    /// Compilation flags and target configuration
    pub config: CodegenConfig,
    /// Type system registration, interface/enum metadata, function signatures
    pub types: TypeContext,
    /// Per-function compilation state (locals, params, return type, ensures)
    pub fctx: FunctionContext,
    /// Monomorphisation worklist and instantiation tracking
    pub mono: MonoContext,
    /// Local variable classification, module-level globals, loop stack
    pub local: LocalContext,
}



impl IrEmitter {
        pub fn new() -> Self {
        Self {
            output: String::new(),
            tmp_counter: 0,
            block_counter: 0,
            str_counter: 0,
            has_llvm_trap_decl: false,
            config: CodegenConfig::default(),
            types: TypeContext::default(),
            fctx: FunctionContext {
                locals: vec![HashMap::new()],
                ..FunctionContext::default()
            },
            mono: MonoContext::default(),
            local: LocalContext::default(),
        }
    }

    pub fn set_target_triple(&mut self, triple: &str) {
        self.config.target_triple = triple.to_string();
    }

    pub fn set_check_contracts(&mut self, enabled: bool) {
        self.config.check_contracts = enabled;
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
    pub(crate) fn llvm_type_byte_size(llvm_ty: &str, type_meta: &HashMap<String, TypeMeta>) -> usize {
        match llvm_ty {
            "i1" | "i8" => 1,
            "i16" => 2,
            "i32" | "float" => 4,
            "i64" | "double" => 8,
            ty if ty.starts_with("%struct.") => {
                let inner = &ty[8..];
                let meta = type_meta.get(inner)
                    .or_else(|| type_meta.iter().find(|(k,_)| k.ends_with(&format!(".{inner}"))).map(|(_,v)| v));
                match meta {
                    Some(m) => {
                        let mut total: usize = 0;
                        for (_, field_ty) in &m.fields {
                            if field_ty.contains('[') && !field_ty.starts_with('[') {
                                total += 8; // generic container → i64 handle
                            } else if field_ty.starts_with('[') {
                                total += 8; // fixed-size array → 8 per element simplified
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
    /// materialized to their real value ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â these are the initializers that
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
                    format!("{f:.6}")
                } else if llvm_ty == "float" {
                    // CG-02: LLVM requires float constants to round-trip exactly
                    // through decimal→double→float. Use ryu crate or manual formatting
                    // with enough digits for the exact float32→float64→decimal→float64
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
    // 5e.7f: Const Evaluation — walk & fold const expressions at compile time
    // ========================================================================

    /// Evaluate a const expression to a literal value by recursively resolving
    /// const references and folding arithmetic. Returns `Some(Expr)` on full
    /// evaluation, `None` if the expression cannot be const-evaluated.
    /// `depth` tracks recursion depth for cycle detection (max 128).
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
    fn evaluate_all_consts(&mut self) {
        // Clone all keys first (can't iterate and mutate simultaneously)
        let keys: Vec<String> = self.local.constants.keys().cloned().collect();
        for name in keys {
            if let Some(expr) = self.local.constants.get(&name).cloned() {
                if let Some(evaluated) = Self::const_eval(&expr, &self.local.constants, 0) {
                    self.local.constants.insert(name, evaluated);
                }
            }
        }
    }

    /// Widen a narrow integer value (`i1`/`i8`/`i16`/`i32`) to `i64` so it can
    /// participate in the emitter's i64 integer arithmetic/comparison model.
    fn widen_to_i64(&mut self, val: &str, ty: &str) -> String {
        match ty {
            "i1" | "i8" => {
                let ext = self.fresh_tmp();
                self.emitln(&format!("  {ext} = zext {ty} {val} to i64"));
                ext
            }
            "i16" | "i32" => {
                let ext = self.fresh_tmp();
                self.emitln(&format!("  {ext} = sext {ty} {val} to i64"));
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
    /// the value "sink" points ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â call arguments, returns, and stores ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â so a value
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
        if let Expr::Call(func, _, _) = e {
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
            Expr::Call(func, args, _) => Self::expr_uses_this(func) || args.iter().any(|a| Self::expr_uses_this(a)),
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
            Stmt::While(cond, body, _, _) => Self::expr_uses_this(cond) || Self::block_uses_this(body),
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

    /// G-20: does the method body reference receiver STATE — either `this`
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
        // 5e.3: check BOTH types and type_meta — catalog-loaded types (e.g.,
        // benchmark modules) may only be in type_meta, not types.
        let types_fields = self.types.types.get(&recv.name)
            .or_else(|| {
                let suffix = format!(".{}", recv.name);
                self.types.types.keys().find(|k| k.ends_with(&suffix)).and_then(|k| self.types.types.get(k))
            })
            .cloned();
        let fields: Option<Vec<String>> = types_fields.or_else(|| {
            self.types.type_meta.get(&recv.name).map(|m| {
                m.fields.iter().map(|(n, _)| n.clone()).collect()
            })
        }).or_else(|| {
            let suffix = format!(".{}", recv.name);
            self.types.type_meta.keys().find(|k| k.ends_with(&suffix)).and_then(|k| {
                self.types.type_meta.get(k).map(|m| {
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
            Stmt::While(_, body, _, _) => Self::collect_bound_names(body, out),
            Stmt::For(binder, _, body, _) => {
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
            Stmt::While(cond, body, _, _) => Self::expr_mentions_any_ident(cond, names) || Self::block_mentions_any_ident(body, names),
            Stmt::For(_, iter, body, _) => Self::expr_mentions_any_ident(iter, names) || Self::block_mentions_any_ident(body, names),
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
            // obj.FIELD: the field NAME is not a bare ident — only scan the object.
            Expr::Field(obj, _, _) => Self::expr_mentions_any_ident(obj, names),
            Expr::Binary(a, _, b, _) => Self::expr_mentions_any_ident(a, names) || Self::expr_mentions_any_ident(b, names),
            Expr::Call(func, args, _) => Self::expr_mentions_any_ident(func, names) || args.iter().any(|a| Self::expr_mentions_any_ident(a, names)),
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
            // 6A.2: Unknown types must not silently compile as i64.
            // Log the unknown type so the user can diagnose the issue.
            _ => {
                eprintln!("xiom: warning: unknown type '{}' — defaulting to i64. This may produce incorrect code.", xiom_ty);
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

    fn xiom_type_name_from_llvm(llvm_ty: &str) -> String {
        // Check pointer types before stripping `*` — `i8*` is Str, not Int8.
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
                format!("Tuple_{}", parts.join("_"))
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
    /// ("Result[Vec[Int], Str]"). Used ONLY for fn_return_xiom â€” the field
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

    /// Resolve a Vec field's element type from its container expression.
    /// For `h.entries[i].name`, the container `h.entries` has field type
    /// `Vec[HttpHeader]` in type_meta. This extracts `HttpHeader` (fully qualified).
    /// 5c.30: For an Option[X]/Result[X, E] type string, return X (the
    /// success payload), respecting nested brackets ("Result[Vec[Int], Str]"
    /// â†’ "Vec[Int]").
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
        let leaf = match func {
            Expr::Ident(id) => id.name.clone(),
            Expr::Field(_, f, _) => f.name.clone(),
            _ => return None,
        };
        if let Some(rt) = self.types.fn_return_xiom.get(&leaf) {
            return Some(rt.clone());
        }
        let suffix = format!(".{leaf}");
        let mut found: Option<&String> = None;
        for (k, v) in self.types.fn_return_xiom.iter() {
            if k.ends_with(&suffix) {
                match found {
                    None => found = Some(v),
                    Some(prev) if prev == v => {}
                    _ => return None, // ambiguous with different types
                }
            }
        }
        found.cloned()
    }

    /// 5c.30: Track boxed-struct payload flow for a `let`/`var` binding.
    /// - `x.pop()` / `x.get(i)` on a Vec-of-struct container â†’ the Option's
    ///   payload is a boxed struct pointer (record in local_opt_payload).
    /// - fn calls returning Option[X]/Result[X, E] â†’ record X.
    /// - `opt.unwrap()` where opt is such an Option â†’ classify the binding:
    ///   Vec[T] payloads are container HANDLES, struct payloads are boxes.
    fn track_boxed_payload_binding(&mut self, name: &str, value: &Expr) {
        self.local.local_opt_payload.remove(name);
        self.local.local_boxed_struct.remove(name);
        self.local.local_vec_handle.remove(name);
        self.local.local_err_payload.remove(name);
        if let Expr::Call(func, _, _) = value {
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
                        if let Expr::Ident(opt_id) = recv.as_ref() {
                            if let Some(t) = self.local.local_opt_payload.get(&opt_id.name).cloned() {
                                if let Some(elem) = t.strip_prefix("Vec[").and_then(|s| s.strip_suffix(']')) {
                                    self.local.local_vec_handle.insert(name.to_string(), elem.to_string());
                                } else {
                                    self.local.local_boxed_struct.insert(name.to_string(), t);
                                }
                                return;
                            }
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
                        self.types.types.keys()
                            .find(|k| k.ends_with(&format!(".{payload}")) || k.as_str() == payload)
                            .cloned()
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

    /// 5c.30: If `expr` is a `Vec[T].new()` / `Vec[T].with_capacity(..)` call,
    /// return the element type name `T` (from the explicit type argument).
    fn vec_ctor_elem_type(expr: &Expr) -> Option<String> {
        if let Expr::Call(func, _, _) = expr {
            if let Expr::Field(obj, method, _) = func.as_ref() {
                if matches!(method.name.as_str(), "new" | "with_capacity") {
                    if let Expr::Index(base, idx, _) = obj.as_ref() {
                        if let Expr::Ident(b) = base.as_ref() {
                            if b.name == "Vec" {
                                match idx.as_ref() {
                                    Expr::Ident(t) => return Some(t.name.clone()),
                                    Expr::Tuple(elems, _) => {
                                        if let Some(Expr::Ident(t)) = elems.first() {
                                            return Some(t.name.clone());
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        // Array literals: infer element type from first element.
        // e.g. [1.5, 2.5] → Float64, [1, 2, 3] → Int
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
        if let Type::Named(ident, type_args) = ty {
            if ident.name == "Vec" {
                if let Some(first) = type_args.first() {
                    return Some(Self::type_from_ast(first));
                }
            }
        }
        None
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
        if let Expr::Field(base, field_expr, _) = container {
            let base_ty = self.infer_struct_type_name(base)?;
            for key in self.types.type_meta.keys() {
                if key.ends_with(&base_ty) || key == &base_ty {
                    if let Some(meta) = self.types.type_meta.get(key) {
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
            return self.types.types.keys()
                .find(|k| k.ends_with(&format!(".{}", elem)) || k.as_str() == elem)
                .cloned();
        }
        if let Expr::Field(base, field_expr, _) = container {
            let base_ty = self.infer_struct_type_name(base)?;
            for key in self.types.type_meta.keys() {
                if key.ends_with(&base_ty) || key == &base_ty {
                    if let Some(meta) = self.types.type_meta.get(key) {
                        for (fname, ftype) in &meta.fields {
                            if fname == &field_expr.name {
                                if let Some(inner) = ftype.strip_prefix("Vec[") {
                                    if let Some(bare_name) = inner.strip_suffix(']') {
                                        // Only return if this is a known struct type
                                        // (not a primitive like Int, Str, Bool, etc.)
                                        if let Some(qualified) = self.types.types.keys()
                                            .find(|k| k.ends_with(&format!(".{}", bare_name)) || k.as_str() == bare_name)
                                            .cloned()
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

    /// FIELD-I64: When obj_val is an i64 from a Vec index of a struct element
    /// (stored inline via memcpy or as val_to_i64 heap pointer), resolve field
    /// access via inttoptr+GEP on a known struct type. Returns None if no
    fn llvm_type_for(&self, type_name: &str) -> Result<String, String> {
        // Parse array types like [N x ElementType] ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â used for fixed-size stack arrays.
        if type_name.starts_with('[') {
            if let Some(rest) = type_name.strip_prefix('[') {
                if let Some(x_pos) = rest.find(" x ") {
                    let n_str = rest[..x_pos].trim();
                    let elem_name = rest[x_pos + 3..].trim();
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
                    // to resolve it as a struct name or builtin ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the caller will get
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
        // Try current module's qualified name first (e.g., "types.Person")
        if let Some(ref module) = self.local.current_module {
            let qualified = format!("{}.{}", module, type_name);
            if self.types.types.contains_key(&qualified) || self.types.type_meta.contains_key(&qualified) {
                return Ok(format!("%struct.{qualified}"));
            }
        }
        // Try exact match
        if self.types.types.contains_key(type_name) || self.types.type_meta.contains_key(type_name) {
            return Ok(format!("%struct.{type_name}"));
        }
        // Search for any module-qualified variant ending with .type_name
        for (key, _) in &self.types.type_meta {
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
        for (enum_key, variants) in &self.types.enum_variants {
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
        // then suffix ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â mirroring the struct lookup above.
        if self.types.enum_variants.contains_key(type_name) {
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
                // Æ’ "i64 (i64)*". Parse the signature and lower each part.
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
                Err(format!("unknown type '{}' — not a registered struct, enum, or builtin", type_name))
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
                // Also check generic_type_names ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â generic types may not
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
    /// fields contribute their own size â€” the old `field_count Ã— 8` math
    /// undercounted (JsonEntry { key: Str, value: JsonValue } is 24 bytes,
    /// not 16), truncating Vec elements on push/index.
    fn struct_byte_size(&self, type_name: &str) -> i64 {
        self.struct_byte_size_depth(type_name, 0)
    }

    fn struct_byte_size_depth(&self, type_name: &str, depth: u32) -> i64 {
        if depth > 8 {
            return 8;
        }
        let meta = self.types.type_meta.get(type_name)
            .or_else(|| {
                self.types.type_meta.iter()
                    .find(|(k, _)| k.ends_with(&format!(".{type_name}")))
                    .map(|(_, v)| v)
            });
        let Some(meta) = meta else {
            return 8
        };
        let mut total = 0i64;
        for (_, fty) in meta.fields.iter() {
            // Generic containers are i64 handles (5c.28h).
            if fty.contains('[') && !fty.starts_with('[') {
                total += 8;
                continue;
            }
            // Fixed-size arrays `[N x T]`: N Ã— 8.
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

    fn field_llvm_type(&self, struct_name: &str, field_idx: usize) -> String {
        let meta = self.types.type_meta.get(struct_name)
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
                self.types.type_meta.iter()
                    .find(|(k, _)| k.ends_with(&format!(".{struct_name}")))
                    .map(|(_, v)| v)
            });
        if let Some(meta) = meta {
            if let Some((_, ty_name)) = meta.fields.get(field_idx) {
                // For generic types (Vec[Int], Map[Str,Int]), return i64
                // to avoid Win64 sret corruption (5c.28 NET crash fix).
                if ty_name.contains('[') {
                    return "i64".to_string();
                }
                return self.llvm_type_for(ty_name).unwrap_or_else(|_| "i64".to_string());
            }
        }
        "i64".to_string()
    }

    /// 5e.1 G-18: byte size of a struct from type_meta field list.
    /// Sums LLVM type widths: i8=1, i16=2, i32=4, i64=8, etc.
    /// Returns 0 for unknown types. Used for C FFI malloc/offsetof.
    pub(crate) fn sizeof_struct(&self, type_name: &str) -> usize {
        let suffix = format!(".{type_name}");
        let meta = self.types.type_meta.get(type_name)
            .or_else(|| self.types.type_meta.iter().find(|(k, _)| k.ends_with(&suffix)).map(|(_, v)| v));
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
            // enum qualifier â€” compare the LEAF segment. Also tolerate
            // qualified/unqualified enum keys.
            let leaf = name.rsplit('.').next().unwrap_or(name);
            let variants = self.types.enum_variants.get(type_name)
                .or_else(|| {
                    self.types.enum_variants.iter()
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
            Pattern::Lit(Literal::Int(..)) | Pattern::Lit(Literal::Bool(..)) => true,
            Pattern::Variant(..) => true,
            Pattern::Some(..) | Pattern::None(..) | Pattern::Ok(..) | Pattern::Err(..) => true,
            Pattern::Ident(ident) => self.ident_is_enum_variant(scrutinee_type, &ident.name),
            Pattern::Or(alternatives, _) => alternatives.iter().any(|a| self.pattern_needs_check(a, scrutinee_type)),
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
            // carry the enum qualifier in the name â€” compare against the LEAF
            // segment. Also tolerate qualified/unqualified enum keys.
            let leaf = variant_name.rsplit('.').next().unwrap_or(variant_name);
            let variants = self.types.enum_variants.get(type_name)
                .or_else(|| {
                    self.types.enum_variants.iter()
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
        if !self.types.types.contains_key("Option") {
            self.types.types.insert("Option".to_string(), vec!["discriminant".to_string(), "value".to_string()]);
            self.types.type_meta.insert("Option".to_string(), TypeMeta {
                fields: vec![("discriminant".to_string(), "Int".to_string()), ("value".to_string(), "Int".to_string())],
                derives: vec![],
                invariants: vec![],
            });
        }
        if !self.types.types.contains_key("Result") {
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
        if !self.types.enum_variants.contains_key("Result") {
            self.types.enum_variants.insert("Result".to_string(), vec![
                ("Err".to_string(), vec!["error".to_string()]),
                ("Ok".to_string(), vec!["value".to_string()]),
            ]);
        }
        // Register Vec type for runtime operations ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ensure 4 fields
        // (data, len, cap, elem_size). The elem_size field tracks the
        // element byte width so narrow types (UInt8ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢1, Int16ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢2, etc.)
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
            self.types.type_meta.entry("Vec".to_string()).or_insert_with(|| TypeMeta {
                fields: full_fields.clone(),
                derives: Vec::new(),
                invariants: Vec::new(),
            });
            for key in &["xiom.collections.Vec".to_string()] {
                if let Some(meta) = self.types.type_meta.get_mut(key) {
                    if meta.fields.len() < 4 {
                        meta.fields.push(("elem_size".to_string(), "Int".to_string()));
                    }
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
            self.types.type_meta.entry("Map".to_string()).or_insert_with(|| TypeMeta {
                fields: map_full_fields,
                derives: Vec::new(),
                invariants: Vec::new(),
            });
        }

        // 5e.3: Register Layout as a builtin type so its struct definition
        // ({i64, i64}) is emitted even when alloc.xi is not compiled directly.
        // The Layout.new constructor is inlined in expr.rs; this ensures the
        // type definition exists for the emitted insertvalue instructions.
        if !self.types.type_meta.contains_key("xiom.alloc.Layout") {
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
        if !self.types.type_meta.contains_key("xiom.rc.RcInner") {
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

        // Register function signatures
        for item in &program.items {
            self.register_functions(item);
        }

        // 5e.7f: Const evaluation pass — fold const expressions after all
        // constants are registered so cross-references resolve correctly.
        self.evaluate_all_consts();

        // Scan interface implementations: for each interface, find all concrete
        // types that implement all its methods (BUG-007 interface dispatch).
        self.scan_interface_impls();

        // Emit module header
        self.emitln("; XIOM v0.50.0 LLVM IR");
        self.emitln("; Auto-generated by xiom\n");
        self.emitln(&format!("target triple = \"{}\"", self.config.target_triple));
        self.emitln("");

        // Emit builtin struct types FIRST so user types can reference them.
        // Vec is emitted from type_meta (with elem_size if registered via
        // the code below) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â NOT hardcoded so narrow-type Vecs get correct layout.
        self.emitln("");

        // Emit struct type definitions using actual field types from type_meta
        for (name, meta) in &self.types.type_meta.clone() {
            let struct_ref = format!("%struct.{name}");
            let field_types: Vec<String> = meta.fields.iter()
                .map(|(_, ty_name)| {
                    let t = self.llvm_type_for(ty_name).unwrap_or_else(|_| "i64".to_string());
                    if t == struct_ref { format!("{t}*") } else { t }
                })
                .collect();
            self.emitln(&format!("%struct.{name} = type {{ {} }}", field_types.join(", ")));
        }
        if !self.types.types.is_empty() {
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

        // 5e.5c: emit hot reload state save/restore functions
        if self.config.hot_reload && !self.config.xiom_hot_globals.is_empty() {
            self.emit_hot_state_functions();
        }

        self.emit_builtin_declares();

        // Emit declares for user-defined extern "C" functions
        // (skips names already in self.mono.already_declared, e.g. malloc)
        // Pre-seed the metadata-accessor names when their tables will be DEFINED
        // below (reflect/contracts), so the user extern block's `declare` for them
        // is skipped ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â otherwise the same symbol is both declared and defined,
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

        // Additive metadata emission: RTTI for the `reflect` stdlib module and a
        // contract table for the `contracts` stdlib module. This is a NEW,
        // self-contained step appended alongside the runtime `declare`s above.
        // It NEVER alters any existing lowering path ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â it only reads
        // already-registered type metadata (`self.types.type_meta` / `self.types.enum_variants`)
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
        // manufacture a *correct* live result ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â only unblock linking.
        self.emit_undefined_symbol_stubs();

        Ok(self.output.clone())
    }

    // Contract runtime checks → see contracts.rs


    /// Infer the struct type name from an expression (if it produces a struct value).
    fn struct_type_from_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Field(obj, field, _) => {
                // 5c.29: Enum variant literal `EnumType.Variant` (e.g.
                // `match DistanceMetric.Cosine { ... }`): the scrutinee type is
                // the enum itself. Without this, bare-ident match arms were
                // treated as bindings and dispatch fell through to the last arm.
                if let Expr::Ident(base_id) = obj.as_ref() {
                    for (ek, vars) in self.types.enum_variants.iter() {
                        if (ek == &base_id.name || ek.ends_with(&format!(".{}", base_id.name)))
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
                        if key.ends_with(&base_type) || key == &base_type {
                            if let Some(meta) = self.types.type_meta.get(key) {
                                for (fname, ftype) in &meta.fields {
                                    if fname == &field.name {
                                        let clean = ftype.trim_start_matches('*');
                                        if self.types.type_meta.contains_key(clean) {
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
            Expr::Call(func, _, _) => {
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
            Expr::Some(_, _) => {
                // Some(x) produces Option[T] ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â not a struct with user invariants
                None
            }
            Expr::Ok(_, _) | Expr::Err(_, _) => {
                None
            }
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
                // Type aliases have no fields ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â nothing to derive or invariant-check
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
        // G-13: a by-value struct return copies EVERY field — including enum
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

    /// 8B/M9: Debug derive for structs — delegates to Display for now
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
            // (a Str field is i8*, a Char field is i8, etc.) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â avoids `add i64, i8*`.
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
                // (a Str field is i8* ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ ptrtoint; a Char field is i8 ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ zext) so the
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
                        // Str stored as i64 handle in struct — inttoptr before strcmp
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
                        // 7d: Vec payload — delegate to Vec.eq() for deep comparison
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
                        // Option payload — delegate to Option.eq()
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
                        // Raw i8* pointer field — compare via strcmp
                        let cr = self.fresh_tmp();
                        let eq = self.fresh_tmp();
                        let ze = self.fresh_tmp();
                        body.push(format!("  {cr} = call i32 @strcmp(i8* {sv}, i8* {ov})"));
                        body.push(format!("  {eq} = icmp eq i32 {cr}, 0"));
                        body.push(format!("  {ze} = zext i1 {eq} to i64"));
                        body.push(format!("  ret i64 {ze}"));
                    } else if fty_name == "Float64" || fty_name == "Float32" {
                        // Float stored as i64 in struct — bitcast before fcmp
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
        let field_count = self.types.types.get(type_name).map(|f| f.len()).unwrap_or(1);
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
                self.emitln("  ret i64 0"); // same variant, no payload → equal
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

    /// 8B/M9: Debug derive for enums — delegates to Display.to_str
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
        const MAX_GENERIC_ITERATIONS: u32 = 65536;
        loop {
            iteration += 1;
            if iteration > MAX_GENERIC_ITERATIONS {
                return Err(format!(
                    "generic monomorphisation exceeded {} iterations ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â possible infinite recursion in generic definitions",
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
                None => continue,
            };
            // Emit each unique specialization at most once. Without this, a
            // generic that (transitively) instantiates itself re-queues the same
            // specialization on every worklist pass, never draining the queue ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â
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
                            // comparison ops) via inline codegen ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â see the primitive
                            // fast-path in compile_expr's method dispatch.
                            let is_builtin_method = matches!(
                                method_name.as_str(),
                                "compare" | "eq" | "ne" | "lt" | "gt" | "le" | "ge" | "hash" | "clone"
                            );
                            if is_builtin_method && Self::is_primitive_type_name(concrete_type) {
                                continue;
                            }
                            if !self.types.functions.contains_key(&method_key) {
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
            let const_map: HashMap<String, i64> = self.mono.const_value_map.get(&specialized_name).cloned().unwrap_or_default();
            for (gp, ct) in fd.generics.iter().zip(concrete_types.iter()) {
                if gp.is_const { continue; } // const params use const_map, not type_map
                type_map.insert(gp.name.name.clone(), ct.clone());
            }
            // Interface-typed params: map interface names to concrete types.
            // For `fn f(r: Reporter)` called with `GoodReporter`, map
            // "Reporter" ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ "GoodReporter" so subst_type can resolve params.
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
            let struct_types: HashSet<String> = self.types.types.keys().cloned().collect();
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
                    // Pointer / mutable-scalar-ref types: substitute the inner
                    // generic, then lower to a REAL pointer (e.g. `mem.swap[Int]`'s
                    // `&mut T` -> `i64*`). Uses the captured `struct_types` set for
                    // struct detection so the closure stays `self`-free. Without this,
                    // such params defaulted to `i64` and `from_mut(x)` fed an `i64*`
                    // address into an `i64` slot ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ miscompiled swap.
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
let subst_elem = Self::substitute_type(t, elem, &type_map);
                                let elem_name = Self::type_from_ast(&subst_elem);
                                let elem_ty = if struct_types.contains(&elem_name) {
                                    format!("%struct.{elem_name}")
                                } else {
                                    Self::xiom_to_llvm_type(&elem_name).to_string()
                                };
                                format!("{elem_ty}*")
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
                        format!("%struct.Tuple_{}", parts.join("_"))
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
            // with NO `self` param is a static constructor ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â no receiver argument.
            let has_self_param = fd.params.iter().any(|p| p.name.name == "self");
            let is_mut_self = fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self);
            let self_llvm_ty = if let (true, Some(r)) = (has_self_param, fd.receiver.as_ref()) {
                let base = self.llvm_type_for(&r.name).unwrap_or_else(|_| {
                    let search = format!(".{}", r.name);
                    for key in self.types.type_meta.keys() {
                        if key.ends_with(&search) { return format!("%struct.{key}"); }
                    }
                    "i64".to_string()
                });
                Some(if is_mut_self && base.starts_with('%') { format!("{base}*") } else { base })
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
                    let names_opt = self.types.types.get(recv_type_name).cloned()
                        .or_else(|| {
                            if let Some(ref module) = self.local.current_module {
                                let qualified = format!("{}.{}", module, recv_type_name);
                                self.types.types.get(&qualified).cloned()
                            } else {
                                self.types.types.keys().find(|k| k.ends_with(&format!(".{recv_type_name}"))).and_then(|k| self.types.types.get(k).cloned())
                            }
                        });
                    if let Some(names) = names_opt {
                        let type_key = self.types.types.get(recv_type_name).map(|_| recv_type_name.clone())
                            .or_else(|| {
                                if let Some(ref module) = self.local.current_module {
                                    let q = format!("{}.{}", module, recv_type_name);
                                    if self.types.types.contains_key(&q) { Some(q) } else { None }
                                } else { None }
                            })
                            .or_else(|| self.types.types.keys().find(|k| k.ends_with(&format!(".{recv_type_name}"))).cloned())
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
                    let names_opt = self.types.types.get(recv_type_name).cloned()
                        .or_else(|| {
                            if let Some(ref module) = self.local.current_module {
                                let qualified = format!("{}.{}", module, recv_type_name);
                                self.types.types.get(&qualified).cloned()
                            } else {
                                self.types.types.iter().find(|(k, _)| k.ends_with(&format!(".{recv_type_name}"))).map(|(_, v)| v.clone())
                            }
                        });
                    if let Some(names) = names_opt {
                        let type_key = self.types.types.get(recv_type_name).map(|_| recv_type_name.clone())
                            .or_else(|| {
                                if let Some(ref module) = self.local.current_module {
                                    let q = format!("{}.{}", module, recv_type_name);
                                    if self.types.types.contains_key(&q) { Some(q) } else { None }
                                } else { None }
                            })
                            .or_else(|| self.types.types.keys().find(|k| k.ends_with(&format!(".{recv_type_name}"))).cloned())
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
            }

            // Set type substitution map for method dispatch in body
            self.mono.current_const_map = const_map.clone();
            self.mono.current_type_map = type_map.clone();

            // Compile body
            if let Some(body) = fd.body.as_ref() {
                self.compile_block(body, fd.return_type.is_some())?;
            }

            // Clear type substitution state
            self.mono.current_type_map.clear();
            self.mono.param_concrete_types.clear();
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
                self.fctx.current_fn = None;
                self.fctx.current_receiver = None;
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
                        let ret_ty = &self.fctx.current_return_type.clone();
                        let result_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {result_alloca} = alloca {ret_ty}"));
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
                        self.emitln(&format!("  ret {ret_ty} {loaded}"));
                        last_result = Some(loaded);
                    } else if is_last && is_expression && matches!(stmt, Stmt::If(..)) {
                        // A tail `if`-expression used as the function's implicit
                        // return value: `fn f() -> T { if c { a } else { b } }`.
                        // Reuse the proven tail-Match mechanism ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â allocate a result
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
    fn infer_match_llvm_type(&self, arms: &[MatchArm]) -> String {
        let mut types: Vec<String> = Vec::new();
        for arm in arms {
            let ty = match &arm.body {
                MatchBody::Expr(e) => {
                    let t = self.infer_llvm_type(e);
                    if t.is_empty() { continue; }
                    t
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
        // Prefer a struct type (wider alloca).  If all types match the first one,
        // use it directly so the codegen sees the exact struct name.
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
        for (enum_key, vars) in &self.types.enum_variants {
            if enum_key.ends_with(&format!(".{type_seg}"))
                && vars.iter().any(|(v, _)| v == variant)
            {
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
                // Fallback: search generic_type_names ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â generic types may not
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
            Expr::Index(container, _, _) => self.resolve_vec_elem_type(container),
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
                        if key.ends_with(&base_struct) || key == &base_struct {
                            if let Some(meta) = self.types.type_meta.get(key) {
                                for (fname, ftype) in &meta.fields {
                                    if fname == &field.name {
                                        // Strip leading `*` from pointer types (e.g. `*SqliteRow`).
                                        let clean = ftype.trim_start_matches('*');
                                        if self.types.type_meta.contains_key(clean)
                                            || self.types.types.contains_key(clean)
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
            Expr::Call(func, _, _) => {
                // Infer type from the return type of a method/function call
                let fn_key = if let Expr::Field(obj, field, _) = func.as_ref() {
                    // Try module-qualified resolution first (e.g. iter.range ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ xiom.iter.range)
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

    /// Parse field LLVM types from a tuple struct name.
    /// `%struct.Tuple_Float32_Float32` → `["float", "float"]`
    /// `%struct.Tuple_Int_Float64` → `["i64", "double"]`
    fn parse_struct_field_types(&self, struct_ty: &str) -> Vec<String> {
        let name = struct_ty.trim_start_matches("%struct.");
        // Struct names for tuples have format: Tuple_Type1_Type2_... or just Type1_Type2
        let parts: Vec<&str> = name.split('_').collect();
        let mut types = Vec::new();
        for part in parts {
            if part == "Tuple" { continue; }
            // Map XIOM type names to LLVM types
            let llvm = match part {
                "Int" | "Bool" | "Int32" | "UInt32" | "UInt64" | "Int64" | "Int8" | "UInt8" | "Int16" | "UInt16" => "i64",
                "Float64" => "double",
                "Float32" => "float",
                "Char" => "i8",
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
        let meta = self.types.type_meta.get(struct_name)
            .or_else(|| {
                self.local.current_module.as_ref()
                    .and_then(|m| self.types.type_meta.get(&format!("{m}.{struct_name}")))
            })
            .or_else(|| {
                self.types.type_meta.iter()
                    .find(|(k, _)| k.ends_with(&format!(".{struct_name}")))
                    .map(|(_, v)| v)
            })?;
        meta.fields.get(field_idx).map(|(_, t)| t.clone())
    }


    /// Extract the element type from an LLVM array type like `[64 x i64]` ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ `i64`.
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
            Expr::Char(_, _) => "i8".to_string(),
            Expr::Ident(ident) => {
                if let Some((_, llvm_ty)) = self.lookup_local(&ident.name) {
                    if llvm_ty == "double" { return "double".to_string(); }
                    return llvm_ty.clone();
                }
                // If the ident is an enum variant name (e.g., DivByZero), return the parent enum's struct type
                if let Some(enum_key) = self.types.enum_variants.iter()
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
                            if let Some(meta) = self.types.type_meta.get(type_name) {
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
                        // Try to resolve method call: obj.method ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ Type.method
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
                        for (k, (_, rt)) in &self.types.functions {
                            if k.ends_with(&suffix) && rt.starts_with("%struct.") {
                                return rt.clone();
                            }
                        }
                    }
                    // Function pointer call ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â look up tracked return type
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
                if self.types.types.contains_key("Option") { "%struct.Option".to_string() } else { "i64".to_string() }
            }
            Expr::Ok(..) | Expr::Err(..) => {
                if self.types.types.contains_key("Result") { "%struct.Result".to_string() } else { "i64".to_string() }
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
            Expr::Match(_scrutinee, arms, _) => self.infer_match_llvm_type(arms),
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
                            if let Some(meta) = self.types.type_meta.get(type_name) {
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
            Expr::Call(_, _, _) | Expr::If(..) => {
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

