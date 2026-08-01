// XIOM Codegen — Statement compilation (extracted from expr.rs, M4.2)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_ast::*;
use crate::llvm_consts::*;

use super::IrEmitter;

impl IrEmitter {
    pub(crate) fn compile_stmt_impl(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, _ty, value, _) => {
                // 5c.30: Empty array `[]` assigned to a Vec-typed variable — emit
                // proper Vec initialization instead of coercing i8* to %struct.Vec.
                let is_empty_array_to_vec = matches!(value, Expr::Array(elems, _) if elems.is_empty())
                    && _ty.as_ref().map_or(false, |t| {
                        let type_name = Self::type_from_ast(t);
                        type_name == "Vec" || type_name.ends_with(".Vec")
                    });
                // Track array-literal bindings for Expr::Index dispatch
                if matches!(value, Expr::Array(..)) {
                    self.local.array_locals.insert(name.name.clone());
                    // 5c-R: record the element LLVM type for typed array indexing (G-11)
                    if let Expr::Array(elems, _) = value {
                        if let Some(first) = elems.first() {
                            let elem_ty = self.infer_llvm_type(first);
                            if elem_ty != "i64" {
                                self.local.local_array_elem.insert(name.name.clone(), elem_ty);
                            }
                        }
                        // 5c.30: record array size for const-generic inference
                        self.local.local_array_sizes.insert(name.name.clone(), elems.len() as i64);
                    }
                }
                // 5c.30: record Vec element type for local Vec bindings.
                if let Some(elem) = Self::vec_ctor_elem_type(value) {
                    self.local.local_vec_elem.insert(name.name.clone(), elem);
                } else if let Some(ty) = _ty {
                    if let Some(elem) = Self::vec_elem_from_type_annotation(ty) {
                        self.local.local_vec_elem.insert(name.name.clone(), elem);
                    } else {
                        self.local.local_vec_elem.remove(&name.name);
                    }
                } else {
                    // 5c.39: Inherit Vec element type from source local
                    // or resolve from function call return types.
                    let inherited = match value {
                        Expr::Ident(id) => self.local.local_vec_elem.get(&id.name).cloned(),
                        Expr::Call(_, args, _) => {
                            // Inherit from first argument's Vec element type
                            args.first().and_then(|a| {
                                if let Expr::Ident(id) = a {
                                    self.local.local_vec_elem.get(&id.name).cloned()
                                } else { None }
                            })
                        }
                        _ => None,
                    };
                    if let Some(elem) = inherited {
                        self.local.local_vec_elem.insert(name.name.clone(), elem);
                    } else {
                        self.local.local_vec_elem.remove(&name.name);
                    }
                }
                // M20-A1: Track closure bindings (also through parens)
                let is_closure = |e: &Expr| -> bool {
                    matches!(e, Expr::PipeClosure(..) | Expr::Closure(..))
                    || matches!(e, Expr::Paren(inner, _) if matches!(inner.as_ref(), Expr::Closure(..) | Expr::PipeClosure(..)))
                };
                if is_closure(value) {
                    self.local.closure_locals.insert(name.name.clone());
                }
                self.track_boxed_payload_binding(&name.name, value);
                // Track Option/Result payload types from the type annotation
                // so nested match dispatch works.
                if let Some(ty) = _ty {
                    if let Some(opt_inner) = Self::option_type_param(ty, "Option") {
                        self.local.local_opt_payload.entry(name.name.clone()).or_insert(opt_inner);
                    }
                    if let Some(res_val) = Self::option_type_param(ty, "Result") {
                        self.local.local_opt_payload.entry(name.name.clone()).or_insert(res_val);
                    }
                    if let Some(err_val) = Self::result_err_type_param(ty) {
                        self.local.local_err_payload.entry(name.name.clone()).or_insert(err_val);
                    }
                }
                // 5c.30: Empty array `[]` assigned to Vec-typed variable — emit
                // proper Vec initialization to avoid i8* → %struct.Vec coercion.
                if is_empty_array_to_vec {
                    let elem_size: i64 = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .and_then(|elem| {
                            // Compute struct size for known types
                            let sname = self.types.types.keys()
                                .find(|k| k.ends_with(&format!(".{}", elem)) || k.as_str() == elem)
                                .cloned()
                                .unwrap_or(elem.to_string());
                            Some(self.struct_byte_size(&sname) as i64)
                        })
                        .unwrap_or(8);
                    let initial_cap: i64 = 16;
                    let alloc_size = initial_cap * elem_size;
                    let struct_alloca = self.fresh_tmp();
                    self.emitln(&format!("  {struct_alloca} = alloca %struct.Vec"));
                    let data_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {data_ptr} = call i8* @malloc(i64 {alloc_size})"));
                    let null_check = self.fresh_tmp();
                    let ok_block = self.fresh_block("vec_init_ok");
                    let trap_block = self.fresh_block("vec_init_trap");
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
                    self.emitln(&format!("  store i64 {initial_cap}, i64* {cap_gep}"));
                    let esz_gep = self.fresh_tmp();
                    self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 3"));
                    self.emitln(&format!("  store i64 {elem_size}, i64* {esz_gep}"));
                    let loaded = self.emit_vec_load_fields(&struct_alloca);
                    self.add_local(&name.name, struct_alloca, &"%struct.Vec".to_string());
                    // Use declared element type, not hardcoded Int
                    let elem_ty = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .unwrap_or_else(|| "Int".to_string());
                    self.local.local_vec_elem.insert(name.name.clone(), elem_ty);
                    return Ok(());
                }
                let (val, val_llvm_ty) = self.compile_expr(value)?;
                let declared_llvm_ty: Option<String> = _ty.as_ref().map(|t| {
                    let name = Self::type_from_ast(t);
                    self.llvm_type_for(&name).unwrap_or_else(|_| LLVM_I64.to_string())
                });
                // M17: Track XIOM type and signedness for narrow-int widening.
                if let Some(ty) = _ty {
                    let xiom_name = Self::type_from_ast(ty);
                    self.local.local_xiom_types.insert(name.name.clone(), xiom_name.clone());
                    if Self::is_signed_xiom_type(&xiom_name) {
                        self.local.signed_locals.insert(name.name.clone());
                    } else {
                        self.local.signed_locals.remove(&name.name);
                    }
                }
                // Use declared struct type when available (handles Option.unwrap
                // round-trip where the value is a heap pointer i64 but the declared
                // type is a struct).
                // M17: Also use declared type for primitive narrow types (i8/i16/i32/float)
                // so Int8/Int16/Int32/Float32 get properly-sized allocas instead of i64.
                let llvm_ty = if declared_llvm_ty.as_ref().map_or(false, |d| d.starts_with('%')) {
                    declared_llvm_ty.clone().expect("declared_llvm_ty is Some when starts_with('%')")
                } else if let Some(ref d) = declared_llvm_ty {
                    // M17: For primitive declared types, prefer the declared type
                    // when it differs from the compiled value's LLVM type.
                    // This ensures Int8→i8, Int16→i16, Int32→i32, Float32→float.
                    // For i64 declared types where the value is also i64, keep i64.
                    if d != &val_llvm_ty || val_llvm_ty == "void" || val.is_empty() {
                        d.clone()
                    } else {
                        val_llvm_ty.clone()
                    }
                } else if val_llvm_ty == "void" || val.is_empty() {
                    declared_llvm_ty.clone().unwrap_or_else(|| LLVM_I64.to_string())
                } else {
                    val_llvm_ty.clone()
                };
                // 5c-E: If the declared type differs from the compiled value type
                // (e.g. Vec[Float32] = [] produces i8* but expected %struct.Vec),
                // coerce the value before storing. Prevents 'store %struct.Vec i8*'
                // llvm type mismatch (clang opaque pointer reject).
                let (val, val_llvm_ty) = if llvm_ty != val_llvm_ty && !val.is_empty() {
                    let coerced = self.coerce_value(&val, &val_llvm_ty, &llvm_ty);
                    (coerced, llvm_ty.clone())
                } else {
                    (val, val_llvm_ty)
                };
                // Track Bool-typed locals
                let is_bool = matches!(_ty.as_deref(), Some(Type::Named(id, _)) if id.name == "Bool")
                    || matches!(value, Expr::Bool(..))
                    || self.expr_is_bool(value);
                if is_bool { self.local.bool_locals.insert(name.name.clone()); } else { self.local.bool_locals.remove(&name.name); }
                if llvm_ty == "void" || val.is_empty() {
                    let alloca = self.fresh_tmp();
                    self.emitln(&format!("  {alloca} = alloca i64"));
                    self.emitln(&format!("  store i64 0, i64* {alloca}"));
                    self.add_local(&name.name, alloca, "i64");
                    return Ok(());
                }
                let store_val = self.coerce_value(&val, &val_llvm_ty, &llvm_ty);
                let store_val = self.zero_val_for(&store_val, &llvm_ty);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {alloca}"));
                self.add_local(&name.name, alloca, &llvm_ty);
                // Check invariants if the value is a struct with invariants
                if self.config.check_contracts {
                    self.maybe_check_value_invariants(value, &val);
                }
            }
            Stmt::Var(name, _ty, value, _) => {
                // M20-A1: Track closure bindings (also through parens)
                if matches!(value, Expr::PipeClosure(..) | Expr::Closure(..))
                    || matches!(value, Expr::Paren(inner, _) if matches!(inner.as_ref(), Expr::Closure(..) | Expr::PipeClosure(..)))
                {
                    self.local.closure_locals.insert(name.name.clone());
                }
                // Track array-literal bindings for Expr::Index dispatch
                if matches!(value, Expr::Array(..)) {
                    self.local.array_locals.insert(name.name.clone());
                    // 5c-R: record the element LLVM type for typed array indexing (G-11)
                    if let Expr::Array(elems, _) = value {
                        if let Some(first) = elems.first() {
                            let elem_ty = self.infer_llvm_type(first);
                            if elem_ty != "i64" {
                                self.local.local_array_elem.insert(name.name.clone(), elem_ty);
                            }
                        }
                        // 5c.30: record array size for const-generic inference
                        self.local.local_array_sizes.insert(name.name.clone(), elems.len() as i64);
                    }
                }
                // 5c.30: record Vec element type for local Vec bindings.
                if let Some(elem) = Self::vec_ctor_elem_type(value) {
                    self.local.local_vec_elem.insert(name.name.clone(), elem);
                } else if let Some(ty) = _ty {
                    if let Some(elem) = Self::vec_elem_from_type_annotation(ty) {
                        self.local.local_vec_elem.insert(name.name.clone(), elem);
                    } else {
                        self.local.local_vec_elem.remove(&name.name);
                    }
                } else {
                    // 5c.39: Inherit Vec element type for Var binding
                    let inherited = match value {
                        Expr::Ident(id) => self.local.local_vec_elem.get(&id.name).cloned(),
                        Expr::Call(_, args, _) => {
                            args.first().and_then(|a| {
                                if let Expr::Ident(id) = a {
                                    self.local.local_vec_elem.get(&id.name).cloned()
                                } else { None }
                            })
                        }
                        _ => None,
                    };
                    if let Some(elem) = inherited {
                        self.local.local_vec_elem.insert(name.name.clone(), elem);
                    } else {
                        self.local.local_vec_elem.remove(&name.name);
                    }
                }
                self.track_boxed_payload_binding(&name.name, value);
                // Track Option/Result payload types from the type annotation
                // so nested match dispatch works.
                if let Some(ty) = _ty {
                    if let Some(opt_inner) = Self::option_type_param(ty, "Option") {
                        self.local.local_opt_payload.entry(name.name.clone()).or_insert(opt_inner);
                    }
                    if let Some(res_val) = Self::option_type_param(ty, "Result") {
                        self.local.local_opt_payload.entry(name.name.clone()).or_insert(res_val);
                    }
                    if let Some(err_val) = Self::result_err_type_param(ty) {
                        self.local.local_err_payload.entry(name.name.clone()).or_insert(err_val);
                    }
                }
                // 5c.30: Empty array `[]` assigned to Vec-typed variable — emit
                // proper Vec initialization.
                let is_empty_array_to_vec_var = matches!(value, Expr::Array(elems, _) if elems.is_empty())
                    && _ty.as_ref().map_or(false, |t| {
                        let type_name = Self::type_from_ast(t);
                        type_name == "Vec" || type_name.ends_with(".Vec")
                    });
                if is_empty_array_to_vec_var {
                    let elem_size: i64 = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .and_then(|elem| {
                            let sname = self.types.types.keys()
                                .find(|k| k.ends_with(&format!(".{}", elem)) || k.as_str() == elem)
                                .cloned()
                                .unwrap_or(elem.to_string());
                            Some(self.struct_byte_size(&sname) as i64)
                        })
                        .unwrap_or(8);
                    let initial_cap: i64 = 16;
                    let struct_alloca = self.fresh_tmp();
                    self.emitln(&format!("  {struct_alloca} = alloca %struct.Vec"));
                    let data_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {data_ptr} = call i8* @malloc(i64 {})", initial_cap * elem_size));
                    let null_check = self.fresh_tmp();
                    let ok_block = self.fresh_block("vec_var_ok");
                    let trap_block = self.fresh_block("vec_var_trap");
                    self.emitln(&format!("  {null_check} = icmp eq i8* {data_ptr}, null"));
                    self.emitln(&format!("  br i1 {null_check}, label %{trap_block}, label %{ok_block}"));
                    self.emitln(&format!("\n{trap_block}:"));
                    self.emitln("  call void @llvm.trap()");
                    self.emitln("  unreachable");
                    self.emitln(&format!("\n{ok_block}:"));
                    let dg = self.fresh_tmp(); self.emitln(&format!("  {dg} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 0"));
                    self.emitln(&format!("  store i8* {data_ptr}, i8** {dg}"));
                    let lg = self.fresh_tmp(); self.emitln(&format!("  {lg} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 1"));
                    self.emitln(&format!("  store i64 0, i64* {lg}"));
                    let cg = self.fresh_tmp(); self.emitln(&format!("  {cg} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 2"));
                    self.emitln(&format!("  store i64 {initial_cap}, i64* {cg}"));
                    let eg = self.fresh_tmp(); self.emitln(&format!("  {eg} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 3"));
                    self.emitln(&format!("  store i64 {elem_size}, i64* {eg}"));
                    let loaded = self.emit_vec_load_fields(&struct_alloca);
                    self.add_local(&name.name, struct_alloca, &"%struct.Vec".to_string());
                    // Use the declared element type if available, not hardcoded Int
                    let elem_ty = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .unwrap_or_else(|| "Int".to_string());
                    self.local.local_vec_elem.insert(name.name.clone(), elem_ty);
                    return Ok(());
                }
                let declared_llvm_ty: Option<String> = _ty.as_ref().map(|t| {
                    let name = Self::type_from_ast(t);
                    self.llvm_type_for(&name).unwrap_or_else(|_| LLVM_I64.to_string())
                });
                // M17: Track XIOM type and signedness for narrow-int widening.
                if let Some(ty) = _ty {
                    let xiom_name = Self::type_from_ast(ty);
                    self.local.local_xiom_types.insert(name.name.clone(), xiom_name.clone());
                    if Self::is_signed_xiom_type(&xiom_name) {
                        self.local.signed_locals.insert(name.name.clone());
                    } else {
                        self.local.signed_locals.remove(&name.name);
                    }
                }
                let (val, val_llvm_ty) = if let Expr::Array(elems, _) = value {
                    // 5c.39: Non-empty array literal assigned to a Vec-typed
                    // variable — convert to Vec via compile_array_as_vec.
                    let elem_ty = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .unwrap_or_else(|| "Int".to_string());
                    self.compile_array_as_vec(elems, &elem_ty)?
                } else {
                    self.compile_expr(value)?
                };
                let orig_val_ty = val_llvm_ty.clone();
                // M17: Use declared type for alloca width when present, falling back
                // to value type. Special cases preserved for zero-init and float→double.
                let llvm_ty = if val_llvm_ty == "i64" && val == "0" {
                    declared_llvm_ty.clone().unwrap_or(val_llvm_ty)
                } else if val_llvm_ty == "void" || val.is_empty() {
                    declared_llvm_ty.clone().unwrap_or_else(|| LLVM_I64.to_string())
                } else if let Some(ref d) = declared_llvm_ty {
                    // M17: When a type annotation exists, prefer the declared type
                    // for the alloca width. This ensures Int8→i8, Int16→i16, etc.
                    // Struct types (starts_with '%') and Float32 special case were
                    // already handled; this generalizes to all declared types.
                    if d.starts_with('%') || d != &val_llvm_ty {
                        d.clone()
                    } else {
                        val_llvm_ty
                    }
                } else {
                    val_llvm_ty
                };
                // 5c-E: If declared type differs from value type, coerce before storing.
                let (val, _val_llvm_ty) = if llvm_ty != orig_val_ty && !val.is_empty() {
                    let coerced = self.coerce_value(&val, &orig_val_ty, &llvm_ty);
                    (coerced, llvm_ty.clone())
                } else {
                    (val, orig_val_ty.clone())
                };
                let is_bool = matches!(_ty.as_deref(), Some(Type::Named(id, _)) if id.name == "Bool")
                    || matches!(value, Expr::Bool(..))
                    || self.expr_is_bool(value);
                if is_bool { self.local.bool_locals.insert(name.name.clone()); } else { self.local.bool_locals.remove(&name.name); }
                if llvm_ty == "void" || val.is_empty() {
                    let alloca = self.fresh_tmp();
                    self.emitln(&format!("  {alloca} = alloca i64"));
                    self.emitln(&format!("  store i64 0, i64* {alloca}"));
                    self.add_local(&name.name, alloca, "i64");
                    return Ok(());
                }
                let store_val = self.zero_val_for(&val, &llvm_ty);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {alloca}"));
                self.add_local(&name.name, alloca, &llvm_ty);
                // Check invariants if the value is a struct with invariants
                if self.config.check_contracts {
                    self.maybe_check_value_invariants(value, &val);
                }
            }
            Stmt::Assign(place, value, _) => {
                // Deref write: `*p = v` (Unary Deref) or `*p = v` via a `&mut`-wrapped
                // place. Compile the pointer, then store the value through it. Handled
                // BEFORE the value is compiled for the plain-ident path so the store
                // uses the pointee type. Only fires for real pointer operands.
                if let Expr::Unary(UnaryOp::Deref, inner, _) = place {
                    let (ptr_val, ptr_ty) = self.compile_expr(inner)?;
                    if ptr_ty.ends_with('*') {
                        let pointee = ptr_ty.trim_end_matches('*').to_string();
                        let (val, val_ty) = self.compile_expr(value)?;
                        let store_val = self.coerce_value(&val, &val_ty, &pointee);
                        self.emitln(&format!("  store {pointee} {store_val}, {ptr_ty} {ptr_val}"));
                        return Ok(());
                    }
                    // 5c.31: Legacy erased-to-i64 path — the pointer value is
                    // held as an i64 (e.g. from `&mut x` ptrtoint). Resolve the
                    // pointee type from the inner expression's XIOM type and
                    // emit inttoptr + store through the real pointer.
                    if ptr_ty == "i64" {
                        let pointee_llvm = if let Expr::Ident(id) = inner.as_ref() {
                            self.local.local_xiom_types.get(&id.name)
                                .and_then(|xiom_ty| {
                                    let stripped = xiom_ty.trim_start_matches("&mut ")
                                        .trim_start_matches('&')
                                        .trim_start_matches('*');
                                    self.llvm_type_for(stripped).ok()
                                })
                        } else { None };
                        let pointee = pointee_llvm.unwrap_or_else(|| LLVM_I64.to_string());
                        let (val, val_ty) = self.compile_expr(value)?;
                        let real_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {real_ptr} = inttoptr i64 {ptr_val} to {pointee}*"));
                        let store_val = self.coerce_value(&val, &val_ty, &pointee);
                        self.emitln(&format!("  store {pointee} {store_val}, {pointee}* {real_ptr}"));
                        return Ok(());
                    }
                }
                let (val, val_ty) = self.compile_expr(value)?;
                if let Expr::Ident(ident) = place {
                    if let Some((ptr, llvm_ty)) = self.lookup_local(&ident.name).cloned() {
                        // Coerce the value to the slot's declared type using the
                        // value's REAL type from compile_expr (e.g. an i8 char
                        // value assigned into an i64 slot).
                        let store_val = self.coerce_value(&val, &val_ty, &llvm_ty);
                        self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {ptr}"));
                    } else if let Some((symbol, llvm_ty)) = self.local.module_globals.get(&ident.name).cloned() {
                        // Assignment to a mutable module-level `var`: store into the
                        // real global so the write persists across calls. Checked
                        // BEFORE the unknown-target path so it is never silently
                        // dropped.
                        let store_val = self.coerce_value(&val, &val_ty, &llvm_ty);
                        self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* @{symbol}"));
                    }
                }
                // M20: Nested field assignment support via compile_lvalue.
                // Only fires for field chain assignments (a.b.c = value),
                // not simple ident, deref, or index assignments.
                if matches!(place, Expr::Field(..)) {
                    if let Some((l_ptr, l_ptr_ty, l_elem_ty)) = self.compile_lvalue(place) {
                        let store_val = self.coerce_value(&val, &val_ty, &l_elem_ty);
                        self.emitln(&format!("  store {l_elem_ty} {store_val}, {l_ptr_ty} {l_ptr}"));
                        return Ok(());
                    }
                }
                // Indexed assignment: `container[idx] = value` into a Vec (builtin
                // {i8*, i64, i64}) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â write an i64-wide slot at data[idx]. Str is
                // immutable at the ABI, so only Vec/Slice are handled.
                if let Expr::Index(container, index, _) = place {
                    let (cont_val, cont_ty) = self.compile_expr(container)?;
                    let (vec_val, vec_ty) = self.resolve_vec_receiver(container, &cont_val, &cont_ty);
                    let is_vec = vec_ty == "%struct.Vec" || vec_ty.ends_with(".Vec")
                        || vec_ty.contains("struct.Vec")
                        || vec_ty == "%struct.Slice" || vec_ty.contains("struct.Slice");
                    if is_vec {
                        let (idx_raw, idx_ty) = self.compile_expr(index)?;
                        let idx = self.val_to_i64(&idx_raw, &idx_ty);
                        let store_i64 = self.val_to_i64(&val, &val_ty);
                        let vslot = self.fresh_tmp();
                        self.emitln(&format!("  {vslot} = alloca %struct.Vec"));
                        self.emit_vec_store_fields(&vec_val, &vslot);
                        // Load elem_size from field 3
                        let esz_gep = self.fresh_tmp();
                        let esz_val = self.fresh_tmp();
                        self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {vslot}, i32 0, i32 3"));
                        self.emitln(&format!("  {esz_val} = load i64, i64* {esz_gep}"));
                        let data_gep = self.fresh_tmp();
                        self.emitln(&format!("  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {vslot}, i32 0, i32 0"));
                        let data_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                        let byte_off = self.fresh_tmp();
                        self.emitln(&format!("  {byte_off} = mul i64 {idx}, {esz_val}"));
                        let elem_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {data_ptr}, i64 {byte_off}"));
                        self.emit_elem_store(&store_i64, &elem_ptr, &esz_val);
                    } else if cont_ty.starts_with('[') && cont_ty.contains(" x ") {
                        let (idx_raw, idx_ty) = self.compile_expr(index)?;
                        let idx = self.val_to_i64(&idx_raw, &idx_ty);
                        // Use the existing local alloca when the container is an
                        // Ident ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â avoids fresh alloca/load/store on every write.
                        let mut is_ident = false;
                        let mut arr_ptr = String::new();
                        let mut arr_ptr_ty = String::new();
                        if let Expr::Ident(id) = &**container {
                            if let Some((slot, _slot_ty)) = self.lookup_local(&id.name).cloned() {
                                (arr_ptr, arr_ptr_ty) = (slot, format!("{cont_ty}*"));
                                is_ident = true;
                            }
                        }
                        if !is_ident {
                            arr_ptr = self.fresh_tmp();
                            arr_ptr_ty = format!("{cont_ty}*");
                            self.emitln(&format!("  {arr_ptr} = alloca {cont_ty}"));
                            self.emitln(&format!("  store {cont_ty} {cont_val}, {cont_ty}* {arr_ptr}"));
                        }
                        let elem_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {elem_ptr} = getelementptr {cont_ty}, {arr_ptr_ty} {arr_ptr}, i64 0, i64 {idx}"));
                        let inner_ty = Self::extract_array_elem_ty(&cont_ty);
                        let store_val = self.coerce_value(&val, &val_ty, &inner_ty);
                        self.emitln(&format!("  store {inner_ty} {store_val}, {inner_ty}* {elem_ptr}"));
                        if !is_ident {
                            // Only need load+store_back when using a fresh alloca
                            let loaded_arr = self.fresh_tmp();
                            self.emitln(&format!("  {loaded_arr} = load {cont_ty}, {cont_ty}* {arr_ptr}"));
                            self.store_back_to_receiver(container, &loaded_arr, &cont_ty);
                        }
                    } else if cont_ty == "i8*" {
                        // Raw byte-buffer store: `buf[i] = v` where `buf: *UInt8`.
                        // The element is one byte; truncate the value to i8. Without
                        // this, `buf[i] = ...` silently emitted nothing (the store was
                        // dropped), leaving heap buffers uninitialized ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ crashes in
                        // str_concat/str_upper/str_slice and other manual builders.
                        let (idx_raw, idx_ty) = self.compile_expr(index)?;
                        let idx = self.val_to_i64(&idx_raw, &idx_ty);
                        let store_i8 = self.coerce_value(&val, &val_ty, "i8");
                        let elem_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {cont_val}, i64 {idx}"));
                        self.emitln(&format!("  store i8 {store_i8}, i8* {elem_ptr}"));
                    } else if cont_ty == "i64" && self.is_ptr_local_expr(container) {
                        // Raw byte-buffer store where the pointer is held in an i64
                        // (a `*T` param lowered to i64): inttoptr then store the byte.
                        let (idx_raw, idx_ty) = self.compile_expr(index)?;
                        let idx = self.val_to_i64(&idx_raw, &idx_ty);
                        let store_i8 = self.coerce_value(&val, &val_ty, "i8");
                        let base_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {base_ptr} = inttoptr i64 {cont_val} to i8*"));
                        let elem_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {base_ptr}, i64 {idx}"));
                        self.emitln(&format!("  store i8 {store_i8}, i8* {elem_ptr}"));
                    }
                }
                // Emit invariant check if the assigned place is a struct with invariants
                if let Expr::Field(obj, field, _) = place {
                    // --- Deref-field write: `(*ptr).field = value` ---
                    // Handle through-pointer field stores (e.g. `(*raw).value = v`
                    // in Cell.set). Matches `(*p).f` and `(*(p)).f` nesting.
                    {
                        let deref_inner: Option<&Expr> = match obj.as_ref() {
                            Expr::Unary(UnaryOp::Deref, inner, _) => Some(inner.as_ref()),
                            Expr::Paren(p, _) => match p.as_ref() {
                                Expr::Unary(UnaryOp::Deref, inner, _) => Some(inner.as_ref()),
                                _ => None,
                            },
                            _ => None,
                        };
                        if let Some(inner) = deref_inner {
                            let (ptr_val, ptr_ty) = self.compile_expr(inner)?;
                            if ptr_ty.ends_with('*') {
                                let pointee = ptr_ty.trim_end_matches('*').to_string();
                                if pointee.starts_with("%struct.") {
                                    let type_name = &pointee[8..];
                        if let Some(field_names) = self.types.types.get(type_name)
                            .or_else(|| {
                                let suffix = format!(".{type_name}");
                                self.types.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                    .and_then(|k| self.types.types.get(k))
                            })
                            .cloned()
                        {
                                        if let Some(field_idx) = field_names.iter().position(|f| f == &field.name) {
                                            let field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                                            let gep = self.fresh_tmp();
                                            let store_val = self.coerce_value(&val, &val_ty, &field_llvm_ty);
                                            self.emitln(&format!("  {gep} = getelementptr {pointee}, {ptr_ty} {ptr_val}, i32 0, i32 {field_idx}"));
                                            self.emitln(&format!("  store {field_llvm_ty} {store_val}, {field_llvm_ty}* {gep}"));
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // --- Regular field assignment: `obj.field = value` ---
                    if let Expr::Ident(obj_ident) = obj.as_ref() {
                        if let Some((obj_ptr, obj_ty)) = self.lookup_local(&obj_ident.name).cloned() {
                            if obj_ty.starts_with("%struct.") {
                                // 5c.29: support POINTER bases (&mut T params like
                                // `req: &mut HttpRequest`): load the pointer from
                                // its alloca, then GEP through it. Previously the
                                // trailing '*' made the type lookup fail and the
                                // whole assignment was silently dropped.
                                let is_ptr_base = obj_ty.ends_with('*');
                                let type_name = obj_ty[8..].trim_end_matches('*').to_string();
                                let base_ty = format!("%struct.{type_name}");
                                let base_ptr = if is_ptr_base {
                                    let p = self.fresh_tmp();
                                    self.emitln(&format!("  {p} = load {obj_ty}, {obj_ty}* {obj_ptr}"));
                                    p
                                } else {
                                    obj_ptr.clone()
                                };
                                // GEP to field and store (with qualified-name fallback)
                                if let Some(field_names) = self.types.types.get(&type_name)
                                    .or_else(|| {
                                        let suffix = format!(".{type_name}");
                                        self.types.types.keys().find(|k| k.ends_with(&suffix))
                                            .and_then(|k| self.types.types.get(k))
                                    })
                                    .cloned()
                                {
                                    if let Some(field_idx) = field_names.iter().position(|f| f == &field.name) {
                                        let field_llvm_ty = self.field_llvm_type(&type_name, field_idx);
                                        // Generic container fields hold i64 handles:
                                        // box a by-value header before storing.
                                        let store_val = if field_llvm_ty == "i64"
                                            && val_ty.starts_with("%struct.")
                                            && !val_ty.ends_with('*')
                                            && self.field_xiom_type(&type_name, field_idx)
                                                .map_or(false, |t| t.contains('['))
                                        {
                                            self.emit_box_struct_handle(&val, &val_ty)
                                        } else {
                                            self.coerce_value(&val, &val_ty, &field_llvm_ty)
                                        };
                                        let gep = self.fresh_tmp();
                                        self.emitln(&format!("  {gep} = getelementptr {base_ty}, {base_ty}* {base_ptr}, i32 0, i32 {field_idx}"));
                                        self.emitln(&format!("  store {field_llvm_ty} {store_val}, {field_llvm_ty}* {gep}"));
                                    }
                                }
                                let has_invariants = self.types.type_meta.get(&type_name)
                                    .map(|m| !m.invariants.is_empty())
                                    .unwrap_or(false);
                                if has_invariants {
                                    let loaded = self.fresh_tmp();
                                    self.emitln(&format!("  {loaded} = load {base_ty}, {base_ty}* {base_ptr}"));
                                    self.compile_invariant_call(&type_name, &loaded);
                                }
                            }
                        }
                    }
                }
            }
            Stmt::Return(expr, _) => {
                if let Some(e) = expr {
                    // Value sink: use the value's real LLVM type from compile_expr.
                    let (mut val, val_ty) = self.compile_expr(e)?;
                    let ret_ty = self.fctx.current_return_type.clone();
                    // Coerce the returned value to the function's declared return
                    // type (int widths, int<->pointer, int<->double, int->struct)
                    // so the `ret` instruction is well-typed.
                    val = self.coerce_value(&val, &val_ty, &ret_ty);
                    // Store result for ensures checks
                    if let Some(res_ptr) = self.fctx.result_ptr.as_ref() {
                        let ret_ty = self.fctx.current_return_type.clone();
                        self.emitln(&format!("  store {ret_ty} {val}, {ret_ty}* {res_ptr}"));
                    }
                    // Check ensures before returning
                    if !self.fctx.current_ensures.is_empty() {
                        self.compile_ensures_checks();
                    }
                    let ret_ty = self.fctx.current_return_type.clone();
                    // Decrement recursion depth
                    let depth_dec = self.fresh_tmp();
                    self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
                    let new_depth_dec = self.fresh_tmp();
                    self.emitln(&format!("  {new_depth_dec} = sub i64 {depth_dec}, 1"));
                    self.emitln(&format!("  store i64 {new_depth_dec}, i64* @xiom_recursion_counter"));
                    self.emitln(&format!("  ret {ret_ty} {val}"));
                } else {
                    if !self.fctx.current_ensures.is_empty() {
                        self.compile_ensures_checks();
                    }
                    // Decrement recursion depth
                    let depth_dec = self.fresh_tmp();
                    self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
                    let new_depth_dec = self.fresh_tmp();
                    self.emitln(&format!("  {new_depth_dec} = sub i64 {depth_dec}, 1"));
                    self.emitln(&format!("  store i64 {new_depth_dec}, i64* @xiom_recursion_counter"));
                    // M16: `return;` in a void-typed function emits `ret void`,
                    // but `main` is always lowered to `i64` so the process exit
                    // code is well-defined.
                    let ret_ty = self.fctx.current_return_type.clone();
                    if ret_ty == "void" {
                        self.emitln("  ret void");
                    } else {
                        self.emitln(&format!("  ret {ret_ty} 0"));
                    }
                }
            }
            Stmt::Expr(expr, _) => {
                self.compile_expr(expr)?;
            }
            Stmt::If(cond, then_block, elifs, else_block, _) => {
                let (cond_raw, cond_ty) = self.compile_expr(cond)?;
                let cond_val = if cond_ty == "i1" {
                    cond_raw
                } else if cond_ty == "i64" {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = icmp ne i64 {cond_raw}, 0"));
                    tmp
                } else if cond_ty.starts_with("%struct.") {
                    // M17: Struct-typed conditions use always-true since
                    // icmp can't compare structs directly.
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = icmp ne i64 1, 0"));
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
                    let (econd_raw, econd_ty) = self.compile_expr(econd)?;
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
                    // No else case for simple if ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the else block is just a merge jump
                    self.emitln(&format!("\n{prev_label}:"));
                    self.emitln(&format!("  br label %{merge_label}"));
                    merge_reachable = true;
                } else if elifs.is_empty() {
                    // prev_label == merge_label ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚ skip redundant label emission
                    merge_reachable = true;
                } else if prev_label != merge_label {
                    self.emitln(&format!("\n{prev_label}:"));
                    self.emitln(&format!("  br label %{merge_label}"));
                    merge_reachable = true;
                } else {
                    // 5c.29: non-empty elif chain with NO else: the last elif's
                    // false edge branches DIRECTLY to the merge label, so the
                    // merge block is reachable. Previously this case fell
                    // through with merge_reachable=false, emitting a stray
                    // `unreachable` ahead of live code (HTTP from_str trap).
                    merge_reachable = true;
                }

                self.emitln(&format!("\n{merge_label}:"));
                if !merge_reachable {
                    self.emitln("  unreachable");
                }
            }
            Stmt::Match(expr_match, arms, _) => {
                // 5c.37: Initialize match result slot when called from Expr::Match
                // wrapper. The wrapper allocates the slot before dispatching here.
                if let (Some(ptr), Some(ty)) = (self.fctx.match_result_ptr.clone(), self.fctx.match_result_ty.clone()) {
                    if ty.starts_with("%struct.") {
                        self.emitln(&format!("  store {ty} zeroinitializer, {ty}* {ptr}"));
                    } else if ty.ends_with('*') {
                        self.emitln(&format!("  store {ty} null, {ty}* {ptr}"));
                    } else {
                        self.emitln(&format!("  store {ty} 0, {ty}* {ptr}"));
                    }
                }
                let (mut val, mut scrutinee_llvm_ty) = self.compile_expr(expr_match)?;
                let merge_label = self.fresh_block("match_merge");

                // Determine scrutinee type for variant pattern matching
                let mut scrutinee_type = self.struct_type_from_expr(expr_match);

                // 5c.29: An enum payload unwrapped from Option/Result arrives as
                // an i64 BOX POINTER with the static type erased (val_to_i64
                // boxes struct payloads on the heap). If the arm patterns name
                // variants of exactly ONE known enum, adopt that enum as the
                // scrutinee type and load the struct through the pointer so
                // discriminant checks work (`match m.unwrap() { PUT => ... }`).
                if scrutinee_type.is_none() && scrutinee_llvm_ty == "i64" {
                    let names: Vec<&str> = arms.iter()
                        .filter_map(|a| match &a.pattern {
                            Pattern::Ident(id) => Some(id.name.as_str()),
                            Pattern::Variant(name, _, _) => Some(name.name.as_str()),
                            _ => None,
                        })
                        .collect();
                    if !names.is_empty() {
                        let cands: Vec<String> = self.types.enum_variants.iter()
                            .filter(|(_, vars)| names.iter().all(|n| vars.iter().any(|(v, _)| v == n)))
                            .map(|(k, _)| k.clone())
                            .collect();
                        if cands.len() == 1 {
                            let ek = cands.into_iter().next().expect("at least one enum key candidate");
                            let struct_ty = format!("%struct.{ek}");
                            let ptr = self.fresh_tmp();
                            self.emitln(&format!("  {ptr} = inttoptr i64 {val} to {struct_ty}*"));
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {ptr}"));
                            val = loaded;
                            scrutinee_llvm_ty = struct_ty;
                            scrutinee_type = Some(ek);
                        }
                    }
                }

                // Store scrutinee value in alloca for field extraction
                let mut scrutinee_alloca_info = None;
                if let Some(ref type_name) = scrutinee_type {
                    let struct_ty = format!("%struct.{type_name}");
                    let alloca = self.fresh_tmp();
                    // If the scrutinee is a pointer to the struct (e.g. JsonValue*)
                    // rather than the struct value itself, load the struct through
                    // the pointer before storing in the match alloca.
                    let store_val = if scrutinee_llvm_ty.ends_with('*') && struct_ty == scrutinee_llvm_ty.trim_end_matches('*') {
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {struct_ty}, {scrutinee_llvm_ty} {val}"));
                        loaded
                    } else {
                        self.zero_val_for(&val, &struct_ty)
                    };
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
                    } else if arm.guard.is_none() && matches!(&arm.pattern, Pattern::Wildcard(_) | Pattern::Ident(_)) {
                        // M18: Only UNGUARDED wildcard/ident arms can serve as the
                        // catch-all default. Guarded arms need explicit evaluation.
                        wildcard_idx = Some(i);
                    }
                }

                // Branch to the first check block (or straight to the default
                // arm / merge block when there are no checks). All indexing is
                // bounds-guarded.
                // M18: If there are guarded arms before the wildcard, branch to the
                // first arm so the guard chain can evaluate. Unguarded wildcards are
                // only used as the fallback target within guard chains.
                let has_guard_before_wildcard = arms.iter().any(|a| a.guard.is_some());
                if let Some(first_check) = check_labels.first() {
                    self.emitln(&format!("  br label %{first_check}"));
                } else if let Some(wi) = wildcard_idx {
                    if has_guard_before_wildcard {
                        // Branch to first arm; guard failure chains to wildcard
                        if let Some(first_label) = arm_labels.first() {
                            self.emitln(&format!("  br label %{first_label}"));
                        } else {
                            self.emitln(&format!("  br label %{merge_label}"));
                        }
                    } else {
                        let target = arm_labels.get(wi).cloned().unwrap_or_else(|| merge_label.clone());
                        self.emitln(&format!("  br label %{target}"));
                    }
                } else if let Some(first_label) = arm_labels.first() {
                    self.emitln(&format!("  br label %{first_label}"));
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
                // bug degrades to a branch-to-merge instead of a panic ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â a
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
                        // Safety fallback ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â unreachable once the build/emit
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
                        Pattern::Or(alternatives, _) => {
                            // For or-patterns with guards that bind variables,
                            // pre-create a shared alloca BEFORE the check blocks
                            // so it dominates all or_bind blocks and the guard can
                            // load from it regardless of which alternative matched.
                            let shared_slot: Option<(String, String, String)> = if arm.guard.is_some() {
                                alternatives.iter().find_map(|alt| {
                                    if let Pattern::Variant(vn, fields, _) = alt {
                                        if !fields.is_empty() && scrutinee_alloca_info.is_some() {
                                            let (_, type_name, _) = scrutinee_alloca_info.as_ref().unwrap();
                                            let leaf = vn.name.rsplit('.').next().unwrap_or(&vn.name);
                                            self.types.enum_variants.get(type_name)
                                                .and_then(|vars| vars.iter().find(|(v, _)| v == leaf || v == &vn.name))
                                                .and_then(|(_, vfs)| vfs.first())
                                                .and_then(|canonical| {
                                                    self.types.types.get(type_name)
                                                        .and_then(|fns| fns.iter().position(|f| f == canonical))
                                                })
                                                .map(|fi| {
                                                    let llvm_ty = self.field_llvm_type(type_name, fi);
                                                    let alloca = self.fresh_tmp();
                                                    self.emitln(&format!("  {alloca} = alloca {llvm_ty}"));
                                                    let var_name = fields[0].name.clone();
                                                    self.add_local(&var_name, alloca.clone(), &llvm_ty);
                                                    (alloca, llvm_ty, var_name)
                                                })
                                        } else { None }
                                    } else { None }
                                })
                            } else { None };
                            for (ai, alt) in alternatives.iter().enumerate() {
                                let is_last = ai == alternatives.len() - 1;
                                let fail_block = if is_last {
                                    next.clone()
                                } else {
                                    let fl = format!("match_or_fail_{i}_{ai}");
                                    self.fresh_block(&fl)
                                };
                                match alt {
                                    Pattern::Lit(Literal::Int(n, _)) => {
                                        let c = self.fresh_tmp();
                                        self.emitln(&format!("  {c} = icmp eq i64 {val}, {n}"));
                                        self.emitln(&format!("  br i1 {c}, label %{arm_label}, label %{fail_block}"));
                                    }
                                    Pattern::Lit(Literal::Bool(b, _)) => {
                                        let c = self.fresh_tmp();
                                        let bv = if *b { "1" } else { "0" };
                                        self.emitln(&format!("  {c} = icmp eq i64 {val}, {bv}"));
                                        self.emitln(&format!("  br i1 {c}, label %{arm_label}, label %{fail_block}"));
                                    }
                                    Pattern::Lit(Literal::Str(s, _)) => {
                                        let cstr = self.intern_cstring(s);
                                        let cmp = self.fresh_tmp();
                                        self.emitln(&format!("  {cmp} = call i32 @strcmp(i8* {val}, i8* {cstr})"));
                                        let eq = self.fresh_tmp();
                                        self.emitln(&format!("  {eq} = icmp eq i32 {cmp}, 0"));
                                        self.emitln(&format!("  br i1 {eq}, label %{arm_label}, label %{fail_block}"));
                                    }
                                    Pattern::Ident(id) => {
                                        self.emit_variant_discriminant_check(&id.name, &scrutinee_alloca_info, &val, &arm_label, &fail_block);
                                    }
                                    Pattern::Variant(vn, fields, _) => {
                                        // For or-patterns with guards, bind the payload BEFORE
                                        // jumping to the shared arm so the guard sees the correct value.
                                        let mut did_bind = false;
                                        if arm.guard.is_some() && !fields.is_empty() && scrutinee_alloca_info.is_some() {
                                            let (alloca, type_name, struct_ty) = scrutinee_alloca_info.as_ref().unwrap();
                                            let leaf = vn.name.rsplit('.').next().unwrap_or(&vn.name).to_string();
                                            // Collect all info before mutating self
                                            let variant_idx_opt = self.types.enum_variants.get(type_name)
                                                .and_then(|vars| vars.iter().position(|(vn2, _)| vn2 == &leaf || vn2 == &vn.name));
                                            let canonical_opt = variant_idx_opt.and_then(|vi| {
                                                self.types.enum_variants.get(type_name)
                                                    .and_then(|vars| vars.get(vi))
                                                    .and_then(|(_, vfs)| vfs.first().cloned())
                                            });
                                            let fi_opt = canonical_opt.as_ref().and_then(|canonical| {
                                                self.types.types.get(type_name)
                                                    .and_then(|fns| fns.iter().position(|f| f == canonical))
                                            });
                                            if let (Some(variant_idx), Some(fi)) = (variant_idx_opt, fi_opt) {
                                                let field_llvm_ty = self.field_llvm_type(type_name, fi);
                                                let field_ident = fields[0].clone();
                                                let alloca_c = alloca.clone();
                                                let struct_ty_c = struct_ty.clone();
                                                let disc_gep = self.fresh_tmp();
                                                let disc_val = self.fresh_tmp();
                                                self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty_c}, {struct_ty_c}* {alloca_c}, i32 0, i32 0"));
                                                self.emitln(&format!("  {disc_val} = load i64, i64* {disc_gep}"));
                                                let check = self.fresh_tmp();
                                                self.emitln(&format!("  {check} = icmp eq i64 {disc_val}, {variant_idx}"));
                                                let bind_block = self.fresh_block("or_bind");
                                                self.emitln(&format!("  br i1 {check}, label %{bind_block}, label %{fail_block}"));
                                                self.emitln(&format!("\n{bind_block}:"));
                                                let gep = self.fresh_tmp();
                                                self.emitln(&format!("  {gep} = getelementptr {struct_ty_c}, {struct_ty_c}* {alloca_c}, i32 0, i32 {fi}"));
                                                let loaded = self.fresh_tmp();
                                                self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                                // Use shared alloca from pre-loop, or create one if not available
                                                let (shared_alloca, shared_ty) = if let Some((ref sa, ref st, _)) = shared_slot {
                                                    (sa.clone(), st.clone())
                                                } else {
                                                    let sa = self.fresh_tmp();
                                                    self.emitln(&format!("  {sa} = alloca {field_llvm_ty}"));
                                                    (sa, field_llvm_ty.clone())
                                                };
                                                self.emitln(&format!("  store {shared_ty} {loaded}, {shared_ty}* {shared_alloca}"));
                                                self.add_local(&field_ident.name, shared_alloca, &shared_ty);
                                                self.emitln(&format!("  br label %{arm_label}"));
                                                did_bind = true;
                                            }
                                        }
                                        if !did_bind {
                                            self.emit_variant_discriminant_check(&vn.name, &scrutinee_alloca_info, &val, &arm_label, &fail_block);
                                        }
                                    }
                                    Pattern::Some(inner, _) | Pattern::Ok(inner, _) => {
                                        // OR alternative with Some/Ok: check discriminant == 1,
                                        // plus inner literal if present (e.g. Some('t'))
                                        if let Some((alloca, _type_name, struct_ty)) = &scrutinee_alloca_info {
                                            let disc_gep = self.fresh_tmp();
                                            let disc_val = self.fresh_tmp();
                                            self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                                            self.emitln(&format!("  {disc_val} = load i64, i64* {disc_gep}"));
                                            let disc_check = self.fresh_tmp();
                                            self.emitln(&format!("  {disc_check} = icmp eq i64 {disc_val}, 1"));
                                            // If inner is a literal, add value check too (M18: unified Char+Int)
                                            let lit_val: Option<i64> = match inner.as_ref() {
                                                Pattern::Lit(Literal::Char(ch, _)) => Some(*ch as i64),
                                                Pattern::Lit(Literal::Int(n, _)) => Some(*n as i64),
                                                _ => None,
                                            };
                                            if let Some(v) = lit_val {
                                                let inner_ok = self.fresh_block("or_inner_ok");
                                                self.emitln(&format!("  br i1 {disc_check}, label %{inner_ok}, label %{fail_block}"));
                                                self.emitln(&format!("\n{inner_ok}:"));
                                                let val_gep = self.fresh_tmp();
                                                let val_loaded = self.fresh_tmp();
                                                let val_check = self.fresh_tmp();
                                                self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
                                                self.emitln(&format!("  {val_loaded} = load i64, i64* {val_gep}"));
                                                self.emitln(&format!("  {val_check} = icmp eq i64 {val_loaded}, {v}"));
                                                self.emitln(&format!("  br i1 {val_check}, label %{arm_label}, label %{fail_block}"));
                                            } else {
                                                self.emitln(&format!("  br i1 {disc_check}, label %{arm_label}, label %{fail_block}"));
                                            }
                                        } else {
                                            self.emitln(&format!("  br label %{fail_block}"));
                                        }
                                    }
                                    Pattern::None(..) | Pattern::Err(..) => {
                                        // OR alternative with None/Err: check discriminant == 0
                                        if let Some((alloca, _type_name, struct_ty)) = &scrutinee_alloca_info {
                                            let disc_gep = self.fresh_tmp();
                                            let disc_val = self.fresh_tmp();
                                            self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                                            self.emitln(&format!("  {disc_val} = load i64, i64* {disc_gep}"));
                                            let check = self.fresh_tmp();
                                            self.emitln(&format!("  {check} = icmp eq i64 {disc_val}, 0"));
                                            self.emitln(&format!("  br i1 {check}, label %{arm_label}, label %{fail_block}"));
                                        } else {
                                            self.emitln(&format!("  br label %{fail_block}"));
                                        }
                                    }
                                    _ => { self.emitln(&format!("  br label %{arm_label}")); }
                                }
                                if !is_last {
                                    self.emitln(&format!("\n{fail_block}:"));
                                }
                            }
                        }
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
                        Pattern::Lit(Literal::Str(s, _)) => {
                            let cstr = self.intern_cstring(s);
                            let cmp = self.fresh_tmp();
                            self.emitln(&format!("  {cmp} = call i32 @strcmp(i8* {val}, i8* {cstr})"));
                            let eq = self.fresh_tmp();
                            self.emitln(&format!("  {eq} = icmp eq i32 {cmp}, 0"));
                            self.emitln(&format!("  br i1 {eq}, label %{arm_label}, label %{next}"));
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
                        Pattern::Some(..) | Pattern::None(..) | Pattern::Ok(..) | Pattern::Err(..) => {
                            // Builtin Option/Result variant dispatch: check the
                            // discriminant and, for Some(pat)/Ok(pat) with a literal
                            // inner pattern, also check the payload value.
                            let expected_disc: i64 = match &arm.pattern {
                                Pattern::Some(..) | Pattern::Ok(..) => 1,
                                Pattern::None(..) | Pattern::Err(..) => 0,
                                _ => unreachable!("match arm pattern is neither Option nor Result discriminant"),
                            };
                            let inner_pat: Option<&Pattern> = match &arm.pattern {
                                Pattern::Some(inner, _) | Pattern::Ok(inner, _) => Some(inner.as_ref()),
                                _ => None,
                            };
                            if let Some((alloca, _type_name, struct_ty)) = &scrutinee_alloca_info {
                                let disc_gep = self.fresh_tmp();
                                let disc_val = self.fresh_tmp();
                                self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                                self.emitln(&format!("  {disc_val} = load i64, i64* {disc_gep}"));
                                let disc_check = self.fresh_tmp();
                                self.emitln(&format!("  {disc_check} = icmp eq i64 {disc_val}, {expected_disc}"));
                                // For Some(inner_lit) / Ok(inner_lit) with a literal
                                // inner pattern, add a second check on the payload.
                                // Handle both Char and Int literals (M18).
                                let lit_val: Option<i64> = match inner_pat {
                                    Some(Pattern::Lit(Literal::Char(ch, _))) => Some(*ch as i64),
                                    Some(Pattern::Lit(Literal::Int(n, _))) => Some(*n as i64),
                                    _ => None,
                                };
                                if let (1i64, Some(val)) = (expected_disc, lit_val) {
                                    let inner_ok = self.fresh_block("match_inner_ok");
                                    self.emitln(&format!("  br i1 {disc_check}, label %{inner_ok}, label %{next}"));
                                    self.emitln(&format!("\n{inner_ok}:"));
                                    let val_gep = self.fresh_tmp();
                                    let val_loaded = self.fresh_tmp();
                                    let val_check = self.fresh_tmp();
                                    self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
                                    self.emitln(&format!("  {val_loaded} = load i64, i64* {val_gep}"));
                                    self.emitln(&format!("  {val_check} = icmp eq i64 {val_loaded}, {val}"));
                                    self.emitln(&format!("  br i1 {val_check}, label %{arm_label}, label %{next}"));
                                } else {
                                    self.emitln(&format!("  br i1 {disc_check}, label %{arm_label}, label %{next}"));
                                }
                            } else {
                                self.emitln(&format!("  br label %{arm_label}"));
                            }
                        }
                        _ => {
                            // Wildcard and non-checkable patterns: keep the block valid.
                            self.emitln(&format!("  br label %{arm_label}"));
                        }
                    }

                    check_idx += 1;
                }

                // Emit arm bodies
                for (i, arm) in arms.iter().enumerate() {
                    let arm_label = arm_labels.get(i).cloned().unwrap_or_else(|| merge_label.clone());
                    self.emitln(&format!("\n{arm_label}:"));
                    // M18: For guarded arms, pre-bind Ident patterns so the guard
                    // can reference the bound variable, then compile the guard.
                    // On guard failure, skip to the next arm.
                    if arm.guard.is_some() {
                        self.push_scope();
                        // Pre-bind Ident pattern for guard access
                        if let Pattern::Ident(ident) = &arm.pattern {
                            let is_variant = scrutinee_type.as_ref().and_then(|tn| {
                                self.types.enum_variants.get(tn)
                                    .map(|vars| vars.iter().any(|(v, _)| v == &ident.name))
                            }).unwrap_or(false);
                            if !is_variant {
                                let bind_ty = if scrutinee_llvm_ty.is_empty() || scrutinee_llvm_ty == "void" {
                                    LLVM_I64.to_string()
                                } else {
                                    scrutinee_llvm_ty.clone()
                                };
                                let store_val = self.zero_val_for(&val, &bind_ty);
                                let match_alloca = self.fresh_tmp();
                                self.emitln(&format!("  {match_alloca} = alloca {bind_ty}"));
                                self.emitln(&format!("  store {bind_ty} {store_val}, {bind_ty}* {match_alloca}"));
                                self.add_local(&ident.name, match_alloca, &bind_ty);
                            }
                        }
                        // M18: Pre-extract Ok/Some/Err payloads for guard access.
                        // The payload field is loaded from the scrutinee struct's alloca
                        // and bound as a local so guard expressions can reference it.
                        let payload_field: Option<(&Pattern, i32)> = match &arm.pattern {
                            Pattern::Some(inner, _) | Pattern::Ok(inner, _) => Some((inner.as_ref(), 1)),
                            Pattern::Err(inner, _) => Some((inner.as_ref(), 2)),
                            _ => None,
                        };
                        if let Some((inner_pat, field_idx)) = payload_field {
                            if let Pattern::Ident(ident) = inner_pat {
                                if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                                    let gep = self.fresh_tmp();
                                    self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {field_idx}"));
                                    let field_llvm_ty = self.field_llvm_type(type_name, field_idx as usize);
                                    let loaded = self.fresh_tmp();
                                    self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                    let inner_alloca = self.fresh_tmp();
                                    self.emitln(&format!("  {inner_alloca} = alloca {field_llvm_ty}"));
                                    self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {inner_alloca}"));
                                    self.add_local(&ident.name, inner_alloca, &field_llvm_ty);
                                    // Track XIOM type for method dispatch (e.g. Str.len())
                                    if let Some(xiom_ty) = self.field_xiom_type(type_name, field_idx as usize) {
                                        self.local.local_xiom_types.insert(ident.name.clone(), xiom_ty);
                                    }
                                }
                            }
                        }
                        // M18: Pre-extract custom enum variant payloads for guard access.
                        // For patterns like `Data(v) if v > 10` or `P(x, y) if x+y > 10`,
                        // all payload fields must be bound BEFORE the guard is evaluated.
                        if let Pattern::Variant(variant_ident, fields, _) = &arm.pattern {
                            if !fields.is_empty() {
                                // Clone field names to avoid borrow conflicts
                                let field_names_clone: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                                let variant_name_clone = variant_ident.name.clone();
                                if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                                    let leaf_variant = variant_name_clone.rsplit('.').next().unwrap_or(&variant_name_clone).to_string();
                                    let variant_info = self.types.enum_variants.get(type_name)
                                        .and_then(|vars| vars.iter().find(|(vn, _)| vn == &leaf_variant || vn == &variant_name_clone))
                                        .map(|(_, vfs)| vfs.clone());
                                    let field_name_map = self.types.types.get(type_name).cloned();
                                    if let (Some(vfields), Some(field_names)) = (variant_info, field_name_map) {
                                        let alloca_c = alloca.clone();
                                        let struct_ty_c = struct_ty.clone();
                                        let type_name_c = type_name.clone();
                                        for (field_idx, canonical) in vfields.iter().enumerate() {
                                            if field_idx < field_names_clone.len() {
                                                if let Some(fi) = field_names.iter().position(|f| f == canonical) {
                                                    let field_llvm_ty = self.field_llvm_type(&type_name_c, fi);
                                                    let gep = self.fresh_tmp();
                                                    self.emitln(&format!("  {gep} = getelementptr {struct_ty_c}, {struct_ty_c}* {alloca_c}, i32 0, i32 {fi}"));
                                                    let loaded = self.fresh_tmp();
                                                    self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                                    let inner_alloca = self.fresh_tmp();
                                                    self.emitln(&format!("  {inner_alloca} = alloca {field_llvm_ty}"));
                                                    self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {inner_alloca}"));
                                                    self.add_local(&field_names_clone[field_idx], inner_alloca, &field_llvm_ty);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Compile guard expression and check result
                        if let Some(ref guard_expr) = arm.guard {
                            // Determine fallback label
                            let guard_fail = if i + 1 < arm_labels.len() {
                                arm_labels[i + 1].clone()
                            } else {
                                merge_label.clone()
                            };
                            let (guard_val, guard_ty) = self.compile_expr(guard_expr)?;
                            let guard_i1 = if guard_ty == "i1" { guard_val } else {
                                let ne = self.fresh_tmp();
                                self.emitln(&format!("  {ne} = icmp ne i64 {guard_val}, 0"));
                                ne
                            };
                            let guard_ok = self.fresh_block("guard_ok");
                            self.emitln(&format!("  br i1 {guard_i1}, label %{guard_ok}, label %{guard_fail}"));
                            self.emitln(&format!("\n{guard_ok}:"));
                        }
                    }
                    // For variant patterns, extract fields before compiling arm body
                    if let Pattern::Variant(variant_ident, fields, _) = &arm.pattern {
                        if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                            let field_names_opt = self.types.types.get(type_name).cloned();
                            if let Some(field_names) = field_names_opt {
                                let type_name_clone = type_name.clone();
                                let variants_opt = self.types.enum_variants.get(type_name).cloned();
                                for (fi, field_ident) in fields.iter().enumerate() {
                                    // First try direct name match
                                    let field_idx_opt = field_names.iter().position(|f| f == &field_ident.name);
                                    // Fallback: use variant field position to find canonical field name.
                                    // 5c.30: qualified variant patterns (`JsonValue.Array(x)`)
                                    // must match on the LEAF variant name.
                                    let leaf_variant = variant_ident.name.rsplit('.').next().unwrap_or(&variant_ident.name).to_string();
                                    let field_idx_opt = field_idx_opt.or_else(|| {
                                        variants_opt.as_ref().and_then(|variants| {
                                            variants.iter().find(|(vn, _)| vn == &leaf_variant || vn == &variant_ident.name)
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
                                        // 5c.30: resolve the VARIANT's declared payload
                                        // type (type_meta dedups payload fields by name,
                                        // losing per-variant types).
                                        let payload_xiom_ty: Option<String> = self.types.enum_variant_field_types.get(&type_name_clone)
                                            .or_else(|| {
                                                self.types.enum_variant_field_types.iter()
                                                    .find(|(k, _)| k.ends_with(&format!(".{type_name_clone}")))
                                                    .map(|(_, v)| v)
                                            })
                                            .and_then(|vft| vft.iter().find(|(vn, _)| vn == &leaf_variant || vn == &variant_ident.name))
                                            .and_then(|(_, ftypes)| ftypes.get(fi))
                                            .cloned();
                                        // Float payloads are RAW BITS in the i64 slot:
                                        // bind them as real floats via bitcast so
                                        // downstream math never sitofp's bit patterns
                                        // (json_stringify(Number(7.0)) hung in a
                                        // float_to_int loop over 4.6e18).
                                        let (bind_val, bind_ty): (String, String) = match (field_llvm_ty.as_str(), payload_xiom_ty.as_deref()) {
                                            ("i64", Some("Float64")) | ("i64", Some("Float")) => {
                                                let d = self.fresh_tmp();
                                                self.emitln(&format!("  {d} = bitcast i64 {loaded} to double"));
                                                (d, "double".to_string())
                                            }
                                            ("i64", Some("Float32")) => {
                                                let t32 = self.fresh_tmp();
                                                self.emitln(&format!("  {t32} = trunc i64 {loaded} to i32"));
                                                let f = self.fresh_tmp();
                                                self.emitln(&format!("  {f} = bitcast i32 {t32} to float"));
                                                (f, "float".to_string())
                                            }
                                            // M19: Str payload stored in i64 slot via ptrtoint —
                                            // convert back to i8* for string operations.
                                            ("i64", Some("Str")) => {
                                                let sptr = self.fresh_tmp();
                                                self.emitln(&format!("  {sptr} = inttoptr i64 {loaded} to i8*"));
                                                (sptr, LLVM_STR_PTR.to_string())
                                            }
                                            _ => (loaded.clone(), field_llvm_ty.clone()),
                                        };
                                        let field_alloca = self.fresh_tmp();
                                        self.emitln(&format!("  {field_alloca} = alloca {bind_ty}"));
                                        self.emitln(&format!("  store {bind_ty} {bind_val}, {bind_ty}* {field_alloca}"));
                                        self.add_local(&field_ident.name, field_alloca, &bind_ty);
                                        // 5c.30: a generic-container payload
                                        // (Vec[T]) binds the i64 HANDLE â€” record
                                        // it so `items.push(..)` / `items.len()`
                                        // dereference the boxed header and
                                        // mutations alias the original enum.
                                        self.local.local_vec_handle.remove(&field_ident.name);
                                        if let Some(fty) = payload_xiom_ty.as_deref() {
                                            if let Some(inner) = fty.strip_prefix("Vec[").and_then(|s| s.strip_suffix(']')) {
                                                self.local.local_vec_handle.insert(field_ident.name.clone(), inner.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // For Ident patterns, bind the matched value to the identifier
                    // (skip for enum variant names, which are handled by the check block)
                    if let Pattern::Ident(ident) = &arm.pattern {
                        let is_variant = scrutinee_type.as_ref().and_then(|tn| {
                            self.types.enum_variants.get(tn)
                                .map(|vars| vars.iter().any(|(v, _)| v == &ident.name))
                        }).unwrap_or(false);
                        if !is_variant {
                            // Bind the scrutinee value to the pattern variable using
                            // its REAL LLVM type (a struct scrutinee like LogLevel
                            // must not be stored as i64). Fall back to i64 for a
                            // scalar/empty value.
                            let bind_ty = if scrutinee_llvm_ty.is_empty() || scrutinee_llvm_ty == "void" {
                                LLVM_I64.to_string()
                            } else {
                                scrutinee_llvm_ty.clone()
                            };
                            let store_val = self.zero_val_for(&val, &bind_ty);
                            let match_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {match_alloca} = alloca {bind_ty}"));
                            self.emitln(&format!("  store {bind_ty} {store_val}, {bind_ty}* {match_alloca}"));
                            self.add_local(&ident.name, match_alloca, &bind_ty);
                        }
                    }
                    // Handle Some(inner) / Ok(inner) payload extraction:
                    // extract field 1 (the payload) and bind to the inner pattern.
                    // Err(inner) extracts field 2 (the error payload).
                    // 5d: bindings are TYPED from the scrutinee's declared payload
                    // types (local_opt_payload / local_err_payload) so `.len()` etc.
                    // dispatch correctly (fixes match Ok(bytes) → bytes.len()).
                    let payload_binding: Option<(&Pattern, i32)> = match &arm.pattern {
                        Pattern::Some(inner, _) | Pattern::Ok(inner, _) => Some((inner.as_ref(), 1)),
                        Pattern::Err(inner, _) => Some((inner.as_ref(), 2)),
                        _ => None,
                    };
                    if let Some((inner, val_field)) = payload_binding {
                        if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                            let val_gep = self.fresh_tmp();
                            self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {val_field}"));
                            let field_ty = self.field_llvm_type(type_name, val_field as usize);
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load {field_ty}, {field_ty}* {val_gep}"));
                            if let Pattern::Ident(ident) = inner {
                                // Declared payload type from scrutinee tracking
                                let declared: Option<String> = if let Expr::Ident(sid) = expr_match {
                                    if val_field == 2 {
                                        self.local.local_err_payload.get(&sid.name).cloned()
                                    } else {
                                        self.local.local_opt_payload.get(&sid.name).cloned()
                                    }
                                } else { None };

                                let (bind_val_inner, bind_ty_inner) = match declared.as_deref() {
                                    Some("Str") if field_ty == "i64" => {
                                        let sptr = self.fresh_tmp();
                                        self.emitln(&format!("  {sptr} = inttoptr i64 {loaded} to i8*"));
                                        (sptr, "i8*".to_string())
                                    }
                                    Some("Float64") if field_ty == "i64" => {
                                        let f = self.fresh_tmp();
                                        self.emitln(&format!("  {f} = bitcast i64 {loaded} to double"));
                                        (f, "double".to_string())
                                    }
                                    // Struct payload: the i64 is a heap pointer to a
                                    // boxed struct (Option/Result/enum). Load the struct
                                    // so nested match dispatch works.
                                    Some(decl) if field_ty == "i64" && !decl.starts_with("Vec[") => {
                                        let s_ty = self.llvm_type_for(decl).unwrap_or_else(|_| format!("%struct.{decl}"));
                                        if s_ty.starts_with('%') {
                                            let sptr = self.fresh_tmp();
                                            self.emitln(&format!("  {sptr} = inttoptr i64 {loaded} to {s_ty}*"));
                                            let sload = self.fresh_tmp();
                                            self.emitln(&format!("  {sload} = load volatile {s_ty}, {s_ty}* {sptr}"));
                                            // Track inner variable's struct type for subsequent matches
                                            self.local.local_boxed_struct.insert(ident.name.clone(), decl.to_string());
                                            (sload, s_ty)
                                        } else {
                                            (loaded.clone(), field_ty.clone())
                                        }
                                    }
                                    _ => {
                                        // M12: Handle the case where field_ty is i64 (fallback for unregistered structs)
                                        // but the actual XIOM type is a pointer (Str, *T, etc.)
                                        if field_ty == "i64" {
                                            let xiom_ty = self.field_xiom_type(type_name, val_field as usize);
                                            if let Some(ref xt) = xiom_ty {
                                                if xt == "Str" || xt.starts_with('*') {
                                                    let sptr = self.fresh_tmp();
                                                    self.emitln(&format!("  {sptr} = inttoptr i64 {loaded} to i8*"));
                                                     (sptr, "i8*".to_string())
                                                } else {
                                                    (loaded.clone(), field_ty.clone())
                                                }
                                            } else {
                                                (loaded.clone(), field_ty.clone())
                                            }
                                        } else {
                                            (loaded.clone(), field_ty.clone())
                                        }
                                    },
                                };
                                let field_alloca = self.fresh_tmp();
                                self.emitln(&format!("  {field_alloca} = alloca {bind_ty_inner}"));
                                self.emitln(&format!("  store {bind_ty_inner} {bind_val_inner}, {bind_ty_inner}* {field_alloca}"));
                                self.add_local(&ident.name, field_alloca, &bind_ty_inner);
                                // Vec[T] payloads are container HANDLES: register so
                                // len/push/index dereference the boxed Vec header.
                                self.local.local_vec_handle.remove(&ident.name);
                                if let Some(decl_ty) = declared.as_deref() {
                                    if let Some(elem) = decl_ty.strip_prefix("Vec[").and_then(|s| s.strip_suffix(']')) {
                                        self.local.local_vec_handle.insert(ident.name.clone(), elem.to_string());
                                    }
                                }
                            }
                        }
                    }
                    match &arm.body {
                        MatchBody::Block(b) => { self.compile_block(b, false)?; }
                        MatchBody::Expr(e) => {
                            let (arm_val, arm_val_ty) = self.compile_expr(e)?;
                            if let Some(ptr) = self.fctx.match_result_ptr.clone() {
                                let ret_ty = self.fctx.match_result_ty.clone().unwrap_or_else(|| self.fctx.current_return_type.clone());
                                // Coerce the arm value's REAL type to the result slot
                                // type. An arm producing a bare scalar (e.g. a literal
                                // wrapped into Option/Result) must be widened into the
                                // struct rather than emitting `store volatile %struct.X 34`.
                                let store_val = self.coerce_value(&arm_val, &arm_val_ty, &ret_ty);
                                self.emitln(&format!("  store {ret_ty} {store_val}, {ret_ty}* {ptr}"));
                            }
                        }
                    }
                    self.emitln(&format!("  br label %{merge_label}"));
                }

                self.emitln(&format!("\n{merge_label}:"));
            }
            Stmt::While(cond, body, _, _) => {
                let loop_cond = self.fresh_block("while_cond");
                let loop_body = self.fresh_block("while_body");
                let loop_exit = self.fresh_block("while_exit");

                self.emitln(&format!("  br label %{loop_cond}"));
                self.emitln(&format!("\n{loop_cond}:"));
                let (cond_raw, cond_ty) = self.compile_expr(cond)?;
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
                self.local.loop_stack.push((loop_cond.clone(), loop_exit.clone()));
                self.compile_block(body, false)?;
                self.local.loop_stack.pop();
                self.emitln(&format!("  br label %{loop_cond}"));
                self.emitln(&format!("\n{loop_exit}:"));
            }
            Stmt::For(_, _, body, _) => {
                // Phase 0: simplified for ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â just execute body once
                self.compile_block(body, false)?;
            }
            Stmt::Destructure(names, value, _) => {
                // Value sink: use the value's real LLVM type from compile_expr.
                let (val, llvm_ty) = self.compile_expr(value)?;
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
            Stmt::Break(..) => {
                if let Some((_, break_label)) = self.local.loop_stack.last().cloned() {
                    self.emitln(&format!("  br label %{break_label}"));
                    let dead = self.fresh_block("after_break");
                    self.emitln(&format!("\n{dead}:"));
                }
            }
            Stmt::Continue(..) => {
                if let Some((cont_label, _)) = self.local.loop_stack.last().cloned() {
                    self.emitln(&format!("  br label %{cont_label}"));
                    let dead = self.fresh_block("after_continue");
                    self.emitln(&format!("\n{dead}:"));
                }
            }
        }
        Ok(())
    }
}
