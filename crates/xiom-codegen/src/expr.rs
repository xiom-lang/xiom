use xiom_ast::*;
use std::collections::HashMap;

use super::IrEmitter;

impl IrEmitter {
    pub(crate) fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, _ty, value, _) => {
                // Track array-literal bindings for Expr::Index dispatch
                if matches!(value, Expr::Array(..)) {
                    self.array_locals.insert(name.name.clone());
                    // 5c-R: record the element LLVM type for typed array indexing (G-11)
                    if let Expr::Array(elems, _) = value {
                        if let Some(first) = elems.first() {
                            let elem_ty = self.infer_llvm_type(first);
                            if elem_ty != "i64" {
                                self.local_array_elem.insert(name.name.clone(), elem_ty);
                            }
                        }
                        // 5c.30: record array size for const-generic inference
                        self.local_array_sizes.insert(name.name.clone(), elems.len() as i64);
                    }
                }
                // 5c.30: record Vec element type for local Vec bindings.
                if let Some(elem) = Self::vec_ctor_elem_type(value) {
                    self.local_vec_elem.insert(name.name.clone(), elem);
                } else {
                    self.local_vec_elem.remove(&name.name);
                }
                self.track_boxed_payload_binding(&name.name, value);
                let (val, val_llvm_ty) = self.compile_expr(value)?;
                let declared_llvm_ty: Option<String> = _ty.as_ref().map(|t| {
                    let name = Self::type_from_ast(t);
                    self.llvm_type_for(&name).unwrap_or_else(|_| "i64".to_string())
                });
                // Use declared struct type when available (handles Option.unwrap
                // round-trip where the value is a heap pointer i64 but the declared
                // type is a struct).
                let llvm_ty = if declared_llvm_ty.as_ref().map_or(false, |d| d.starts_with('%')) {
                    declared_llvm_ty.clone().unwrap()
                } else if val_llvm_ty == "void" || val.is_empty() {
                    declared_llvm_ty.clone().unwrap_or_else(|| "i64".to_string())
                } else {
                    val_llvm_ty.clone()
                };
                // Track Bool-typed locals
                let is_bool = matches!(_ty.as_deref(), Some(Type::Named(id, _)) if id.name == "Bool")
                    || matches!(value, Expr::Bool(..))
                    || self.expr_is_bool(value);
                if is_bool { self.bool_locals.insert(name.name.clone()); } else { self.bool_locals.remove(&name.name); }
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
                if self.check_contracts {
                    self.maybe_check_value_invariants(value, &val);
                }
            }
            Stmt::Var(name, _ty, value, _) => {
                // Track array-literal bindings for Expr::Index dispatch
                if matches!(value, Expr::Array(..)) {
                    self.array_locals.insert(name.name.clone());
                    // 5c-R: record the element LLVM type for typed array indexing (G-11)
                    if let Expr::Array(elems, _) = value {
                        if let Some(first) = elems.first() {
                            let elem_ty = self.infer_llvm_type(first);
                            if elem_ty != "i64" {
                                self.local_array_elem.insert(name.name.clone(), elem_ty);
                            }
                        }
                        // 5c.30: record array size for const-generic inference
                        self.local_array_sizes.insert(name.name.clone(), elems.len() as i64);
                    }
                }
                // 5c.30: record Vec element type for local Vec bindings.
                if let Some(elem) = Self::vec_ctor_elem_type(value) {
                    self.local_vec_elem.insert(name.name.clone(), elem);
                } else {
                    self.local_vec_elem.remove(&name.name);
                }
                self.track_boxed_payload_binding(&name.name, value);
                let declared_llvm_ty: Option<String> = _ty.as_ref().map(|t| {
                    let name = Self::type_from_ast(t);
                    self.llvm_type_for(&name).unwrap_or_else(|_| "i64".to_string())
                });
                let (val, val_llvm_ty) = self.compile_expr(value)?;
                let llvm_ty = if val_llvm_ty == "i64" && val == "0" {
                    declared_llvm_ty.clone().unwrap_or(val_llvm_ty)
                } else if val_llvm_ty == "void" || val.is_empty() {
                    declared_llvm_ty.clone().unwrap_or_else(|| "i64".to_string())
                } else if declared_llvm_ty.as_ref().map_or(false, |d| d.starts_with('%')) {
                    // Declared type is a struct ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â prefer it over the value's
                    // raw i64 type (handles Option.unwrap() round-trip where
                    // the heap pointer needs inttoptr+load coercion).
                    declared_llvm_ty.clone().unwrap()
                } else {
                    val_llvm_ty
                };
                let is_bool = matches!(_ty.as_deref(), Some(Type::Named(id, _)) if id.name == "Bool")
                    || matches!(value, Expr::Bool(..))
                    || self.expr_is_bool(value);
                if is_bool { self.bool_locals.insert(name.name.clone()); } else { self.bool_locals.remove(&name.name); }
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
                if self.check_contracts {
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
                    // Not a real pointer (legacy erased-to-i64 path): fall through so
                    // the value is still evaluated for side effects; nothing stored.
                }
                let (val, val_ty) = self.compile_expr(value)?;
                if let Expr::Ident(ident) = place {
                    if let Some((ptr, llvm_ty)) = self.lookup_local(&ident.name).cloned() {
                        // Coerce the value to the slot's declared type using the
                        // value's REAL type from compile_expr (e.g. an i8 char
                        // value assigned into an i64 slot).
                        let store_val = self.coerce_value(&val, &val_ty, &llvm_ty);
                        self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {ptr}"));
                    } else if let Some((symbol, llvm_ty)) = self.module_globals.get(&ident.name).cloned() {
                        // Assignment to a mutable module-level `var`: store into the
                        // real global so the write persists across calls. Checked
                        // BEFORE the unknown-target path so it is never silently
                        // dropped.
                        let store_val = self.coerce_value(&val, &val_ty, &llvm_ty);
                        self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* @{symbol}"));
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
                        if let Some(field_names) = self.types.get(type_name)
                            .or_else(|| {
                                let suffix = format!(".{type_name}");
                                self.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                    .and_then(|k| self.types.get(k))
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
                                if let Some(field_names) = self.types.get(&type_name)
                                    .or_else(|| {
                                        let suffix = format!(".{type_name}");
                                        self.types.keys().find(|k| k.ends_with(&suffix))
                                            .and_then(|k| self.types.get(k))
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
                                let has_invariants = self.type_meta.get(&type_name)
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
                    let ret_ty = self.current_return_type.clone();
                    // Coerce the returned value to the function's declared return
                    // type (int widths, int<->pointer, int<->double, int->struct)
                    // so the `ret` instruction is well-typed.
                    val = self.coerce_value(&val, &val_ty, &ret_ty);
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
                let (cond_raw, cond_ty) = self.compile_expr(cond)?;
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
                        let cands: Vec<String> = self.enum_variants.iter()
                            .filter(|(_, vars)| names.iter().all(|n| vars.iter().any(|(v, _)| v == n)))
                            .map(|(k, _)| k.clone())
                            .collect();
                        if cands.len() == 1 {
                            let ek = cands.into_iter().next().unwrap();
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
                                    Pattern::Ident(id) => {
                                        self.emit_variant_discriminant_check(&id.name, &scrutinee_alloca_info, &val, &arm_label, &fail_block);
                                    }
                                    Pattern::Variant(vn, _, _) => {
                                        self.emit_variant_discriminant_check(&vn.name, &scrutinee_alloca_info, &val, &arm_label, &fail_block);
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
                                            // If inner is a literal, add value check too
                                            match inner.as_ref() {
                                                Pattern::Lit(Literal::Char(ch, _)) => {
                                                    let inner_ok = self.fresh_block("or_inner_ok");
                                                    self.emitln(&format!("  br i1 {disc_check}, label %{inner_ok}, label %{fail_block}"));
                                                    self.emitln(&format!("\n{inner_ok}:"));
                                                    let val_gep = self.fresh_tmp();
                                                    let val_loaded = self.fresh_tmp();
                                                    let val_check = self.fresh_tmp();
                                                    let ch_val = *ch as u32 as i64;
                                                    self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
                                                    self.emitln(&format!("  {val_loaded} = load i64, i64* {val_gep}"));
                                                    self.emitln(&format!("  {val_check} = icmp eq i64 {val_loaded}, {ch_val}"));
                                                    self.emitln(&format!("  br i1 {val_check}, label %{arm_label}, label %{fail_block}"));
                                                }
                                                Pattern::Lit(Literal::Int(n, _)) => {
                                                    let inner_ok = self.fresh_block("or_inner_ok");
                                                    self.emitln(&format!("  br i1 {disc_check}, label %{inner_ok}, label %{fail_block}"));
                                                    self.emitln(&format!("\n{inner_ok}:"));
                                                    let val_gep = self.fresh_tmp();
                                                    let val_loaded = self.fresh_tmp();
                                                    let val_check = self.fresh_tmp();
                                                    self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
                                                    self.emitln(&format!("  {val_loaded} = load i64, i64* {val_gep}"));
                                                    self.emitln(&format!("  {val_check} = icmp eq i64 {val_loaded}, {n}"));
                                                    self.emitln(&format!("  br i1 {val_check}, label %{arm_label}, label %{fail_block}"));
                                                }
                                                _ => {
                                                    self.emitln(&format!("  br i1 {disc_check}, label %{arm_label}, label %{fail_block}"));
                                                }
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
                                    _ => { self.emitln(&format!("  br label %{fail_block}")); }
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
                                _ => unreachable!(),
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
                                if let (1i64, Some(_pat @ Pattern::Lit(Literal::Char(ch, _)))) = (expected_disc, inner_pat) {
                                    let inner_ok = self.fresh_block("match_inner_ok");
                                    self.emitln(&format!("  br i1 {disc_check}, label %{inner_ok}, label %{next}"));
                                    self.emitln(&format!("\n{inner_ok}:"));
                                    let val_gep = self.fresh_tmp();
                                    let val_loaded = self.fresh_tmp();
                                    let val_check = self.fresh_tmp();
                                    let ch_val = *ch as u32 as i64;
                                    self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
                                    self.emitln(&format!("  {val_loaded} = load i64, i64* {val_gep}"));
                                    self.emitln(&format!("  {val_check} = icmp eq i64 {val_loaded}, {ch_val}"));
                                    self.emitln(&format!("  br i1 {val_check}, label %{arm_label}, label %{next}"));
                                } else if let (1i64, Some(Pattern::Lit(Literal::Int(n, _)))) = (expected_disc, inner_pat) {
                                    let inner_ok = self.fresh_block("match_inner_ok");
                                    self.emitln(&format!("  br i1 {disc_check}, label %{inner_ok}, label %{next}"));
                                    self.emitln(&format!("\n{inner_ok}:"));
                                    let val_gep = self.fresh_tmp();
                                    let val_loaded = self.fresh_tmp();
                                    let val_check = self.fresh_tmp();
                                    self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
                                    self.emitln(&format!("  {val_loaded} = load i64, i64* {val_gep}"));
                                    self.emitln(&format!("  {val_check} = icmp eq i64 {val_loaded}, {n}"));
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
                                        let payload_xiom_ty: Option<String> = self.enum_variant_field_types.get(&type_name_clone)
                                            .or_else(|| {
                                                self.enum_variant_field_types.iter()
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
                                        self.local_vec_handle.remove(&field_ident.name);
                                        if let Some(fty) = payload_xiom_ty.as_deref() {
                                            if let Some(inner) = fty.strip_prefix("Vec[").and_then(|s| s.strip_suffix(']')) {
                                                self.local_vec_handle.insert(field_ident.name.clone(), inner.to_string());
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
                            self.enum_variants.get(tn)
                                .map(|vars| vars.iter().any(|(v, _)| v == &ident.name))
                        }).unwrap_or(false);
                        if !is_variant {
                            // Bind the scrutinee value to the pattern variable using
                            // its REAL LLVM type (a struct scrutinee like LogLevel
                            // must not be stored as i64). Fall back to i64 for a
                            // scalar/empty value.
                            let bind_ty = if scrutinee_llvm_ty.is_empty() || scrutinee_llvm_ty == "void" {
                                "i64".to_string()
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
                    if let Pattern::Some(inner, _) | Pattern::Ok(inner, _) = &arm.pattern {
                        if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                            let val_gep = self.fresh_tmp();
                            self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 1"));
                            let field_ty = self.field_llvm_type(type_name, 1);
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load {field_ty}, {field_ty}* {val_gep}"));
                            if let Pattern::Ident(ident) = inner.as_ref() {
                                let field_alloca = self.fresh_tmp();
                                self.emitln(&format!("  {field_alloca} = alloca {field_ty}"));
                                self.emitln(&format!("  store {field_ty} {loaded}, {field_ty}* {field_alloca}"));
                                self.add_local(&ident.name, field_alloca, &field_ty);
                            }
                        }
                    }
                    match &arm.body {
                        MatchBody::Block(b) => { self.compile_block(b, false)?; }
                        MatchBody::Expr(e) => {
                            let (arm_val, arm_val_ty) = self.compile_expr(e)?;
                            if let Some(ptr) = self.match_result_ptr.clone() {
                                let ret_ty = self.match_result_ty.clone().unwrap_or_else(|| self.current_return_type.clone());
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
            Stmt::While(cond, body, _) => {
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
                self.loop_stack.push((loop_cond.clone(), loop_exit.clone()));
                self.compile_block(body, false)?;
                self.loop_stack.pop();
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

    pub(crate) fn compile_expr(&mut self, expr: &Expr) -> Result<(String, String), String> {
        // Flush any concrete struct types that were registered during
        // compilation (e.g. %struct.Option__Point) so they appear before
        // the current function body.
        self.flush_deferred_types();
        match expr {
            Expr::Ident(ident) => {
                // `this` keyword in method bodies maps to the receiver `self`.
                // 5c.30 const-generic: substitute compile-time const value (e.g. N=5)
                if let Some(val) = self.current_const_map.get(&ident.name) {
                    return Ok((val.to_string(), "i64".to_string()));
                }
                let lookup_name: &str = if ident.name == "this" { "self" } else { &ident.name };
                if let Some((ptr, llvm_ty)) = self.lookup_local(lookup_name).cloned() {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                    // Propagate array-value tracking through let-bound locals:
                    // if `ident` was bound from an Expr::Array, the loaded value
                    // also originates from an array buffer so val_to_struct can
                    // construct a proper Vec from it.
                    if self.array_locals.contains(&ident.name) {
                        self.array_value_regs.insert(tmp.clone());
                    }
                    Ok((tmp, llvm_ty))
                } else if let Some((symbol, llvm_ty)) = self.module_globals.get(&ident.name).cloned() {
                    // Mutable module-level `var`: load the current value from the
                    // real global. Checked BEFORE enum-variant / constant fallbacks
                    // so a live global is never mistaken for a compile-time literal.
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* @{symbol}"));
                    Ok((tmp, llvm_ty))
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
                            Ok((loaded, struct_ty))
                    } else {
                        // TAIL-TODO: a generic call whose type params can't be inferred
                        // from value arguments (e.g. `hash[T](v)` where T is bound by
                        // an interface, or interface default methods like
                        // `Iterator[T].sum`/`.next`). The parser also discards explicit
                        // `[T]` type args, so these can't be monomorphised and fall back
                        // to 0. Blocks: iter (sum/product), hash (Hash dispatch),
                        // cell/rc/sync (wrapper .get/.clone + nested `mod.Type.new`).
                        Ok(("0".to_string(), "i64".to_string()))
                    }
                } else {
                        Ok(("0".to_string(), "i64".to_string()))
                    }
                } else {
                    // A bare reference to a module/global constant: substitute its
                    // literal value (constants aren't materialized as globals).
                    if let Some(cval) = self.constants.get(&ident.name).cloned() {
                        return self.compile_expr(&cval);
                    }
                    // Function name used as value (e.g. v.push(add_one)):
                    // resolve to a function pointer via ptrtoint of the IR symbol.
                    // The functions map has both bare names and module-qualified names.
                    let fn_full: Option<(String, String, Vec<String>)> = {
                        let exact = self.functions.get(&ident.name).map(|(p, r)| (ident.name.clone(), r.clone(), p.clone()));
                        exact.or_else(|| {
                            self.functions.iter().find(|(k, _)| k.ends_with(&format!(".{}", ident.name)))
                                .map(|(k, (p, r))| (k.clone(), r.clone(), p.clone()))
                        })
                    };
                    if let Some((fn_name, ret_ty, param_tys)) = fn_full {
                        let fpty = format!("{ret_ty} ({})*", param_tys.join(", "));
                        let fp = self.fresh_tmp();
                        self.emitln(&format!("  {fp} = ptrtoint {fpty} @{fn_name} to i64"));
                        return Ok((fp, "i64".to_string()));
                    }
                    Ok(("0".to_string(), "i64".to_string()))
                }
            }
            Expr::Int(n, _) => {
                Ok((format!("{n}"), "i64".to_string()))
            }
            Expr::Float(f, _) => {
                Ok((format!("{f:.6}"), "double".to_string()))
            }
            Expr::Bool(b, _) => {
                Ok((if *b { "1".to_string() } else { "0".to_string() }, "i64".to_string()))
            }
            Expr::Str(s, _) => {
                let tmp = self.intern_cstring(s);
                Ok((tmp, "i8*".to_string()))
            }
            Expr::Char(c, _) => {
                Ok((format!("{}", *c as u32), "i8".to_string()))
            }
            Expr::Paren(inner, _) => self.compile_expr(inner),
            Expr::Tuple(items, _) => {
                if items.is_empty() {
                    Ok(("0".to_string(), "void".to_string()))
                } else {
                    let struct_ty = self.infer_llvm_type(expr);
                    if !struct_ty.starts_with("%struct.") {
                        return Ok(("0".to_string(), "i64".to_string()));
                    }
                    let alloca = self.fresh_tmp();
                    self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                    for (i, item) in items.iter().enumerate() {
                        let (item_val, item_ty) = self.compile_expr(item)?;
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
                        self.emitln(&format!("  store {item_ty} {item_val}, {item_ty}* {gep}"));
                    }
                    let loaded = self.fresh_tmp();
                    self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
                    Ok((loaded, struct_ty))
                }
            }
            Expr::Unary(op, inner, _) => {
                let (val, inner_ty) = self.compile_expr(inner)?;
                let tmp = self.fresh_tmp();
                match op {
                    UnaryOp::Neg => {
                        if inner_ty == "double" || inner_ty == "float" {
                            self.emitln(&format!("  {tmp} = fneg {inner_ty} {val}"));
                            return Ok((tmp, inner_ty.clone()));
                        } else {
                            self.emitln(&format!("  {tmp} = sub i64 0, {val}"));
                            return Ok((tmp, "i64".to_string()));
                        }
                    }
                    UnaryOp::Not => {
                        let val_ty = inner_ty.clone();
                        let xor_val = if val_ty == "i1" || val_ty == "i8" {
                            let ext = self.fresh_tmp();
                            self.emitln(&format!("  {ext} = zext {val_ty} {val} to i64"));
                            ext
                        } else {
                            val
                        };
                        self.emitln(&format!("  {tmp} = xor i64 {xor_val}, 1"));
                        return Ok((tmp, "i64".to_string()));
                    }
                    UnaryOp::BitNot => {
                        self.emitln(&format!("  {tmp} = xor i64 {val}, -1"));
                        return Ok((tmp, "i64".to_string()));
                    }
                    UnaryOp::Deref => {
                        // `*p`: load through a real pointer. `inner_ty` is e.g. `i64*`
                        // (from a `*T` value). Load the pointee type. If the operand is
                        // not a pointer (legacy path where a `*T` erased to i64), return
                        // it unchanged so no invalid `load` is emitted.
                        if !inner_ty.ends_with('*') {
                            return Ok((val, inner_ty));
                        }
                        let pointee = inner_ty.trim_end_matches('*').to_string();
                        self.emitln(&format!("  {tmp} = load {pointee}, {inner_ty} {val}"));
                        return Ok((tmp, pointee));
                    }
                    UnaryOp::Ref | UnaryOp::MutRef => return Ok((val, inner_ty)),
                }
            }
            Expr::Binary(left, op, right, _) => {
                let (mut l, mut lt) = self.compile_expr(left)?;
                let (mut r, mut rt) = self.compile_expr(right)?;
                let tmp = self.fresh_tmp();
                // Str + Str: concatenate at runtime, not `add i64` on pointers.
                // A Str is `i8*` at the ABI; `add` on two pointers is invalid IR
                // and semantically wrong. Lower to a call to the runtime concat.
                // Fires when EITHER operand is a Str pointer (the other side is
                // coerced to i8*), which also keeps IR valid where a Str-returning
                // callee was resolved to a fallback i64 signature.
                if matches!(op, BinOp::Add) && (lt == "i8*" || rt == "i8*") {
                    let lp = self.val_to_i8ptr(&l, &lt);
                    let rp = self.val_to_i8ptr(&r, &rt);
                    let res = self.fresh_tmp();
                    self.emitln(&format!("  {res} = call i8* @xiom_str_concat(i8* {lp}, i8* {rp})"));
                    return Ok((res, "i8*".to_string()));
                }
                let is_float = self.is_float_expr(left) || self.is_float_expr(right)
                    || lt == "float" || lt == "double" || rt == "float" || rt == "double";
                // Determine the actual float type from the operands.
                // If either operand is `float` (Float32), use `float` for the
                // comparison; otherwise default to `double` (Float64).
                let float_ty = if lt == "float" || rt == "float" { "float" } else { "double" };
                if matches!(op, BinOp::And | BinOp::Or) {
                    let is_or = matches!(op, BinOp::Or);
                    let widen = |s: &mut Self, val: &str, ty: &str| -> String {
                        if ty == "i64" { val.to_string() } else {
                            let ext = s.fresh_tmp();
                            s.emitln(&format!("  {ext} = zext {ty} {val} to i64"));
                            ext
                        }
                    };
                    let lw = widen(self, &l, &lt);
                    let rw = widen(self, &r, &rt);
                    let op_name = if is_or { "or" } else { "and" };
                    let result = self.fresh_tmp();
                    self.emitln(&format!("  {result} = {op_name} i64 {lw}, {rw}"));
                    return Ok((result, "i64".to_string()));
                }
                // For struct-typed equality/inequality, call derived eq() instead of icmp.
                // Exclude pointer-to-struct types (e.g. `%struct.ArcInner*`) which end
                // with `*`; those compare pointer identity, not struct contents.
                if matches!(op, BinOp::Eq | BinOp::Neq) {
                    let lt_is_struct = lt.starts_with("%struct.") && !lt.ends_with('*');
                    let rt_is_struct = rt.starts_with("%struct.") && !rt.ends_with('*');
                    if lt_is_struct || rt_is_struct {
                        let struct_name = if lt_is_struct { &lt[8..] } else { &rt[8..] };
                        let eq_fn = format!("{}.eq", struct_name);
                        let eq_result = self.fresh_tmp();
                        if self.functions.contains_key(&eq_fn) {
                            self.emitln(&format!("  {eq_result} = call i64 @{eq_fn}({lt} {l}, {rt} {r})"));
                        } else {
                            // No derived `.eq` (e.g. builtin Ordering/Option enums):
                            // For Option/Result types, compare field 1 (the value)
                            // with the scalar; for other structs, compare field 0.
                            let l_i = if lt_is_struct {
                                if struct_name == "Option" || struct_name.ends_with(".Option")
                                   || struct_name == "Result" || struct_name.ends_with(".Result")
                                {
                                    self.extract_scalar_field1(&l, &lt)
                                } else {
                                    self.extract_scalar_field0(&l, &lt)
                                }
                            } else {
                                self.val_to_i64(&l, &lt)
                            };
                            let r_i = if rt_is_struct {
                                if struct_name == "Option" || struct_name.ends_with(".Option")
                                   || struct_name == "Result" || struct_name.ends_with(".Result")
                                {
                                    self.extract_scalar_field1(&r, &rt)
                                } else {
                                    self.extract_scalar_field0(&r, &rt)
                                }
                            } else {
                                self.val_to_i64(&r, &rt)
                            };
                            let eqb = self.fresh_tmp();
                            self.emitln(&format!("  {eqb} = icmp eq i64 {l_i}, {r_i}"));
                            self.emitln(&format!("  {eq_result} = zext i1 {eqb} to i64"));
                        }
                        if matches!(op, BinOp::Neq) {
                            let negated = self.fresh_tmp();
                            self.emitln(&format!("  {negated} = xor i64 {eq_result}, 1"));
                            return Ok((negated, "i64".to_string()));
                        }
                        return Ok((eq_result, "i64".to_string()));
                    }
                    // Str == Str / Str != Str: compare by CONTENT via strcmp, not by
                    // pointer identity. A Str is `i8*` at the ABI; a raw `icmp eq i8*`
                    // only tests whether the two pointers are the same object, which
                    // is wrong for value equality (`int_to_string(42) == "42"`).
                    // Also fires when only ONE side is a known i8* and the other is an
                    // i64 whose real value is a Str pointer (e.g. `opt.unwrap() == "x"`
                    // where unwrap's ABI return is i64 but holds an i8*): coerce the
                    // i64 side to i8* so the content compare is well-typed.
                    if (lt == "i8*" || rt == "i8*") && (lt == "i8*" || lt == "i64") && (rt == "i8*" || rt == "i64") {
                        // 5c-E G6: pointer-to-null comparison. When comparing an i8*
                        // data pointer to literal 0, use icmp eq i8* NULL, not strcmp.
                        // strcmp(NULL, ...) crashes with ACCESS_VIOLATION.
                        if (lt == "i8*" && r == "0") || (rt == "i8*" && l == "0") {
                            let lp = self.val_to_i8ptr(&l, &lt);
                            let rp = self.val_to_i8ptr(&r, &rt);
                            let _nullp = if l == "0" { "null".to_string() } else { "null".to_string() };
                            let cmp = self.fresh_tmp();
                            let icmp_val = if l == "0" { &rp } else { &lp };
                            self.emitln(&format!("  {cmp} = icmp eq i8* {icmp_val}, null"));
                            let zext = self.fresh_tmp();
                            self.emitln(&format!("  {zext} = zext i1 {cmp} to i64"));
                            if matches!(op, BinOp::Neq) {
                                let neg = self.fresh_tmp();
                                self.emitln(&format!("  {neg} = xor i64 {zext}, 1"));
                                return Ok((neg, "i64".to_string()));
                            }
                            return Ok((zext, "i64".to_string()));
                        }
                        let lp = self.val_to_i8ptr(&l, &lt);
                        let rp = self.val_to_i8ptr(&r, &rt);
                        let cmp = self.fresh_tmp();
                        self.emitln(&format!("  {cmp} = call i32 @strcmp(i8* {lp}, i8* {rp})"));
                        let is_eq = self.fresh_tmp();
                        // strcmp == 0 means equal.
                        let want = if matches!(op, BinOp::Neq) { "ne" } else { "eq" };
                        self.emitln(&format!("  {is_eq} = icmp {want} i32 {cmp}, 0"));
                        let ext = self.fresh_tmp();
                        self.emitln(&format!("  {ext} = zext i1 {is_eq} to i64"));
                        return Ok((ext, "i64".to_string()));
                    }
                }
                // Auto-deref pointer operands for relational comparisons (Lt/Gt/Le/Ge).
                // A field like `count: *Int` loaded from the struct is a pointer (e.g.
                // `i64*`); when compared to an integer, load through the pointer so the
                // comparison is on the pointed-to scalar, not the pointer address.
                if matches!(op, BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge) {
                    if lt.ends_with('*') {
                        let inner = lt.trim_end_matches('*');
                        let deref_l = self.fresh_tmp();
                        self.emitln(&format!("  {deref_l} = load {inner}, {lt} {l}"));
                        l = deref_l;
                        lt = inner.to_string();
                    }
                    if rt.ends_with('*') {
                        let inner = rt.trim_end_matches('*');
                        let deref_r = self.fresh_tmp();
                        self.emitln(&format!("  {deref_r} = load {inner}, {rt} {r}"));
                        r = deref_r;
                        rt = inner.to_string();
                    }
                }
                let (ty, inst) = match op {
                    BinOp::Add => (if is_float { float_ty } else { "i64" }, if is_float { "fadd" } else { "add" }),
                    BinOp::Sub => (if is_float { float_ty } else { "i64" }, if is_float { "fsub" } else { "sub" }),
                    BinOp::Mul => (if is_float { float_ty } else { "i64" }, if is_float { "fmul" } else { "mul" }),
                    BinOp::Div => (if is_float { float_ty } else { "i64" }, if is_float { "fdiv" } else { "sdiv" }),
                    BinOp::Rem => (if is_float { float_ty } else { "i64" }, if is_float { "frem" } else { "srem" }),
                    BinOp::BitXor => ("i64", "xor"),
                    BinOp::BitAnd => ("i64", "and"),
                    BinOp::BitOr => ("i64", "or"),
                    BinOp::Shl => ("i64", "shl"),
                    BinOp::Shr => ("i64", "ashr"),
                    BinOp::Eq => (if is_float { float_ty } else { "i64" }, if is_float { "fcmp oeq" } else { "icmp eq" }),
                    BinOp::Neq => (if is_float { float_ty } else { "i64" }, if is_float { "fcmp one" } else { "icmp ne" }),
                    BinOp::Lt => (if is_float { float_ty } else { "i64" }, if is_float { "fcmp olt" } else { "icmp slt" }),
                    BinOp::Gt => (if is_float { float_ty } else { "i64" }, if is_float { "fcmp ogt" } else { "icmp sgt" }),
                    BinOp::Le => (if is_float { float_ty } else { "i64" }, if is_float { "fcmp ole" } else { "icmp sle" }),
                    BinOp::Ge => (if is_float { float_ty } else { "i64" }, if is_float { "fcmp oge" } else { "icmp sge" }),
                    BinOp::Assign => return Ok((r, rt)),
                    _ => unreachable!(),
                };
                // For non-float comparisons, use the actual operand LLVM type
                // (handles pointer types like i8* for string comparisons)
                let ty = if !is_float && inst.starts_with("icmp") {
                    if lt.contains('*') { lt.clone() } else if rt.contains('*') { rt.clone() } else { ty.to_string() }
                } else {
                    ty.to_string()
                };
                // Convert literal 0 to null pointer when comparing with pointer types
                if ty.contains('*') {
                    if l == "0" && lt == "i64" {
                        let null_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {null_tmp} = inttoptr i64 0 to {ty}"));
                        l = null_tmp;
                    }
                    if r == "0" && rt == "i64" {
                        let null_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {null_tmp} = inttoptr i64 0 to {ty}"));
                        r = null_tmp;
                    }
                }
                // Coerce i64 operands to double when in float context (mixed-type expressions)
                if is_float {
                    // A single-scalar-backed struct operand (Option/Ordering etc.)
                    // in a float context: extract its leading i64 field first.
                    if lt.starts_with("%struct.") {
                        l = self.extract_scalar_field0(&l, &lt);
                    }
                    if rt.starts_with("%struct.") {
                        r = self.extract_scalar_field0(&r, &rt);
                    }
                    if lt == "i64" || lt.starts_with("%struct.") {
                        let conv = self.fresh_tmp();
                        // 5c.29: `opt.unwrap()` returns the float payload as RAW
                        // BITS in an i64 (Some(x) stores via bitcast) â€” so the
                        // conversion must bit-reinterpret, never sitofp.
                        if lt == "i64" && Self::expr_is_unwrap_call(left) {
                            if float_ty == "double" {
                                self.emitln(&format!("  {conv} = bitcast i64 {l} to double"));
                            } else {
                                let t32 = self.fresh_tmp();
                                self.emitln(&format!("  {t32} = trunc i64 {l} to i32"));
                                self.emitln(&format!("  {conv} = bitcast i32 {t32} to float"));
                            }
                        } else {
                            self.emitln(&format!("  {conv} = sitofp i64 {l} to {float_ty}"));
                        }
                        l = conv;
                    }
                    if rt == "i64" || rt.starts_with("%struct.") {
                        let conv = self.fresh_tmp();
                        if rt == "i64" && Self::expr_is_unwrap_call(right) {
                            if float_ty == "double" {
                                self.emitln(&format!("  {conv} = bitcast i64 {r} to double"));
                            } else {
                                let t32 = self.fresh_tmp();
                                self.emitln(&format!("  {t32} = trunc i64 {r} to i32"));
                                self.emitln(&format!("  {conv} = bitcast i32 {t32} to float"));
                            }
                        } else {
                            self.emitln(&format!("  {conv} = sitofp i64 {r} to {float_ty}"));
                        }
                        r = conv;
                    }
                    // Narrow double ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ float when the operation uses float
                    // (Float32) but an operand is double (Float64 literal).
                    if is_float && float_ty == "float" {
                        if lt == "double" {
                            let conv = self.fresh_tmp();
                            self.emitln(&format!("  {conv} = fptrunc double {l} to float"));
                            l = conv;
                        }
                        if rt == "double" {
                            let conv = self.fresh_tmp();
                            self.emitln(&format!("  {conv} = fptrunc double {r} to float"));
                            r = conv;
                        }
                    }
                }
                // A1: widen narrow integer operands (Char/UInt8 = i8, Bool = i1,
                // Int16 = i16, Int32 = i32) to i64 so they match the i64 integer
                // operation type. Skips float ops and pointer comparisons (string
                // `==`), whose `ty` was resolved to `double`/`i8*` above.
                if !is_float && ty == "i64" {
                    // A single-scalar-backed struct operand (Option/Ordering whose
                    // first field is the i64 discriminant/value) against a plain
                    // integer: extract field 0 so the integer op is well-typed.
                    // Handles idioms like `opt >= 0` / `find(...) < n`.
                    if lt.starts_with("%struct.") && !rt.starts_with("%struct.") {
                        l = self.extract_scalar_field0(&l, &lt);
                    }
                    if rt.starts_with("%struct.") && !lt.starts_with("%struct.") {
                        r = self.extract_scalar_field0(&r, &rt);
                    }
                    l = self.widen_to_i64(&l, &lt);
                    r = self.widen_to_i64(&r, &rt);
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
                let (result, result_ty) = if inst.starts_with("icmp") || inst.starts_with("fcmp") {
                    let ext = self.fresh_tmp();
                    self.emitln(&format!("  {ext} = zext i1 {tmp} to i64"));
                    (ext, "i64".to_string())
                } else {
                    (tmp, ty.clone())
                };
                if let Some(cont_block) = div_cont {
                    self.emitln(&format!("  br label %{cont_block}"));
                    self.emitln(&format!("\n{cont_block}:"));
                }
                Ok((result, result_ty))
            }
            Expr::Try(inner, _span) => {
                let (val, _inner_ty) = self.compile_expr(inner)?;
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
                            // If function is defined and its return type is a struct with ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â°ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¤2 fields
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
                    let tag_bool = self.fresh_tmp();
                    self.emitln(&format!("  {tag_bool} = icmp ne i64 {tag}, 0"));
                    self.emitln(&format!("  br i1 {tag_bool}, label %{some_block}, label %{none_block}"));
                    self.emitln(&format!("\n{none_block}:"));
                    let ret_ty = self.current_return_type.clone();
                    let default_val = if ret_ty.starts_with('%') { "zeroinitializer".to_string() } else { "0".to_string() };
                    self.emitln(&format!("  ret {ret_ty} {default_val}"));
                    self.emitln(&format!("\n{some_block}:"));
                    let val_gep = self.fresh_tmp();
                    let some_val = self.fresh_tmp();
                    self.emitln(&format!("  {val_gep} = getelementptr {opt_ty}, {opt_ty}* {opt_alloca}, i32 0, i32 1"));
                    self.emitln(&format!("  {some_val} = load i64, i64* {val_gep}"));
                    Ok((some_val, "i64".to_string()))
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
                    let tag_bool = self.fresh_tmp();
                    self.emitln(&format!("  {tag_bool} = icmp ne i64 {tag}, 0"));
                    self.emitln(&format!("  br i1 {tag_bool}, label %{ok_block}, label %{err_block}"));
                    self.emitln(&format!("\n{err_block}:"));
                    let ret_ty = self.current_return_type.clone();
                    // Propagate the error. When the enclosing function returns a
                    // Result/struct, return the ORIGINAL result value unchanged
                    // (the error variant flows through). Only when the function
                    // returns a scalar do we return the raw error field.
                    if ret_ty.starts_with("%struct.") {
                        let ev = self.coerce_value(&val, result_ty, &ret_ty);
                        self.emitln(&format!("  ret {ret_ty} {ev}"));
                    } else {
                        let err_gep = self.fresh_tmp();
                        let err_val = self.fresh_tmp();
                        self.emitln(&format!("  {err_gep} = getelementptr {result_ty}, {result_ty}* {result_alloca}, i32 0, i32 2"));
                        self.emitln(&format!("  {err_val} = load i64, i64* {err_gep}"));
                        let ev = self.coerce_value(&err_val, "i64", &ret_ty);
                        self.emitln(&format!("  ret {ret_ty} {ev}"));
                    }
                    self.emitln(&format!("\n{ok_block}:"));
                    let val_gep = self.fresh_tmp();
                    let ok_val = self.fresh_tmp();
                    self.emitln(&format!("  {val_gep} = getelementptr {result_ty}, {result_ty}* {result_alloca}, i32 0, i32 1"));
                    self.emitln(&format!("  {ok_val} = load i64, i64* {val_gep}"));
                    Ok((ok_val, "i64".to_string()))
                }
            }
            Expr::Imply(left, right, _) => {
                let (l, _lt) = self.compile_expr(left)?;
                let (r, _rt) = self.compile_expr(right)?;
                let tmp1 = self.fresh_tmp();
                let tmp2 = self.fresh_tmp();
                self.emitln(&format!("  {tmp1} = xor i64 {l}, 1"));
                self.emitln(&format!("  {tmp2} = or i64 {tmp1}, {r}"));
                Ok((tmp2, "i64".to_string()))
            }
            Expr::Is(expr, pattern, _) => {
                let (val, ty) = self.compile_expr(expr)?;
                if ty.starts_with("%struct.") {
                    let variant_name = match &pattern {
                        xiom_ast::Pattern::Some(..) => "Some",
                        xiom_ast::Pattern::None(..) => "None",
                        xiom_ast::Pattern::Ok(..) => "Ok",
                        xiom_ast::Pattern::Err(..) => "Err",
                        _ => { return Ok(("1".to_string(), "i64".to_string())); }
                    };
                    let type_name = &ty[8..];
                    if let Some(variants) = self.enum_variants.get(type_name) {
                        if let Some((disc, _)) = variants.iter().enumerate()
                            .find(|(_, (v, _))| v == variant_name)
                        {
                            let disc_val = disc as i64;
                            let alloca = self.fresh_tmp();
                            self.emitln(&format!("  {alloca} = alloca {ty}"));
                            self.emitln(&format!("  store {ty} {val}, {ty}* {alloca}"));
                            let gep = self.fresh_tmp();
                            self.emitln(&format!("  {gep} = getelementptr {ty}, {ty}* {alloca}, i32 0, i32 0"));
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load i64, i64* {gep}"));
                            let cmp = self.fresh_tmp();
                            self.emitln(&format!("  {cmp} = icmp eq i64 {loaded}, {disc_val}"));
                            let ext = self.fresh_tmp();
                            self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
                            return Ok((ext, "i64".to_string()));
                        }
                    }
                    // For Option/Result types not registered as enum variants:
                    // field 0 discriminator; Some/Ok = disc != 0, None/Err = disc == 0.
                    let alloca = self.fresh_tmp();
                    self.emitln(&format!("  {alloca} = alloca {ty}"));
                    self.emitln(&format!("  store {ty} {val}, {ty}* {alloca}"));
                    let gep = self.fresh_tmp();
                    self.emitln(&format!("  {gep} = getelementptr {ty}, {ty}* {alloca}, i32 0, i32 0"));
                    let loaded = self.fresh_tmp();
                    self.emitln(&format!("  {loaded} = load i64, i64* {gep}"));
                    if variant_name == "Some" || variant_name == "Ok" {
                        let cmp = self.fresh_tmp();
                        self.emitln(&format!("  {cmp} = icmp ne i64 {loaded}, 0"));
                        let ext = self.fresh_tmp();
                        self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
                        return Ok((ext, "i64".to_string()));
                    } else {
                        let cmp = self.fresh_tmp();
                        self.emitln(&format!("  {cmp} = icmp eq i64 {loaded}, 0"));
                        let ext = self.fresh_tmp();
                        self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
                        return Ok((ext, "i64".to_string()));
                    }
                }
                Ok(("1".to_string(), "i64".to_string()))
            }
            Expr::Field(obj, field, _) => {
                // Module-qualified constant, e.g. `simd.SIMD_SSE`: when the object is
                // NOT a value instance (a module path), and the leaf names a known
                // constant, substitute its literal value. Constants are keyed by their
                // bare name, so `simd.SIMD_SSE` resolves via `SIMD_SSE`.
                if !self.receiver_is_instance(obj) {
                    if let Some(cval) = self.constants.get(&field.name).cloned() {
                        return self.compile_expr(&cval);
                    }
                }
                // `(*p).field` on a raw pointer to a struct: GEP directly into the
                // pointee (`%struct.X*`) rather than loading a by-value struct first.
                // Enables Arc's `(*ptr).value` / `(*ptr).count` deref-field reads.
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
                                if let Some(field_names) = self.types.get(type_name)
                                    .or_else(|| {
                                        let suffix = format!(".{type_name}");
                                        self.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                            .and_then(|k| self.types.get(k))
                                    })
                                    .cloned() {
                                    if let Some(field_idx) = field_names.iter().position(|f| f == &field.name) {
                                        let field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                                        let gep = self.fresh_tmp();
                                        let loaded = self.fresh_tmp();
                                        self.emitln(&format!("  {gep} = getelementptr {pointee}, {ptr_ty} {ptr_val}, i32 0, i32 {field_idx}"));
                                        self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                        return Ok((loaded, field_llvm_ty));
                                    }
                                }
                            }
                        }
                    }
                }
                // Simplified field access: if the object is an ident in locals, load the field via GEP
                if let Expr::Ident(obj_ident) = obj.as_ref() {
                    if let Some((ptr, llvm_ty)) = self.lookup_local(&obj_ident.name).cloned() {
                        // 5c.30: i64 local holding a BOXED struct pointer (from
                        // `opt.unwrap()` of a Vec-of-struct pop/get): typed
                        // inttoptr + GEP field access on the heap box.
                        if llvm_ty == "i64" {
                            if let Some(tn) = self.local_boxed_struct.get(&obj_ident.name).cloned() {
                                if let Some(field_names) = self.types.get(&tn)
                                    .or_else(|| {
                                        let suffix = format!(".{tn}");
                                        self.types.keys().find(|k| k.ends_with(&suffix))
                                            .and_then(|k| self.types.get(k))
                                    })
                                    .cloned()
                                {
                                    if let Some(fi) = field_names.iter().position(|f| f == &field.name) {
                                        let sty = format!("%struct.{tn}");
                                        let field_llvm_ty = self.field_llvm_type(&tn, fi);
                                        let hv = self.fresh_tmp();
                                        self.emitln(&format!("  {hv} = load i64, i64* {ptr}"));
                                        let sp = self.fresh_tmp();
                                        self.emitln(&format!("  {sp} = inttoptr i64 {hv} to {sty}*"));
                                        let gp = self.fresh_tmp();
                                        let ld = self.fresh_tmp();
                                        self.emitln(&format!("  {gp} = getelementptr {sty}, {sty}* {sp}, i32 0, i32 {fi}"));
                                        self.emitln(&format!("  {ld} = load {field_llvm_ty}, {field_llvm_ty}* {gp}"));
                                        return Ok((ld, field_llvm_ty));
                                    }
                                }
                            }
                        }
                        // Auto-deref a pointer-to-struct local (`p: *Struct`): load the
                        // pointer from its slot, then GEP into the pointee. Fires for
                        // struct fields whose type is a real `*Struct` (e.g. Arc's
                        // `ptr: *ArcInner`), where `ptr.count` means `(*ptr).count`.
                        if llvm_ty.ends_with('*') && llvm_ty.starts_with("%struct.") {
                            let pointee = llvm_ty.trim_end_matches('*').to_string();
                            let type_name = &pointee[8..];
                            if let Some(field_names) = self.types.get(type_name)
                                .or_else(|| {
                                    let suffix = format!(".{type_name}");
                                    self.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                        .and_then(|k| self.types.get(k))
                                })
                                .cloned() {
                                if let Some(field_idx) = field_names.iter().position(|f| f == &field.name) {
                                    let field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                                    let ptr_val = self.fresh_tmp();
                                    self.emitln(&format!("  {ptr_val} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                                    let gep = self.fresh_tmp();
                                    let loaded = self.fresh_tmp();
                                    self.emitln(&format!("  {gep} = getelementptr {pointee}, {llvm_ty} {ptr_val}, i32 0, i32 {field_idx}"));
                                    self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                    return Ok((loaded, field_llvm_ty));
                                }
                            }
                        }
                        // Handle .is_ok / .is_some / .is_err / .is_none pseudo-fields
                        // on Result/Option enum types. These check the discriminant
                        // (field 0) against the success variant index.
                        if llvm_ty.starts_with("%struct.") && !llvm_ty.ends_with('*') {
                            let type_name = &llvm_ty[8..];
                            let is_result = type_name.ends_with("Result") || type_name.contains(".Result");
                            let is_option = type_name.ends_with("Option") || type_name.contains(".Option");
                            let field_name = &field.name;
                            if (is_result && (field_name == "is_ok" || field_name == "is_err"))
                                || (is_option && (field_name == "is_some" || field_name == "is_none"))
                            {
                                // is_ok/is_some: discriminant == 1 (the success variant)
                                // is_err/is_none: discriminant == 0
                                let success_variant = matches!(field_name.as_str(), "is_ok" | "is_some");
                                let struct_val = self.fresh_tmp();
                                self.emitln(&format!("  {struct_val} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                                let alloca_tmp = self.fresh_tmp();
                                self.emitln(&format!("  {alloca_tmp} = alloca {llvm_ty}"));
                                self.emitln(&format!("  store {llvm_ty} {struct_val}, {llvm_ty}* {alloca_tmp}"));
                                let disc_gep = self.fresh_tmp();
                                self.emitln(&format!("  {disc_gep} = getelementptr {llvm_ty}, {llvm_ty}* {alloca_tmp}, i32 0, i32 0"));
                                let disc_val = self.fresh_tmp();
                                self.emitln(&format!("  {disc_val} = load i64, i64* {disc_gep}"));
                                let cmp = self.fresh_tmp();
                                if success_variant {
                                    self.emitln(&format!("  {cmp} = icmp eq i64 {disc_val}, 1"));
                                } else {
                                    self.emitln(&format!("  {cmp} = icmp eq i64 {disc_val}, 0"));
                                }
                                let result = self.fresh_tmp();
                                self.emitln(&format!("  {result} = zext i1 {cmp} to i64"));
                                return Ok((result, "i64".to_string()));
                            }
                        }
                        // Check if it's a (by-value) struct type. Exclude pointer
                        // types (handled above) so `%struct.X*` never takes this path.
                        if llvm_ty.starts_with("%struct.") && !llvm_ty.ends_with('*') {
                            // Find field index
                            let type_name = &llvm_ty[8..];
                            if let Some(field_names) = self.types.get(type_name)
                                .or_else(|| {
                                    let suffix = format!(".{type_name}");
                                    self.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                        .and_then(|k| self.types.get(k))
                                })
                            {
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
                                    return Ok((loaded, field_llvm_ty));
                                }
                            }
                        }
                    }
                }
                // Qualified enum-variant path, e.g. `xiom.log.LogLevel.Warn` or
                // `LogLevel.Warn`: the field name is a variant and the parent
                // resolves to the enum type. Emit the discriminant struct (mirrors
                // the bare-Ident enum-variant construction above).
                if let Some(enum_key) = self.resolve_enum_for_variant(obj, &field.name) {
                    if let Some(vars) = self.enum_variants.get(&enum_key) {
                        if let Some(var_idx) = vars.iter().position(|(v, _)| v == &field.name) {
                            if let Ok(struct_ty) = self.llvm_type_for(&enum_key) {
                                let alloca = self.fresh_tmp();
                                self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                                let disc_gep = self.fresh_tmp();
                                self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                                self.emitln(&format!("  store i64 {var_idx}, i64* {disc_gep}"));
                                let loaded = self.fresh_tmp();
                                self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
                                return Ok((loaded, struct_ty));
                            }
                        }
                    }
                }
                // General struct field access on a computed value (e.g.
                // `data.get(i).value` ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â a Field over a Call result). The object
                // isn't a bound local, so compile it and GEP the field by index.
                // Without this, such accesses fell through to the `0` default,
                // silently discarding Option payloads passed as call arguments.
                {
                    let (obj_val, ov_ty) = self.compile_expr(obj)?;
                    if ov_ty.starts_with("%struct.") {
                        let type_name = &ov_ty[8..];
                        // Handle .is_ok / .is_some / .is_err / .is_none pseudo-fields
                        // on computed values (e.g. `find_first_match(...).is_some`).
                        let is_result = type_name.ends_with("Result") || type_name.contains(".Result");
                        let is_option = type_name.ends_with("Option") || type_name.contains(".Option");
                        let field_name = &field.name;
                        if (is_result && (field_name == "is_ok" || field_name == "is_err"))
                            || (is_option && (field_name == "is_some" || field_name == "is_none"))
                        {
                            let success_variant = matches!(field_name.as_str(), "is_ok" | "is_some");
                            let struct_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {struct_alloca} = alloca {ov_ty}"));
                            self.emitln(&format!("  store {ov_ty} {obj_val}, {ov_ty}* {struct_alloca}"));
                            let disc_gep = self.fresh_tmp();
                            self.emitln(&format!("  {disc_gep} = getelementptr {ov_ty}, {ov_ty}* {struct_alloca}, i32 0, i32 0"));
                            let disc_val = self.fresh_tmp();
                            self.emitln(&format!("  {disc_val} = load i64, i64* {disc_gep}"));
                            let cmp = self.fresh_tmp();
                            if success_variant {
                                self.emitln(&format!("  {cmp} = icmp eq i64 {disc_val}, 1"));
                            } else {
                                self.emitln(&format!("  {cmp} = icmp eq i64 {disc_val}, 0"));
                            }
                            let result = self.fresh_tmp();
                            self.emitln(&format!("  {result} = zext i1 {cmp} to i64"));
                            return Ok((result, "i64".to_string()));
                        }
                        if let Some(field_names) = self.types.get(type_name)
                            .or_else(|| {
                                let suffix = format!(".{type_name}");
                                self.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                    .and_then(|k| self.types.get(k))
                            })
                            .cloned()
                        {
                            if let Some(field_idx) = field_names.iter().position(|f| f == &field.name) {
                                let field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                                let struct_alloca = self.fresh_tmp();
                                self.emitln(&format!("  {struct_alloca} = alloca {ov_ty}"));
                                self.emitln(&format!("  store {ov_ty} {obj_val}, {ov_ty}* {struct_alloca}"));
                                let gep = self.fresh_tmp();
                                let loaded = self.fresh_tmp();
                                self.emitln(&format!("  {gep} = getelementptr {ov_ty}, {ov_ty}* {struct_alloca}, i32 0, i32 {field_idx}"));
                                self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                return Ok((loaded, field_llvm_ty));
                            }
                        }
                    }
                }
                Ok(("0".to_string(), "i64".to_string()))
            }
            Expr::Call(func, args, _) => {
                // Determine function name and receiver for both direct and method call forms.
                // A callee shaped `base[Type]` (Expr::Index) is an explicit generic
                // instantiation whose type arg the parser preserved as an index;
                // unwrap to the underlying callee `base` so `ptr.null[Int]()` and
                // `foo[T]()` resolve to the function, not a bogus index expression.
                // Only unwrap when the index is a TYPE expression (known type name
                // or generic param), not a VALUE expression like `tests[i]()` where
                // `i` is a loop variable.
                let idx_is_type = |idx: &Expr| -> bool {
                    match idx {
                        Expr::Ident(id) => {
                            Self::is_primitive_type_name(&id.name)
                                || self.types.contains_key(&id.name)
                                || self.type_meta.contains_key(&id.name)
                                || (id.name.len() == 1 && id.name.chars().next().map_or(false, |c| c.is_ascii_uppercase()))
                        }
                        Expr::Field(_, _, _) => true, Expr::Tuple(elems, _) => elems.iter().all(|e| matches!(e, Expr::Ident(_) | Expr::Field(_, _, _))),
                        
                        _ => false, // integer literal, binary expr, etc. — always a value index
                    }
                };
                let (func_unwrapped, mut type_arg): (&Expr, Option<&Expr>) = match &**func {
                    Expr::Index(base, idx, _) if idx_is_type(idx) => (base.as_ref(), Some(idx.as_ref())),
                    other => (other, None),
                };
                let (fn_name_opt, receiver_expr) = match func_unwrapped {
                    Expr::Ident(name) => (Some(name.name.clone()), None),
                    Expr::Field(obj, field, _) => (Some(field.name.clone()), Some(obj)),
                    _ => (None, None),
                };
                // 5c.30: G-10 implicit-self method calls (via receiver_expr
                // handling below; resolution deferred to compile time)
                // Capture type args from receiver_expr for `Map[Str,JsonValue].new()`.
                if type_arg.is_none() {
                    if let Some(ref r) = receiver_expr {
                        if let Expr::Index(_, idx, _) = r.as_ref() {
                            type_arg = Some(idx.as_ref());
                        }
                    }
                }
                let fn_name = match fn_name_opt {
                    Some(ref n) => n.clone(),
                    None => {
                        // The callee is not a simple Ident or Field ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â it may be an
                        // Expr::Index (e.g. `tests[i]()`) that produces a function pointer
                        // value. Compile the expression and call the result.
                        if let Expr::Index(ref container, ref index, _) = **func {
                            return self.compile_index_fn_ptr_call(container, index, args);
                        }
                        // For other complex callee expressions (e.g. chained calls
                        // like `get_fn()()`), compile the callee and inttoptr.
                        let (callee_val, callee_ty) = self.compile_expr(func_unwrapped)?;
                        if callee_ty == "i64" || callee_ty == "i8*" || callee_ty.ends_with('*') {
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
                            let val_i64 = self.val_to_i64(&callee_val, &callee_ty);
                            self.emitln(&format!("  {fn_ptr} = inttoptr i64 {val_i64} to {fn_ptr_ty}"));
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = call i64 {fn_ptr}({args_str})"));
                            return Ok((tmp, "i64".to_string()));
                        }
                        return Ok(("0".to_string(), "i64".to_string()));
                    }
                };
                // Enum variant constructor: TypeName.Variant(args)
                // e.g. `JsonValue.Integer(42)` or `SqliteValue.Text("hello")`
                if let Some(recv) = receiver_expr {
                    let recv_ty = self.infer_struct_type_name(recv);
                    if let Some(recv_name) = recv_ty {
                        let variant_key = format!("{}.{}", recv_name, &fn_name);
                        if self.enum_variants.contains_key(&variant_key)
                            || self.enum_variants.get(&recv_name)
                                .map_or(false, |vars| vars.iter().any(|(v, _)| v == &fn_name))
                        {
                            return self.compile_enum_constructor(&recv_name, &fn_name, args);
                        }
                    }
                }
                // Check for contract collection methods ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â only intercept when
                // there is no user-defined function with the same name; otherwise
                // a regular `fn is_sorted(arr: &Vec[Int]) -> Bool` gets hijacked
                // and replaced with a `call @xiom_is_sorted` builtin.
                let is_contract_method = matches!(fn_name.as_str(), "is_sorted" | "all" | "none" | "contains");
                if is_contract_method {
                    // Skip contract builtin if a user function with this name exists
                    // in the current module or has already been emitted.
                    let has_user_fn = self.emitted_fns.contains(fn_name.as_str())
                        || self.functions.contains_key(fn_name.as_str())
                        || self.emitted_fns.iter().any(|k| k.ends_with(&format!(".{}", fn_name)))
                        || self.functions.keys().any(|k| k.ends_with(&format!(".{}", fn_name)));
                    if !has_user_fn {
                    let tmp = self.fresh_tmp();
                    if let Some(receiver) = &receiver_expr {
                        if self.receiver_is_instance(receiver) {
                            // Method form: receiver.method(args)
                            let (recv_val, recv_llvm_ty) = self.compile_expr(receiver)?;
                            let recv_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {recv_alloca} = alloca {recv_llvm_ty}"));
                            self.emitln(&format!("  store {recv_llvm_ty} {recv_val}, {recv_llvm_ty}* {recv_alloca}"));
                            let ptr = self.fresh_tmp();
                            self.emitln(&format!("  {ptr} = bitcast {recv_llvm_ty}* {recv_alloca} to i8*"));
                            let extra_args: Vec<String> = args.iter()
                                .map(|a| self.compile_expr(a).map(|(v, _)| v))
                                .collect::<Result<Vec<_>, _>>()?;
                            match fn_name.as_str() {
                                "is_sorted" => {
                                    self.emitln(&format!("  {tmp} = call i64 @xiom_is_sorted(i8* {ptr})"));
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
                            return Ok((tmp, "i64".to_string()));
                        }
                    }
                    // Direct form: method(args) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â compile all args
                    let compiled_args: Vec<String> = args.iter()
                        .map(|a| self.compile_expr(a).map(|(v, _)| v))
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
                            self.emitln(&format!("  {tmp} = call i64 @xiom_is_sorted(i8* {ptr})"));
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
                    return Ok((tmp, "i64".to_string()));
                }
                } // if !has_user_fn ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â contract builtin guard
                // Primitive interface methods (Ord.compare, Eq.eq/ne, comparison ops,
                // Hash.hash, Clone.clone) are emitted inline for scalar receivers, so
                // primitives satisfy Ord/Eq/Hash/Clone bounds without a user method.
                let is_builtin_iface_method = matches!(
                    fn_name.as_str(),
                    "compare" | "eq" | "ne" | "lt" | "gt" | "le" | "ge" | "hash" | "clone"
                );
                if is_builtin_iface_method {
                    if let Some(receiver) = receiver_expr {
                        let is_value_instance = self.receiver_is_instance(receiver);
                        // Skip static/type-name receivers (e.g. Int.compare(a, b)).
                        let receiver_is_type_name = matches!(&**receiver, Expr::Ident(id)
                            if Self::is_primitive_type_name(&id.name)
                                || self.types.contains_key(&id.name)
                                || self.type_meta.contains_key(&id.name));
                        // Builtin interface methods only apply to value instances (e.g.
                        // `x.hash()` or `42.hash()`) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â module-qualified calls like
                        // `hash.hash(42)` must fall through to generic dispatch.
                        if !is_value_instance {
                            // module-qualified call ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â fall through to generic dispatch
                        } else {
                        let recv_llvm_ty = self.infer_llvm_type(receiver);
                        // Only scalar (integer/float) receivers get inline handling;
                        // structs use derived/user impls, pointers (Str) fall through.
                        let is_scalar = !receiver_is_type_name
                            && !recv_llvm_ty.starts_with("%struct.")
                            && recv_llvm_ty != "i8*"
                            && recv_llvm_ty != "void";
                        if is_scalar {
                            let (recv_val, _recv_val_ty) = self.compile_expr(receiver)?;
                            let is_float = recv_llvm_ty == "double" || recv_llvm_ty == "float";
                            match fn_name.as_str() {
                                "clone" => return Ok((recv_val, recv_llvm_ty.clone())),
                                "hash" if args.is_empty() => {
                                    if is_float {
                                        let bits = if recv_llvm_ty == "double" { "i64" } else { "i32" };
                                        let cast = self.fresh_tmp();
                                        self.emitln(&format!("  {cast} = bitcast {recv_llvm_ty} {recv_val} to {bits}"));
                                        if bits == "i64" {
                                            return Ok((cast, "i64".to_string()));
                                        }
                                        let ext = self.fresh_tmp();
                                        self.emitln(&format!("  {ext} = sext i32 {cast} to i64"));
                                        return Ok((ext, "i64".to_string()));
                                    }
                                    if recv_llvm_ty == "i64" {
                                        return Ok((recv_val, "i64".to_string()));
                                    }
                                    let ext = self.fresh_tmp();
                                    self.emitln(&format!("  {ext} = sext {recv_llvm_ty} {recv_val} to i64"));
                                    return Ok((ext, "i64".to_string()));
                                }
                                // "hash" with args (Hash interface method call like
                                // `value.hash(hasher)` inside a generic body) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â exit
                                // builtin path entirely so regular method dispatch
                                // resolves to the concrete `Type.hash` function below.
                                "hash" => {}
                                _ => {}
                            }
                            // Skip comparison-op compilation when we handled `hash`
                            // with args via the empty fallthrough above.
                            if fn_name == "hash" {
                                // Exit is_scalar to fall through to regular dispatch
                            } else {
                            let arg_val = if let Some(a) = args.first() {
                                self.compile_expr(a)?.0
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
                                return Ok((res, "i64".to_string()));
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
                            return Ok((res, "i64".to_string()));
                            } // end else (fn_name != "hash")
                        }
                        } // end else (is_value_instance)
                    }
                }
                // Check for memory allocation/free builtins
                if fn_name == "alloc" {
                    let tmp = self.fresh_tmp();
                    if let Some(size_arg) = args.first() {
                        let (size_raw, size_ty) = self.compile_expr(size_arg)?;
                        let size_val = self.val_to_i64(&size_raw, &size_ty);
                        self.emitln(&format!("  {tmp} = call i8* @malloc(i64 {size_val})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i8* @malloc(i64 0)"));
                    }
                    return Ok((tmp, "i8*".to_string()));
                }
                // ptr.null[T]() / ptr.null_mut[T]() / ptr.dangling[T]() ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â generic
                // pointer constructors with NO value arguments. The parser discards
                // explicit type args (`[T]`), so type inference can't specialise them
                // and the generic path returns a bogus 0. Inline them: null ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ 0 (a
                // null pointer), dangling ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ a non-null sentinel (1). Only fires for
                // the zero-arg module form (a module-qualified receiver, not a value
                // instance), so it never shadows a user method on a struct value.
                if args.is_empty()
                    && matches!(fn_name.as_str(), "null" | "null_mut" | "dangling")
                    && receiver_expr.map(|r| !self.receiver_is_instance(r)).unwrap_or(true)
                {
                    let v = if fn_name == "dangling" { "1" } else { "0" };
                    return Ok((v.to_string(), "i64".to_string()));
                }
                // to_string(Int) / x.to_str() / x.to_string() on an integer value:
                // lower to the C runtime `xiom_int_to_string`. The pure-XIOM
                // `to_string` uses fixed stack arrays the codegen can't materialize.
                // 5c.29: never hijack a USER-DEFINED `Type.to_str`/`Type.to_string`
                // (e.g. `HttpMethod.to_str(m)`) â€” fall through to normal dispatch.
                let user_defined_to_str = matches!(fn_name.as_str(), "to_string" | "to_str")
                    && receiver_expr
                        .and_then(|r| self.infer_struct_type_name(r))
                        .map_or(false, |tn| {
                            let key = format!("{tn}.{fn_name}");
                            self.functions.contains_key(&key)
                                || self.functions.keys().any(|k| k.ends_with(&format!(".{key}")))
                        });
                if matches!(fn_name.as_str(), "to_string" | "to_str") && !user_defined_to_str {
                    // Determine the single integer operand (receiver for method form,
                    // first arg for the free-function form).
                    let operand: Option<&Expr> = if let Some(r) = receiver_expr {
                        if self.receiver_is_instance(r) { Some(r) } else { args.first() }
                    } else {
                        args.first()
                    };
                    if let Some(op_expr) = operand {
                        // Bool operand: emit "true"/"false" via a select on the value.
                        if self.expr_is_bool(op_expr) {
                            let (val, vty) = self.compile_expr(op_expr)?;
                            let iv = self.val_to_i64(&val, &vty);
                            let cond = self.fresh_tmp();
                            self.emitln(&format!("  {cond} = icmp ne i64 {iv}, 0"));
                            let tstr = self.intern_cstring("true");
                            let fstr = self.intern_cstring("false");
                            let sel = self.fresh_tmp();
                            self.emitln(&format!("  {sel} = select i1 {cond}, i8* {tstr}, i8* {fstr}"));
                            return Ok((sel, "i8*".to_string()));
                        }
                        // Infer the operand type WITHOUT emitting, so non-integer
                        // receivers (Str, structs) fall through cleanly to normal
                        // dispatch with no double side effects.
                        let ty = self.infer_llvm_type(op_expr);
                        if matches!(ty.as_str(), "i64" | "i32" | "i16" | "i8" | "i1") {
                            let (val, vty) = self.compile_expr(op_expr)?;
                            let iv = self.val_to_i64(&val, &vty);
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = call i8* @xiom_int_to_string(i64 {iv})"));
                            return Ok((tmp, "i8*".to_string()));
                        }
                    }
                }
                // ptr.from_ref(x) / ptr.from_mut(x) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â take a reference to an lvalue
                // and return its ADDRESS as a real pointer. For a plain local/param
                // `x`, return its alloca (the address of the slot). If `x` is itself a
                // pointer local (a `*T`/`&mut Scalar` param, already an address),
                // return that pointer value as-is. Falls back to stashing an rvalue in
                // a fresh alloca and returning its address.
                if matches!(fn_name.as_str(), "from_ref" | "from_mut") && args.len() == 1 {
                    let arg_expr = &args[0];
                    // Unwrap a `&x` / `&mut x` wrapper so `from_mut(&mut a)` still
                    // reaches the underlying lvalue.
                    let inner_expr: &Expr = match arg_expr {
                        Expr::Ref(i, _) | Expr::MutRef(i, _) => i.as_ref(),
                        Expr::Unary(UnaryOp::Ref, i, _) | Expr::Unary(UnaryOp::MutRef, i, _) => i.as_ref(),
                        other => other,
                    };
                    if let Expr::Ident(id) = inner_expr {
                        if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                            if slot_ty.ends_with('*') {
                                // The local already holds a pointer value (a `*T`
                                // param) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â load and return it (identity address).
                                let (val, vty) = self.compile_expr(inner_expr)?;
                                return Ok((val, vty));
                            } else {
                                // Return the address of the local's slot.
                                return Ok((slot, format!("{slot_ty}*")));
                            }
                        }
                    }
                    // Fallback: compile the value, stash it in a fresh alloca, and
                    // return that address so callees receive a valid pointer.
                    let (val, ty) = self.compile_expr(inner_expr)?;
                    if ty == "void" || val.is_empty() {
                        return Ok(("null".to_string(), "i8*".to_string()));
                    }
                    let slot = self.fresh_tmp();
                    self.emitln(&format!("  {slot} = alloca {ty}"));
                    self.emitln(&format!("  store {ty} {val}, {ty}* {slot}"));
                    return Ok((slot, format!("{ty}*")));
                }
                if fn_name == "free" {
                    if let Some(ptr_arg) = args.first() {
                        let (ptr_val, ptr_ty) = self.compile_expr(ptr_arg)?;
                        // Coerce the freed pointer to i8* (it may be typed i64 or a
                        // typed pointer). free() takes i8*.
                        let ptr_i8 = self.val_to_i8ptr(&ptr_val, &ptr_ty);
                        self.emitln(&format!("  call void @free(i8* {ptr_i8})"));
                    }
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "memcpy" && args.len() >= 3 {
                    let (dest_val, _) = self.compile_expr(&args[0])?;
                    let (src_val, _) = self.compile_expr(&args[1])?;
                    let (size_val, _) = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {dest_val}, i8* {src_val}, i64 {size_val}, i1 false)"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                // Vec.new() ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â static method on Vec type
                if let Some(receiver) = receiver_expr {
                    // Accept both `Vec.new()` (Ident receiver) and `Vec[T].new()`
                    // (Index receiver ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the parser wraps the explicit type arg as an
                    // index expression `Vec[T]`). Without unwrapping the Index, the
                    // latter fell through to the fragile general `.new` resolution,
                    // which could bind to an unrelated injected `X.new` (e.g. a
                    // pub-generic `Reverse.new`) returning the wrong struct type.
                    let recv_ident: Option<&str> = match &**receiver {
                        Expr::Ident(id) => Some(id.name.as_str()),
                        Expr::Index(base, _, _) => match base.as_ref() {
                            Expr::Ident(id) => Some(id.name.as_str()),
                            _ => None,
                        },
                        _ => None,
                    };
                    if recv_ident == Some("Vec") && fn_name == "new" {
                            // Determine element size from the type argument.
                            // Vec[UInt8] ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ 1, Vec[Int16] ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ 2, Vec[Int32] ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ 4, default ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ 8.
                            let elem_size: i64 = if let Some(type_arg) = type_arg {
                                let type_name = match type_arg {
                                    Expr::Ident(id) => id.name.clone(),
                                    Expr::Tuple(elems, _) => elems.first()
                                        .map(|e| match e { Expr::Ident(id) => id.name.clone(), _ => "Int".to_string() })
                                        .unwrap_or_else(|| "Int".to_string()),
                                    _ => "Int".to_string(),
                                };
                                match type_name.as_str() {
                                    "UInt8" | "Int8" | "Char" | "Bool" => 1,
                                    "Int16" | "UInt16" => 2,
                                    "Int32" | "UInt32" | "Float32" => 4,
                                    _ => {
                                        // For struct types, compute the REAL layout
                                        // size (nested by-value struct fields count
                                        // fully â€” 5c.30, field_countÃ—8 truncated
                                        // JsonEntry-style elements).
                                        self.struct_byte_size(&type_name)
                                    }
                                }
                            } else { 8 };
                            let initial_cap: i64 = 16;
                            let alloc_size = initial_cap * elem_size;
                            let struct_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {struct_alloca} = alloca %struct.Vec"));
                            let data_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {data_ptr} = call i8* @malloc(i64 {alloc_size})"));
                            // Null check on malloc ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â trap on OOM
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
                            self.emitln(&format!("  store i64 {initial_cap}, i64* {cap_gep}"));
                            let esz_gep = self.fresh_tmp();
                            self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 3"));
                            self.emitln(&format!("  store i64 {elem_size}, i64* {esz_gep}"));
                            let loaded = self.emit_vec_load_fields(&struct_alloca);
                            return Ok((loaded, "%struct.Vec".to_string()));
                    }
                    // 5c-R: Vec.with_capacity(n) — same as Vec.new but with
                    // user-specified initial capacity (G-06).
                    if recv_ident == Some("Vec") && fn_name == "with_capacity" && args.len() == 1 {
                            let (cap_val, cap_ty) = self.compile_expr(&args[0])?;
                            let cap_i64 = self.val_to_i64(&cap_val, &cap_ty);
                            let elem_size: i64 = if let Some(type_arg) = type_arg {
                                let type_name = match type_arg {
                                    Expr::Ident(id) => id.name.clone(),
                                    Expr::Tuple(elems, _) => elems.first()
                                        .map(|e| match e { Expr::Ident(id) => id.name.clone(), _ => "Int".to_string() })
                                        .unwrap_or_else(|| "Int".to_string()),
                                    _ => "Int".to_string(),
                                };
                                match type_name.as_str() {
                                    "UInt8" | "Int8" | "Char" | "Bool" => 1,
                                    "Int16" | "UInt16" => 2,
                                    "Int32" | "UInt32" | "Float32" => 4,
                                    _ => self.struct_byte_size(&type_name),
                                }
                            } else { 8 };
                            let struct_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {struct_alloca} = alloca %struct.Vec"));
                            let data_ptr = self.fresh_tmp();
                            let alloc_size = self.fresh_tmp();
                            self.emitln(&format!("  {alloc_size} = mul i64 {elem_size}, {cap_i64}"));
                            self.emitln(&format!("  {data_ptr} = call i8* @malloc(i64 {alloc_size})"));
                            let null_check = self.fresh_tmp();
                            let ok_block = self.fresh_block("vec_wc_ok");
                            let trap_block = self.fresh_block("vec_wc_trap");
                            self.emitln(&format!("  {null_check} = icmp eq i8* {data_ptr}, null"));
                            self.emitln(&format!("  br i1 {null_check}, label %{trap_block}, label %{ok_block}"));
                            self.emitln(&format!("\n{trap_block}:"));
                            self.emitln("  call void @llvm.trap()");
                            self.emitln("  unreachable");
                            self.emitln(&format!("\n{ok_block}:"));
                            let dg = self.fresh_tmp();
                            self.emitln(&format!("  {dg} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 0"));
                            self.emitln(&format!("  store i8* {data_ptr}, i8** {dg}"));
                            let lg = self.fresh_tmp();
                            self.emitln(&format!("  {lg} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 1"));
                            self.emitln(&format!("  store i64 0, i64* {lg}"));
                            let cg = self.fresh_tmp();
                            self.emitln(&format!("  {cg} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 2"));
                            self.emitln(&format!("  store i64 {cap_i64}, i64* {cg}"));
                            let eg = self.fresh_tmp();
                            self.emitln(&format!("  {eg} = getelementptr %struct.Vec, %struct.Vec* {struct_alloca}, i32 0, i32 3"));
                            self.emitln(&format!("  store i64 {elem_size}, i64* {eg}"));
                            let loaded = self.emit_vec_load_fields(&struct_alloca);
                            return Ok((loaded, "%struct.Vec".to_string()));
                    }
                }
                // Vec.push(vec, val) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â method call on Vec
                if fn_name == "push" && args.len() >= 1 {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        // Accept any Vec-typed receiver (Vec[Int], Vec[UInt8], a
                        // module-qualified `%struct.xiom.collections.Vec`, etc.),
                        // including i64 container-field handles (5c.29).
                        let is_vec = recv_ty == "%struct.Vec" || recv_ty.ends_with(".Vec") || recv_ty.contains("struct.Vec")
                            || self.is_container_vec_field(receiver);
                        if !is_vec {
                            // Not a Vec receiver ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚ fall through to general method dispatch
                        } else {
                        let (recv_val, recv_actual_ty) = self.compile_expr(receiver)?;
                        let (recv_vec, _) = self.resolve_vec_receiver(receiver, &recv_val, &recv_actual_ty);
                        let (val_raw, val_ty) = self.compile_expr(&args[0])?;
                        let vec_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                        // Store Vec via extractvalue+individual stores to prevent
                        // LLVM SROA from decomposing the struct write (5c.28).
                        let vec_data = self.fresh_tmp();
                        let vec_len = self.fresh_tmp();
                        let vec_cap = self.fresh_tmp();
                        let vec_esz = self.fresh_tmp();
                        self.emitln(&format!("  {vec_data} = extractvalue %struct.Vec {recv_vec}, 0"));
                        self.emitln(&format!("  {vec_len} = extractvalue %struct.Vec {recv_vec}, 1"));
                        self.emitln(&format!("  {vec_cap} = extractvalue %struct.Vec {recv_vec}, 2"));
                        self.emitln(&format!("  {vec_esz} = extractvalue %struct.Vec {recv_vec}, 3"));
                        let d_gep = self.fresh_tmp();
                        let l_gep = self.fresh_tmp();
                        let c_gep = self.fresh_tmp();
                        let e_gep = self.fresh_tmp();
                        self.emitln(&format!("  {d_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
                        self.emitln(&format!("  store i8* {vec_data}, i8** {d_gep}"));
                        self.emitln(&format!("  {l_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
                        self.emitln(&format!("  store i64 {vec_len}, i64* {l_gep}"));
                        self.emitln(&format!("  {c_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 2"));
                        self.emitln(&format!("  store i64 {vec_cap}, i64* {c_gep}"));
                        self.emitln(&format!("  {e_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
                        self.emitln(&format!("  store i64 {vec_esz}, i64* {e_gep}"));
                        // Load elem_size early ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â needed to decide struct vs scalar path
                        let esz_gep = self.fresh_tmp();
                        let esz_val = self.fresh_tmp();
                        self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
                        self.emitln(&format!("  {esz_val} = load i64, i64* {esz_gep}"));
                        // For struct elements >8 bytes, skip val_to_i64 (which would
                        // heap-allocate) and use memcpy to store the struct inline.
                        let is_struct_elem = val_ty.starts_with('%') && {
                            let tn = val_ty.trim_start_matches("%struct.").trim_end_matches('*');
                            self.types.contains_key(tn)
                                || self.types.keys().any(|k| k.ends_with(&format!(".{tn}")))
                        };
                        let val = if !is_struct_elem {
                            self.val_to_i64(&val_raw, &val_ty)
                        } else {
                            // For structs, the raw value is preserved for memcpy.
                            // We emit a dummy i64; the store block will use val_raw directly.
                            val_raw.clone() // not used as i64 ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the store block checks is_struct_elem
                        };
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
                        // Capacity guard: trap if exceeding max (2^20 elements ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â°ÃƒÆ’Ã¢â‚¬Â¹ÃƒÂ¢Ã¢â€šÂ¬Ã‚Â  8MB)
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
                        self.emitln(&format!("  {new_size} = mul i64 {new_cap}, {esz_val}"));
                        let grow_data_gep = self.fresh_tmp();
                        let grow_data_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {grow_data_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
                        self.emitln(&format!("  {grow_data_ptr} = load i8*, i8** {grow_data_gep}"));
                        let new_data = self.fresh_tmp();
                        self.emitln(&format!("  {new_data} = call i8* @realloc(i8* {grow_data_ptr}, i64 {new_size})"));
                        // Null check on realloc ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â trap on OOM
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
                        self.emitln(&format!("  {offset} = mul i64 {len_val}, {esz_val}"));
                        let dest = self.fresh_tmp();
                        self.emitln(&format!("  {dest} = getelementptr i8, i8* {store_data_ptr}, i64 {offset}"));
                        // Store with the correct element width: narrow types (UInt8/Char)
                        // use truncated stores to avoid overwriting adjacent elements.
                        // For struct elements >8 bytes, memcpy the full struct from
                        // a temporary alloca (val_raw is the struct value, val_ty is
                        // its LLVM type like %struct.HttpHeader).
                        if is_struct_elem {
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = alloca {val_ty}"));
                            self.emitln(&format!("  store {val_ty} {val_raw}, {val_ty}* {tmp}"));
                            let tmp_i8 = self.fresh_tmp();
                            self.emitln(&format!("  {tmp_i8} = bitcast {val_ty}* {tmp} to i8*"));
                            self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {dest}, i8* {tmp_i8}, i64 {esz_val}, i1 false)"));
                        } else {
                            self.emit_elem_store(&val, &dest, &esz_val);
                        }
                        let new_len = self.fresh_tmp();
                        self.emitln(&format!("  {new_len} = add i64 {len_val}, 1"));
                        self.emitln(&format!("  store i64 {new_len}, i64* {len_gep}"));
                        let loaded = self.emit_vec_load_fields(&vec_alloca);
                        // Write the mutated Vec back to the receiver variable so the
                        // updated len/cap/data persist (value semantics: `v.push(x)`
                        // must be observable via `v` afterwards).
                        self.store_back_to_receiver(receiver, &loaded, "%struct.Vec");
                        return Ok((loaded, "%struct.Vec".to_string()));
                        }
                    }
                }
                // Vec.pop(vec) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â method call on Vec. Returns Option[T]: None when
                // empty (discriminant 0), else Some(last element) (discriminant 1,
                // value = element). A Vec is the builtin {i8*, i64, i64}; elements
                // are i64-wide slots. The pop reads the last live element; the
                // returned struct is a %struct.Option so `v.pop() == Some(x)` typechecks.
                // Vec.insert(idx, val) / Vec.remove(idx) â€” inline builtins
                // (5c.29). The stdlib generic versions relied on general method
                // dispatch, which cannot resolve container-field receivers
                // (`tree.nodes[idx].keys.insert(...)` misdispatched to a stub).
                // Shifting uses llvm.memmove so it is element-size agnostic
                // (works for 32-byte struct elements too).
                if (fn_name == "insert" && args.len() == 2) || (fn_name == "remove" && args.len() == 1) {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_vec = recv_ty == "%struct.Vec" || recv_ty.ends_with(".Vec") || recv_ty.contains("struct.Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            let is_insert = fn_name == "insert";
                            if !is_insert { self.used_builtins.insert("Option".to_string()); }
                            let (hdr, needs_store_back) = self.resolve_vec_receiver_ptr(receiver)?;
                            let (idx_raw, idx_ty) = self.compile_expr(&args[0])?;
                            let idx = self.val_to_i64(&idx_raw, &idx_ty);
                            // Header fields
                            let esz_gep = self.fresh_tmp();
                            let esz = self.fresh_tmp();
                            self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {hdr}, i32 0, i32 3"));
                            self.emitln(&format!("  {esz} = load i64, i64* {esz_gep}"));
                            let len_gep = self.fresh_tmp();
                            let len = self.fresh_tmp();
                            self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {hdr}, i32 0, i32 1"));
                            self.emitln(&format!("  {len} = load i64, i64* {len_gep}"));
                            // Option result slot (remove); insert ignores it.
                            let opt_slot = self.fresh_tmp();
                            self.emitln(&format!("  {opt_slot} = alloca %struct.Option"));
                            let od_gep = self.fresh_tmp();
                            self.emitln(&format!("  {od_gep} = getelementptr %struct.Option, %struct.Option* {opt_slot}, i32 0, i32 0"));
                            self.emitln(&format!("  store i64 0, i64* {od_gep}"));
                            let ov_gep = self.fresh_tmp();
                            self.emitln(&format!("  {ov_gep} = getelementptr %struct.Option, %struct.Option* {opt_slot}, i32 0, i32 1"));
                            self.emitln(&format!("  store i64 0, i64* {ov_gep}"));
                            // Bounds: insert allows 0..=len, remove allows 0..len.
                            let ge0 = self.fresh_tmp();
                            self.emitln(&format!("  {ge0} = icmp sge i64 {idx}, 0"));
                            let ub_ok = self.fresh_tmp();
                            if is_insert {
                                self.emitln(&format!("  {ub_ok} = icmp sle i64 {idx}, {len}"));
                            } else {
                                self.emitln(&format!("  {ub_ok} = icmp slt i64 {idx}, {len}"));
                            }
                            let in_bounds = self.fresh_tmp();
                            self.emitln(&format!("  {in_bounds} = and i1 {ge0}, {ub_ok}"));
                            let body_block = self.fresh_block("vecmod_body");
                            let done_block = self.fresh_block("vecmod_done");
                            self.emitln(&format!("  br i1 {in_bounds}, label %{body_block}, label %{done_block}"));
                            self.emitln(&format!("\n{body_block}:"));
                            if is_insert {
                                let (val_raw, val_ty) = self.compile_expr(&args[1])?;
                                let is_struct_elem = val_ty.starts_with('%') && {
                                    let tn = val_ty.trim_start_matches("%struct.").trim_end_matches('*');
                                    self.types.contains_key(tn)
                                        || self.types.keys().any(|k| k.ends_with(&format!(".{tn}")))
                                };
                                let val_i64 = if is_struct_elem { val_raw.clone() } else { self.val_to_i64(&val_raw, &val_ty) };
                                // Grow when len == cap.
                                let cap_gep = self.fresh_tmp();
                                let cap = self.fresh_tmp();
                                self.emitln(&format!("  {cap_gep} = getelementptr %struct.Vec, %struct.Vec* {hdr}, i32 0, i32 2"));
                                self.emitln(&format!("  {cap} = load i64, i64* {cap_gep}"));
                                let need_grow = self.fresh_tmp();
                                self.emitln(&format!("  {need_grow} = icmp uge i64 {len}, {cap}"));
                                let grow_block = self.fresh_block("vecmod_grow");
                                let shift_block = self.fresh_block("vecmod_shift");
                                self.emitln(&format!("  br i1 {need_grow}, label %{grow_block}, label %{shift_block}"));
                                self.emitln(&format!("\n{grow_block}:"));
                                let new_cap = self.fresh_tmp();
                                self.emitln(&format!("  {new_cap} = mul i64 {cap}, 2"));
                                let cap_ok = self.fresh_tmp();
                                let cap_ok_block = self.fresh_block("vecmod_cap_ok");
                                let cap_trap_block = self.fresh_block("vecmod_cap_trap");
                                self.emitln(&format!("  {cap_ok} = icmp ule i64 {new_cap}, 1048576"));
                                self.emitln(&format!("  br i1 {cap_ok}, label %{cap_ok_block}, label %{cap_trap_block}"));
                                self.emitln(&format!("\n{cap_trap_block}:"));
                                self.emitln("  call void @llvm.trap()");
                                self.emitln("  unreachable");
                                self.emitln(&format!("\n{cap_ok_block}:"));
                                let new_size = self.fresh_tmp();
                                self.emitln(&format!("  {new_size} = mul i64 {new_cap}, {esz}"));
                                let gd_gep = self.fresh_tmp();
                                let gd_ptr = self.fresh_tmp();
                                self.emitln(&format!("  {gd_gep} = getelementptr %struct.Vec, %struct.Vec* {hdr}, i32 0, i32 0"));
                                self.emitln(&format!("  {gd_ptr} = load i8*, i8** {gd_gep}"));
                                let new_data = self.fresh_tmp();
                                self.emitln(&format!("  {new_data} = call i8* @realloc(i8* {gd_ptr}, i64 {new_size})"));
                                let re_null = self.fresh_tmp();
                                let re_ok = self.fresh_block("vecmod_realloc_ok");
                                let re_trap = self.fresh_block("vecmod_realloc_trap");
                                self.emitln(&format!("  {re_null} = icmp eq i8* {new_data}, null"));
                                self.emitln(&format!("  br i1 {re_null}, label %{re_trap}, label %{re_ok}"));
                                self.emitln(&format!("\n{re_trap}:"));
                                self.emitln("  call void @llvm.trap()");
                                self.emitln("  unreachable");
                                self.emitln(&format!("\n{re_ok}:"));
                                self.emitln(&format!("  store i8* {new_data}, i8** {gd_gep}"));
                                self.emitln(&format!("  store i64 {new_cap}, i64* {cap_gep}"));
                                self.emitln(&format!("  br label %{shift_block}"));
                                // Shift [idx..len) right by one element via memmove.
                                self.emitln(&format!("\n{shift_block}:"));
                                let d_gep = self.fresh_tmp();
                                let d_ptr = self.fresh_tmp();
                                self.emitln(&format!("  {d_gep} = getelementptr %struct.Vec, %struct.Vec* {hdr}, i32 0, i32 0"));
                                self.emitln(&format!("  {d_ptr} = load i8*, i8** {d_gep}"));
                                let src_off = self.fresh_tmp();
                                self.emitln(&format!("  {src_off} = mul i64 {idx}, {esz}"));
                                let src_ptr = self.fresh_tmp();
                                self.emitln(&format!("  {src_ptr} = getelementptr i8, i8* {d_ptr}, i64 {src_off}"));
                                let dst_off = self.fresh_tmp();
                                self.emitln(&format!("  {dst_off} = add i64 {src_off}, {esz}"));
                                let dst_ptr = self.fresh_tmp();
                                self.emitln(&format!("  {dst_ptr} = getelementptr i8, i8* {d_ptr}, i64 {dst_off}"));
                                let tail_elems = self.fresh_tmp();
                                self.emitln(&format!("  {tail_elems} = sub i64 {len}, {idx}"));
                                let tail_bytes = self.fresh_tmp();
                                self.emitln(&format!("  {tail_bytes} = mul i64 {tail_elems}, {esz}"));
                                self.emitln(&format!("  call void @llvm.memmove.p0i8.p0i8.i64(i8* {dst_ptr}, i8* {src_ptr}, i64 {tail_bytes}, i1 false)"));
                                // Store the new element at idx.
                                if is_struct_elem {
                                    let tmp = self.fresh_tmp();
                                    self.emitln(&format!("  {tmp} = alloca {val_ty}"));
                                    self.emitln(&format!("  store {val_ty} {val_raw}, {val_ty}* {tmp}"));
                                    let tmp_i8 = self.fresh_tmp();
                                    self.emitln(&format!("  {tmp_i8} = bitcast {val_ty}* {tmp} to i8*"));
                                    self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {src_ptr}, i8* {tmp_i8}, i64 {esz}, i1 false)"));
                                } else {
                                    self.emit_elem_store(&val_i64, &src_ptr, &esz);
                                }
                                let new_len = self.fresh_tmp();
                                self.emitln(&format!("  {new_len} = add i64 {len}, 1"));
                                self.emitln(&format!("  store i64 {new_len}, i64* {len_gep}"));
                                self.emitln(&format!("  br label %{done_block}"));
                            } else {
                                // remove(idx): capture the old element, shift left, len-1.
                                let d_gep = self.fresh_tmp();
                                let d_ptr = self.fresh_tmp();
                                self.emitln(&format!("  {d_gep} = getelementptr %struct.Vec, %struct.Vec* {hdr}, i32 0, i32 0"));
                                self.emitln(&format!("  {d_ptr} = load i8*, i8** {d_gep}"));
                                let at_off = self.fresh_tmp();
                                self.emitln(&format!("  {at_off} = mul i64 {idx}, {esz}"));
                                let at_ptr = self.fresh_tmp();
                                self.emitln(&format!("  {at_ptr} = getelementptr i8, i8* {d_ptr}, i64 {at_off}"));
                                let old_val = self.emit_elem_payload_load(receiver, &at_ptr, &esz);
                                let next_off = self.fresh_tmp();
                                self.emitln(&format!("  {next_off} = add i64 {at_off}, {esz}"));
                                let next_ptr = self.fresh_tmp();
                                self.emitln(&format!("  {next_ptr} = getelementptr i8, i8* {d_ptr}, i64 {next_off}"));
                                let tail_elems = self.fresh_tmp();
                                self.emitln(&format!("  {tail_elems} = sub i64 {len}, {idx}"));
                                let tail_elems1 = self.fresh_tmp();
                                self.emitln(&format!("  {tail_elems1} = sub i64 {tail_elems}, 1"));
                                let tail_bytes = self.fresh_tmp();
                                self.emitln(&format!("  {tail_bytes} = mul i64 {tail_elems1}, {esz}"));
                                self.emitln(&format!("  call void @llvm.memmove.p0i8.p0i8.i64(i8* {at_ptr}, i8* {next_ptr}, i64 {tail_bytes}, i1 false)"));
                                let new_len = self.fresh_tmp();
                                self.emitln(&format!("  {new_len} = sub i64 {len}, 1"));
                                self.emitln(&format!("  store i64 {new_len}, i64* {len_gep}"));
                                self.emitln(&format!("  store i64 1, i64* {od_gep}"));
                                self.emitln(&format!("  store i64 {old_val}, i64* {ov_gep}"));
                                self.emitln(&format!("  br label %{done_block}"));
                            }
                            self.emitln(&format!("\n{done_block}:"));
                            // Persist mutations for non-handle receivers.
                            if needs_store_back {
                                let loaded = self.emit_vec_load_fields(&hdr);
                                self.store_back_to_receiver(receiver, &loaded, "%struct.Vec");
                            }
                            if is_insert {
                                let loaded = self.emit_vec_load_fields(&hdr);
                                return Ok((loaded, "%struct.Vec".to_string()));
                            }
                            let opt_val = self.fresh_tmp();
                            self.emitln(&format!("  {opt_val} = load %struct.Option, %struct.Option* {opt_slot}"));
                            return Ok((opt_val, "%struct.Option".to_string()));
                        }
                    }
                }
                if fn_name == "pop" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_vec = recv_ty == "%struct.Vec" || recv_ty.ends_with(".Vec") || recv_ty.contains("struct.Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            self.used_builtins.insert("Option".to_string());
                            let (recv_val, recv_actual_ty) = self.compile_expr(receiver)?;
                            let (recv_vec, _) = self.resolve_vec_receiver(receiver, &recv_val, &recv_actual_ty);
                            let vec_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                            self.emit_vec_store_fields(&recv_vec, &vec_alloca);
                            let len_gep = self.fresh_tmp();
                            self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
                            let len_val = self.fresh_tmp();
                            self.emitln(&format!("  {len_val} = load i64, i64* {len_gep}"));
                            let opt_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {opt_alloca} = alloca %struct.Option"));
                            let is_empty = self.fresh_tmp();
                            self.emitln(&format!("  {is_empty} = icmp sle i64 {len_val}, 0"));
                            let empty_block = self.fresh_block("vec_pop_empty");
                            let some_block = self.fresh_block("vec_pop_some");
                            let done_block = self.fresh_block("vec_pop_done");
                            self.emitln(&format!("  br i1 {is_empty}, label %{empty_block}, label %{some_block}"));
                            // None
                            self.emitln(&format!("\n{empty_block}:"));
                            let none_disc = self.fresh_tmp();
                            self.emitln(&format!("  {none_disc} = getelementptr %struct.Option, %struct.Option* {opt_alloca}, i32 0, i32 0"));
                            self.emitln(&format!("  store i64 0, i64* {none_disc}"));
                            let none_val = self.fresh_tmp();
                            self.emitln(&format!("  {none_val} = getelementptr %struct.Option, %struct.Option* {opt_alloca}, i32 0, i32 1"));
                            self.emitln(&format!("  store i64 0, i64* {none_val}"));
                            self.emitln(&format!("  br label %{done_block}"));
                            // Some(last)
                            self.emitln(&format!("\n{some_block}:"));
                            let new_len = self.fresh_tmp();
                            self.emitln(&format!("  {new_len} = sub i64 {len_val}, 1"));
                            self.emitln(&format!("  store i64 {new_len}, i64* {len_gep}"));
                            // Load elem_size for byte-offset
                            let esz_gep = self.fresh_tmp();
                            let esz_val = self.fresh_tmp();
                            self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
                            self.emitln(&format!("  {esz_val} = load i64, i64* {esz_gep}"));
                            let data_gep = self.fresh_tmp();
                            self.emitln(&format!("  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
                            let data_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                            let byte_off = self.fresh_tmp();
                            self.emitln(&format!("  {byte_off} = mul i64 {new_len}, {esz_val}"));
                            let elem_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {data_ptr}, i64 {byte_off}"));
                            let elem = self.emit_elem_payload_load(receiver, &elem_ptr, &esz_val);
                            let some_disc = self.fresh_tmp();
                            self.emitln(&format!("  {some_disc} = getelementptr %struct.Option, %struct.Option* {opt_alloca}, i32 0, i32 0"));
                            self.emitln(&format!("  store i64 1, i64* {some_disc}"));
                            let some_val = self.fresh_tmp();
                            self.emitln(&format!("  {some_val} = getelementptr %struct.Option, %struct.Option* {opt_alloca}, i32 0, i32 1"));
                            self.emitln(&format!("  store i64 {elem}, i64* {some_val}"));
                            self.emitln(&format!("  br label %{done_block}"));
                            self.emitln(&format!("\n{done_block}:"));
                            // Persist the (possibly decremented) Vec back to the
                            // receiver variable so the pop is observable via `v`.
                            let vec_back = self.emit_vec_load_fields(&vec_alloca);
                            self.store_back_to_receiver(receiver, &vec_back, "%struct.Vec");
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load %struct.Option, %struct.Option* {opt_alloca}"));
                            return Ok((loaded, "%struct.Option".to_string()));
                        }
                    }
                }
                // Vec.get(vec, idx) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â method call on Vec. Returns Option[T]: None
                // when idx is out of range (discriminant 0), else Some(data[idx])
                // (discriminant 1, value = element). Mirrors the pop lowering.
                if fn_name == "get" && args.len() == 1 {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_vec = recv_ty == "%struct.Vec" || recv_ty.ends_with(".Vec") || recv_ty.contains("struct.Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            self.used_builtins.insert("Option".to_string());
                            let (recv_val, recv_actual_ty) = self.compile_expr(receiver)?;
                            let (recv_vec, _) = self.resolve_vec_receiver(receiver, &recv_val, &recv_actual_ty);
                            let (idx_raw, idx_ty) = self.compile_expr(&args[0])?;
                            let idx = self.val_to_i64(&idx_raw, &idx_ty);
                            let vec_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                            self.emit_vec_store_fields(&recv_vec, &vec_alloca);
                            let len_gep = self.fresh_tmp();
                            self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
                            let len_val = self.fresh_tmp();
                            self.emitln(&format!("  {len_val} = load i64, i64* {len_gep}"));
                            let opt_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {opt_alloca} = alloca %struct.Option"));
                            // in range iff (unsigned) idx < len ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â also rejects idx<0.
                            let in_range = self.fresh_tmp();
                            self.emitln(&format!("  {in_range} = icmp ult i64 {idx}, {len_val}"));
                            let some_block = self.fresh_block("vec_get_some");
                            let none_block = self.fresh_block("vec_get_none");
                            let done_block = self.fresh_block("vec_get_done");
                            self.emitln(&format!("  br i1 {in_range}, label %{some_block}, label %{none_block}"));
                            self.emitln(&format!("\n{none_block}:"));
                            let none_disc = self.fresh_tmp();
                            self.emitln(&format!("  {none_disc} = getelementptr %struct.Option, %struct.Option* {opt_alloca}, i32 0, i32 0"));
                            self.emitln(&format!("  store i64 0, i64* {none_disc}"));
                            let none_val = self.fresh_tmp();
                            self.emitln(&format!("  {none_val} = getelementptr %struct.Option, %struct.Option* {opt_alloca}, i32 0, i32 1"));
                            self.emitln(&format!("  store i64 0, i64* {none_val}"));
                            self.emitln(&format!("  br label %{done_block}"));
                            self.emitln(&format!("\n{some_block}:"));
                            // Load elem_size for byte-offset
                            let esz_gep = self.fresh_tmp();
                            let esz_val = self.fresh_tmp();
                            self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
                            self.emitln(&format!("  {esz_val} = load i64, i64* {esz_gep}"));
                            let data_gep = self.fresh_tmp();
                            self.emitln(&format!("  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
                            let data_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                            let byte_off = self.fresh_tmp();
                            self.emitln(&format!("  {byte_off} = mul i64 {idx}, {esz_val}"));
                            let elem_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {data_ptr}, i64 {byte_off}"));
                            let elem = self.emit_elem_payload_load(receiver, &elem_ptr, &esz_val);
                            let some_disc = self.fresh_tmp();
                            self.emitln(&format!("  {some_disc} = getelementptr %struct.Option, %struct.Option* {opt_alloca}, i32 0, i32 0"));
                            self.emitln(&format!("  store i64 1, i64* {some_disc}"));
                            let some_val = self.fresh_tmp();
                            self.emitln(&format!("  {some_val} = getelementptr %struct.Option, %struct.Option* {opt_alloca}, i32 0, i32 1"));
                            self.emitln(&format!("  store i64 {elem}, i64* {some_val}"));
                            self.emitln(&format!("  br label %{done_block}"));
                            self.emitln(&format!("\n{done_block}:"));
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load %struct.Option, %struct.Option* {opt_alloca}"));
                            return Ok((loaded, "%struct.Option".to_string()));
                        }
                    }
                }
                // Vec.len(vec) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â method call on Vec
                // Vec.len(vec) â€” method call on Vec
                if fn_name == "len" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        if self.infer_llvm_type(receiver) != "%struct.Vec"
                            && !self.is_container_vec_field(receiver)
                        {
                            // Not a Vec receiver â€” fall through to general method dispatch
                        } else {
                        let (recv_raw, recv_raw_ty) = self.compile_expr(receiver)?;
                        let (recv_val, _) = self.resolve_vec_receiver(receiver, &recv_raw, &recv_raw_ty);
                        let vec_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                        self.emit_vec_store_fields(&recv_val, &vec_alloca);
                        let len_gep = self.fresh_tmp();
                        let len_val = self.fresh_tmp();
                        self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
                        self.emitln(&format!("  {len_val} = load i64, i64* {len_gep}"));
                        return Ok((len_val, "i64".to_string()));
                        }
                    }
                }
                // Str.len(s) / Vec.len / Slice.len ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â method call
                if fn_name == "len" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        if recv_ty == "i8*" || recv_ty == "ptr" {
                            let (recv_val, _) = self.compile_expr(receiver)?;
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = call i64 @xiom_str_len(i8* {recv_val})"));
                            return Ok((tmp, "i64".to_string()));
                        }
                        // Pointer-typed array references from monomorphised generics
                        // (e.g. &Slice[Int] -> i64*): length is at buf[0].
                        if recv_ty.ends_with('*') && recv_ty != "i8*" {
                            let (recv_val, _) = self.compile_expr(receiver)?;
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = load i64, {recv_ty} {recv_val}"));
                            return Ok((tmp, "i64".to_string()));
                        }
                        // Vec/Slice: length is field 1 of the {ptr, len, cap} struct.
                        if recv_ty == "%struct.Vec" || recv_ty.ends_with(".Vec")
                            || recv_ty == "%struct.Slice" || recv_ty.ends_with(".Slice")
                            || recv_ty.contains("struct.Vec") || recv_ty.contains("struct.Slice")
                            || self.is_container_vec_field(receiver)
                        {
                            let (recv_val, rty) = self.compile_expr(receiver)?;
                            let (recv_vec, vec_ty) = self.resolve_vec_receiver(receiver, &recv_val, &rty);
                            let slot = self.fresh_tmp();
                            self.emitln(&format!("  {slot} = alloca {vec_ty}"));
                            self.emitln(&format!("  store {vec_ty} {recv_vec}, {vec_ty}* {slot}"));
                            let gep = self.fresh_tmp();
                            self.emitln(&format!("  {gep} = getelementptr {vec_ty}, {vec_ty}* {slot}, i32 0, i32 1"));
                            let lenv = self.fresh_tmp();
                            self.emitln(&format!("  {lenv} = load i64, i64* {gep}"));
                            return Ok((lenv, "i64".to_string()));
                        }
                    }
                }
                // Str.c_str() ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â identity on the string pointer (Str is already i8*).
                // Str.len() / Str.byte_len() ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â return the string length.
                if (fn_name == "c_str" || fn_name == "byte_len" || fn_name == "len") && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_str = recv_ty == "i8*" || recv_ty.contains(".Str");
                        if fn_name == "c_str" {
                            let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                            let ptr = self.coerce_value(&recv_val, &recv_ty, "i8*");
                            return Ok((ptr, "i8*".to_string()));
                        } else if is_str {
                            // .len() / .byte_len(): only for Str receivers
                            let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                            let ptr = self.coerce_value(&recv_val, &recv_ty, "i8*");
                            let len_tmp = self.fresh_tmp();
                            self.emitln(&format!("  {len_tmp} = call i64 @strlen(i8* {ptr})"));
                            return Ok((len_tmp, "i64".to_string()));
                        }
                    }
                }
                // Str.from_cstring(ptr) / from_c_str / from_utf8 ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â reinterpret a
                // C string / byte buffer as a Str. A Str is `i8*` at the ABI and a
                // C string is already a NUL-terminated i8*, so this is an identity
                // on the pointer (coerced to i8*). Emitted inline since there is no
                // runtime function.
                if matches!(fn_name.as_str(), "from_cstring" | "from_c_str" | "from_utf8" | "from_bytes")
                    && !args.is_empty()
                {
                    let (arg_val, arg_ty) = self.compile_expr(&args[0])?;
                    let as_ptr = self.coerce_value(&arg_val, &arg_ty, "i8*");
                    return Ok((as_ptr, "i8*".to_string()));
                }
                let compiled_args: Vec<(String, String)> = args.iter()
                    .map(|a| self.compile_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;
                if fn_name == "io" {
                    if let Some((arg, _)) = compiled_args.first() {
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i32 @puts(i8* {arg})"));
                        return Ok((tmp, "i32".to_string()));
                    }
                    return Ok(("0".to_string(), "i64".to_string()));
                }
                // Extern runtime functions for file I/O
                if fn_name == "xiom_read_file" {
                    let tmp = self.fresh_tmp();
                    if let Some(path_arg) = args.first() {
                        let (path_ptr, _) = self.compile_expr(path_arg)?;
                        self.emitln(&format!("  {tmp} = call i8* @xiom_read_file(i8* {path_ptr})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i8* @xiom_read_file(i8* null)"));
                    }
                    let tmp_int = self.fresh_tmp();
                    self.emitln(&format!("  {tmp_int} = ptrtoint i8* {tmp} to i64"));
                    return Ok((tmp_int, "i64".to_string()));
                }
                if fn_name == "xiom_file_size" {
                    let tmp = self.fresh_tmp();
                    if let Some(path_arg) = args.first() {
                        let (path_ptr, _) = self.compile_expr(path_arg)?;
                        self.emitln(&format!("  {tmp} = call i64 @xiom_file_size(i8* {path_ptr})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i64 @xiom_file_size(i8* null)"));
                    }
                    return Ok((tmp, "i64".to_string()));
                }
                if fn_name == "xiom_free" {
                    if let Some(ptr_arg) = args.first() {
                        let (ptr_val, ptr_ty) = self.compile_expr(ptr_arg)?;
                        let ptr_ptr = self.val_to_i8ptr(&ptr_val, &ptr_ty);
                        self.emitln(&format!("  call void @xiom_free(i8* {ptr_ptr})"));
                    }
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_char_at" && args.len() >= 2 {
                    let (src, src_ty) = self.compile_expr(&args[0])?;
                    let (pos, _) = self.compile_expr(&args[1])?;
                    let tmp = self.fresh_tmp();
                    let tmp_ext = self.fresh_tmp();
                    let src_ptr = self.val_to_i8ptr(&src, &src_ty);
                    self.emitln(&format!("  {tmp} = call i8 @xiom_char_at(i8* {src_ptr}, i64 {pos})"));
                    self.emitln(&format!("  {tmp_ext} = zext i8 {tmp} to i64"));
                    return Ok((tmp_ext, "i64".to_string()));
                }
                if fn_name == "xiom_str_len" && args.len() >= 1 {
                    let (src, src_ty) = self.compile_expr(&args[0])?;
                    let tmp = self.fresh_tmp();
                    let src_ptr = self.val_to_i8ptr(&src, &src_ty);
                    self.emitln(&format!("  {tmp} = call i64 @xiom_str_len(i8* {src_ptr})"));
                    return Ok((tmp, "i64".to_string()));
                }
                // v0.9.4 string-based IR emission externs
                if fn_name == "xiom_ir_define_s" && args.len() >= 2 {
                    let (name, _) = self.compile_expr(&args[0])?;
                    let (ret_type, _) = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @xiom_ir_define_s(i8* {name}, i8* {ret_type})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_param_int" && args.len() >= 1 {
                    let (index, _) = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_param_int(i64 {index})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_param_double" && args.len() >= 1 {
                    let (index, _) = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_param_double(i64 {index})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_alloca_s" && args.len() >= 1 {
                    let (reg, _) = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_alloca_s(i64 {reg})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_store_param" && args.len() >= 2 {
                    let (reg, _) = self.compile_expr(&args[0])?;
                    let (param, _) = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @xiom_ir_store_param(i64 {reg}, i64 {param})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_load_s" && args.len() >= 2 {
                    let (reg, _) = self.compile_expr(&args[0])?;
                    let (from_reg, _) = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @xiom_ir_load_s(i64 {reg}, i64 {from_reg})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_add" && args.len() >= 3 {
                    let (dst, _) = self.compile_expr(&args[0])?;
                    let (left, _) = self.compile_expr(&args[1])?;
                    let (right, _) = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @xiom_ir_add(i64 {dst}, i64 {left}, i64 {right})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_fmul" && args.len() >= 3 {
                    let (dst, _) = self.compile_expr(&args[0])?;
                    let (left, _) = self.compile_expr(&args[1])?;
                    let (right, _) = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @xiom_ir_fmul(i64 {dst}, i64 {left}, i64 {right})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_call_fn" && args.len() >= 3 {
                    let (dst, _) = self.compile_expr(&args[0])?;
                    let (fn_name_str, _) = self.compile_expr(&args[1])?;
                    let (ret_type, _) = self.compile_expr(&args[2])?;
                    self.emitln(&format!("  call void @xiom_ir_call_fn(i64 {dst}, i8* {fn_name_str}, i8* {ret_type})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_call_arg_lit" && args.len() >= 2 {
                    let (ty, _) = self.compile_expr(&args[0])?;
                    let (val, _) = self.compile_expr(&args[1])?;
                    self.emitln(&format!("  call void @xiom_ir_call_arg_lit(i8* {ty}, i8* {val})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_ret_reg" && args.len() >= 1 {
                    let (reg, _) = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_ret_reg(i64 {reg})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                if fn_name == "xiom_ir_ret_lit" && args.len() >= 1 {
                    let (val, _) = self.compile_expr(&args[0])?;
                    self.emitln(&format!("  call void @xiom_ir_ret_lit(i64 {val})"));
                    return Ok(("0".to_string(), "void".to_string()));
                }
                // Builtin len on array (i8*) buffer: read count from slot 0.
                // Array literals are compiled as `[i64 count, i64 elem...]`
                // cast to i8*. This avoids the const-generic N propagation issue.
                if fn_name == "len" && args.len() >= 1 {
                    let (arg_val, arg_ty) = self.compile_expr(&args[0])?;
                    if arg_ty == "i8*" {
                        let buf = self.fresh_tmp();
                        self.emitln(&format!("  {buf} = bitcast i8* {arg_val} to i64*"));
                        let len_val = self.fresh_tmp();
                        self.emitln(&format!("  {len_val} = load i64, i64* {buf}"));
                        return Ok((len_val, "i64".to_string()));
                    }
                }
                // Builtin write(ptr, value): store value through raw pointer.
                // ptr.write is generic but type inference fails for *T types,
                // so it ends up as a zero-arg stub. This inline handler
                // emits the store directly, bypassing monomorphization.
                if fn_name == "write" && args.len() >= 2 {
                    let (ptr_val, ptr_ty) = self.compile_expr(&args[0])?;
                    if ptr_ty.ends_with('*') {
                        let pointee = ptr_ty.trim_end_matches('*').to_string();
                        let (val, val_ty) = self.compile_expr(&args[1])?;
                        let store_val = self.coerce_value(&val, &val_ty, &pointee);
                        self.emitln(&format!("  store {pointee} {store_val}, {ptr_ty} {ptr_val}"));
                        return Ok((String::new(), "void".to_string()));
                    }
                }
                // Builtin read(ptr): load value through raw pointer.
                // ptr.read is generic with the same *T inference issue as write.
                if fn_name == "read" && args.len() >= 1 {
                    let (ptr_val, ptr_ty) = self.compile_expr(&args[0])?;
                    if ptr_ty.ends_with('*') {
                        let pointee = ptr_ty.trim_end_matches('*').to_string();
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = load {pointee}, {ptr_ty} {ptr_val}"));
                        return Ok((tmp, pointee));
                    }
                }
                // Builtin size_of[T](): return the LLVM size in bytes of type T.
                // The type arg is parsed as `Expr::Index` and captured in type_arg.
                if fn_name == "size_of" || fn_name == "align_of" {
                    if let Some(ta) = type_arg {
                        let xiom_ty = match ta {
                            Expr::Ident(id) => id.name.clone(),
                            Expr::Field(_, f, _) => f.name.clone(),
                            _ => String::new(),
                        };
                        if !xiom_ty.is_empty() {
                            let llvm_ty = self.llvm_type_for(&xiom_ty)
                                .unwrap_or_else(|_| Self::xiom_to_llvm_type(&xiom_ty).to_string());
                            let size = if llvm_ty.starts_with("%struct.") {
                                let type_name = llvm_ty[8..].to_string();
                                self.struct_byte_size(&type_name)
                            } else {
                                match llvm_ty.as_str() {
                                    "i8" => 1,
                                    "i16" => 2,
                                    "i32" => 4,
                                    "i64" | "double" | "i8*" | "ptr" => 8,
                                    _ => 8,
                                }
                            };
                            if fn_name == "align_of" {
                                let align = if llvm_ty.starts_with("%struct.") { 8 } else { size };
                                return Ok((align.to_string(), "i64".to_string()));
                            }
                            return Ok((size.to_string(), "i64".to_string()));
                        }
                    }
                    return Ok(("8".to_string(), "i64".to_string()));
                }
                // Enum variant constructor: `TypeName.Variant(args)`.
                // Detects when the call is constructing an enum variant and emits
                // the proper discriminant + payload struct.
                if let Some(receiver) = receiver_expr {
                    if let Expr::Ident(type_id) = &**receiver {
                        let enum_key = self.enum_variants.keys()
                            .find(|k| **k == type_id.name || k.ends_with(&format!(".{}", type_id.name)))
                            .cloned();
                        if let Some(ek) = enum_key {
                            if let Some(variants) = self.enum_variants.get(&ek) {
                                let var_info: Option<(usize, Vec<String>)> = variants.iter().enumerate()
                                    .find(|(_, (v, _))| v == &fn_name)
                                    .map(|(idx, (_, fields))| (idx, fields.clone()));
                                if let Some((var_idx, payload_fields)) = var_info {
                                    // Gather parent field layout BEFORE mutating self.
                                    let parent_field_names = self.types.get(&ek).cloned().unwrap_or_default();
                                    if let Ok(struct_ty) = self.llvm_type_for(&ek) {
                                        let alloca = self.fresh_tmp();
                                        self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                                        // Set discriminant to variant index
                                        let disc_gep = self.fresh_tmp();
                                        self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                                        self.emitln(&format!("  store i64 {var_idx}, i64* {disc_gep}"));
                                        // Set payload fields from call args
                                        let compiled: Vec<(String, String)> = args.iter()
                                            .map(|a| self.compile_expr(a))
                                            .collect::<Result<Vec<_>, _>>()?;
                                        // Map variant fields to their parent enum offsets.
                                        // Variants share field names across the parent
                                        // enum (e.g. Object::entries maps to field 3, not 1).
                                        for (pi, (val, val_ty)) in compiled.iter().enumerate() {
                                            let field_name = payload_fields.get(pi).cloned().unwrap_or_default();
                                            let field_idx = parent_field_names.iter()
                                                .position(|f| f == &field_name)
                                                .unwrap_or(pi + 1);
                                            let gep = self.fresh_tmp();
                                            let fty = self.field_llvm_type(&ek, field_idx);
                                            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {field_idx}"));
                                            let sv = self.coerce_value(val, val_ty, &fty);
                                            self.emitln(&format!("  store {fty} {sv}, {fty}* {gep}"));
                                        }
                                        let loaded = self.fresh_tmp();
                                        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
                                        return Ok((loaded, struct_ty));
                                    }
                                }
                            }
                        }
                    }
                }
                // Inline Option.unwrap() / Result.unwrap() / Result.unwrap_err()
                // when called as a method on a known Option/Result value.
                // Avoids relying on the hardcoded @Option.unwrap stub which may
                // not be emitted if used_builtins wasn't set via ?/is_some/is_none.
                if (fn_name == "unwrap" || fn_name == "unwrap_err") && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_option = recv_ty == "%struct.Option"
                            || recv_ty.ends_with(".Option")
                            || recv_ty.contains("Option__"); // concrete Option__Point etc.
                        let is_result = recv_ty == "%struct.Result"
                            || recv_ty.ends_with(".Result")
                            || recv_ty.contains("Result__"); // concrete Result__X__Y etc.
                        if is_option || is_result {
                            if is_option { self.used_builtins.insert("Option".to_string()); }
                            else { self.used_builtins.insert("Result".to_string()); }
                            let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                            let struct_ty = recv_ty.clone(); // Use the actual concrete type
                            let alloca = self.fresh_tmp();
                            self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                            self.emitln(&format!("  store {struct_ty} {recv_val}, {struct_ty}* {alloca}"));
                            // Read discriminant
                            let disc_gep = self.fresh_tmp();
                            self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                            let disc = self.fresh_tmp();
                            self.emitln(&format!("  {disc} = load i64, i64* {disc_gep}"));
                            if fn_name == "unwrap_err" || is_option {
                                // unwrap: expect disc != 0 (Some/Ok); unwrap_err: expect disc == 0 (Err)
                                let ok_cond = if fn_name == "unwrap_err" { "eq" } else { "ne" };
                                let ok = self.fresh_tmp();
                                self.emitln(&format!("  {ok} = icmp {ok_cond} i64 {disc}, 0"));
                                let ok_block = self.fresh_block("unwrap_ok");
                                let fail_block = self.fresh_block("unwrap_fail");
                                self.emitln(&format!("  br i1 {ok}, label %{ok_block}, label %{fail_block}"));
                                self.emitln(&format!("\n{fail_block}:"));
                                self.emitln("  call void @llvm.trap()");
                                self.emitln("  unreachable");
                                self.emitln(&format!("\n{ok_block}:"));
                            }
                            // Read the value payload (field 1 for Option, field 1 for Result.ok, field 2 for Result.err)
                            let val_field = if fn_name == "unwrap_err" { 2 } else { 1 };
                            let type_name = struct_ty.trim_start_matches("%struct.");
                            let field_ty = self.field_llvm_type(type_name, val_field);
                            let val_gep = self.fresh_tmp();
                            self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {val_field}"));
                            let val = self.fresh_tmp();
                            self.emitln(&format!("  {val} = load {field_ty}, {field_ty}* {val_gep}"));
                            // If the payload is already a struct type, return it directly.
                            if field_ty.starts_with('%') {
                                return Ok((val, field_ty));
                            }
                            // When field_ty is i64, the payload may be a heap pointer
                            // from val_to_i64 for struct payloads.  Determine the actual
                            // struct type by resolving the generic return type of the
                            // concrete instantiation (e.g. `Option.unwrap[Point] ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ Point`).
                            let struct_type_hint: Option<String> = {
                                let fn_key = if is_option { "Option.unwrap" } else { "Result.unwrap" };
                                self.generic_fn_decls.iter().find(|(k, _)| k == fn_key || k.ends_with(&format!(".{}", fn_name)))
                                    .and_then(|(_, fd)| fd.return_type.as_ref().map(|t| Self::type_from_ast(t)))
                            };
                            if let Some(ref hint) = struct_type_hint {
                                // hint is the XIOM type name (e.g. "Point" for T=Point).
                                // Convert to LLVM struct type.
                                let struct_llvm = if self.types.contains_key(hint) || self.type_meta.contains_key(hint) {
                                    format!("%struct.{hint}")
                                } else {
                                    // Check if it resolves via type_meta
                                    let full_key = self.type_meta.keys().find(|k| k.ends_with(&format!(".{hint}"))).cloned();
                                    match full_key {
                                        Some(k) => format!("%struct.{k}"),
                                        None => return Ok((self.val_to_i64(&val, &field_ty), "i64".to_string())),
                                    }
                                };
                                // The val is a heap pointer (i64). Inttoptr to the struct type, load.
                                let ptr = self.fresh_tmp();
                                self.emitln(&format!("  {ptr} = inttoptr i64 {val} to {struct_llvm}*"));
                                let loaded = self.fresh_tmp();
                                self.emitln(&format!("  {loaded} = load {struct_llvm}, {struct_llvm}* {ptr}"));
                                return Ok((loaded, struct_llvm));
                            }
                            let result = self.val_to_i64(&val, &field_ty);
                            return Ok((result, "i64".to_string()));
                        }
                    }
                }
                // Check if this is a call to a generic function and track instantiation
                let fn_key = if let Some(receiver) = receiver_expr {
                    if let Some(recv_type) = self.infer_struct_type_name(receiver) {
                        format!("{}.{}", recv_type, fn_name)
                    } else if self.receiver_is_instance(receiver) {
                        // Scalar value instance receiver (e.g. `value.hash(hasher)`
                        // inside a generic monomorphised body). Resolve via
                        // param_concrete_types to get `Int.hash` not bare `hash`.
                        let obj_var_name = match &**receiver {
                            Expr::Ident(id) => id.name.clone(),
                            _ => String::new(),
                        };
                        if !obj_var_name.is_empty() {
                            if let Some(concrete) = self.param_concrete_types.get(&obj_var_name) {
                                format!("{}.{}", concrete, fn_name)
                            } else if let Some((_, llvm_ty)) = self.lookup_local(&obj_var_name) {
                                let ty_name = Self::xiom_type_name_from_llvm(llvm_ty);
                                format!("{}.{}", ty_name, fn_name)
                            } else {
                                self.resolve_module_call(receiver, &fn_name)
                            }
                        } else {
                            self.resolve_module_call(receiver, &fn_name)
                        }
                    } else {
                        // Receiver is a module name (not a struct type) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â resolve
                        // to module-qualified function name if registered.
                        self.resolve_module_call(receiver, &fn_name)
                    }
                } else {
                    fn_name.clone()
                };
                // Interface dispatch fallback: when the receiver type is a known
                // interface (e.g. `Error.description`), search all registered
                // concrete functions for one that matches `*.method_name` (static
                // dispatch ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the first matching implementation wins).
                let fn_key = if !self.functions.contains_key(&fn_key)
                    && !self.generic_fn_decls.iter().any(|(k, _)| k == &fn_key)
                {
                    // Extract interface name and method from fn_key ("Error.description").
                    if let Some(dot_pos) = fn_key.find('.') {
                        let iface_name = &fn_key[..dot_pos];
                        let method_name = &fn_key[dot_pos + 1..];
                        if self.interfaces.contains_key(iface_name) {
                            let suffix = format!(".{}", method_name);
                            self.functions.keys()
                                .find(|k| k.ends_with(&suffix) && !k.starts_with(iface_name))
                                .cloned()
                                .unwrap_or(fn_key)
                        } else {
                            fn_key
                        }
                    } else {
                        fn_key
                    }
                } else {
                    fn_key
                };
                let is_generic = self.generic_fn_decls.iter().any(|(k, _)| k == &fn_key)
                    || (!self.functions.contains_key(&fn_key)
                        && self.generic_fn_decls.iter().any(|(k, _)| k.ends_with(&format!(".{}", fn_key))));
                if is_generic {
                    // Infer concrete types from argument types
                    let mut concrete_types: Vec<String> = Vec::new();
                    let mut const_values: HashMap<String, i64> = HashMap::new();
                    // Find the generic function declaration
                    if let Some((_, fd)) = self.generic_fn_decls.iter().find(|(k, _)| k == &fn_key)
                        .or_else(|| self.generic_fn_decls.iter().find(|(k_2, _)| k_2.ends_with(&format!(".{}", fn_key)))) {
                        let fd = fd.clone();
                        for gp in &fd.generics {
                            // Const-generic params: extract the integer value from the
                            // explicit type arg (e.g. `len[Int, 5](arr)`).
                            if gp.is_const {
                                let mut found_const = false;
                                if let Some(ta) = type_arg {
                                    let const_expr: Option<Expr> = match ta {
                                        Expr::Int(n, _) => Some(Expr::Int(*n, Span::new(0, 0))),
                                        Expr::Tuple(elems, _) => {
                                            elems.iter().find(|e| matches!(e, Expr::Int(..))).cloned()
                                        }
                                        _ => None,
                                    };
                                    if let Some(Expr::Int(n, _)) = const_expr {
                                        const_values.insert(gp.name.name.clone(), n as i64);
                                        found_const = true;
                                    }
                                }
                                // 5c.30: If no explicit type arg, infer const-generic value
                                // from any argument that references an array local.
                                // We search ALL params because const-generic names
                                // may not appear in the parameter type AST (parser
                                // lowers [N]T as Slice(T), losing N).
                                if !found_const {
                                    for (_param, arg_expr) in fd.params.iter().zip(args.iter()) {
                                        let inner_expr: &Expr = match arg_expr {
                                            Expr::Ref(i, _) | Expr::MutRef(i, _)
                                            | Expr::Unary(UnaryOp::Ref, i, _)
                                            | Expr::Unary(UnaryOp::MutRef, i, _) => i.as_ref(),
                                            other => other,
                                        };
                                        if let Expr::Ident(id) = inner_expr {
                                            if let Some(size) = self.local_array_sizes.get(&id.name) {
                                                const_values.insert(gp.name.name.clone(), *size);
                                                break;
                                            }
                                        }
                                    }
                                }
                                concrete_types.push("Int".to_string());
                                continue;
                            }
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
                                            } else if let Some((_, _llvm_ty)) = self.lookup_local(&id.name) {
                                                self.resolve_local_xiom_type(&id.name).unwrap_or_else(|| "Int".to_string())
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
                                // Nested generic: e.g. `Option[T]` ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ extract T from type args
                                let arg_names = Self::extract_type_arg_names(&param.ty);
                                if let Some(_pos) = arg_names.iter().position(|a| a == &gp.name.name) {
                                    let concrete_ty = "Int".to_string();
                                    concrete_types.push(concrete_ty);
                                    inferred = true;
                                    break;
                                }
                            }
                            if !inferred {
                                // Use explicit type args from the call syntax
                                // (e.g. `Map[Str, JsonValue].new()` ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ type_arg = Tuple([Str, JsonValue])).
                                if let Some(ta) = type_arg {
                                    let type_names: Vec<String> = match ta {
                                        Expr::Ident(id) => vec![id.name.clone()],
                                        Expr::Tuple(elems, _) => elems.iter()
                                            .map(|e| match e {
                                                Expr::Ident(id) => id.name.clone(),
                                                _ => "Int".to_string(),
                                            })
                                            .collect(),
                                        _ => vec!["Int".to_string()],
                                    };
                                    // Map each generic param to the corresponding type name
                                    let gp_idx = fd.generics.iter().position(|g| g.name.name == gp.name.name);
                                    if let Some(idx) = gp_idx {
                                        if idx < type_names.len() {
                                            concrete_types.push(type_names[idx].clone());
                                            continue;
                                        }
                                    }
                                }
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
                                            } else if let Some((_, _llvm_ty)) = self.lookup_local(&id.name) {
                                                self.resolve_local_xiom_type(&id.name).unwrap_or_else(|| "Int".to_string())
                                            } else {
                                                gp.name.name.clone()
                                            }
                                        }
                                        _ => "Int".to_string(),
                                    };
                                    concrete_types.push(concrete_ty);
                                } else if receiver_expr
                                    .map(|r| self.infer_struct_type_name(r).is_some())
                                    .unwrap_or(false)
                                {
                                    // Receiver-bound generic with NO explicit args
                                    // (e.g. `Cell[T].get(self) -> T`, `Rc[T].get(self)`).
                                    // T lives only on the receiver type; the concrete
                                    // instance already collapsed to a single struct
                                    // layout whose fields lower to i64-width slots at
                                    // the ABI, so default T to `Int` (its i64 lowering).
                                    // This lets get/count-style accessors monomorphise
                                    // instead of falling back to a constant-0 stub.
                                    concrete_types.push("Int".to_string());
                                }
                            }
                        }
                        // Interface-typed function: no explicit generics but has
                        // interface-typed params. Infer concrete struct types from the
                        // actual arguments (BUG-007 interface dispatch).
                        if concrete_types.is_empty() && fd.generics.is_empty() {
                            for (param, arg_expr) in fd.params.iter().zip(args.iter()) {
                                let param_name = Self::type_from_ast(&param.ty);
                                if self.interfaces.contains_key(&param_name) {
                                    let concrete_ty = match arg_expr {
                                        Expr::Ident(id) => {
                                            if let Some((_, _llvm_ty)) = self.lookup_local(&id.name) {
                                                self.resolve_local_xiom_type(&id.name).unwrap_or_else(|| "Int".to_string())
                                            } else { String::new() }
                                        }
                                        _ => String::new(),
                                    };
                                    if !concrete_ty.is_empty() {
                                        concrete_types.push(concrete_ty);
                                    }
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
                            if !const_values.is_empty() {
                                self.const_value_map.insert(specialized_name.clone(), const_values.clone());
                            }
                            // Also insert even without const values to avoid repeated lookups
                            self.const_value_map.entry(specialized_name.clone()).or_insert_with(|| const_values.clone());
                        }
                        // Call the specialized version
                        let (ret_ty, param_types) = if let Some((pts, rt)) = self.functions.get(&specialized_name) {
                            (rt.clone(), pts.clone())
                        } else {
                            // Not yet registered - use the generic signature with
                            // argument-inferred param types, prepending the receiver
                            // type if the generic decl has a self parameter.
                            let mut inferred_types: Vec<String> = args.iter()
                                .map(|a| self.infer_llvm_type(a))
                                .collect();
                            // Check if the generic decl has a self param (receiver)
                            let has_self = self.generic_fn_decls.iter()
                                .find(|(k, _)| k == &fn_key)
                                .map(|(_, fd)| fd.receiver.is_some()
                                    && fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self))
                                .unwrap_or(false);
                            if has_self {
                                // Prepend the receiver's pointer type
                                if let Some(recv_name) = self.generic_fn_decls.iter()
                                    .find(|(k, _)| k == &fn_key)
                                    .and_then(|(_, fd)| fd.receiver.as_ref())
                                {
                                    let recv_ty = self.llvm_type_for(&recv_name.name)
                                        .unwrap_or_else(|_| "i64".to_string());
                                    let recv_ptr = if recv_ty.starts_with('%') { format!("{recv_ty}*") } else { recv_ty };
                                    inferred_types.insert(0, recv_ptr);
                                }
                            }
                            let generic_ret = self.functions.get(&fn_key)
                                .map(|(_, rt)| rt.clone())
                                .unwrap_or_else(|| "i64".to_string());
                            (generic_ret, inferred_types)
                        };
                        // Include receiver argument only if it's an actual struct instance
                        // AND it's not already in the registered param_types
                        let mut all_args: Vec<String> = compiled_args.iter().map(|(v, _)| v.clone()).collect();
                        let mut all_arg_types: Vec<String> = compiled_args.iter().map(|(_, t)| t.clone()).collect();
                        let mut all_param_types = param_types.clone();
                        if let Some(receiver) = receiver_expr {
                            let is_instance = self.receiver_is_instance(receiver);
                            // Check if param_types already includes a receiver (from monomorphised registration)
                            let has_receiver_in_params = !all_param_types.is_empty() && all_param_types.len() > all_args.len();
                            // Check if the generic function declaration has a self param.
                            // Methods like Map.insert(key, value) have receiver type
                            // but no self param ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â don't pass the receiver instance.
                            // For generic functions in generic_fn_decls, check if the
                            // declaration has a self param. For non-generic methods
                            // (not in generic_fn_decls), default to true ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the
                            // has_receiver_in_params check above handles the rest.
                            let generic_has_self = self.generic_fn_decls.iter()
                                .find(|(k, _)| k == &fn_key)
                                .map(|(_, fd)| fd.params.iter().any(|p| p.name.name == "self"))
                                .unwrap_or(true);
                            // Pass the receiver if it's an instance AND either the
                            // signature includes a self param or the generic decl does.
                            if is_instance && (has_receiver_in_params || generic_has_self) {
                                let (recv_val, recv_llvm_ty) = self.compile_expr(receiver)?;
                                // If callee expects a pointer self (&mut Struct),
                                // pass the receiver's alloca address instead.
                                let (recv_val, recv_llvm_ty) = if has_receiver_in_params {
                                    if let Some(p0) = all_param_types.first() {
                                        if p0.ends_with('*') && !recv_llvm_ty.ends_with('*') {
                                            if let Expr::Ident(id) = &**receiver {
                                                if let Some((slot, _slot_ty)) = self.lookup_local(&id.name).cloned() {
                                                    (slot, format!("{recv_llvm_ty}*"))
                                                } else { (recv_val, recv_llvm_ty) }
                                            } else { (recv_val, recv_llvm_ty) }
                                        } else { (recv_val, recv_llvm_ty) }
                                    } else { (recv_val, recv_llvm_ty) }
                                } else { (recv_val, recv_llvm_ty) };
                                if has_receiver_in_params {
                                    // Receiver type already in param_types, just need the value
                                    all_args.insert(0, recv_val);
                                    all_arg_types.insert(0, all_param_types.first().cloned().unwrap_or_else(|| "i64".to_string()));
                                } else {
                                    all_param_types.insert(0, recv_llvm_ty.clone());
                                    all_args.insert(0, recv_val);
                                    all_arg_types.insert(0, recv_llvm_ty);
                                }
                            } else if !is_instance && has_receiver_in_params {
                                // Type name or module name receiver ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â remove extra param type
                                all_param_types.remove(0);
                            }
                            // else: is_instance && !generic_has_self ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ no self param to pass
                        }
                        // Coerce each arg to the callee's declared param type (its real
                        // compiled type may differ, e.g. an enum-variant arg compiled to
                        // a struct while the callee expects that struct). A pointer
                        // param fed `&x`/`&mut x` receives the scalar's slot address.
                        let arg_offset = all_args.len().saturating_sub(args.len());
                        let args_str = all_args.iter().enumerate()
                            .map(|(i, arg)| {
                                let pty = all_param_types.get(i).cloned()
                                    .unwrap_or_else(|| all_arg_types.get(i).cloned().unwrap_or_else(|| "i64".to_string()));
                                let from = all_arg_types.get(i).cloned().unwrap_or_else(|| pty.clone());
                                let coerced = if i >= arg_offset {
                                    match args.get(i - arg_offset) {
                                        Some(ae) => self.coerce_arg_for_param(ae, arg, &from, &pty),
                                        None => self.coerce_value(arg, &from, &pty),
                                    }
                                } else {
                                    self.coerce_value(arg, &from, &pty)
                                };
                                format!("{pty} {coerced}")
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        let tmp = self.fresh_tmp();
                        if ret_ty == "void" {
                            self.emitln(&format!("  call void @{specialized_name}({args_str})"));
                            Ok((String::new(), "void".to_string()))
                        } else {
                            self.emitln(&format!("  {tmp} = call {ret_ty} @{specialized_name}({args_str})"));
                            if let Some(receiver) = receiver_expr {
                                if ret_ty.starts_with("%struct.") {
                                    self.store_back_to_receiver(receiver, &tmp, &ret_ty);
                                }
                            }
                            Ok((tmp, ret_ty.clone()))
                        }
                    } else {
                        Ok(("0".to_string(), "i64".to_string()))
                    }
                } else {
                    // For method calls, resolve the fully qualified function name
                    let mut resolved_fn_key = if let Some(receiver) = receiver_expr {
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
                            // Receiver is a module name ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â use module-qualified resolution
                            self.resolve_module_call(receiver, &fn_name)
                        }
                    } else {
                        fn_key.clone()
                    };
                    // Fallback: when the resolved key is not a known function (e.g.
                    // "is_match" from an i64-typed receiver), search for any registered
                    // function whose name ends with ".method_name" (e.g. "Regex.is_match").
                    // IMPORTANT: for bare function calls (no "." in key AND no receiver),
                    // only match bare function names ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â do NOT match instance methods.
                    // Method calls (receiver_expr is Some) may resolve through this path
                    // when the receiver type is i64 (not a named struct), so the bare-call
                    // skip must NOT apply.
                    if !self.functions.contains_key(&resolved_fn_key) {
                        let suffix = format!(".{fn_name}");
                        let mut found = String::new();
                        let is_bare_call = !resolved_fn_key.contains('.') && receiver_expr.is_none();
                        for key in self.functions.keys() {
                            if key.ends_with(&suffix) {
                                // Skip method names when resolving truly bare calls
                                if is_bare_call && key.contains('.') {
                                    continue;
                                }
                                if found.is_empty() {
                                    found = key.clone();
                                } else if found != *key {
                                    found.clear();
                                    break;
                                }
                            }
                        }
                        if !found.is_empty() {
                            resolved_fn_key = found;
                        }
                    }
                    let args_str = if let Some(receiver) = receiver_expr {
                        // Check if receiver is a real struct instance (local variable)
                        // vs a type name (TrafficLight.xxx()) or module name (pipeline.xxx()).
                        // A module path like `xiom.char` (nested Field rooted in a
                        // non-local) is NOT an instance ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ no phantom receiver arg.
                        let is_instance = self.receiver_is_instance(receiver);
                        // Registered callee param types (used to coerce args to the
                        // exact types the callee declares).
                        let callee_pts = self.functions.get(&resolved_fn_key).map(|(p, _)| p.clone());
                        if is_instance {
                            // Value sink: use the receiver's real compiled LLVM type
                            // (from compile_expr) rather than a re-inference.
                            let (recv_val, recv_llvm_ty) = self.compile_expr(receiver)?;
                            // If the callee expects a pointer self param (&mut Struct),
                            // pass the receiver's alloca ADDRESS instead of the
                            // loaded value so mutations propagate to the caller.
                            let (recv_val, recv_llvm_ty) = if let Some(p0) = callee_pts.as_ref().and_then(|p| p.first()) {
                                if p0.ends_with('*') && !recv_llvm_ty.ends_with('*') {
                                    if let Expr::Ident(id) = &**receiver {
                                        if let Some((slot, _slot_ty)) = self.lookup_local(&id.name).cloned() {
                                            (slot, format!("{recv_llvm_ty}*"))
                                        } else {
                                            (recv_val, recv_llvm_ty)
                                        }
                                    } else {
                                        (recv_val, recv_llvm_ty)
                                    }
                                } else {
                                    (recv_val, recv_llvm_ty)
                                }
                            } else {
                                (recv_val, recv_llvm_ty)
                            };
                            // Coerce receiver when it is an i64 pointer (e.g. from
                            // Result.unwrap on a struct-typed Result) but the callee
                            // expects a struct value. Emit inttoptr + load to
                            // dereference the heap-allocated struct.
                            let (recv_val, recv_llvm_ty) = if let Some(p0) = callee_pts.as_ref().and_then(|p| p.first().cloned()) {
                                if p0.starts_with("%struct.") && recv_llvm_ty == "i64" {
                                    if p0.ends_with('*') {
                                        // 5c.30: callee expects a POINTER receiver
                                        // (this-based method): the i64 heap box IS
                                        // the struct â€” inttoptr directly, never
                                        // load through it as a pointer-to-pointer
                                        // (that read the discriminant as an
                                        // address: as_string AV at 0x3).
                                        let struct_ptr = self.fresh_tmp();
                                        self.emitln(&format!("  {struct_ptr} = inttoptr i64 {recv_val} to {p0}"));
                                        (struct_ptr, p0)
                                    } else {
                                        let struct_ptr = self.fresh_tmp();
                                        let struct_val = self.fresh_tmp();
                                        self.emitln(&format!("  {struct_ptr} = inttoptr i64 {recv_val} to {p0}*"));
                                        self.emitln(&format!("  {struct_val} = load {p0}, {p0}* {struct_ptr}"));
                                        (struct_val, p0)
                                    }
                                } else {
                                    (recv_val, recv_llvm_ty)
                                }
                            } else {
                                (recv_val, recv_llvm_ty)
                            };
                            // When registered, param types include the receiver at [0];
                            // explicit args map to [1..].
                            let rest_str: Vec<String> = compiled_args.iter().enumerate()
                                .map(|(i, (arg_val, arg_ty))| {
                                    let pty = callee_pts.as_ref()
                                        .and_then(|p| p.get(i + 1).cloned())
                                        .unwrap_or_else(|| arg_ty.clone());
                                    let coerced = match args.get(i) {
                                        Some(ae) => self.coerce_arg_for_param(ae, arg_val, arg_ty, &pty),
                                        None => self.coerce_value(arg_val, arg_ty, &pty),
                                    };
                                    format!("{pty} {coerced}")
                                })
                                .collect();
                            if rest_str.is_empty() {
                                format!("{recv_llvm_ty} {recv_val}")
                            } else {
                                format!("{recv_llvm_ty} {recv_val}, {}", rest_str.join(", "))
                            }
                        } else {
                            // Type name or module name ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â no receiver argument. Coerce
                            // each arg to the callee's declared param type (its real
                            // compiled type may differ, e.g. an enum-variant arg
                            // compiled to a struct while the callee expects it).
                            compiled_args.iter().enumerate()
                                .map(|(i, (arg_val, arg_ty))| {
                                    let pty = callee_pts.as_ref()
                                        .and_then(|p| p.get(i).cloned())
                                        .unwrap_or_else(|| arg_ty.clone());
                                    let coerced = match args.get(i) {
                                        Some(ae) => self.coerce_arg_for_param(ae, arg_val, arg_ty, &pty),
                                        None => self.coerce_value(arg_val, arg_ty, &pty),
                                    };
                                    format!("{pty} {coerced}")
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    } else {
                        // Use registered param types when available (correct for extern
                        // functions with non-default types like Int32ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢i32, Float32ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢float).
                        // 5c.30: G-10 implicit-self â€” override the fn key so
                        // bare `greet("Hi")` resolves to `Greeter.greet`.
                        // No self injection: method bodies already have receiver
                        // fields as locals via the prologue.
                        if receiver_expr.is_none() {
                            if let Some(isk) = self.resolve_implicit_self_call(&fn_name) {
                                resolved_fn_key = isk;
                            }
                        }
                        let use_registered = self.functions.get(&resolved_fn_key)
                            .map(|(pts, _)| pts.len() == compiled_args.len())
                            .unwrap_or(false);
                        if use_registered {
                            let pts = self.functions[&resolved_fn_key].0.clone();
                            let mut parts: Vec<String> = Vec::new();
                            for (i, (arg_val, arg_ty)) in compiled_args.iter().enumerate() {
                                let pty = pts[i].clone();
                                let coerced = match args.get(i) {
                                    Some(ae) => self.coerce_arg_for_param(ae, arg_val, arg_ty, &pty),
                                    None => self.coerce_value(arg_val, arg_ty, &pty),
                                };
                                parts.push(format!("{pty} {coerced}"));
                            }
                            parts.join(", ")
                        } else {
                            compiled_args.iter()
                                .map(|(arg_val, arg_ty)| format!("{arg_ty} {arg_val}"))
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
                            Ok((String::new(), "void".to_string()))
                        } else {
                            self.emitln(&format!("  {tmp} = call {actual_ret_ty} {fn_ptr}({args_str})"));
                            Ok((tmp, actual_ret_ty))
                        }
                    } else if ret_ty == "void" {
                        self.emitln(&format!("  call void @{resolved_fn_key}({args_str})"));
                        Ok((String::new(), "void".to_string()))
                    } else {
                        self.emitln(&format!("  {tmp} = call {ret_ty} @{resolved_fn_key}({args_str})"));
                        // Store result back to receiver variable for mutating methods
                        // (by-value semantics: callee receives a copy; store the
                        // returned struct so caller sees the mutation).
                        if let Some(receiver) = receiver_expr {
                            if ret_ty.starts_with("%struct.") {
                                self.store_back_to_receiver(receiver, &tmp, &ret_ty);
                            }
                        }
                        Ok((tmp, ret_ty.clone()))
                    }
                }
            }
            Expr::Index(container, index, _) => {
                // Index into a Vec (builtin {i8*, i64, i64}) or a Str (i8*).
                // Fixed-size arrays [N x T] (from Expr::Array literals or stack
                // arrays) are handled by the `[N x T]` GEP path below.
                let (cont_val, cont_ty) = self.compile_expr(container)?;
                let (idx_raw, idx_ty) = self.compile_expr(index)?;
                let idx = self.val_to_i64(&idx_raw, &idx_ty);
                // Index into an Expr::Array literal buffer (i8* with length at [0]).
                // The buffer layout is: [length: i64][elem0: i64][elem1: i64]...
                // Skip past the leading length slot and read the element at index+1.
                // Also handles local variables bound from array literals (let arr = [...];
                // arr[i]) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â detected by cont_ty == i8* and the ident resolves to a
                // buffer that wasn't interned as a C string.
                let is_array_buf = cont_ty == "i8*" && (
                    matches!(container.as_ref(), Expr::Array(..))
                    || (if let Expr::Ident(ident) = container.as_ref() { self.array_locals.contains(&ident.name) } else { false })
                );
                if is_array_buf {
                    let base_ptr = self.fresh_tmp();
                    // 5c-R: use the declared element type for arrays (G-11).
                    // Default to i64 for backward-compat. Tracked via local_array_elem.
                    let arr_elem_ty = if let Expr::Ident(ident) = container.as_ref() {
                        self.local_array_elem.get(&ident.name).cloned().unwrap_or_else(|| "i64".to_string())
                    } else { "i64".to_string() };
                    self.emitln(&format!("  {base_ptr} = bitcast i8* {cont_val} to {arr_elem_ty}*"));
                    // Element is at position index+1 (slot 0 is the length).
                    let offset = self.fresh_tmp();
                    self.emitln(&format!("  {offset} = add i64 {idx}, 1"));
                    let elem_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {elem_ptr} = getelementptr {arr_elem_ty}, {arr_elem_ty}* {base_ptr}, i64 {offset}"));
                    let elem = self.fresh_tmp();
                    self.emitln(&format!("  {elem} = load {arr_elem_ty}, {arr_elem_ty}* {elem_ptr}"));
                    return Ok((elem, arr_elem_ty));
                }
                // Raw `*T` pointer held in an i64 (a pointer param): inttoptr and read
                // one byte. Checked before the Str path since these lower to i64.
                if cont_ty == "i64" && self.is_ptr_local_expr(container) {
                    let base_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {base_ptr} = inttoptr i64 {cont_val} to i8*"));
                    let elem_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {base_ptr}, i64 {idx}"));
                    let ch = self.fresh_tmp();
                    self.emitln(&format!("  {ch} = load i8, i8* {elem_ptr}"));
                    let ext = self.fresh_tmp();
                    self.emitln(&format!("  {ext} = zext i8 {ch} to i64"));
                    return Ok((ext, "i64".to_string()));
                }
                // Str: char access via the raw runtime accessor, returned as i64.
                if cont_ty == "i8*" {
                    let ch = self.fresh_tmp();
                    self.emitln(&format!("  {ch} = call i8 @xiom_char_at(i8* {cont_val}, i64 {idx})"));
                    let ext = self.fresh_tmp();
                    self.emitln(&format!("  {ext} = zext i8 {ch} to i64"));
                    return Ok((ext, "i64".to_string()));
                }
                // Vec/Slice: element is an i64-wide slot at data[index].
                let (mut vec_val, mut vec_ty) = self.resolve_vec_value(&cont_val, &cont_ty);
                // If field_llvm_type returned i64 (generic type like Vec[Int]),
                // the Vec was loaded as i64. Inttoptr to %struct.Vec* and
                // reload as the proper struct type (5c.28).
                if vec_ty == "i64" && self.is_container_vec_field(container) {
                    let vp = self.fresh_tmp();
                    self.emitln(&format!("  {vp} = inttoptr i64 {vec_val} to %struct.Vec*"));
                    let vl = self.fresh_tmp();
                    self.emitln(&format!("  {vl} = load volatile %struct.Vec, %struct.Vec* {vp}"));
                    vec_val = vl;
                    vec_ty = "%struct.Vec".to_string();
                }
                let is_vec = vec_ty == "%struct.Vec" || vec_ty.ends_with(".Vec")
                    || vec_ty.contains("struct.Vec")
                    || vec_ty == "%struct.Slice" || vec_ty.contains("struct.Slice");
                if is_vec {
                    let vslot = self.fresh_tmp();
                    self.emitln(&format!("  {vslot} = alloca %struct.Vec"));
                    // Zero-init the Vec alloca to prevent stale stack data
                    // from prior function calls when LLVM SROA skips stores.
                    let vslot_i8 = self.fresh_tmp();
                    self.emitln(&format!("  {vslot_i8} = bitcast %struct.Vec* {vslot} to i8*"));
                    self.emitln(&format!("  call void @llvm.memset.p0i8.i64(i8* {vslot_i8}, i8 0, i64 32, i1 false)"));
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
                    // For struct elements with a known element type, load the
                    // struct directly from Vec data via memcpy, bypassing the
                    // ptrtoint/inttoptr chain of emit_elem_load+val_to_struct.
                    // (5c.28 ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â counter pattern fix)
                    if let Some(elem_type_name) = self.resolve_vec_elem_type(container) {
                        let struct_ty = format!("%struct.{elem_type_name}");
                        let struct_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {struct_alloca} = alloca {struct_ty}"));
                        let dst_i8 = self.fresh_tmp();
                        self.emitln(&format!("  {dst_i8} = bitcast {struct_ty}* {struct_alloca} to i8*"));
                        self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {dst_i8}, i8* {elem_ptr}, i64 {esz_val}, i1 false)"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {struct_alloca}"));
                        return Ok((loaded, struct_ty));
                    }
                    // Fallback: use emit_elem_load for unknown element types.
                    let elem = self.emit_elem_load(&elem_ptr, &esz_val);
                    // 5c.29: float elements round-trip as raw bits â€” reinterpret
                    // them instead of letting callers sitofp the bit pattern.
                    if let Some(fty) = self.vec_elem_float_type(container) {
                        if fty == "float" {
                            let t32 = self.fresh_tmp();
                            self.emitln(&format!("  {t32} = trunc i64 {elem} to i32"));
                            let f = self.fresh_tmp();
                            self.emitln(&format!("  {f} = bitcast i32 {t32} to float"));
                            return Ok((f, "float".to_string()));
                        } else {
                            let d = self.fresh_tmp();
                            self.emitln(&format!("  {d} = bitcast i64 {elem} to double"));
                            return Ok((d, "double".to_string()));
                        }
                    }
                    return Ok((elem, "i64".to_string()));
                }
                // Fixed-size stack array [N x T]: use the existing alloca for
                // Ident containers (no fresh alloca per access) or stash into an
                // alloca and GEP for non-local array values.
                if cont_ty.starts_with('[') && cont_ty.contains(" x ") {
                    let (arr_ptr, arr_ptr_ty) = if let Expr::Ident(id) = &**container {
                        if let Some((slot, _slot_ty)) = self.lookup_local(&id.name) {
                            // Use the existing alloca pointer directly ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â avoids
                            // creating a fresh alloca on every loop iteration.
                            (slot.clone(), format!("{cont_ty}*"))
                        } else {
                            let arr_slot = self.fresh_tmp();
                            self.emitln(&format!("  {arr_slot} = alloca {cont_ty}"));
                            self.emitln(&format!("  store {cont_ty} {cont_val}, {cont_ty}* {arr_slot}"));
                            (arr_slot, format!("{cont_ty}*"))
                        }
                    } else {
                        let arr_slot = self.fresh_tmp();
                        self.emitln(&format!("  {arr_slot} = alloca {cont_ty}"));
                        self.emitln(&format!("  store {cont_ty} {cont_val}, {cont_ty}* {arr_slot}"));
                        (arr_slot, format!("{cont_ty}*"))
                    };
                    let elem_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {elem_ptr} = getelementptr {cont_ty}, {arr_ptr_ty} {arr_ptr}, i64 0, i64 {idx}"));
                    let inner_ty = Self::extract_array_elem_ty(&cont_ty);
                    let elem = self.fresh_tmp();
                    self.emitln(&format!("  {elem} = load {inner_ty}, {inner_ty}* {elem_ptr}"));
                    let result = self.val_to_i64(&elem, &inner_ty);
                    return Ok((result, "i64".to_string()));
                }
                // Handle pointer-typed array references from monomorphised generic params.
                if cont_ty.ends_with('*') && cont_ty != "i8*" {
                    let elem_ty = cont_ty.trim_end_matches('*');
                    // Array buffers store the length at [0], so elements start at [1].
                    let offset = self.fresh_tmp();
                    self.emitln(&format!("  {offset} = add i64 {idx}, 1"));
                    let elem_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {elem_ptr} = getelementptr {elem_ty}, {cont_ty} {cont_val}, i64 {offset}"));
                    let elem = self.fresh_tmp();
                    self.emitln(&format!("  {elem} = load {elem_ty}, {elem_ty}* {elem_ptr}"));
                    let result = self.val_to_i64(&elem, &elem_ty);
                    return Ok((result, "i64".to_string()));
                }
                // safe default.
                Ok(("0".to_string(), "i64".to_string()))
            }
            Expr::AtPre(inner, _) => {
                // If inner is `self` or any variable, resolve to its pre-state snapshot
                if let Expr::Ident(id) = inner.as_ref() {
                    let pre_name = if id.name == "self" {
                        "__self_pre".to_string()
                    } else {
                        format!("__{}_pre", id.name)
                    };
                    if let Some((ptr, llvm_ty)) = self.lookup_local(&pre_name).cloned() {
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                        return Ok((tmp, llvm_ty));
                    }
                    // Fallback: if no pre snapshot, use current value
                }
                self.compile_expr(inner)
            }
            Expr::Ref(inner, _) | Expr::MutRef(inner, _) => {
                // When `&this.field` (or `&self.field`) appears inside a method body,
                // the field was registered as a GEP pointer during the prologue.
                // Return that pointer directly instead of compiling the inner
                // expression (which loads the field value), so that this-based
                // methods receive a proper pointer receiver.
                // GEP always produces a pointer; if the registered field type is
                // a plain struct, append `*` so the pointer propagates correctly
                // through coerce_value and call-site receiver handling.
                if let Expr::Field(base, field_name_expr, _) = inner.as_ref() {
                    if let Expr::Ident(base_ident) = base.as_ref() {
                        let is_self_base = base_ident.name == "this" || base_ident.name == "self";
                        if is_self_base {
                            // The prologue registered SELF's field GEP pointers
                            // under their bare names.
                            if let Some((gep_ptr, gep_ty)) = self.lookup_local(&field_name_expr.name).cloned() {
                                if self.lookup_local("self").is_some() {
                                    let ptr_ty = if gep_ty.starts_with("%struct.") && !gep_ty.ends_with('*') {
                                        format!("{gep_ty}*")
                                    } else {
                                        gep_ty
                                    };
                                    return Ok((gep_ptr, ptr_ty));
                                }
                            }
                        } else if let Some((slot, slot_ty)) = self.lookup_local(&base_ident.name).cloned() {
                            // 5c.30: `&local.field` â€” emit a REAL GEP into the
                            // local's storage. Previously the bare field name was
                            // looked up as a local, so `&addr3.ip` silently bound
                            // to an unrelated variable named `ip` (TFR test 7).
                            if slot_ty.starts_with("%struct.") {
                                let type_name = slot_ty[8..].trim_end_matches('*').to_string();
                                let base_ty = format!("%struct.{type_name}");
                                let base_ptr = if slot_ty.ends_with('*') {
                                    let p = self.fresh_tmp();
                                    self.emitln(&format!("  {p} = load {slot_ty}, {slot_ty}* {slot}"));
                                    p
                                } else {
                                    slot
                                };
                                if let Some(field_names) = self.types.get(&type_name)
                                    .or_else(|| {
                                        let suffix = format!(".{type_name}");
                                        self.types.keys().find(|k| k.ends_with(&suffix))
                                            .and_then(|k| self.types.get(k))
                                    })
                                    .cloned()
                                {
                                    if let Some(fi) = field_names.iter().position(|f| f == &field_name_expr.name) {
                                        let field_llvm_ty = self.field_llvm_type(&type_name, fi);
                                        // Only take the GEP path for struct-typed
                                        // fields; scalar/handle fields keep the
                                        // by-value fallback below.
                                        if field_llvm_ty.starts_with("%struct.") && !field_llvm_ty.ends_with('*') {
                                            let gep = self.fresh_tmp();
                                            self.emitln(&format!("  {gep} = getelementptr {base_ty}, {base_ty}* {base_ptr}, i32 0, i32 {fi}"));
                                            return Ok((gep, format!("{field_llvm_ty}*")));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // When a fixed array literal (e.g. [1,2,3]) is used with & in
                // a Vec context, materialise a proper %struct.Vec from the
                // array buffer instead of forwarding the raw i8* pointer.
                // This ensures the Vec owns a heap copy, preventing stack
                // corruption when the Vec is modified.
                if let Expr::Array(elems, _) = inner.as_ref() {
                    let n = elems.len() as i64;
                    let alloc_count = n + 1;
                    let buf = self.fresh_tmp();
                    self.emitln(&format!("  {buf} = alloca i64, i64 {alloc_count}"));
                    let gep0 = self.fresh_tmp();
                    self.emitln(&format!("  {gep0} = getelementptr i64, i64* {buf}, i64 0"));
                    self.emitln(&format!("  store i64 {n}, i64* {gep0}"));
                    for (i, e) in elems.iter().enumerate() {
                        let (v, _) = self.compile_expr(e)?;
                        let gep = self.fresh_tmp();
                        let idx = (i + 1) as i64;
                        self.emitln(&format!("  {gep} = getelementptr i64, i64* {buf}, i64 {idx}"));
                        let store_val = self.val_to_i64(&v, &self.infer_llvm_type(e));
                        self.emitln(&format!("  store i64 {store_val}, i64* {gep}"));
                    }
                    let ptr = self.fresh_tmp();
                    self.emitln(&format!("  {ptr} = bitcast i64* {buf} to i8*"));
                    // Now build a proper Vec from the array buffer
                    let vec_alloca = self.fresh_tmp();
                    let struct_ty = "%struct.Vec";
                    self.emitln(&format!("  {vec_alloca} = alloca {struct_ty}"));
                    // len from buffer[0]
                    let len_slot = self.fresh_tmp();
                    self.emitln(&format!("  {len_slot} = bitcast i8* {ptr} to i64*"));
                    let len_val = self.fresh_tmp();
                    self.emitln(&format!("  {len_val} = load i64, i64* {len_slot}"));
                    // heap copy of elements
                    let byte_count = self.fresh_tmp();
                    self.emitln(&format!("  {byte_count} = mul i64 {len_val}, 8"));
                    let heap_copy = self.fresh_tmp();
                    self.emitln(&format!("  {heap_copy} = call i8* @malloc(i64 {byte_count})"));
                    let malloc_ok = self.fresh_block("ref_arr_malloc_ok");
                    let malloc_fail = self.fresh_block("ref_arr_malloc_fail");
                    let malloc_check = self.fresh_tmp();
                    self.emitln(&format!("  {malloc_check} = icmp eq i8* {heap_copy}, null"));
                    self.emitln(&format!("  br i1 {malloc_check}, label %{malloc_fail}, label %{malloc_ok}"));
                    self.emitln(&format!("\n{malloc_fail}:"));
                    self.emitln("  call void @llvm.trap()");
                    self.emitln("  unreachable");
                    self.emitln(&format!("\n{malloc_ok}:"));
                    let src_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {src_ptr} = getelementptr i8, i8* {ptr}, i64 8"));
                    self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {heap_copy}, i8* {src_ptr}, i64 {byte_count}, i1 false)"));
                    // store Vec fields
                    let g0 = self.fresh_tmp();
                    self.emitln(&format!("  {g0} = getelementptr {struct_ty}, {struct_ty}* {vec_alloca}, i32 0, i32 0"));
                    self.emitln(&format!("  store i8* {heap_copy}, i8** {g0}"));
                    let g1 = self.fresh_tmp();
                    self.emitln(&format!("  {g1} = getelementptr {struct_ty}, {struct_ty}* {vec_alloca}, i32 0, i32 1"));
                    self.emitln(&format!("  store i64 {len_val}, i64* {g1}"));
                    let g2 = self.fresh_tmp();
                    self.emitln(&format!("  {g2} = getelementptr {struct_ty}, {struct_ty}* {vec_alloca}, i32 0, i32 2"));
                    self.emitln(&format!("  store i64 {len_val}, i64* {g2}"));
                    let g3 = self.fresh_tmp();
                    self.emitln(&format!("  {g3} = getelementptr {struct_ty}, {struct_ty}* {vec_alloca}, i32 0, i32 3"));
                    self.emitln(&format!("  store i64 8, i64* {g3}"));
                    let loaded = self.fresh_tmp();
                    self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {vec_alloca}"));
                    return Ok((loaded, struct_ty.to_string()));
                }
                // Compile the inner expression and return a pointer to the value.
                // For struct-typed idents, use the alloca pointer directly so
                // this-based methods receive a proper pointer receiver.
                // coerce_value handles both directions (structÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Âpointer) for safety.
                if let Expr::Ident(id) = inner.as_ref() {
                    if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                        if slot_ty.starts_with("%struct.") {
                            return Ok((slot, format!("{slot_ty}*")));
                        }
                    }
                }
                self.compile_expr(inner)
            }
            Expr::Some(inner, _) => {
                self.used_builtins.insert("Option".to_string());
                let (val, inner_ty) = self.compile_expr(inner)?;
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
                Ok((loaded, opt_ty.to_string()))
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
                Ok((loaded, opt_ty.to_string()))
            }
            Expr::Ok(inner, _) => {
                self.used_builtins.insert("Result".to_string());
                let (val, inner_ty) = self.compile_expr(inner)?;
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
                Ok((loaded, result_ty.to_string()))
            }
            Expr::Err(inner, _) => {
                self.used_builtins.insert("Result".to_string());
                let (val, inner_ty) = self.compile_expr(inner)?;
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
                Ok((loaded, result_ty.to_string()))
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
                        let (field_val, field_val_ty) = self.compile_expr(val)?;
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
                        } else {
                            // Coerce the field value to the field slot's declared
                            // type (e.g. an i8 char value into an i64 field).
                            self.coerce_value(&field_val, &field_val_ty, &field_llvm_ty)
                        };
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {parent_field_idx}"));
                        self.emitln(&format!("  store {field_llvm_ty} {store_val}, {field_llvm_ty}* {gep}"));
                    }
                } else {
                    for (i, (_, val)) in fields.iter().enumerate() {
                        let (mut field_val, mut field_val_ty) = self.compile_expr(val)?;
                        let mut field_llvm_ty = self.field_llvm_type(&name.name, i);
                        // 5c.29: Generic container fields (Vec[Int], ...) are i64
                        // HANDLES (5c.28h). A by-value container header must be
                        // BOXED on the heap and the pointer stored as the handle;
                        // storing the 32-byte %struct.Vec into the 8-byte i64 slot
                        // corrupted the stack and broke every handle reader.
                        let is_generic_container_field = self
                            .field_xiom_type(&name.name, i)
                            .map_or(false, |t| t.contains('['));
                        if field_llvm_ty == "i64"
                            && is_generic_container_field
                            && field_val_ty.starts_with("%struct.")
                            && !field_val_ty.ends_with('*')
                        {
                            field_val = self.emit_box_struct_handle(&field_val, &field_val_ty);
                            field_val_ty = "i64".to_string();
                        } else if field_llvm_ty == "i64" {
                            // For generic types, field_llvm_type may return "i64" for unresolved type params (like T).
                            // Fall back to the field value's actual compiled LLVM type.
                            if field_val_ty != "i64" {
                                field_llvm_ty = field_val_ty.clone();
                            }
                        }
                        let store_val = if field_val == "0" && (field_llvm_ty.ends_with('*') || field_llvm_ty.starts_with('\"')) {
                            "null".to_string()
                        } else if field_llvm_ty.ends_with('*') && field_val.chars().all(|c| c.is_ascii_digit() || c == '-') {
                            let ptr_tmp = self.fresh_tmp();
                            self.emitln(&format!("  {ptr_tmp} = inttoptr i64 {field_val} to {field_llvm_ty}"));
                            ptr_tmp
                        } else {
                            // Coerce the field value to the field slot's declared
                            // type (e.g. an i8 char value into an i64 field).
                            self.coerce_value(&field_val, &field_val_ty, &field_llvm_ty)
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
                Ok((loaded, struct_ty))
            }
            Expr::Array(elems, _) => {
                // Materialize a fixed-size `[N]T` array literal. 5c-R: use the
                // ACTUAL element LLVM type (Float64 → double, Int → i64, struct →
                // %struct.Name) instead of always coercing to i64. The element
                // type is determined from the first element's compiled type.
                let n = elems.len() as i64;
                let elem_llvm_ty = if let Some(first) = elems.first() {
                    let t = self.infer_llvm_type(first);
                    if t == "double" || t == "float" || t.starts_with("%struct.") { t } else { "i64".to_string() }
                } else { "i64".to_string() };
                let buf = self.fresh_tmp();
                let alloc_count = n + 1;
                self.emitln(&format!("  {buf} = alloca {elem_llvm_ty}, i64 {alloc_count}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {elem_llvm_ty}, {elem_llvm_ty}* {buf}, i64 0"));
                // Store length as i64 for all types (the length is always an integer)
                let gep0_i64 = self.fresh_tmp();
                self.emitln(&format!("  {gep0_i64} = bitcast {elem_llvm_ty}* {gep0} to i64*"));
                self.emitln(&format!("  store i64 {n}, i64* {gep0_i64}"));
                for (i, e) in elems.iter().enumerate() {
                    let (v, val_ty) = self.compile_expr(e)?;
                    let gep = self.fresh_tmp();
                    let idx = (i + 1) as i64;
                    self.emitln(&format!("  {gep} = getelementptr {elem_llvm_ty}, {elem_llvm_ty}* {buf}, i64 {idx}"));
                    let store_val = self.coerce_value(&v, &val_ty, &elem_llvm_ty);
                    self.emitln(&format!("  store {elem_llvm_ty} {store_val}, {elem_llvm_ty}* {gep}"));
                }
                let ptr = self.fresh_tmp();
                self.emitln(&format!("  {ptr} = bitcast {elem_llvm_ty}* {buf} to i8*"));
                // Track this register as originating from an array literal
                // so val_to_struct can distinguish array-buffer i8* from generic i8*.
                self.array_value_regs.insert(ptr.clone());
                Ok((ptr, "i8*".to_string()))
            }
            Expr::Closure(_, _, _, _) | Expr::PipeClosure(_, _, _) => Ok(("0".to_string(), "i64".to_string())),
            Expr::As(inner, ty, _) => {
                // Use the compiled value's REAL LLVM type as the source of the
                // cast (from the refactor), falling back to infer only when the
                // real type is unknown. This ensures e.g. `c as Int` where `c` is
                // a Char (i8) actually sign-extends i8->i64 rather than emitting an
                // untyped/mis-typed value.
                let (val, val_ty) = self.compile_expr(inner)?;
                let mut inner_llvm_ty = if !val_ty.is_empty() && val_ty != "void" {
                    val_ty.clone()
                } else {
                    self.infer_llvm_type(inner)
                };
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
                        Ok((tmp, "double".to_string()))
                    }
                    ("double", "i64") => {
                        self.emitln(&format!("  {tmp} = fptosi double {val} to i64"));
                        Ok((tmp, "i64".to_string()))
                    }
                    (a, b) if a == b => Ok((val, target_llvm_ty.clone())),
                    // 5c-E G2: &local as Int — emit ADDRESS not VALUE
                    (a, b) if matches!(inner.as_ref(), Expr::Ref(_, _) | Expr::MutRef(_, _))
                        && int_width(b).is_some()
                        && int_width(a).is_some() =>
                    {
                        // Re-compile to get the ADDRESS pointer, not the value
                        if let Expr::Ref(ri, _) | Expr::MutRef(ri, _) = inner.as_ref() {
                            if let Expr::Ident(id) = ri.as_ref() {
                                if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                                    let addr_val = format!("{}*", slot_ty);
                                    let ptr_reg = self.fresh_tmp();
                                    self.emitln(&format!("  {ptr_reg} = ptrtoint {addr_val} {slot} to {b}"));
                                    return Ok((ptr_reg, target_llvm_ty.clone()));
                                }
                            }
                        }
                        // Fallback
                        self.emitln(&format!("  {tmp} = sext {a} {val} to {b}"));
                        Ok((tmp, target_llvm_ty.clone()))
                    }
                    // Integer <-> integer width conversions (e.g. Int<->Char, Int<->Int8/16/32).
                    // Char is i8 and Int is i64, so Int->Char truncs and Char->Int sign-extends.
                    (a, b) if int_width(a).is_some() && int_width(b).is_some() => {
                        let aw = int_width(a).unwrap();
                        let bw = int_width(b).unwrap();
                        if bw < aw {
                            self.emitln(&format!("  {tmp} = trunc {a} {val} to {b}"));
                            Ok((tmp, target_llvm_ty.clone()))
                        } else {
                            self.emitln(&format!("  {tmp} = sext {a} {val} to {b}"));
                            Ok((tmp, target_llvm_ty.clone()))
                        }
                    }
                    // Fallback: coerce the value to the declared target type so the
                    // As expression's reported type always matches the value.
                    _ => {
                        let coerced = self.coerce_value(&val, &inner_llvm_ty, &target_llvm_ty);
                        Ok((coerced, target_llvm_ty.clone()))
                    }
                }
            }
            Expr::Await(inner, _) => self.compile_expr(inner),
            Expr::Comptime(inner, _) => self.compile_expr(inner),
            Expr::Unsafe(block, _) => {
                // An `unsafe { ... }` block is an expression whose value is its
                // tail. Compile every statement (Let/Var/Assign/Return/ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â¦) and
                // return the value of the final expression, so
                // `let x = unsafe { ffi_call() }` and
                // `fn f() -> T { unsafe { ffi_call() } }` yield a real SSA value
                // instead of an empty operand (previously returned String::new(),
                // producing invalid `store T ,` / `ret T ` IR).
                let mut last = String::new();
                let mut last_ty = String::new();
                let n = block.stmts.len();
                for (i, item) in block.stmts.iter().enumerate() {
                    let is_last = i + 1 == n;
                    match item {
                        xiom_ast::StmtOrExpr::Expr(e) => {
                            let (v, vt) = self.compile_expr(e)?;
                            if is_last {
                                last = v;
                                last_ty = vt;
                            }
                        }
                        xiom_ast::StmtOrExpr::Stmt(s) => {
                            if is_last {
                                if let Stmt::Expr(e, ..) = s {
                                    let (v, vt) = self.compile_expr(e)?;
                                    last = v;
                                    last_ty = vt;
                                } else {
                                    self.compile_stmt(s)?;
                                }
                            } else {
                                self.compile_stmt(s)?;
                            }
                        }
                    }
                }
                if last.is_empty() {
                    // No value-producing tail expression.
                    Ok(("0".to_string(), "void".to_string()))
                } else {
                    Ok((last, last_ty))
                }
            }
            Expr::If(cond, then_block, elifs, else_block, _) => {
                // Value-producing if-expression (e.g. `let x = if c { 1 } else { 0 }`).
                // Emit conditional branches ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â  la Stmt::If, but have each arm store its
                // tail expression into a result alloca.  At the merge point, load the
                // result and return it.
                
                // Compute condition value
                let (cond_raw, cond_ty) = self.compile_expr(cond)?;
                let cond_val = if cond_ty == "i1" {
                    cond_raw
                } else {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = icmp ne {cond_ty} {cond_raw}, 0"));
                    tmp
                };

                let result_ty = "i64".to_string();
                let result_alloca = self.fresh_tmp();
                self.emitln(&format!("  {result_alloca} = alloca {result_ty}"));

                let then_label = self.fresh_block("if_then");
                let else_label = if !elifs.is_empty() || else_block.is_some() {
                    self.fresh_block("if_else")
                } else {
                    self.fresh_block("if_merge")
                };
                let merge_label = self.fresh_block("if_merge");

                let block_ends_with_ret = |b: &Block| -> bool {
                    b.stmts.last().map_or(false, |s| matches!(s, StmtOrExpr::Stmt(Stmt::Return(..))))
                };
                let mut merge_reachable = false;

                self.emitln(&format!("  br i1 {cond_val}, label %{then_label}, label %{else_label}"));
                self.emitln(&format!("\n{then_label}:"));
                self.compile_if_arm_value(then_block, &result_alloca, &result_ty)?;
                if !block_ends_with_ret(then_block) {
                    self.emitln(&format!("  br label %{merge_label}"));
                    merge_reachable = true;
                }

                // Elif chain
                let mut prev_label = else_label.clone();
                for (i, (econd, eblock)) in elifs.iter().enumerate() {
                    self.emitln(&format!("\n{prev_label}:"));
                    let (ec_raw, ec_ty) = self.compile_expr(econd)?;
                    let ec_val = if ec_ty == "i1" { ec_raw } else {
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = icmp ne {ec_ty} {ec_raw}, 0"));
                        tmp
                    };
                    let elif_then = self.fresh_block("elif_then");
                    let elif_next = if i + 1 < elifs.len() || else_block.is_some() {
                        self.fresh_block("elif_next")
                    } else {
                        merge_label.clone()
                    };
                    self.emitln(&format!("  br i1 {ec_val}, label %{elif_then}, label %{elif_next}"));
                    self.emitln(&format!("\n{elif_then}:"));
                    self.compile_if_arm_value(eblock, &result_alloca, &result_ty)?;
                    if !block_ends_with_ret(eblock) {
                        self.emitln(&format!("  br label %{merge_label}"));
                        merge_reachable = true;
                    }
                    prev_label = elif_next;
                }

                // Else block
                if let Some(eb) = else_block {
                    self.emitln(&format!("\n{prev_label}:"));
                    self.compile_if_arm_value(eb, &result_alloca, &result_ty)?;
                    if !block_ends_with_ret(eb) {
                        self.emitln(&format!("  br label %{merge_label}"));
                        merge_reachable = true;
                    }
                } else if elifs.is_empty() {
                    // No elifs, no else ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â the original else_label IS the merge_label
                } else if prev_label != merge_label {
                    self.emitln(&format!("\n{prev_label}:"));
                    self.emitln(&format!("  br label %{merge_label}"));
                    merge_reachable = true;
                }

                self.emitln(&format!("\n{merge_label}:"));
                if !merge_reachable {
                    self.emitln("  unreachable");
                }
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {result_ty}, {result_ty}* {result_alloca}"));
                Ok((loaded, result_ty))
            }
            Expr::Error(_guarantee, _span) => {
                // Error-poisoned node: the checker already emitted a diagnostic.
                // Return a dummy i64 value so compilation continues without
                // cascading errors (rustc lesson: ErrorGuaranteed).
                Ok(("0".to_string(), "i64".to_string()))
            }
            Expr::Match(scrutinee, arms, span) => {
                // Compile a match-expression by allocating a result slot, running the
                // statement-form match (whose arm bodies store their value into
                // `match_result_ptr`), then loading the slot as this expression's value.
                // The result type is the widest type across all arms (struct > i64),
                // with `coerce_value` handling per-arm conversions during the store.
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
                Ok((loaded, result_ty))
            }
        }
    }

    /// Returns true if `receiver` in `receiver.method(args)` is a real VALUE
    /// instance (so its value must be passed as the `self` argument), vs a module
    /// path (`xiom.char`) or bare type name (`LogLevel`) used only for name
    /// qualification (no receiver argument).
    ///
    /// Module paths and type names are NOT instances: `xiom` is not a local, and
    /// `xiom.char` does not resolve to a struct type. This prevents miscompiling
    /// `xiom.char.to_uppercase(x)` as a method call with a phantom receiver arg.
    pub(crate) fn receiver_is_instance(&self, receiver: &Expr) -> bool {
        match receiver {
            // A bare identifier is an instance only if it's a bound local/param
            // (a value). Bare type names (`LogLevel`) and module roots (`xiom`)
            // are not locals ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ not instances.
            Expr::Ident(ident) => self.lookup_local(&ident.name).is_some(),
            // `a.b`: instance iff its base chain is rooted in a value (local/self),
            // e.g. `obj.field`. A module path like `xiom.char` is rooted in `xiom`
            // (not a local) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ NOT an instance. Also an instance if the whole
            // expression has a concrete struct type.
            Expr::Field(base, field, _) => {
                // `module.Type` static path (e.g. `alloc.Layout`) is NOT an instance:
                // the base is a module (not a value) and the leaf names a known type.
                if !self.receiver_is_instance(base)
                    && (self.types.contains_key(&field.name)
                        || self.type_meta.contains_key(&field.name)
                        || self.type_meta.keys().any(|k| k.ends_with(&format!(".{}", field.name))))
                {
                    return false;
                }
                self.receiver_is_instance(base) || self.infer_struct_type_name(receiver).is_some()
            }
            // Calls / indexing / parens evaluate to values.
            Expr::Call(..) | Expr::Index(..) | Expr::Paren(..) => true,
            // Any other receiver form evaluates to a value ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â preserve the prior
            // "complex receiver is an instance" behavior (only the Ident type-name
            // and Field module-path shapes above are treated as non-instances).
            _ => true,
        }
    }

}
