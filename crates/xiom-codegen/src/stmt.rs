// XIOM Codegen -- Statement compilation (extracted from expr.rs, M4.2)
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use xiom_ast::*;
use std::collections::HashSet;
use crate::llvm_consts::*;

use super::IrEmitter;

impl IrEmitter {
    /// round-15: infer the XIOM element NAME of a scalar array-literal
    /// element -- As-target aware so `200 as UInt8` yields "UInt8" (not the
    /// LLVM-width "Int8"). Falls back to the local's registered type for
    /// Ident elements; structs/tuples return None (handled elsewhere).
    pub(crate) fn infer_scalar_elem_xiom(&self, e: &Expr) -> Option<String> {
        match e {
            Expr::As(_, ty, _) => Some(Self::type_from_ast(ty)),
            Expr::Int(..) => Some("Int".to_string()),
            Expr::Float(..) => Some("Float64".to_string()),
            Expr::Bool(..) => Some("Bool".to_string()),
            Expr::Char(..) => Some("Char".to_string()),
            Expr::Str(..) => Some("Str".to_string()),
            Expr::Ident(id) => self.xiom_type_of_local(&id.name),
            _ => None,
        }
    }

    pub(crate) fn compile_stmt_impl(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, _ty, value, _) => {
                // 5c.30: Empty array `[]` assigned to a Vec-typed variable -- emit
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
                        // round-15: record the XIOM element NAME (As-target
                        // aware) so generic-arg inference keeps signedness
                        // (`[200 as UInt8, ...]` -> "UInt8", not "Int8").
                        if let Some(elem_xiom) = elems.first().and_then(|e| self.infer_scalar_elem_xiom(e)) {
                            self.local.local_array_elem_xiom.insert(name.name.clone(), elem_xiom);
                        }
                        // 5c.30: record array size for const-generic inference
                        self.local.local_array_sizes.insert(name.name.clone(), elems.len() as i64);
                        // M33: When an array literal of struct elements is
                        // converted to a Vec, record the element type so that
                        // `resolve_vec_elem_type` finds it later. Without this,
                        // `items[i].val` loads the raw i64 handle and field
                        // access returns 0 instead of dereferencing the boxed
                        // struct (affects m33_a08, m33_a16, m33_a17, m35_l07, etc.)
                        if let Some(elem_xiom) = elems.first()
                            .and_then(|e| self.infer_struct_type_name(e))
                        {
                            self.local.local_vec_elem.insert(name.name.clone(), elem_xiom);
                        }
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
                        // R7 (2026-09-10): a CONTAINER-typed FIELD binding
                        // (`var vals = m.values;`) keeps the element type so
                        // `vals[i]` takes the struct-load path -- the old
                        // fallthrough dropped the record and the element read
                        // scalar-loaded the tag (p_map_json wrong variant).
                        Expr::Field(..) => self.resolve_vec_elem_xiom(value),
                        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
                            // Inherit from first argument's Vec element type
                            let from_arg = args.first().and_then(|a| {
                                if let Expr::Ident(id) = a {
                                    self.local.local_vec_elem.get(&id.name).cloned()
                                } else { None }
                            });
                            // Then fall back to the callee's declared return type
                            // (e.g. `Vec[Tuple__Str__Str]`).
                            from_arg.or_else(|| {
                                self.callee_return_xiom(func).and_then(|rt| {
                                    rt.strip_prefix("Vec[").and_then(|rest| rest.strip_suffix(']')).map(|e| e.to_string())
                                })
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
                    // B-007: a local bound from an fn-typed CONTAINER ELEMENT
                    // (`var t = tw.tasks[i]; t();` -- Vec[fn()]) holds a closure
                    // ENV -- calling it must go through the M20-A1 env path.
                    || matches!(e, Expr::Index(container, _, _) if {
                        let elem = self.resolve_vec_container_elem_xiom(container);
                        elem.map_or(false, |x| x.starts_with("fn("))
                    })
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
                // 5c.30: Empty array `[]` assigned to Vec-typed variable -- emit
                // proper Vec initialization to avoid i8* -> %struct.Vec coercion.
                if is_empty_array_to_vec {
                    let elem_size: i64 = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .and_then(|elem| {
                            // Compute struct size for known types. R39: pick
                            // deterministically when several same-leaf types
                            // are module-qualified (raw HashMap order sized
                            // the buffer differently per process).
                            let suffix = format!(".{}", elem);
                            let candidates: Vec<String> = self.types.types.keys().into_iter()
                                .filter(|k| k.ends_with(&suffix) || k.as_str() == elem)
                                .collect();
                            let sname = self.pick_deterministic(candidates)
                                .unwrap_or_else(|| elem.to_string());
                            Some(self.vec_elem_storage_size(&sname))
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
                    let _loaded = self.emit_vec_load_fields(&struct_alloca);
                    self.add_local(&name.name, struct_alloca, &"%struct.Vec".to_string());
                    // Use declared element type, not hardcoded Int
                    let elem_ty = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .unwrap_or_else(|| "Int".to_string());
                    self.local.local_vec_elem.insert(name.name.clone(), elem_ty);
                    return Ok(());
                }
                // LET-array P1 (docs/LET_ARRAY_DECISION.md): an ANNOTATED fixed
                // array (`let c: [3]Int = [7,8,9]`) stores elements DIRECTLY
                // into the [N x T] slot, mirroring the VAR BUG 53 path. The
                // M33 Vec conversion below fed a %struct.Vec value into the
                // declared [3 x i64] slot (invalid IR: `%tmp defined with type
                // 'i64/pt' but expected '[3 x i64]'`).
                if let (Expr::Array(elems, _), Some(t)) = (value, _ty.as_ref()) {
                    if matches!(&**t, Type::Array(..)) {
                        let type_name = self.concrete_type_for(t);
                        if let Ok(arr_ty) = self.llvm_type_for(&type_name) {
                            if arr_ty.starts_with('[') && arr_ty.contains(" x ") {
                                let elem_llvm = Self::extract_array_elem_ty(&arr_ty);
                                let slot = self.fresh_tmp();
                                if self.local.loop_depth > 0 {
                                    self.local.hoisted_allocas.push((slot.clone(), arr_ty.clone()));
                                } else {
                                    self.emitln(&format!("  {slot} = alloca {arr_ty}"));
                                }
                                for (i, e) in elems.iter().enumerate() {
                                    let (v, vt) = self.compile_expr(e)?;
                                    let cv = self.coerce_value(&v, &vt, &elem_llvm);
                                    let gep = self.fresh_tmp();
                                    self.emitln(&format!("  {gep} = getelementptr {arr_ty}, {arr_ty}* {slot}, i64 0, i64 {i}"));
                                    self.emitln(&format!("  store {elem_llvm} {cv}, {elem_llvm}* {gep}"));
                                }
                                self.add_local(&name.name, slot, &arr_ty);
                                self.local.array_locals.insert(name.name.clone());
                                return Ok(());
                            }
                        }
                    }
                }
                // LET-array P3 (docs/LET_ARRAY_DECISION.md): an UNANNOTATED
                // array literal binds a FIXED array `[N]T` (var/annotation
                // parity). The old M33 conversion made `let a = [...]` a
                // %struct.Vec while annotated/var fixed-array bindings were
                // `[N x T]`, so the representation depended on the binding
                // form. Empty literals and explicitly Vec/Slice-annotated
                // bindings keep the M33 bridge below (the collection API
                // path); only `let` is flipped -- `var` literals can be
                // pushed to and stay Vec.
                if let Expr::Array(elems, _) = value {
                    if _ty.is_none() && !elems.is_empty() {
                        let elem_xiom = elems.first()
                            .and_then(|e| self.infer_struct_type_name(e))
                            .or_else(|| elems.first().and_then(|e| self.infer_scalar_elem_xiom(e)))
                            .unwrap_or_else(|| "Int".to_string());
                        if let Ok(elem_llvm) = self.llvm_type_for(&elem_xiom) {
                            let arr_ty = format!("[{} x {}]", elems.len(), elem_llvm);
                            let slot = self.fresh_tmp();
                            if self.local.loop_depth > 0 {
                                self.local.hoisted_allocas.push((slot.clone(), arr_ty.clone()));
                            } else {
                                self.emitln(&format!("  {slot} = alloca {arr_ty}"));
                            }
                            for (i, e) in elems.iter().enumerate() {
                                let (v, vt) = self.compile_expr(e)?;
                                let cv = self.coerce_value(&v, &vt, &elem_llvm);
                                let gep = self.fresh_tmp();
                                self.emitln(&format!("  {gep} = getelementptr {arr_ty}, {arr_ty}* {slot}, i64 0, i64 {i}"));
                                self.emitln(&format!("  store {elem_llvm} {cv}, {elem_llvm}* {gep}"));
                            }
                            self.add_local(&name.name, slot, &arr_ty);
                            self.local.array_locals.insert(name.name.clone());
                            return Ok(());
                        }
                    }
                }
                let (val, val_llvm_ty) = if let Expr::Array(elems, _) = value {
                    // M33 (retained for Vec/Slice-annotated and EMPTY
                    // literals): convert to Vec so `&arr` produces a proper
                    // Vec pointer instead of an i8* buffer pointer. Fixes
                    // ACCESS_VIOLATION on `binary_search(&arr, ...)` where arr
                    // is a let-bound array.
                    let elem_ty = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .or_else(|| {
                            elems.first().and_then(|e| self.infer_struct_type_name(e))
                        })
                        .unwrap_or_else(|| "Int".to_string());
                    self.compile_array_as_vec(elems, &elem_ty)?
                } else {
                    self.compile_expr(value)?
                };
                let declared_llvm_ty: Option<String> = _ty.as_ref().map(|t| {
                    // M65 R7 (2026-09-10): an ANNOTATED local slot must use the
                    // CONCRETE container type -- `var found: Option[JsonValue];`
                    // allocated %struct.Option (opaque i64 payload) while the
                    // assignments/match built Option__JsonValue, so the store
                    // was an invalid-IR struct mismatch (smoke_stress_serialize_
                    // large_json clang failure). concrete_type_for keeps scalars
                    // and non-container types unchanged.
                    let name = self.concrete_type_for(t);
                    self.llvm_type_for(&name).unwrap_or_else(|_| LLVM_I64.to_string())
                });
                // M17: Track XIOM type and signedness for narrow-int widening.
                if let Some(ty) = _ty {
                    // R49: preserve generic args for user named generics
                    // ("PriorityQueue[Task]") -- the receiver-type-arg
                    // inference needs them (L6-28).
                    let xiom_name = Self::type_annotation_name(ty);
                    self.local.local_xiom_types.insert(name.name.clone(), xiom_name.clone());
                    if Self::is_signed_xiom_type(&xiom_name) {
                        self.local.signed_locals.insert(name.name.clone());
                    } else {
                        self.local.signed_locals.remove(&name.name);
                    }
                } else if let Some(inferred) = Self::infer_value_xiom_type(value) {
                    // BUG 14: `var big = x as UInt128` -- infer signedness from
                    // the cast target when there is no annotation.
                    // round-14c: ALSO track signed_locals -- `var n8 = n as
                    // Int8` widened the load ZEXT (128 != -128 in
                    // smoke_string_narrow) because the local was unsigned.
                    self.local.local_xiom_types.insert(name.name.clone(), inferred.clone());
                    if Self::is_signed_xiom_type(&inferred) {
                        self.local.signed_locals.insert(name.name.clone());
                    } else {
                        self.local.signed_locals.remove(&name.name);
                    }
                } else if let Some(ix) = self.infer_if_xiom_type(value) {
                    // gzip fix (2026-08-19): `let v = if c { f() } else { g() };`
                    // -- the binding keeps the arm tail's XIOM type ("Vec[UInt8]").
                    self.local.local_xiom_types.insert(name.name.clone(), ix);
                } else if let Some(tx) = self.infer_try_xiom_type(value) {
                    // Round-6 fix (2026-08-19): `let name = f()?;` -- the binding
                    // keeps the Option/Result payload XIOM type ("Str") so
                    // method dispatch on it works.
                    self.local.local_xiom_types.insert(name.name.clone(), tx);
                } else if let Some(px) = self.infer_field_payload_xiom(value) {
                    // gzip-DECOMPRESS fix (2026-08-19): `let v = r.value;` --
                    // the binding keeps the Option/Result payload type
                    // ("Vec[UInt8]") so method dispatch/indexing/&passing on it
                    // work (was untyped -> degraded to Str/i64 -> AV).
                    self.local.local_xiom_types.insert(name.name.clone(), px);
                } else if let Some(rt) = self.infer_call_return_xiom(value) {
                    // BUG 52: `var m = make_map()` -- a local bound to a fn
                    // call keeps the callee's declared return type ("Map[Str,
                    // MyVal]") so generic METHOD calls on it can infer type
                    // args (m.get("b") must mono Map.get[Str, MyVal], not
                    // [Str, Str]).
                    self.local.local_xiom_types.insert(name.name.clone(), rt);
                }
                // round-13 (tuple payloads): derive the Vec ELEMENT type from
                // the recorded XIOM type ("Vec[(Int, Int)]" -> "(Int, Int)") so
                // resolve_vec_elem_type finds it for METHOD-CHAIN returns
                // (iter.range(...).enumerate().collect() -- callee_return_xiom
                // can't resolve the chained receiver, so the elem lookup failed
                // and get() loaded the first 8 bytes of the 16-byte tuple slot).
                if let Some(xiom_ty) = self.local.local_xiom_types.get(&name.name) {
                    if let Some(elem) = xiom_ty.strip_prefix("Vec[").and_then(|r| r.strip_suffix(']')) {
                        self.local.local_vec_elem.entry(name.name.clone()).or_insert_with(|| elem.to_string());
                    }
                }
                // BUG 44: `var p = &s` / `var p: &Str = ...` -- track ref-locals
                // (address-carrying) so deref and &T->T coercion load through.
                self.track_ref_local(&name.name, _ty.as_deref(), value);
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
                    // This ensures Int8->i8, Int16->i16, Int32->i32, Float32->float.
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
                    // BUG 22 #6: loop-body binding allocas hoist to fn entry.
                    if self.local.loop_depth > 0 {
                        self.local.hoisted_allocas.push((alloca.clone(), "i64".to_string()));
                    } else {
                        self.emitln(&format!("  {alloca} = alloca i64"));
                    }
                    self.emitln(&format!("  store i64 0, i64* {alloca}"));
                    self.add_local(&name.name, alloca, "i64");
                    return Ok(());
                }
                let store_val = self.coerce_value(&val, &val_llvm_ty, &llvm_ty);
                let store_val = self.zero_val_for(&store_val, &llvm_ty);
                let alloca = self.fresh_tmp();
                // D1: i128/fp128 allocas need 16-byte alignment on x86-64.
                // BUG 22 #6: loop-body binding allocas hoist to fn entry.
                if self.local.loop_depth > 0 {
                    self.local.hoisted_allocas.push((alloca.clone(), llvm_ty.clone()));
                } else {
                    self.emitln(&format!("  {alloca} = alloca {llvm_ty}{}", self.alloca_align(&llvm_ty)));
                }
                self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {alloca}{}", self.store_align(&llvm_ty)));
                self.add_local(&name.name, alloca, &llvm_ty);
                // Check invariants if the value is a struct with invariants
                if self.config.check_contracts {
                    self.maybe_check_value_invariants(value, &val);
                }
            }
            Stmt::Var(name, _ty, value, _) => {
                // M20-A1: Track closure bindings (also through parens)
                // B-007: plus locals bound from fn-typed container ELEMENTS
                // (`var t = tw.tasks[i]; t();` -- Vec[fn()] holds closure envs).
                if matches!(value, Expr::PipeClosure(..) | Expr::Closure(..))
                    || matches!(value, Expr::Paren(inner, _) if matches!(inner.as_ref(), Expr::Closure(..) | Expr::PipeClosure(..)))
                    || matches!(value, Expr::Index(container, _, _) if {
                        self.resolve_vec_container_elem_xiom(container)
                            .map_or(false, |x| x.starts_with("fn("))
                    })
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
                        // round-15: record the XIOM element NAME (As-target
                        // aware) so generic-arg inference keeps signedness
                        // (`[200 as UInt8, ...]` -> "UInt8", not "Int8").
                        if let Some(elem_xiom) = elems.first().and_then(|e| self.infer_scalar_elem_xiom(e)) {
                            self.local.local_array_elem_xiom.insert(name.name.clone(), elem_xiom);
                        }
                        // 5c.30: record array size for const-generic inference
                        self.local.local_array_sizes.insert(name.name.clone(), elems.len() as i64);
                        // M33: track Vec element type for array-literal-to-Vec conversion
                        if let Some(elem_xiom) = elems.first()
                            .and_then(|e| self.infer_struct_type_name(e))
                        {
                            self.local.local_vec_elem.insert(name.name.clone(), elem_xiom);
                        }
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
                        // R7: see the let-binding arm above (m.values -> JsonValue).
                        Expr::Field(..) => self.resolve_vec_elem_xiom(value),
                        Expr::Call(func, args, _) | Expr::GenericCall(func, _, args, _) => {
                            // BUG 23 #1 fix: inherit from the callee's DECLARED
                            // return type as well as from the first argument --
                            // `var v = module.mk_vecf()` (catalog fn returning
                            // Vec[Float64]) must register "Float64", otherwise
                            // v[i] element reads degrade to the elem_size switch
                            // (raw bit-pattern garbage for float elements). The
                            // let-path had this fallback; the var-path did not.
                            let from_arg = args.first().and_then(|a| {
                                if let Expr::Ident(id) = a {
                                    self.local.local_vec_elem.get(&id.name).cloned()
                                } else { None }
                            });
                            from_arg.or_else(|| {
                                self.callee_return_xiom(func).and_then(|rt| {
                                    rt.strip_prefix("Vec[").and_then(|rest| rest.strip_suffix(']')).map(|e| e.to_string())
                                })
                            })
                        }
                        // M33: Array literal bound to Var -- keep the element
                        // type that was inferred above (from first struct element).
                        // The `remove` below would otherwise clear it.
                        Expr::Array(..) => self.local.local_vec_elem.get(&name.name).cloned(),
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
                // 5c.30: Empty array `[]` assigned to Vec-typed variable -- emit
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
                            // R39: deterministic same-leaf pick (see the let
                            // path above).
                            let suffix = format!(".{}", elem);
                            let candidates: Vec<String> = self.types.types.keys().into_iter()
                                .filter(|k| k.ends_with(&suffix) || k.as_str() == elem)
                                .collect();
                            let sname = self.pick_deterministic(candidates)
                                .unwrap_or_else(|| elem.to_string());
                            Some(self.vec_elem_storage_size(&sname))
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
                    let _loaded = self.emit_vec_load_fields(&struct_alloca);
                    self.add_local(&name.name, struct_alloca, &"%struct.Vec".to_string());
                    // Use the declared element type if available, not hardcoded Int
                    let elem_ty = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .unwrap_or_else(|| "Int".to_string());
                    self.local.local_vec_elem.insert(name.name.clone(), elem_ty);
                    return Ok(());
                }
                let declared_llvm_ty: Option<String> = _ty.as_ref().map(|t| {
                    // M65 R7 (2026-09-10): an ANNOTATED local slot must use the
                    // CONCRETE container type -- `var found: Option[JsonValue];`
                    // allocated %struct.Option (opaque i64 payload) while the
                    // assignments/match built Option__JsonValue, so the store
                    // was an invalid-IR struct mismatch (smoke_stress_serialize_
                    // large_json clang failure). concrete_type_for keeps scalars
                    // and non-container types unchanged.
                    let name = self.concrete_type_for(t);
                    self.llvm_type_for(&name).unwrap_or_else(|_| LLVM_I64.to_string())
                });
                // M17: Track XIOM type and signedness for narrow-int widening.
                if let Some(ty) = _ty {
                    // R49: preserve generic args for user named generics
                    // ("PriorityQueue[Task]") -- the receiver-type-arg
                    // inference needs them (L6-28).
                    let xiom_name = Self::type_annotation_name(ty);
                    self.local.local_xiom_types.insert(name.name.clone(), xiom_name.clone());
                    if Self::is_signed_xiom_type(&xiom_name) {
                        self.local.signed_locals.insert(name.name.clone());
                    } else {
                        self.local.signed_locals.remove(&name.name);
                    }
                } else if let Some(inferred) = Self::infer_value_xiom_type(value) {
                    // BUG 14: `var big = x as UInt128` -- infer signedness from
                    // the cast target when there is no annotation.
                    // round-14c: ALSO track signed_locals (see the let arm).
                    self.local.local_xiom_types.insert(name.name.clone(), inferred.clone());
                    if Self::is_signed_xiom_type(&inferred) {
                        self.local.signed_locals.insert(name.name.clone());
                    } else {
                        self.local.signed_locals.remove(&name.name);
                    }
                } else if let Some(ix) = self.infer_if_xiom_type(value) {
                    // gzip fix (2026-08-19): `let v = if c { f() } else { g() };`
                    // -- the binding keeps the arm tail's XIOM type ("Vec[UInt8]").
                    self.local.local_xiom_types.insert(name.name.clone(), ix);
                } else if let Some(tx) = self.infer_try_xiom_type(value) {
                    // Round-6 fix (2026-08-19): `let name = f()?;` -- the binding
                    // keeps the Option/Result payload XIOM type ("Str") so
                    // method dispatch on it works.
                    self.local.local_xiom_types.insert(name.name.clone(), tx);
                } else if let Some(px) = self.infer_field_payload_xiom(value) {
                    // gzip-DECOMPRESS fix (2026-08-19): `let v = r.value;` --
                    // the binding keeps the Option/Result payload type
                    // ("Vec[UInt8]") so method dispatch/indexing/&passing on it
                    // work (was untyped -> degraded to Str/i64 -> AV).
                    self.local.local_xiom_types.insert(name.name.clone(), px);
                } else if let Some(rt) = self.infer_call_return_xiom(value) {
                    // BUG 52: `var m = make_map()` -- a local bound to a fn
                    // call keeps the callee's declared return type ("Map[Str,
                    // MyVal]") so generic METHOD calls on it can infer type
                    // args (m.get("b") must mono Map.get[Str, MyVal], not
                    // [Str, Str]).
                    self.local.local_xiom_types.insert(name.name.clone(), rt);
                }
                // round-13 (tuple payloads): derive the Vec ELEMENT type from
                // the recorded XIOM type ("Vec[(Int, Int)]" -> "(Int, Int)") so
                // resolve_vec_elem_type finds it for METHOD-CHAIN returns
                // (iter.range(...).enumerate().collect() -- callee_return_xiom
                // can't resolve the chained receiver, so the elem lookup failed
                // and get() loaded the first 8 bytes of the 16-byte tuple slot).
                if let Some(xiom_ty) = self.local.local_xiom_types.get(&name.name) {
                    if let Some(elem) = xiom_ty.strip_prefix("Vec[").and_then(|r| r.strip_suffix(']')) {
                        self.local.local_vec_elem.entry(name.name.clone()).or_insert_with(|| elem.to_string());
                    }
                }
                // BUG 44: `var p = &s` / `var p: &Str = ...` -- track ref-locals
                // (address-carrying) so deref and &T->T coercion load through.
                self.track_ref_local(&name.name, _ty.as_deref(), value);
                let (val, val_llvm_ty) = if let Expr::Array(elems, _) = value {
                    // BUG 53 (2026-08-18): FIXED-ARRAY annotated bindings
                    // (`var a: [5]Int = [10, ...]`) store elements DIRECTLY
                    // into the [N x T] slot. The old path converted the
                    // literal to a %struct.Vec and coerced the Vec VALUE to
                    // the declared [N x T] type -- coerce_value extracted
                    // field 0 (the data POINTER) and stored it as the array
                    // (invalid IR: `store [5 x i64] %data_ptr`; reads then
                    // returned garbage). Register the local and return.
                    let is_fixed_arr = declared_llvm_ty.as_deref()
                        .map_or(false, |d| d.starts_with('[') && d.contains(" x "));
                    if is_fixed_arr {
                        let arr_ty = declared_llvm_ty.clone().unwrap();
                        let elem_llvm = Self::extract_array_elem_ty(&arr_ty);
                        let slot = self.fresh_tmp();
                        if self.local.loop_depth > 0 {
                            self.local.hoisted_allocas.push((slot.clone(), arr_ty.clone()));
                        } else {
                            self.emitln(&format!("  {slot} = alloca {arr_ty}"));
                        }
                        for (i, e) in elems.iter().enumerate() {
                            let (v, vt) = self.compile_expr(e)?;
                            let cv = self.coerce_value(&v, &vt, &elem_llvm);
                            let gep = self.fresh_tmp();
                            self.emitln(&format!("  {gep} = getelementptr {arr_ty}, {arr_ty}* {slot}, i64 0, i64 {i}"));
                            self.emitln(&format!("  store {elem_llvm} {cv}, {elem_llvm}* {gep}"));
                        }
                        self.add_local(&name.name, slot, &arr_ty);
                        self.local.array_locals.insert(name.name.clone());
                        return Ok(());
                    }
                    // 5c.39: Non-empty array literal assigned to a Vec-typed
                    // 5c.39: Non-empty array literal assigned to a Vec-typed
                    // variable -- convert to Vec via compile_array_as_vec.
                    // M33: Infer element type from first element for struct
                    // arrays when no type annotation is present. Defaults to
                    // "Int" for scalar elements. This ensures struct elements
                    // are stored inline (correct elem_size) rather than boxed
                    // as i64 pointers on the heap.
                    let elem_ty = _ty.as_ref()
                        .and_then(|t| Self::vec_elem_from_type_annotation(t))
                        .or_else(|| {
                            elems.first().and_then(|e| self.infer_struct_type_name(e))
                        })
                        .or_else(|| {
                            // round-15 (narrow VAR array literals): infer the
                            // SCALAR element type from the first element --
                            // `var arr8 = [1 as Int8, 2, 3]` must build a
                            // 1-byte-element Vec. The old "Int" default made
                            // 8-byte slots while the mono'd &[N]T body reads
                            // data[idx] as i8 -- every element except 0 read
                            // garbage (probe_arr8: first returned 0). The
                            // As-aware helper keeps signedness ("UInt8").
                            elems.first().and_then(|e| self.infer_scalar_elem_xiom(e))
                        })
                        .unwrap_or_else(|| "Int".to_string());
                    self.compile_array_as_vec(elems, &elem_ty)?
                } else {
                    self.compile_expr(value)?
                };
                let orig_val_ty = val_llvm_ty.clone();
                // M17: Use declared type for alloca width when present, falling back
                // to value type. Special cases preserved for zero-init and float->double.
                let llvm_ty = if val_llvm_ty == "i64" && val == "0" {
                    declared_llvm_ty.clone().unwrap_or(val_llvm_ty)
                } else if val_llvm_ty == "void" || val.is_empty() {
                    declared_llvm_ty.clone().unwrap_or_else(|| LLVM_I64.to_string())
                } else if let Some(ref d) = declared_llvm_ty {
                    // M17: When a type annotation exists, prefer the declared type
                    // for the alloca width. This ensures Int8->i8, Int16->i16, etc.
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
                    // BUG 22 #6: loop-body binding allocas hoist to fn entry.
                    if self.local.loop_depth > 0 {
                        self.local.hoisted_allocas.push((alloca.clone(), "i64".to_string()));
                    } else {
                        self.emitln(&format!("  {alloca} = alloca i64"));
                    }
                    self.emitln(&format!("  store i64 0, i64* {alloca}"));
                    self.add_local(&name.name, alloca, "i64");
                    return Ok(());
                }
                let store_val = self.zero_val_for(&val, &llvm_ty);
                let alloca = self.fresh_tmp();
                // D1: i128/fp128 allocas need 16-byte alignment on x86-64.
                // BUG 22 #6: loop-body binding allocas hoist to fn entry.
                if self.local.loop_depth > 0 {
                    self.local.hoisted_allocas.push((alloca.clone(), llvm_ty.clone()));
                } else {
                    self.emitln(&format!("  {alloca} = alloca {llvm_ty}{}", self.alloca_align(&llvm_ty)));
                }
                self.emitln(&format!("  store {llvm_ty} {store_val}, {llvm_ty}* {alloca}{}", self.store_align(&llvm_ty)));
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
                        // R19: strip exactly ONE star. `trim_end_matches('*')`
                        // mapped `i8**` (`*mut Str`) to `i8`, so the store of a
                        // Str value emitted `store i8 %ptr, i8**` (clang
                        // ptr/i8 mismatch in ptr.replace_Str). Same class as
                        // BUG 44 in the deref-load path.
                        let pointee = ptr_ty.strip_suffix('*').unwrap_or(&ptr_ty).to_string();
                        let (val, val_ty) = self.compile_expr(value)?;
                        let store_val = self.coerce_value(&val, &val_ty, &pointee);
                        self.emitln(&format!("  store {pointee} {store_val}, {ptr_ty} {ptr_val}"));
                        return Ok(());
                    }
                    // 5c.31: Legacy erased-to-i64 path -- the pointer value is
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
                // {i8*, i64, i64}) -- write an i64-wide slot at data[idx]. Str is
                // immutable at the ABI, so only Vec/Slice are handled.
                if let Expr::Index(container, index, _) = place {
                    let (cont_val, cont_ty) = self.compile_expr(container)?;
                    let (vec_val, vec_ty) = self.resolve_vec_receiver(container, &cont_val, &cont_ty);
let is_vec = Self::is_llvm_struct_named(&vec_ty, "Vec")
|| Self::is_llvm_struct_named(&vec_ty, "Slice");
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
                        // BUG 52 (2026-08-18): STRUCT/ENUM elements must be
                        // memcpy'd INLINE into the slot -- the old val_to_i64
                        // path stored a BOXED POINTER as the first 8 bytes and
                        // emit_elem_store wrote only that i64, so a 16-byte
                        // enum element was (a) truncated to the boxed handle
                        // and (b) read back as 16 bytes from the handle +
                        // adjacent slot bytes (garbage payload -> AV in
                        // Map.insert's duplicate-key update `values[i] = v`).
                        let mut stored_struct = false;
                        if val_ty.starts_with('%') {
                            if let Some(elem_name) = self.resolve_vec_elem_type(container) {
                                let struct_ty = if elem_name.starts_with("Vec[") {
                                    "%struct.Vec".to_string()
                                } else {
                                    format!("%struct.{elem_name}")
                                };
                                if struct_ty == val_ty {
                                    let tmp = self.fresh_tmp();
                                    self.emitln(&format!("  {tmp} = alloca {struct_ty}"));
                                    self.emitln(&format!("  store {struct_ty} {val}, {struct_ty}* {tmp}"));
                                    let tmp_i8 = self.fresh_tmp();
                                    self.emitln(&format!("  {tmp_i8} = bitcast {struct_ty}* {tmp} to i8*"));
                                    self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {elem_ptr}, i8* {tmp_i8}, i64 {esz_val}, i1 false)"));
                                    stored_struct = true;
                                }
                            }
                        }
                        if !stored_struct {
                            self.emit_elem_store(&store_i64, &elem_ptr, &esz_val);
                        }
                    } else if cont_ty.starts_with('[') && cont_ty.contains(" x ") {
                        let (idx_raw, idx_ty) = self.compile_expr(index)?;
                        let idx = self.val_to_i64(&idx_raw, &idx_ty);
                        // Use the existing local alloca when the container is an
                        // Ident -- avoids fresh alloca/load/store on every write.
                        let mut is_ident = false;
                        let mut arr_ptr = String::new();
                        let mut arr_ptr_ty = String::new();
                        if let Expr::Ident(id) = &**container {
                            if let Some((slot, _slot_ty)) = self.lookup_local(&id.name).cloned() {
                                (arr_ptr, arr_ptr_ty) = (slot, format!("{cont_ty}*"));
                                is_ident = true;
                            } else if let Some((symbol, _gty)) = self.local.module_globals.get(&id.name).cloned() {
                                // Stdlib finding 3b-2 #8 (module-level arrays):
                                // write THROUGH the real global. The old path
                                // wrote into a stack copy of the loaded value and
                                // the store-back was dropped -- array writes were
                                // silently lost (crc tables stayed all zeros).
                                (arr_ptr, arr_ptr_ty) = (format!("@{symbol}"), format!("{cont_ty}*"));
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
                    } else if cont_ty.ends_with('*') && cont_ty != "i8*" {
                        // BUG 53 write-facet (2026-08-18): `&mut [N]T` params
                        // lower to the ELEMENT pointer (i64*/double*/... -- the
                        // mono subst + the caller's array-local coercion). The
                        // old code had no branch for these -- the write was
                        // DROPPED (array.sort/sort_by silently no-oped, and
                        // `set_first(&mut a, 99)` left the array unchanged).
                        let (idx_raw, idx_ty) = self.compile_expr(index)?;
                        let idx = self.val_to_i64(&idx_raw, &idx_ty);
                        let inner_ty = cont_ty.trim_end_matches('*');
                        let elem_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {elem_ptr} = getelementptr {inner_ty}, {cont_ty} {cont_val}, i64 {idx}"));
                        let store_val = self.coerce_value(&val, &val_ty, inner_ty);
                        self.emitln(&format!("  store {inner_ty} {store_val}, {inner_ty}* {elem_ptr}"));
                    } else if cont_ty == "i8*" {
                        // Raw byte-buffer store: `buf[i] = v` where `buf: *UInt8`.
                        // The element is one byte; truncate the value to i8. Without
                        // this, `buf[i] = ...` silently emitted nothing (the store was
                        // dropped), leaving heap buffers uninitialized -> crashes in
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
                        if let Some(field_names) = self.types.types.get(&type_name.to_string())
                            .or_else(|| {
                                let suffix = format!(".{type_name}");
                                self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                    .and_then(|k|self.types.types.get(&k))
                            })
                            
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
                                        self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix))
                                            .and_then(|k|self.types.types.get(&k))
                                    })
                                    
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
                if self.fctx.is_never_return {
                    // P2-4: Never-returning functions -- compile the expression
                    // (which may itself be a !-returning call), then unreachable.
                    if let Some(e) = expr {
                        let _ = self.compile_expr(e)?;
                    }
                    self.emitln("  unreachable");
                } else if let Some(e) = expr {
                    // Value sink: use the value's real LLVM type from compile_expr.
                    let (mut val, val_ty) = self.compile_expr(e)?;
                    let ret_ty = self.fctx.current_return_type.clone();
                    // D2.1 (Unsafe Confinement Phase 3, requirement i -- Copy-Out):
                    // a `return` INSIDE an unsafe block returns a value whose heap
                    // payload lives on the guard arena. It must be COPIED to the
                    // main heap BEFORE the arena resets at block exit, or the
                    // caller's Str/Vec would dangle (use-after-free).
                    // Uses xiom_guard_copy_str (single C call) -- NOT inline
                    // strlen -- which would leak the recursion counter in the
                    // confined block (alwaysinline imbalance, 500-depth trap).
                    // D2.1 (Phase 3/4): a `return` INSIDE an unsafe block must
                    // (a) Copy-Out a Str tail to the main heap before the arena
                    // resets (UAF fix), and (b) ALWAYS discard the guard arena
                    // + disarm the stack guard page -- the block-exit emission
                    // after the tail loop is skipped for early returns.
                    if self.guard_heap_depth > 0 {
                        if val_ty == LLVM_STR_PTR {
                            let copy_tmp = self.fresh_tmp();
                            self.emitln(&format!("  {copy_tmp} = call i8* @xiom_guard_copy_str(i8* {val})"));
                            let not_null = self.fresh_tmp();
                            self.emitln(&format!("  {not_null} = icmp ne i8* {copy_tmp}, null"));
                            let sel = self.fresh_tmp();
                            self.emitln(&format!("  {sel} = select i1 {not_null}, i8* {copy_tmp}, i8* {val}"));
                            val = sel;
                        }
                        // D2.1 (Phase 5): inside an unsafe-block fn, a `return`
                        // must signal the call site to return from the ENCLOSING
                        // fn (not just from the block fn), so the block value is
                        // propagated out.
                        if self.in_unsafe_block_fn {
                            self.emitln("  call void @xiom_trampoline_set_returned()");
                        }
                        self.emitln("  call void @xiom_guard_heap_exit()");
                        self.emitln("  call void @xiom_guard_page_disarm()");
                        self.emitln("  call void @xiom_trap_leave()");
                    }
                    // Coerce the returned value to the function's declared return
                    // type (int widths, int<->pointer, int<->double, int->struct)
                    // so the `ret` instruction is well-typed.
                    if self.in_unsafe_block_fn {
                        // D2.1 (block-fn ABI): `fctx.current_return_type` is the
                        // block fn's i64 (see the Unsafe expression lowering), so
                        // the generic coerce would WRONGLY extract a scalar field
                        // from a struct tail (e.g. `return Some(v - 1)` returned
                        // Option's field-1 VALUE as i64). The trampoline's
                        // round-trip contract is val_to_i64 on the EXPRESSION's
                        // type: STRUCT values are stored to a heap slot and the
                        // ADDRESS flows back, and unsafe_result_i64_to_val
                        // re-materializes with the enclosing fn's return type.
                        // (The guard arena is already exited above, so the slot
                        // lives on the main heap.)
                        val = self.val_to_i64(&val, &val_ty);
                    } else {
                        val = self.coerce_value(&val, &val_ty, &ret_ty);
                    }
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
                    // P0-2: Emit deferred cleanups before return
                    self.compile_deferred_cleanups()?;
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
                    // P0-2: Emit deferred cleanups before return
                    self.compile_deferred_cleanups()?;
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
                    // No else case for simple if -- the else block is just a merge jump
                    self.emitln(&format!("\n{prev_label}:"));
                    self.emitln(&format!("  br label %{merge_label}"));
                    merge_reachable = true;
                } else if elifs.is_empty() {
                    // prev_label == merge_label --  skip redundant label emission
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
                    } else if ty == "double" || ty == "float" {
                        // R48 (playground C17): `store double 0` is invalid IR.
                        self.emitln(&format!("  store {ty} 0.0, {ty}* {ptr}"));
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
                        let cands: Vec<String> = self.types.enum_variants.entries().into_iter()
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
                // round-13 (cb2 residual): a call-scrutinee with an UNKNOWN
                // resolved type but a CONCRETE struct VALUE -- e.g. a closure
                // local's call returning %struct.Ordering
                // (`match compare(&a, &b) { Less => a; ... }` in cmp.min_by,
                // where `compare` is a fn-typed param, not a registered fn),
                // or an fn-typed FIELD call returning %struct.Option
                // (`match self.next_fn() { Some(v) => ... }` in the iter
                // adapters). Adopt the struct as the scrutinee type so the
                // discriminant checks emit -- otherwise the match branched
                // STRAIGHT to the last arm (min_by always returned b;
                // MapIter.next applied f to the None case).
                if scrutinee_type.is_none()
                    && scrutinee_llvm_ty.starts_with("%struct.")
                    && !scrutinee_llvm_ty.ends_with('*')
                {
                    let name = scrutinee_llvm_ty[8..].trim_end_matches('*').to_string();
                    // Registered enums, Option/Result (field 0 = discriminant /
                    // is_some/is_ok) and any other registered struct type.
                    let is_registered = self.types.types.contains_key(&name)
                        || self.types.types.keys().into_iter().any(|k| k.ends_with(&format!(".{name}")));
                    if is_registered {
                        scrutinee_type = Some(name);
                    }
                }

                // Store scrutinee value in alloca for field extraction
                let mut scrutinee_alloca_info = None;
                if let Some(ref type_name) = scrutinee_type {
                    // Use the concrete LLVM type (e.g. %struct.Result__X__Y) when
                    // available, falling back to %struct.{type_name} for generic types.
                    // This prevents type mismatches when matching on concretized
                    // enum/struct types like Result[JsonValue, SerializeError].
                    let struct_ty = if scrutinee_llvm_ty.starts_with("%struct.") && !scrutinee_llvm_ty.ends_with('*') && scrutinee_llvm_ty != "i64" {
                        scrutinee_llvm_ty.clone()
                    } else if scrutinee_llvm_ty.starts_with("%struct.") && scrutinee_llvm_ty.ends_with('*') {
                        // Pointer-to-struct scrutinee (e.g. %struct.JsonValue*):
                        // deref for the match alloca -- the discriminant GEP must
                        // index the STRUCT, not the pointer.
                        scrutinee_llvm_ty.trim_end_matches('*').to_string()
                    } else {
                        format!("%struct.{type_name}")
                    };
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
                // bug degrades to a branch-to-merge instead of a panic -- a
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
                        // Safety fallback -- unreachable once the build/emit
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
                                            self.types.enum_variants.get(&type_name.to_string())
                                                .and_then(|vars| vars.into_iter().find(|(v, _)| v == leaf || v == &vn.name))
                                                    .and_then(|(_, vfs)| vfs.first().cloned())
                                                .and_then(|canonical| {
                                                    self.types.types.get(&type_name.to_string())
                                                    .and_then(|fns| fns.iter().position(|f| f == &canonical))
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
                                    Pattern::Lit(Literal::Float(f, _)) => {
                                        let fval = if scrutinee_llvm_ty == "double" || scrutinee_llvm_ty == "float" {
                                            val.clone()
                                        } else {
                                            let conv = self.fresh_tmp();
                                            self.emitln(&format!("  {conv} = sitofp i64 {val} to double"));
                                            conv
                                        };
                                        let c = self.fresh_tmp();
                                        self.emitln(&format!("  {c} = fcmp oeq double {fval}, {f:.16e}"));
                                        self.emitln(&format!("  br i1 {c}, label %{arm_label}, label %{fail_block}"));
                                    }
                                    Pattern::Lit(Literal::Char(cv, _)) => {
                                        let c = self.fresh_tmp();
                                        self.emitln(&format!("  {c} = icmp eq i64 {val}, {}", *cv as i64));
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
                                            let variant_idx_opt = self.types.enum_variants.get(&type_name.to_string())
                                                .and_then(|vars| vars.iter().position(|(vn2, _)| vn2 == &leaf || vn2 == &vn.name));
                                            let canonical_opt = variant_idx_opt.and_then(|vi| {
                                                self.types.enum_variants.get(&type_name.to_string())
                                                    .and_then(|vars| vars.get(vi).cloned())
                                                .and_then(|(_, vfs)| vfs.first().cloned())
                                            });
                                               let fi_opt = canonical_opt.as_ref().and_then(|canonical| {
                                                self.types.types.get(&type_name.to_string())
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
                                    Pattern::Struct(..) | Pattern::Tuple(..) => {
                                        if let Some((_alloca, _type_name, _struct_ty)) = &scrutinee_alloca_info {
                                            self.emitln(&format!("  br label %{arm_label}"));
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
                        Pattern::Lit(Literal::Float(f, _)) => {
                            // P1-3: Float literal pattern -- use fcmp for double-precision comparison.
                            let fval = if scrutinee_llvm_ty == "double" || scrutinee_llvm_ty == "float" {
                                val.clone()
                            } else {
                                let conv = self.fresh_tmp();
                                self.emitln(&format!("  {conv} = sitofp i64 {val} to double"));
                                conv
                            };
                            let check = self.fresh_tmp();
                            self.emitln(&format!("  {check} = fcmp oeq double {fval}, {f:.16e}"));
                            self.emitln(&format!("  br i1 {check}, label %{arm_label}, label %{next}"));
                        }
                        Pattern::Lit(Literal::Char(c, _)) => {
                            let check = self.fresh_tmp();
                            self.emitln(&format!("  {check} = icmp eq i64 {val}, {}", *c as i64));
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
                        Pattern::Struct(name, _, _) => {
                            if let Some((_alloca, type_name, _struct_ty)) = &scrutinee_alloca_info {
                                if type_name == &name.name {
                                    self.emitln(&format!("  br label %{arm_label}"));
                                } else {
                                    self.emitln(&format!("  br label %{next}"));
                                }
                            } else {
                                self.emitln(&format!("  br label %{arm_label}"));
                            }
                        }
                        Pattern::Tuple(..) => {
                            if scrutinee_alloca_info.is_some() {
                                self.emitln(&format!("  br label %{arm_label}"));
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
                                self.types.enum_variants.get(&tn.to_string())
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
                                    let mut field_llvm_ty = self.field_llvm_type(type_name, field_idx as usize);
                                    let loaded = self.fresh_tmp();
                                    self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                    // BUG 22 #4 fix: Some(5.0)/Ok(2.5) store the
                                    // FLOAT BITS in the i64 payload slot -- bitcast
                                    // back when the payload's XIOM type is a float,
                                    // so match-bound vars carry real doubles/floats
                                    // (unary minus / arithmetic on them was garbage).
                                    let scrutinee_name = if let Expr::Ident(sid) = expr_match { Some(sid.name.clone()) } else { None };
                                    let scrutinee_payload = self.scrutinee_payload_xiom(expr_match, field_idx);
                                    // R23: a FN-MARKER payload (`Some(task)` off
                                    // `Vec[fn()].pop()`) holds closure ENV bits.
                                    // Mark it so `task()` takes the env-first
                                    // closure path -- the raw fn-pointer path
                                    // inttoptr'd the env box as code (0xC0000005).
                                    // The marker can come from the scrutinee's
                                    // recorded type or, for `vec.pop()`, from the
                                    // Vec's element marker.
                                    let payload_marker = scrutinee_payload.clone().or_else(|| {
                                        if field_idx != 1 { return None; }
                                        let Expr::Call(callee, _, _) = expr_match else { return None };
                                        let Expr::Field(obj, mname, _) = callee.as_ref() else { return None };
                                        if mname.name != "pop" { return None; }
                                        self.resolve_vec_container_elem_xiom(obj.as_ref())
                                    });
                                    if matches!(&payload_marker, Some(px) if px.starts_with("fn(")) {
                                        self.local.closure_locals.insert(ident.name.clone());
                                        if let Some(ret_str) = payload_marker.as_ref()
                                            .and_then(|px| px.rsplit_once(") -> ").map(|(_, r)| r.trim().to_string()))
                                        {
                                            self.local.fn_local_returns.insert(ident.name.clone(), ret_str);
                                        }
                                    }
                                    if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() {
                                        eprintln!("[matchpay] scrutinee={scrutinee_name:?} field_llvm_ty={field_llvm_ty} payload_xiom={:?}",
                                            scrutinee_name.as_ref().and_then(|n| self.local.local_opt_payload_xiom.get(n)));
                                    }
                                    let mut bind_val = loaded.clone();
                                    if field_llvm_ty == "i64" {
                                        if let Some(px) = &scrutinee_payload {
                                            if px == "Float64" {
                                                let bc = self.fresh_tmp();
                                                self.emitln(&format!("  {bc} = bitcast i64 {loaded} to double"));
                                                bind_val = bc;
                                                field_llvm_ty = "double".to_string();
                                            } else if px == "Float32" {
                                                let t32 = self.fresh_tmp();
                                                self.emitln(&format!("  {t32} = trunc i64 {loaded} to i32"));
                                                let bc = self.fresh_tmp();
                                                self.emitln(&format!("  {bc} = bitcast i32 {t32} to float"));
                                                bind_val = bc;
                                                field_llvm_ty = "float".to_string();
                                            }
                                        }
                                    }
                                    // BUG 25 #10 (crypto AES-NI follow-up): when the
                                    // payload is a Vec container stored as an i64
                                    // heap HANDLE in the Option/Result slot, bind a
                                    // REAL %struct.Vec local (inttoptr + load) instead
                                    // of the raw handle. A handle-typed binding made
                                    // `&v` (Ref arm) take the ADDRESS OF THE HANDLE
                                    // SLOT, so passing the match-bound payload to a
                                    // `&Vec[T]` param read garbage (len=1 vs 2) and
                                    // crypto's aes_decrypt(&key, &ciphertext) crashed.
                                    let payload_xiom = self.field_xiom_type(type_name, field_idx as usize)
                                        // round-8 (rw1): builtin Option/Result have no
                                        // field_xiom_type -- fall back to the
                                        // SCRUTINEE's declared payload type
                                        // ("Option[&Str]" -> "&Str") so reference
                                        // payloads keep the & and auto-deref on use.
                                        .or_else(|| scrutinee_payload.clone());
                                    if field_llvm_ty == "i64" {
                                        if let Some(pt) = &payload_xiom {
                                            if pt.starts_with("Vec[") || pt.contains(".Vec") || pt.ends_with("]Vec") {
                                                let vp = self.fresh_tmp();
                                                self.emitln(&format!("  {vp} = inttoptr i64 {bind_val} to %struct.Vec*"));
                                                let vl = self.fresh_tmp();
                                                self.emitln(&format!("  {vl} = load volatile %struct.Vec, %struct.Vec* {vp}"));
                                                bind_val = vl;
                                                field_llvm_ty = "%struct.Vec".to_string();
                                            } else if let Some(agg) = self.registered_struct_llvm_for(pt)
                                                .or_else(|| self.boxed_aggregate_llvm_for(pt))
                                            {
                                                // R49 (playground C17 residue): a
                                                // STRUCT/ENUM payload in the erased
                                                // i64 slot is a heap BOX pointer
                                                // (Vec.get on struct elements). The
                                                // old binding used the raw pointer ->
                                                // `v.id` read ticket-garbage and
                                                // `s.name` a bogus Str (L5-34 "Vec:
                                                // not found"). Deref the box.
                                                let vp = self.fresh_tmp();
                                                self.emitln(&format!("  {vp} = inttoptr i64 {bind_val} to {agg}*"));
                                                let vl = self.fresh_tmp();
                                                self.emitln(&format!("  {vl} = load {agg}, {agg}* {vp}"));
                                                bind_val = vl;
                                                field_llvm_ty = agg;
                                            }
                                        }
                                    }
                                    let inner_alloca = self.fresh_tmp();
                                    self.emitln(&format!("  {inner_alloca} = alloca {field_llvm_ty}"));
                                    self.emitln(&format!("  store {field_llvm_ty} {bind_val}, {field_llvm_ty}* {inner_alloca}"));
                                    self.add_local(&ident.name, inner_alloca, &field_llvm_ty);
                                    // Track XIOM type for method dispatch (e.g. Str.len())
                                    if let Some(xiom_ty) = payload_xiom {
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
                                    let variant_info = self.types.enum_variants.get(&type_name.to_string())
                                        .and_then(|vars| vars.into_iter().find(|(vn, _)| vn == &leaf_variant || vn == &variant_name_clone))
                                        .map(|(_, vfs)| vfs.clone());
                                    let field_name_map = self.types.types.get(&type_name.to_string());
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
                        // Pre-extract struct destructure fields for guard access
                        if let Pattern::Struct(_, fields, _) = &arm.pattern {
                            if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                                let field_names_opt = self.types.types.get(&type_name.to_string());
                                if let Some(field_names) = field_names_opt {
                                    for (field_name, field_pat) in fields {
                                        if let Some(field_idx) = field_names.iter().position(|f| f == &field_name.name) {
                                            let gep = self.fresh_tmp();
                                            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {field_idx}"));
                                            let loaded = self.fresh_tmp();
                                            let field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                                            self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                            let field_alloca = self.fresh_tmp();
                                            self.emitln(&format!("  {field_alloca} = alloca {field_llvm_ty}"));
                                            self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {field_alloca}"));
                                            if let Pattern::Ident(ident) = field_pat {
                                                self.add_local(&ident.name, field_alloca, &field_llvm_ty);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Pre-extract tuple elements for guard access
                        if let Pattern::Tuple(elements, _) = &arm.pattern {
                            if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                                for (i, elem) in elements.iter().enumerate() {
                                    let gep = self.fresh_tmp();
                                    self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
                                    let loaded = self.fresh_tmp();
                                    let field_llvm_ty = self.field_llvm_type(type_name, i);
                                    self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                    let field_alloca = self.fresh_tmp();
                                    self.emitln(&format!("  {field_alloca} = alloca {field_llvm_ty}"));
                                    self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {field_alloca}"));
                                    if let Pattern::Ident(ident) = elem {
                                        self.add_local(&ident.name, field_alloca, &field_llvm_ty);
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
                            let field_names_opt = self.types.types.get(&type_name.to_string());
                            if let Some(field_names) = field_names_opt {
                                let type_name_clone = type_name.clone();
                                let variants_opt = self.types.enum_variants.get(&type_name.to_string());
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
                                                self.types.enum_variant_field_types.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{type_name_clone}")))
                                                    .map(|(_, v)| v)
                                            })
                                            .and_then(|vft| vft.into_iter().find(|(vn, _)| vn == &leaf_variant || vn == &variant_ident.name))
                                            .and_then(|(_, ftypes)| ftypes.get(fi).cloned());
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
                                            // M19: Str payload stored in i64 slot via ptrtoint --
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
                                        // (Vec[T]) binds the i64 HANDLE -- record
                                        // it so `items.push(..)` / `items.len()`
                                        // dereference the boxed header and
                                        // mutations alias the original enum.
                                        self.local.local_vec_handle.remove(&field_ident.name);
                                        if let Some(fty) = payload_xiom_ty.as_deref() {
                                            if let Some(inner) = fty.strip_prefix("Vec[").and_then(|s| s.strip_suffix(']')) {
                                                self.local.local_vec_handle.insert(field_ident.name.clone(), inner.to_string());
                                            }
                                            // M65 Part 2 (json heap layer): record the
                                            // DECLARED payload type with its generic args
                                            // ("Map[Str, JsonValue]") so field reads that
                                            // need concrete args (`entries.values[i]`)
                                            // resolve through resolve_generic_field_vec_elem.
                                            // Vec payloads keep local_vec_handle only --
                                            // avoid double-typing the handle ABI.
                                            if !fty.starts_with("Vec[") {
                                                self.local.local_xiom_types.insert(field_ident.name.clone(), fty.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Struct destructure: extract fields from struct alloca
                    if let Pattern::Struct(_, fields, _) = &arm.pattern {
                        if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                            let field_names_opt = self.types.types.get(&type_name.to_string());
                            if let Some(field_names) = field_names_opt {
                                for (field_name, field_pat) in fields {
                                    if let Some(field_idx) = field_names.iter().position(|f| f == &field_name.name) {
                                        let gep = self.fresh_tmp();
                                        self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {field_idx}"));
                                        let loaded = self.fresh_tmp();
                                        let field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                                        self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                        let field_alloca = self.fresh_tmp();
                                        self.emitln(&format!("  {field_alloca} = alloca {field_llvm_ty}"));
                                        self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {field_alloca}"));
                                        if let Pattern::Ident(ident) = field_pat {
                                            self.add_local(&ident.name, field_alloca, &field_llvm_ty);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Tuple destructure: extract elements from tuple alloca
                    if let Pattern::Tuple(elements, _) = &arm.pattern {
                        if let Some((ref alloca, ref type_name, ref struct_ty)) = scrutinee_alloca_info {
                            for (i, elem) in elements.iter().enumerate() {
                                let gep = self.fresh_tmp();
                                self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
                                let loaded = self.fresh_tmp();
                                let field_llvm_ty = self.field_llvm_type(type_name, i);
                                self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                let field_alloca = self.fresh_tmp();
                                self.emitln(&format!("  {field_alloca} = alloca {field_llvm_ty}"));
                                self.emitln(&format!("  store {field_llvm_ty} {loaded}, {field_llvm_ty}* {field_alloca}"));
                                if let Pattern::Ident(ident) = elem {
                                    self.add_local(&ident.name, field_alloca, &field_llvm_ty);
                                }
                            }
                        }
                    }
                    // For Ident patterns, bind the matched value to the identifier
                    // (skip for enum variant names, which are handled by the check block)
                    if let Pattern::Ident(ident) = &arm.pattern {
                        let is_variant = scrutinee_type.as_ref().and_then(|tn| {
                            self.types.enum_variants.get(&tn.to_string())
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
                    // dispatch correctly (fixes match Ok(bytes) -> bytes.len()).
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
                                // (BUG 22 #4: scalar float payloads come from
                                // local_opt_payload_xiom -- `var o = Some(5.0)` --
                                // struct payloads from local_opt_payload).
                                // BUG 43: direct-call scrutinees resolve the
                                // callee's declared Option/Result return type.
                                let declared = self.scrutinee_payload_xiom(expr_match, val_field);

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
                                    Some("Float32") if field_ty == "i64" => {
                                        let t32 = self.fresh_tmp();
                                        self.emitln(&format!("  {t32} = trunc i64 {loaded} to i32"));
                                        let f = self.fresh_tmp();
                                        self.emitln(&format!("  {f} = bitcast i32 {t32} to float"));
                                        (f, "float".to_string())
                                    }
                                    // Vec payload: the i64 is a heap HANDLE to the
                                    // boxed Vec header (5c.28h). Bind a REAL
                                    // %struct.Vec local (inttoptr + load) instead
                                    // of the raw handle: handle-typed bindings made
                                    // `&v` (Ref arm) take the address OF THE HANDLE
                                    // SLOT, so passing a match-bound payload to a
                                    // `&Vec[T]` param read garbage (m37 crypto
                                    // roundtrip: len=1 vs 2, aes_decrypt 0xC0000005).
                                    Some(decl) if field_ty == "i64" && (decl.starts_with("Vec[") || decl.contains(".Vec")) => {
                                        let vp = self.fresh_tmp();
                                        self.emitln(&format!("  {vp} = inttoptr i64 {loaded} to %struct.Vec*"));
                                        let vl = self.fresh_tmp();
                                        self.emitln(&format!("  {vl} = load volatile %struct.Vec, %struct.Vec* {vp}"));
                                        (vl, "%struct.Vec".to_string())
                                    }
                                    // Struct payload: the i64 is a heap pointer to a
                                    // boxed struct (Option/Result/enum). Load the struct
                                    // so nested match dispatch works.
                                    Some(decl) if field_ty == "i64" && decl.starts_with('&') => {
                                        // round-8 (rw1): REFERENCE payloads
                                        // (Option<&T> -- rand.weighted_pick): the
                                        // payload is the T SLOT ADDRESS. Bind it
                                        // raw; the "&T" xiom record below makes
                                        // value uses auto-deref.
                                        (loaded.clone(), field_ty.clone())
                                    }
                                    Some(decl) if field_ty == "i64" && !decl.starts_with("Vec[") => {
                                        // round-13 (tuple payloads -- iter
                                        // enumerate/zip, btree first_entry):
                                        // normalize "(Int, Int)" -> "Tuple__Int__Int"
                                        // so the boxed-tuple deref resolves (the
                                        // raw "(Int, Int)" failed llvm_type_for and
                                        // the binding kept the box pointer -- Vec
                                        // pushes stored 8 bytes into 16-byte slots).
                                        let norm = Self::tuple_xiom_to_struct_name(decl);
                                        let s_ty = self.llvm_type_for(&norm)
                                            .or_else(|_| self.llvm_type_for(decl))
                                            .unwrap_or_else(|_| format!("%struct.{norm}"));
                                        if s_ty.starts_with('%') {
                                            let sptr = self.fresh_tmp();
                                            self.emitln(&format!("  {sptr} = inttoptr i64 {loaded} to {s_ty}*"));
                                            let sload = self.fresh_tmp();
                                            self.emitln(&format!("  {sload} = load volatile {s_ty}, {s_ty}* {sptr}"));
                                            // Track inner variable's struct type for subsequent matches
                                            self.local.local_boxed_struct.insert(ident.name.clone(), norm.clone());
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
                                // R23: FN-MARKER payload (`Vec[fn()].pop()` ->
                                // `Some(task)`): the i64 slot holds closure ENV
                                // bits. Mark the binding so `task()` takes the
                                // env-first closure path; the raw fn-pointer path
                                // inttoptr'd the env box as code (0xC0000005 in
                                // the async executor's reduced shapes).
                                let payload_marker = declared.clone().filter(|d| d.starts_with("fn(")).or_else(|| {
                                    if val_field != 1 { return None; }
                                    let Expr::Call(callee, _, _) = expr_match else { return None };
                                    let Expr::Field(obj, mname, _) = callee.as_ref() else { return None };
                                    if mname.name != "pop" { return None; }
                                    self.resolve_vec_container_elem_xiom(obj.as_ref())
                                });
                                if matches!(&payload_marker, Some(px) if px.starts_with("fn(")) {
                                    self.local.closure_locals.insert(ident.name.clone());
                                    if let Some(ret_str) = payload_marker.as_ref()
                                        .and_then(|px| px.rsplit_once(") -> ").map(|(_, r)| r.trim().to_string()))
                                    {
                                        self.local.fn_local_returns.insert(ident.name.clone(), ret_str);
                                    }
                                }
                                // Vec[T] payloads bound as i64 handles are registered
                                // for handle deref; a %struct.Vec binding needs no
                                // handle registration (all consumers use it directly).
                                self.local.local_vec_handle.remove(&ident.name);
                                if bind_ty_inner == "i64" {
                                    if let Some(decl_ty) = declared.as_deref() {
                                        if let Some(elem) = decl_ty.strip_prefix("Vec[").and_then(|s| s.strip_suffix(']')) {
                                            self.local.local_vec_handle.insert(ident.name.clone(), elem.to_string());
                                        }
                                    }
                                }
                                // round-8 (rw1): record REFERENCE payloads'
                                // declared XIOM type ("&Str"/"&Int") so value
                                // uses (strcmp/icmp) auto-deref the bound slot
                                // address instead of reading it as a plain T.
                                if let Some(decl_ty) = declared.as_deref() {
                                    if decl_ty.starts_with('&') {
                                        self.local.local_xiom_types.insert(ident.name.clone(), decl_ty.to_string());
                                    }
                                }
                            } else if let Pattern::Tuple(elements, _) = inner {
                                // round-13 (TUPLE payloads -- iter enumerate/zip,
                                // btree first_entry): `Some((a, b))` -- the payload
                                // slot is an i64 BOX POINTER to a heap
                                // %struct.Tuple__X__Y (val_to_i64 boxing). Deref the
                                // box and bind each tuple element. The old code
                                // skipped the payload binding for non-Ident inner
                                // patterns -- the elements read literal 0.
                                if field_ty == "i64" {
                                    if let Some(decl) = self.scrutinee_payload_xiom(expr_match, val_field) {
                                        let norm = Self::tuple_xiom_to_struct_name(&decl);
                                        let s_ty = self.llvm_type_for(&norm)
                                            .or_else(|_| self.llvm_type_for(&decl))
                                            .unwrap_or_else(|_| format!("%struct.{norm}"));
                                        if s_ty.starts_with('%') {
                                            let sptr = self.fresh_tmp();
                                            self.emitln(&format!("  {sptr} = inttoptr i64 {loaded} to {s_ty}*"));
                                            let sload = self.fresh_tmp();
                                            self.emitln(&format!("  {sload} = load volatile {s_ty}, {s_ty}* {sptr}"));
                                            let t_alloca = self.fresh_tmp();
                                            self.emitln(&format!("  {t_alloca} = alloca {s_ty}"));
                                            self.emitln(&format!("  store {s_ty} {sload}, {s_ty}* {t_alloca}"));
                                            for (ti, elem) in elements.iter().enumerate() {
                                                let gep = self.fresh_tmp();
                                                self.emitln(&format!("  {gep} = getelementptr {s_ty}, {s_ty}* {t_alloca}, i32 0, i32 {ti}"));
                                                let field_llvm_ty = self.field_llvm_type(&norm, ti);
                                                let eload = self.fresh_tmp();
                                                self.emitln(&format!("  {eload} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                                let e_alloca = self.fresh_tmp();
                                                self.emitln(&format!("  {e_alloca} = alloca {field_llvm_ty}"));
                                                self.emitln(&format!("  store {field_llvm_ty} {eload}, {field_llvm_ty}* {e_alloca}"));
                                                if let Pattern::Ident(ident) = elem {
                                                    self.add_local(&ident.name, e_alloca, &field_llvm_ty);
                                                }
                                            }
                                        }
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
            Stmt::While(cond, body, _, _, label) => {
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
                // P0-3 fix: the labeled-loop field (5th) must reach the stack --
                // previously dropped, so `break @label` fell back to the
                // innermost loop.
                let label_name = label.as_ref().map(|l| l.name.clone());
                self.local.loop_stack.push((label_name, loop_cond.clone(), loop_exit.clone()));
                // BUG 22 #6: bindings inside the body hoist their alloca to the
                // fn entry (loop-body allocas don't dominate later blocks).
                self.local.loop_depth += 1;
                self.compile_block(body, false)?;
                self.local.loop_depth -= 1;
                self.local.loop_stack.pop();
                self.emitln(&format!("  br label %{loop_cond}"));
                self.emitln(&format!("\n{loop_exit}:"));
            }
            Stmt::For(var, iter, body, _, label) => {
                // P0-1 FIX: Implement proper for..in iteration.
                // Strategy: compile iter expression into an alloca, then in a
                // loop check start<end, extract start, increment, run body.
                // This works for Range {start: Int, end: Int} and similar
                // iterator types. Falls back to iterator protocol (.next()
                // call) for non-Range types.

                // 1. Compile the iterator expression
                let (iter_val, iter_ty) = self.compile_expr(iter)?;

                // 2. Alloca the iterator struct
                let iter_alloca = self.fresh_tmp();
                self.emitln(&format!("  {iter_alloca} = alloca {iter_ty}"));
                self.emitln(&format!("  store {iter_ty} {iter_val}, {iter_ty}* {iter_alloca}"));

                // 3. Create loop blocks
                let loop_cond = self.fresh_block("for_cond");
                let loop_body = self.fresh_block("for_body");
                let loop_exit = self.fresh_block("for_exit");

                self.emitln(&format!("  br label %{loop_cond}"));
                self.emitln(&format!("\n{loop_cond}:"));

                // 4. Direct Range iteration: check start < end
                // Range struct layout: field 0 = start (i64), field 1 = end (i64)
                let start_gep = self.fresh_tmp();
                self.emitln(&format!("  {start_gep} = getelementptr {iter_ty}, {iter_ty}* {iter_alloca}, i32 0, i32 0"));
                let start_val = self.fresh_tmp();
                self.emitln(&format!("  {start_val} = load i64, i64* {start_gep}"));

                let end_gep = self.fresh_tmp();
                self.emitln(&format!("  {end_gep} = getelementptr {iter_ty}, {iter_ty}* {iter_alloca}, i32 0, i32 1"));
                let end_val = self.fresh_tmp();
                self.emitln(&format!("  {end_val} = load i64, i64* {end_gep}"));

                let cond = self.fresh_tmp();
                self.emitln(&format!("  {cond} = icmp slt i64 {start_val}, {end_val}"));
                self.emitln(&format!("  br i1 {cond}, label %{loop_body}, label %{loop_exit}"));
                self.emitln(&format!("\n{loop_body}:"));

                // Bind loop variable to start_val
                let var_alloca = self.fresh_tmp();
                self.emitln(&format!("  {var_alloca} = alloca i64"));
                self.emitln(&format!("  store i64 {start_val}, i64* {var_alloca}"));
                self.add_local(&var.name, var_alloca, "i64");

                // Increment start for next iteration (before body so break/continue work)
                let next_start = self.fresh_tmp();
                self.emitln(&format!("  {next_start} = add i64 {start_val}, 1"));
                self.emitln(&format!("  store i64 {next_start}, i64* {start_gep}"));

                // Push loop stack for break/continue support
                let label_name = label.as_ref().map(|l| l.name.clone());
                self.local.loop_stack.push((label_name, loop_cond.clone(), loop_exit.clone()));

                // BUG 22 #6: bindings inside the body hoist their alloca to the
                // fn entry (loop-body allocas don't dominate later blocks).
                self.local.loop_depth += 1;
                self.compile_block(body, false)?;
                self.local.loop_depth -= 1;

                self.local.loop_stack.pop();

                // Jump back to condition
                self.emitln(&format!("  br label %{loop_cond}"));

                // Loop exit
                self.emitln(&format!("\n{loop_exit}:"));
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
            Stmt::Spawn(body, _span, _move) => {
                // v0.55/R2: Spawn -- compile body as separate function, call xiom_thread_spawn.
                // (xiom_thread_spawn is already declared at module level in compile_program)
                let spawn_id = self.local.spawn_counter;
                self.local.spawn_counter += 1;
                let fn_name = format!("_xiom_spawn_{spawn_id}");

                // R2: Collect captured variables -- names used in body that are
                // declared in outer scopes (not inside the spawn block itself).
                let captures: Vec<String> = self.collect_spawn_captures(body);

                let saved_output = std::mem::take(&mut self.output);
                let saved_tmp = self.tmp_counter;
                let saved_block = self.block_counter;
                self.tmp_counter = 0;
                self.block_counter = 0;

                // Push fresh scope for spawn function locals (capture alloca
                // registrations must not pollute the parent function's locals).
                self.push_scope();

                // Emit spawn function header and capture unpacking
                if captures.is_empty() {
                    self.emitln(&format!("\ndefine void @{fn_name}(i8* %_xiom_spawn_arg) {{"));
                    self.emitln("entry:");
                } else {
                    self.emitln(&format!("\ndefine void @{fn_name}(i8* %_xiom_spawn_arg) {{"));
                    self.emitln("entry:");
                    for (i, cap) in captures.iter().enumerate() {
                        let offset = i as i64 * 8;
                        let ptr = self.fresh_tmp();
                        let val = self.fresh_tmp();
                        let alloca = self.fresh_tmp();
                        if offset == 0 {
                            self.emitln(&format!("  {ptr} = bitcast i8* %_xiom_spawn_arg to i64*"));
                        } else {
                            let gep = self.fresh_tmp();
                            self.emitln(&format!("  {gep} = getelementptr i8, i8* %_xiom_spawn_arg, i64 {offset}"));
                            self.emitln(&format!("  {ptr} = bitcast i8* {gep} to i64*"));
                        }
                        self.emitln(&format!("  {val} = load i64, i64* {ptr}"));
                        self.emitln(&format!("  {alloca} = alloca i64"));
                        self.emitln(&format!("  store i64 {val}, i64* {alloca}"));
                        self.add_local(cap, alloca, "i64");
                    }
                }

                // Bump counters past capture unpacking to avoid name conflicts with body
                self.tmp_counter = 1000;
                self.block_counter = 1000;
                self.compile_block(body, false)?;
                self.emitln("  ret void");
                self.emitln("}");
                self.pop_scope(); // R2: pop the spawn function's scope
                let spawn_fn_ir = std::mem::take(&mut self.output);
                self.output = saved_output;
                self.tmp_counter = saved_tmp;
                self.block_counter = saved_block;

                // Append spawn function at end of module
                self.local.deferred_closure_defs.push(spawn_fn_ir);

                if captures.is_empty() {
                    let handle = self.fresh_tmp();
                    self.emitln(&format!(
                        "  {handle} = call i64 @xiom_thread_spawn(ptr @{fn_name}, ptr null)"
                    ));
                } else {
                    // Allocate env buffer and store captures
                    let env_size = captures.len() as i64 * 8;
                    let env_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {env_ptr} = call i8* @malloc(i64 {env_size})"));
                    // Null check
                    let null_ok = self.fresh_tmp();
                    let ok_block = self.fresh_block("spawn_env_ok");
                    let trap_block = self.fresh_block("spawn_env_trap");
                    self.emitln(&format!("  {null_ok} = icmp eq i8* {env_ptr}, null"));
                    self.emitln(&format!("  br i1 {null_ok}, label %{trap_block}, label %{ok_block}"));
                    self.emitln(&format!("\n{trap_block}:"));
                    self.emitln("  call void @llvm.trap()");
                    self.emitln("  unreachable");
                    self.emitln(&format!("\n{ok_block}:"));
                    // Store each capture into env
                    for (i, cap) in captures.iter().enumerate() {
                        // Load the captured variable's value directly using its name.
                        // The variable is already in scope and its alloca is accessible.
                        let val = self.fresh_tmp();
                        let offset = i as i64 * 8;
                        let dst = self.fresh_tmp();
                        // Look up the alloca via the local variable registry
                        let alloca_name = self.lookup_local(cap)
                            .map(|(a, _)| a.clone())
                            .unwrap_or_else(|| format!("%_missing_{cap}"));
                        self.emitln(&format!("  {val} = load i64, i64* {alloca_name}"));
                        if offset == 0 {
                            self.emitln(&format!("  {dst} = bitcast i8* {env_ptr} to i64*"));
                        } else {
                            let gep = self.fresh_tmp();
                            self.emitln(&format!("  {gep} = getelementptr i8, i8* {env_ptr}, i64 {offset}"));
                            self.emitln(&format!("  {dst} = bitcast i8* {gep} to i64*"));
                        }
                        self.emitln(&format!("  store i64 {val}, i64* {dst}"));
                    }
                    let handle = self.fresh_tmp();
                    self.emitln(&format!(
                        "  {handle} = call i64 @xiom_thread_spawn(ptr @{fn_name}, ptr {env_ptr})"
                    ));
                }
            }
            Stmt::Break(label, _) => {
                // P0-3 FIX: Respect labeled break. Search loop_stack for matching
                // label (top-down, LIFO). If no label, use innermost loop.
                let target = if let Some(lab) = label {
                    self.local.loop_stack.iter().rev()
                        .find(|(l, _, _)| l.as_ref().map_or(false, |l_name| l_name == &lab.name))
                        .cloned()
                        .or_else(|| self.local.loop_stack.last().cloned())
                } else {
                    self.local.loop_stack.last().cloned()
                };
                if let Some((_, _cont, break_label)) = target {
                    self.emitln(&format!("  br label %{break_label}"));
                    let dead = self.fresh_block("after_break");
                    self.emitln(&format!("\n{dead}:"));
                }
            }
            Stmt::Continue(label, _) => {
                // P0-3 FIX: Respect labeled continue. Search loop_stack for matching
                // label (top-down, LIFO). If no label, use innermost loop.
                let target = if let Some(lab) = label {
                    self.local.loop_stack.iter().rev()
                        .find(|(l, _, _)| l.as_ref().map_or(false, |l_name| l_name == &lab.name))
                        .cloned()
                        .or_else(|| self.local.loop_stack.last().cloned())
                } else {
                    self.local.loop_stack.last().cloned()
                };
                if let Some((_, cont_label, _)) = target {
                    self.emitln(&format!("  br label %{cont_label}"));
                    let dead = self.fresh_block("after_continue");
                    self.emitln(&format!("\n{dead}:"));
                }
            }
            // BUG 27: assert(cond[, "msg"]) -- runtime-checked invariant.
            // On false the message (or a default with the source location)
            // goes through xiom_panic (clean stderr + exit 1).
            Stmt::Assert(cond, msg, span) => {
                // Security review (2026-08-13): release builds strip assert
                // statements (Rust debug_assert! policy) -- no assertion
                // messages or debug-only logic in shipped binaries.
                if self.config.strip_debug_checks {
                    return Ok(());
                }
                let (c, ct) = self.compile_expr(cond)?;
                let c_i1 = if ct == "i1" { c.clone() } else {
                    let ne = self.fresh_tmp();
                    self.emitln(&format!("  {ne} = icmp ne {ct} {c}, 0"));
                    ne
                };
                let ok_block = self.fresh_block("assert_ok");
                let fail_block = self.fresh_block("assert_fail");
                self.emitln(&format!("  br i1 {c_i1}, label %{ok_block}, label %{fail_block}"));
                self.emitln(&format!("\n{fail_block}:"));
                let msg_ptr = match msg {
                    Some(m) => {
                        let (mv, mt) = self.compile_expr(m)?;
                        if mt == "i8*" {
                            // Str message -- use the compiled pointer directly.
                            mv
                        } else {
                            let fallback = format!("assertion failed at {}:{}", span.line, span.col);
                            self.intern_cstring(&fallback)
                        }
                    }
                    None => self.intern_cstring(&format!("assertion failed at {}:{}", span.line, span.col)),
                };
                self.emitln(&format!("  call void @xiom_panic(i8* {msg_ptr})"));
                self.emitln("  unreachable");
                self.emitln(&format!("\n{ok_block}:"));
            }
            // BUG 27: debugger; -- break into the attached debugger (no-op
            // without one; used with the xiom-dbg DAP server).
            Stmt::Debugger(_) => {
                // Security review (2026-08-13): debugger; is stripped from
                // release builds along with assert/dbg!.
                if !self.config.strip_debug_checks {
                    self.emitln("  call void @xiom_debugger_break()");
                }
            }
            xiom_ast::Stmt::Asm(ab) => {
                // Emit inline assembly as LLVM IR call void asm sideeffect
                let asm_str = ab.template.replace('\n', "\\0A");
                let mut constraints = String::new();
                for (c, _) in &ab.outputs {
                    if !constraints.is_empty() { constraints.push(','); }
                    constraints.push_str(c);
                }
                for (c, _) in &ab.inputs {
                    if !constraints.is_empty() { constraints.push(','); }
                    constraints.push_str(c);
                }
                let clobber_part = if ab.clobbers.is_empty() {
                    "~{dirflag},~{fpsr},~{flags}".to_string()
                } else {
                    let mut c = ab.clobbers.clone();
                    // Always include default clobbers
                    for d in &["dirflag", "fpsr", "flags"] {
                        let s = d.to_string();
                        if !c.contains(&s) { c.push(s); }
                    }
                    c.join(",")
                };
                self.emitln(&format!(
                    "  call void asm sideeffect \"{}\", \"~{{{}}}\"()",
                    asm_str, clobber_part
                ));
            }
            xiom_ast::Stmt::Defer(block, _) => {
                // P0-2 FIX: Push deferred block onto stack for LIFO scope-exit execution.
                // Previously executed immediately which is wrong.
                self.local.defer_stack.push(block.clone());
            }
        }
        Ok(())
    }

    /// R2: Collect variable names captured by a spawn block -- names referenced
    /// inside the body that are available in the current scope (not declared
    /// within the spawn block itself).
    fn collect_spawn_captures(&self, body: &Block) -> Vec<String> {
        let mut refs = HashSet::new();
        Self::collect_block_var_refs(body, &mut refs);
        // Filter: only keep names that exist in the current locals scope
        let captures: Vec<String> = refs.into_iter()
            .filter(|name| self.lookup_local(name).is_some())
            .collect();
        captures
    }

    fn collect_block_var_refs(block: &Block, refs: &mut HashSet<String>) {
        for se in &block.stmts {
            match se {
                StmtOrExpr::Stmt(s) => Self::collect_stmt_var_refs(s, refs),
                StmtOrExpr::Expr(e) => Self::collect_expr_var_refs(e, refs),
            }
        }
    }

    fn collect_stmt_var_refs(stmt: &Stmt, refs: &mut HashSet<String>) {
        match stmt {
            Stmt::Let(_, _, e, _) | Stmt::Var(_, _, e, _) => Self::collect_expr_var_refs(e, refs),
            Stmt::Assign(a, b, _) => { Self::collect_expr_var_refs(a, refs); Self::collect_expr_var_refs(b, refs); }
            Stmt::Return(Some(e), _) => Self::collect_expr_var_refs(e, refs),
            Stmt::Return(None, _) => {}
            Stmt::Expr(e, _) => Self::collect_expr_var_refs(e, refs),
            Stmt::If(c, t, elifs, els, _) => {
                Self::collect_expr_var_refs(c, refs);
                Self::collect_block_var_refs(t, refs);
                for (ec, eb) in elifs { Self::collect_expr_var_refs(ec, refs); Self::collect_block_var_refs(eb, refs); }
                if let Some(eb) = els { Self::collect_block_var_refs(eb, refs); }
            }
            Stmt::While(c, b, _, _, _) => { Self::collect_expr_var_refs(c, refs); Self::collect_block_var_refs(b, refs); }
            Stmt::For(_, e, b, _, _) => { Self::collect_expr_var_refs(e, refs); Self::collect_block_var_refs(b, refs); }
            Stmt::Spawn(b, _, _) => Self::collect_block_var_refs(b, refs),
            Stmt::Match(e, arms, _) => {
                Self::collect_expr_var_refs(e, refs);
                for arm in arms {
                    if let Some(g) = &arm.guard { Self::collect_expr_var_refs(g, refs); }
                    match &arm.body {
                        MatchBody::Block(b) => Self::collect_block_var_refs(b, refs),
                        MatchBody::Expr(e) => Self::collect_expr_var_refs(e, refs),
                    }
                }
            }
            Stmt::Destructure(_, e, _) => Self::collect_expr_var_refs(e, refs),
            Stmt::Break(..) | Stmt::Continue(..) | Stmt::Asm(_) | Stmt::Defer(_, _) => {}
            Stmt::Assert(c, m, _) => {
                Self::collect_expr_var_refs(c, refs);
                if let Some(msg) = m { Self::collect_expr_var_refs(msg, refs); }
            }
            Stmt::Debugger(_) => {},
        }
    }

    fn collect_expr_var_refs(expr: &Expr, refs: &mut HashSet<String>) {
        match expr {
            Expr::Ident(id) => { refs.insert(id.name.clone()); }
            Expr::Field(b, _, _) => Self::collect_expr_var_refs(b, refs),
            Expr::Call(f, args, _) | Expr::GenericCall(f, _, args, _) => {
                Self::collect_expr_var_refs(f, refs);
                for a in args { Self::collect_expr_var_refs(a, refs); }
            }
            Expr::Index(a, b, _) => { Self::collect_expr_var_refs(a, refs); Self::collect_expr_var_refs(b, refs); }
            Expr::Binary(a, _, b, _) | Expr::Imply(a, b, _) => {
                Self::collect_expr_var_refs(a, refs); Self::collect_expr_var_refs(b, refs);
            }
            Expr::Unary(_, e, _) | Expr::Paren(e, _) | Expr::Try(e, _) | Expr::Ref(e, _)
            | Expr::MutRef(e, _) | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _)
            | Expr::Comptime(e, _) | Expr::As(e, _, _) => Self::collect_expr_var_refs(e, refs),
            Expr::Struct(_, fields, base, _) => {
                for (_, v) in fields { Self::collect_expr_var_refs(v, refs); }
                if let Some(b) = base { Self::collect_expr_var_refs(b, refs); }
            }
            Expr::Array(elems, _) | Expr::Tuple(elems, _) => {
                for e in elems { Self::collect_expr_var_refs(e, refs); }
            }
            Expr::If(c, t, elifs, els, _) => {
                Self::collect_expr_var_refs(c, refs);
                Self::collect_block_var_refs(t, refs);
                for (ec, eb) in elifs { Self::collect_expr_var_refs(ec, refs); Self::collect_block_var_refs(eb, refs); }
                if let Some(eb) = els { Self::collect_block_var_refs(eb, refs); }
            }
            Expr::Match(e, arms, _) => {
                Self::collect_expr_var_refs(e, refs);
                for arm in arms {
                    if let Some(g) = &arm.guard { Self::collect_expr_var_refs(g, refs); }
                    match &arm.body {
                        MatchBody::Block(b) => Self::collect_block_var_refs(b, refs),
                        MatchBody::Expr(e) => Self::collect_expr_var_refs(e, refs),
                    }
                }
            }
            Expr::Is(e, _, _) => Self::collect_expr_var_refs(e, refs),
            Expr::Closure(_, _, b, _) | Expr::BlockExpr(b, _) => Self::collect_block_var_refs(b, refs),
            Expr::PipeClosure(_, e, _) => Self::collect_expr_var_refs(e, refs),
            _ => {}
        }
    }

    // P0-2: Emit all deferred blocks in LIFO order at scope exit.
    // Called before `ret` instructions to guarantee defer execution.
    // Does NOT pop the defer_stack -- multiple return paths must all emit
    // the same defers. The stack is cleared at function epilogue.
    pub(crate) fn compile_deferred_cleanups(&mut self) -> Result<(), String> {
        let blocks: Vec<Block> = self.local.defer_stack.iter().rev().cloned().collect();
        for block in &blocks {
            for soe in &block.stmts {
                match soe {
                    xiom_ast::StmtOrExpr::Stmt(s) => self.compile_stmt(s)?,
                    xiom_ast::StmtOrExpr::Expr(e) => { self.compile_expr(e)?; }
                }
            }
        }
        Ok(())
    }

    /// Clear the defer stack (call after function body compilation is complete)
    pub(crate) fn clear_deferred_cleanups(&mut self) {
        self.local.defer_stack.clear();
    }
}
