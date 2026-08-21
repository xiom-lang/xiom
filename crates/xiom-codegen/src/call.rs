// XIOM Codegen -- Function/method call compilation (extracted from expr.rs, M4.2)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_ast::*;
use crate::llvm_consts::*;
use std::collections::HashMap;

use super::IrEmitter;

impl IrEmitter {
    /// BUG 23 #2: render a generic type-argument EXPRESSION to its type NAME,
    /// handling nested Vec[...] chains (`Vec[Vec[Int]]` parses as nested
    /// Expr::Index, not a plain Ident). Used by the Vec::new() element-size
    /// computation so Vec[Vec[T]] allocates 32-byte elements (a Vec struct),
    /// not 8-byte truncated handles.
    pub(crate) fn type_arg_to_name(e: &Expr) -> String {
        match e {
            Expr::Ident(id) => id.name.clone(),
            Expr::Index(base, idx, _) => {
                if let Expr::Ident(b) = base.as_ref() {
                    if b.name == "Vec" {
                        format!("Vec[{}]", Self::type_arg_to_name(idx))
                    } else {
                        "Int".to_string()
                    }
                } else {
                    "Int".to_string()
                }
            }
            _ => "Int".to_string(),
        }
    }

    pub(crate) fn compile_call(&mut self, func: &Expr, args: &[Expr]) -> Result<(String, String), String> {
        self.compile_call_with_types(func, args, None)
    }

    /// D1: compile_call with explicit generic type args (from `fn[Type](args)`).
    pub(crate) fn compile_call_with_types(
        &mut self,
        func: &Expr,
        args: &[Expr],
        explicit_types: Option<&[xiom_ast::Type]>,
    ) -> Result<(String, String), String> {
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
                                || self.types.types.contains_key(&id.name)
                                || self.types.type_meta.contains_key(&id.name)
                                || (id.name.len() == 1 && id.name.chars().next().map_or(false, |c| c.is_ascii_uppercase()))
                        }
                        Expr::Field(_, _, _) => true,
                        Expr::Tuple(elems, _) => elems.iter().all(|e| matches!(e, Expr::Ident(_) | Expr::Field(_, _, _))),
                        // 5e.3: parameterized type args like RcInner[T] parse as
                        // Expr::Index(Ident(base), Ident(T)). Recognize when the
                        // base is a known type so size_of/align_of receive type_arg.
                        Expr::Index(base, inner, _) => {
                            matches!(base.as_ref(), Expr::Ident(b) if
                                Self::is_primitive_type_name(&b.name)
                                || self.types.types.contains_key(&b.name)
                                || self.types.type_meta.contains_key(&b.name))
                            && matches!(inner.as_ref(), Expr::Ident(i) if
                                i.name.len() == 1 && i.name.chars().next().map_or(false, |c| c.is_ascii_uppercase()))
                        }
                        _ => false, // integer literal, binary expr, etc. -- always a value index
                    }
                };
                // D1: capture explicit generic type args from `fn[TypeArgs](...)`
                // (GenericCall) so the generic-call path can map T->concrete
                // (e.g. `add2[Float32]` must monomorphise as Float32, not Int).
                let explicit_generic_types: Vec<String> = match explicit_types {
                    Some(types) => types.iter().map(|t| Self::type_from_ast(t)).collect(),
                    None => match func {
                        Expr::GenericCall(_, types, _, _) => types.iter()
                            .map(|t| Self::type_from_ast(t))
                            .collect(),
                        _ => Vec::new(),
                    },
                };
let (func_unwrapped, mut type_arg): (&Expr, Option<&Expr>) = match func {
                    Expr::Index(base, idx, _) if idx_is_type(&idx) => {
                        (base.as_ref(), Some(idx.as_ref()))
                    }
                    // D1: `fn[TypeArgs](args)` -- GenericCall with explicit type
                    // args (captured in explicit_generic_types above).
                    Expr::GenericCall(base, _, _, _) => (base.as_ref(), None),
                    other => (other, None),
                };
                let (fn_name_opt, receiver_expr) = match func_unwrapped {
                    Expr::Ident(name) => (Some(name.name.clone()), None),
                    Expr::Field(obj, field, _) => (Some(field.name.clone()), Some(obj)),
                    _ => (None, None),
                };
                // M20-A1: Closure call detection -- if the callee is a local
                // variable (bare Ident, not a known function), check if it's
                // a closure and dispatch with env pointer.
                if receiver_expr.is_none() {
                    if let Some(ref name) = fn_name_opt {
                        if self.local.closure_locals.contains(name) {
                            // Local variable -- could be a closure
                            let compiled_args: Vec<(String, String)> = args.iter()
                                .map(|a| self.compile_expr(a).map(|(v, t)| (v, t)))
                                .collect::<Result<Vec<_>, _>>()?;
                            // Load the closure variable
                            let (callee_val, _callee_ty) = self.compile_expr(func_unwrapped)?;
                            let env_ptr_val = self.val_to_i64(&callee_val, &_callee_ty);
                            // Load fn_ptr from env struct (field 0)
                            let env_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {env_ptr} = inttoptr i64 {env_ptr_val} to i64*"));
                            let loaded_fn = self.fresh_tmp();
                            self.emitln(&format!("  {loaded_fn} = load i64, i64* {env_ptr}"));
                            // Build args: env_ptr first, then closure arguments
                            let mut closure_args = vec![(env_ptr_val, "i64".to_string())];
                            for a in &compiled_args { closure_args.push(a.clone()); }
                            let args_str = closure_args.iter()
                                .map(|(v, t)| format!("{t} {v}"))
                                .collect::<Vec<_>>().join(", ");
                            let param_types: Vec<String> = closure_args.iter()
                                .map(|(_, t)| t.clone())
                                .collect();
                            // B-007: the closure's REAL return type (by-value
                            // struct returns -- Option/Result -- are NOT
                            // pointers; hardcoding i64 + inttoptr deref'd the
                            // struct bits and crashed Option.and_then).
                            let ret_llvm = self.local.fn_local_returns.get(name)
                                .and_then(|rt| self.llvm_type_for(rt).ok())
                                .unwrap_or_else(|| LLVM_I64.to_string());
                            let fn_ptr_ty = format!("{ret_llvm} ({})*", param_types.join(", "));
                            let fn_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {fn_ptr} = inttoptr i64 {loaded_fn} to {fn_ptr_ty}"));
                            let tmp = self.fresh_tmp();
                            if ret_llvm == "void" {
                                self.emitln(&format!("  call {ret_llvm} {fn_ptr}({args_str})"));
                                return Ok((String::new(), "void".to_string()));
                            }
                            self.emitln(&format!("  {tmp} = call {ret_llvm} {fn_ptr}({args_str})"));
                            return Ok((tmp, ret_llvm));
                        }
                    }
                }
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
                    Some(ref n) => {
                        n.clone()
                    }
                    None => {
                        // The callee is not a simple Ident or Field -- it may be an
                        // Expr::Index (e.g. `tests[i]()`) that produces a function pointer
                        // value. Compile the expression and call the result.
                        if let Expr::Index(container, index, _) = func {
                            return self.compile_index_fn_ptr_call(container, index, args);
                        }
                        // For other complex callee expressions (e.g. chained calls
                        // like `get_fn()()`), compile the callee and inttoptr.
                        let (callee_val, callee_ty) = self.compile_expr(func_unwrapped)?;
                        if callee_ty == "i64" || callee_ty == "i8*" || callee_ty.ends_with('*') {
                            let compiled_args: Vec<(String, String)> = args.iter()
                                .map(|a| self.compile_expr(a).map(|(v, t)| (v, t)))
                                .collect::<Result<Vec<_>, _>>()?;
                            
                            // M20-A1: If the callee is a closure (env struct pointer),
                            // load fn_ptr from field 0 and call through it with env_ptr as first arg.
                            // Detect closure: the value is an i64 that ptrtoint'd an env struct.
                            let (call_target, effective_args) = if callee_ty == "i64" {
                                // Load fn_ptr from the env struct (field 0 is always i64 fn_ptr)
                                let env_ptr_val = self.val_to_i64(&callee_val, &callee_ty);
                                let env_ptr = self.fresh_tmp();
                                self.emitln(&format!("  {env_ptr} = inttoptr i64 {env_ptr_val} to i64*"));
                                let loaded_fn = self.fresh_tmp();
                                self.emitln(&format!("  {loaded_fn} = load i64, i64* {env_ptr}"));
                                let _fn_ptr = self.fresh_tmp();
                                // Build argument list with env_ptr as first hidden arg
                                let mut closure_args = vec![(env_ptr_val.clone(), "i64".to_string())];
                                for a in &compiled_args {
                                    closure_args.push(a.clone());
                                }
                                (loaded_fn, closure_args)
                            } else {
                                (callee_val.clone(), compiled_args)
                            };
                            
                            let args_str = effective_args.iter()
                                .map(|(v, t)| format!("{t} {v}"))
                                .collect::<Vec<_>>().join(", ");
                            let param_types: Vec<String> = effective_args.iter()
                                .map(|(_, t)| t.clone())
                                .collect();
                            let fn_ptr_ty = format!("i64 ({})*", param_types.join(", "));
                            let fn_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {fn_ptr} = inttoptr i64 {call_target} to {fn_ptr_ty}"));
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = call i64 {fn_ptr}({args_str})"));
                            return Ok((tmp, LLVM_I64.to_string()));
                        }
                        return Ok(("0".to_string(), LLVM_I64.to_string()));
                    }
                };
                // D1 (2026-08-08): interface impl dispatch --
                // `Trait[Arg].method(args)` resolves to the impl's freestanding
                // `Type.method` fn (produced by expand_impl_blocks). Intercept
                // BEFORE generic dispatch so two impls of the same trait method
                // (e.g. Num[Int].add and Num[Float64].add) don't collide.
                if let Some(recv) = receiver_expr {
                    if let Some(impl_type) = self.resolve_impl_receiver(recv) {
                        // 3c: impls inside catalog-loaded modules are registered
                        // MODULE-QUALIFIED (e.g. `core.Float64.add` from
                        // `module xiom.math.core`). Try the bare name first,
                        // then any module-qualified variant whose tail matches
                        // `{concrete_type}.{method}`.
                        let bare = format!("{}.{}", impl_type, fn_name);
                        let mut candidates: Vec<String> = vec![bare.clone()];
                        for key in self.types.functions.keys() {
                            if key.ends_with(&format!(".{bare}")) {
                                candidates.push(key.clone());
                            }
                        }
                        for key in self.mono.generic_fn_decls.iter().map(|(k, _)| k.clone()) {
                            if key.ends_with(&format!(".{bare}")) {
                                candidates.push(key);
                            }
                        }
                        for impl_fn in &candidates {
                            if self.types.functions.contains_key(impl_fn)
                                || self.mono.emitted_fns.contains(impl_fn)
                                || self.mono.generic_fn_decls.iter().any(|(k, _)| k == impl_fn)
                            {
                                return self.compile_impl_method_call(impl_fn, args);
                            }
                        }
                    }
                }
                // Enum variant constructor: TypeName.Variant(args)
                // e.g. `JsonValue.Integer(42)` or `SqliteValue.Text("hello")`
                if let Some(recv) = receiver_expr {
                    let recv_ty = self.infer_struct_type_name(recv);
                    if let Some(recv_name) = recv_ty {
                        let variant_key = format!("{}.{}", recv_name, &fn_name);
                        if self.types.enum_variants.contains_key(&variant_key)
                            || self.types.enum_variants.get(&recv_name)
                                .map_or(false, |vars| vars.iter().any(|(v, _)| v == &fn_name))
                        {
                            return self.compile_enum_constructor(&recv_name, &fn_name, args);
                        }
                    }
                }
                // M22: Bare enum variant constructor: `Data(args)` (no TypeName. prefix).
                // Search all registered enum variants for a matching constructor name.
                if receiver_expr.is_none() {
                    let enum_key_opt = self.types.enum_variants.entries().into_iter()
    .find(|(_, vars)| vars.iter().any(|(v, _)| v == &fn_name))
                        .map(|(ek, _)| ek.clone());
                    if let Some(enum_key) = enum_key_opt {
                        return self.compile_enum_constructor(&enum_key, &fn_name, args);
                    }
                }
                // Check for contract collection methods -- only intercept when
                // there is no user-defined function with the same name; otherwise
                // a regular `fn is_sorted(arr: &Vec[Int]) -> Bool` gets hijacked
                // and replaced with a `call @xiom_is_sorted` builtin.
                let is_contract_method = matches!(fn_name.as_str(), "is_sorted" | "all" | "none" | "contains");
                if is_contract_method {
                    // Skip contract builtin if a user function with this name exists
                    // in the current module or has already been emitted.
                    let has_user_fn = self.mono.emitted_fns.contains(fn_name.as_str())
                        || self.types.functions.contains_key(&fn_name.to_string())
                        || self.mono.emitted_fns.iter().any(|k| k.ends_with(&format!(".{}", fn_name)))
                        || self.types.functions.keys().into_iter().any(|k| k.ends_with(&format!(".{}", fn_name)));
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
                                _ => unreachable!("set method with unexpected argument count"),
                            }
                            return Ok((tmp, LLVM_I64.to_string()));
                        }
                    }
                    // Direct form: method(args) -- compile all args
                    let compiled_args: Vec<String> = args.iter()
                        .map(|a| self.compile_expr(a).map(|(v, _)| v))
                        .collect::<Result<Vec<_>, _>>()?;
                    // Convert first argument to i8* pointer via alloca+bitcast
                    let ptr_val = compiled_args.first().cloned().unwrap_or_else(|| "0".to_string());
                    let ptr_ty = if let Some(arg) = args.first() { self.infer_llvm_type(arg) } else { LLVM_I64.to_string() };
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
                        _ => unreachable!("contains method with unexpected argument count"),
                    }
                    return Ok((tmp, LLVM_I64.to_string()));
                }
                } // if !has_user_fn -- contract builtin guard
                // Primitive interface methods (Ord.compare, Eq.eq/ne, comparison ops,
                // Hash.hash, Clone.clone) are emitted inline for scalar receivers, so
                // primitives satisfy Ord/Eq/Hash/Clone bounds without a user method.
                let is_builtin_iface_method = matches!(
                    fn_name.as_str(),
                    "compare" | "eq" | "ne" | "lt" | "gt" | "le" | "ge" | "hash" | "clone" | "to_owned"
                );
                if is_builtin_iface_method {
                    if let Some(receiver) = receiver_expr {
                        let is_value_instance = self.receiver_is_instance(receiver);
                        // Skip static/type-name receivers (e.g. Int.compare(a, b)).
                        let receiver_is_type_name = matches!(&**receiver, Expr::Ident(id)
                            if Self::is_primitive_type_name(&id.name)
                                || self.types.types.contains_key(&id.name)
                                || self.types.type_meta.contains_key(&id.name));
                        // Builtin interface methods only apply to value instances (e.g.
                        // `x.hash()` or `42.hash()`) -- module-qualified calls like
                        // `hash.hash(42)` must fall through to generic dispatch.
                        if !is_value_instance {
                            // module-qualified call -- fall through to generic dispatch
                        } else {
                        let recv_llvm_ty = self.infer_llvm_type(receiver);
                        // Gap A fix: clone/to_owned on Str (i8*) receivers.
                        // XIOM strings are immutable, so duplication can share the
                        // pointer soundly. Previously this fell through to generic
                        // dispatch and MISCOMPILED into a call to an undefined
                        // @clone symbol (silent corruption, wrong results).
                        if recv_llvm_ty == "i8*"
                            && matches!(fn_name.as_str(), "clone" | "to_owned")
                            && args.is_empty()
                            && !receiver_is_type_name
                        {
                            let (recv_val, _) = self.compile_expr(receiver)?;
                            return Ok((recv_val, LLVM_STR_PTR.to_string()));
                        }
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
                                "clone" | "to_owned" => return Ok((recv_val, recv_llvm_ty.clone())),
                                "hash" if args.is_empty() => {
                                    if is_float {
                                        let bits = if recv_llvm_ty == "double" { "i64" } else { "i32" };
                                        let cast = self.fresh_tmp();
                                        self.emitln(&format!("  {cast} = bitcast {recv_llvm_ty} {recv_val} to {bits}"));
                                        if bits == "i64" {
                                            return Ok((cast, LLVM_I64.to_string()));
                                        }
                                        let ext = self.fresh_tmp();
                                        self.emitln(&format!("  {ext} = sext i32 {cast} to i64"));
                                        return Ok((ext, LLVM_I64.to_string()));
                                    }
                                    if recv_llvm_ty == "i64" {
                                        return Ok((recv_val, LLVM_I64.to_string()));
                                    }
                                    let ext = self.fresh_tmp();
                                    self.emitln(&format!("  {ext} = sext {recv_llvm_ty} {recv_val} to i64"));
                                    return Ok((ext, LLVM_I64.to_string()));
                                }
                                // "hash" with args (Hash interface method call like
                                // `value.hash(hasher)` inside a generic body) -- exit
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
                                // For builtin interface methods on scalar receivers,
                                // arguments like `&value` should be dereferenced to
                                // their VALUE, not the address. Otherwise icmp compares
                                // the element value against the parameter's alloca address.
                                // e.g. items[i].eq(&value) where T=Int should compare
                                // two i64 values, not value vs &value.
                                let arg_expr = if let Expr::Ref(inner, _) = a {
                                    inner.as_ref()
                                } else {
                                    a
                                };
                                let (arg_raw, arg_ty) = self.compile_expr(arg_expr)?;
                                // Reference-typed args compile to a pointer (i64* from
                                // &mut T / *T) OR an address-as-i64 (plain `&T` param).
                                // Deref to the scalar VALUE for the comparison.
                                let is_ref_param = if let Expr::Ident(id) = arg_expr {
                                    self.local.ref_params.contains(&id.name)
                                } else { false };
                                if arg_ty.ends_with('*') || is_ref_param {
                                    let loaded = self.fresh_tmp();
                                    let base = if arg_ty.ends_with('*') {
                                        arg_ty.trim_end_matches('*').to_string()
                                    } else {
                                        "i64".to_string()
                                    };
                                    let ptr = if arg_ty.ends_with('*') {
                                        arg_raw.clone()
                                    } else {
                                        let p = self.fresh_tmp();
                                        self.emitln(&format!("  {p} = inttoptr i64 {arg_raw} to i64*"));
                                        p
                                    };
                                    self.emitln(&format!("  {loaded} = load {base}, {base}* {ptr}"));
                                    loaded
                                } else {
                                    arg_raw
                                }
                            } else {
                                "0".to_string()
                            };
                            if fn_name == "compare" || fn_name == "cmp" {
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
                                return Ok((res, LLVM_I64.to_string()));
                            }
                            // round-10 (Ord tower): min/max on scalar receivers --
                            // min(a,b) = a < b ? a : b (select on the lt/gt result).
                            if fn_name == "min" || fn_name == "max" {
                                let op = if is_float {
                                    if fn_name == "min" { "fcmp olt" } else { "fcmp ogt" }
                                } else {
                                    if fn_name == "min" { "icmp slt" } else { "icmp sgt" }
                                };
                                let cond = self.fresh_tmp();
                                self.emitln(&format!("  {cond} = {op} {recv_llvm_ty} {recv_val}, {arg_val}"));
                                let res = self.fresh_tmp();
                                self.emitln(&format!("  {res} = select i1 {cond}, {recv_llvm_ty} {recv_val}, {recv_llvm_ty} {arg_val}"));
                                return Ok((res, recv_llvm_ty.clone()));
                            }
                            let op = match (fn_name.as_str(), is_float) {
                                ("eq", false) => "icmp eq",  ("eq", true) => "fcmp oeq",
                                // BUG 19 fix: float `.ne()` must be fcmp une --
                                // `one` is false for NaN operands (IEEE: NaN != NaN is true)
                                ("ne", false) => "icmp ne",  ("ne", true) => "fcmp une",
                                ("lt", false) => "icmp slt", ("lt", true) => "fcmp olt",
                                ("gt", false) => "icmp sgt", ("gt", true) => "fcmp ogt",
                                ("le", false) => "icmp sle", ("le", true) => "fcmp ole",
                                ("ge", false) => "icmp sge", ("ge", true) => "fcmp oge",
                                _ => unreachable!("unknown comparison operator"),
                            };
                            let cmp = self.fresh_tmp();
                            self.emitln(&format!("  {cmp} = {op} {recv_llvm_ty} {recv_val}, {arg_val}"));
                            let res = self.fresh_tmp();
                            self.emitln(&format!("  {res} = zext i1 {cmp} to i64"));
                            return Ok((res, LLVM_I64.to_string()));
                            } // end else (fn_name != "hash")
                        }
                        } // end else (is_value_instance)
                    }
                }
                // Check for memory allocation/free builtins
                if fn_name == "alloc" {
                    if let Some(size_arg) = args.first() {
                        let (size_raw, size_ty) = self.compile_expr(size_arg)?;
                        let size_val = self.val_to_i64(&size_raw, &size_ty);
                        let tmp = self.emit_alloc(&size_val);
                        return Ok((tmp, LLVM_STR_PTR.to_string()));
                    } else {
                        let tmp = self.emit_alloc("0");
                        return Ok((tmp, LLVM_STR_PTR.to_string()));
                    }
                }
                // ptr.null[T]() / ptr.null_mut[T]() / ptr.dangling[T]() -- generic
                // pointer constructors with NO value arguments. The parser discards
                // explicit type args (`[T]`), so type inference can't specialise them
                // and the generic path returns a bogus 0. Inline them: null -> 0 (a
                // null pointer), dangling -> a non-null sentinel (1). Only fires for
                // the zero-arg module form (a module-qualified receiver, not a value
                // instance), so it never shadows a user method on a struct value.
                if args.is_empty()
                    && matches!(fn_name.as_str(), "null" | "null_mut" | "dangling")
                    && receiver_expr.map(|r| !self.receiver_is_instance(r)).unwrap_or(true)
                {
                    let v = if fn_name == "dangling" { "1" } else { "0" };
                    return Ok((v.to_string(), LLVM_I64.to_string()));
                }
                // to_string(Int) / x.to_str() / x.to_string() on an integer value:
                // lower to the C runtime `xiom_int_to_string`. The pure-XIOM
                // `to_string` uses fixed stack arrays the codegen can't materialize.
                // 5c.29: never hijack a USER-DEFINED `Type.to_str`/`Type.to_string`
                // (e.g. `HttpMethod.to_str(m)`) -- fall through to normal dispatch.
                let user_defined_to_str = matches!(fn_name.as_str(), "to_string" | "to_str")
                    && receiver_expr
                        .and_then(|r| self.infer_struct_type_name(r))
                        .map_or(false, |tn| {
                            let key = format!("{tn}.{fn_name}");
                            self.types.functions.contains_key(&key)
                                || self.types.functions.keys().into_iter().any(|k| k.ends_with(&format!(".{key}")))
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
                            return Ok((sel, LLVM_STR_PTR.to_string()));
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
                            return Ok((tmp, LLVM_STR_PTR.to_string()));
                        }
                    }
                }
                // ptr.from_ref(x) / ptr.from_mut(x) -- take a reference to an lvalue
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
                                // param) -- load and return it (identity address).
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
                        return Ok(("null".to_string(), LLVM_STR_PTR.to_string()));
                    }
                    let slot = self.fresh_tmp();
                    self.emitln(&format!("  {slot} = alloca {ty}{}", self.alloca_align(&ty)));
                    self.emitln(&format!("  store {ty} {val}, {ty}* {slot}{}", self.store_align(&ty)));
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
                // Vec.new() -- static method on Vec type
                if let Some(receiver) = receiver_expr {
                    // Accept both `Vec.new()` (Ident receiver) and `Vec[T].new()`
                    // (Index receiver -- the parser wraps the explicit type arg as an
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
                        // 5e.3: module-qualified type paths like alloc.Layout
                        // produce Expr::Field receivers whose leaf is the type.
                        Expr::Field(_, field, _) => Some(field.name.as_str()),
                        _ => None,
                    };
                    // 5e.3: Layout.new(size) -- inline struct constructor { size, align: 8 }.
                    // Layout is defined in xiom.alloc and may not be compiled into
                    // the current module's IR. Inlining here avoids the need for
                    // cross-module function resolution.
                    if (recv_ident == Some("Layout") || recv_ident == Some("alloc.Layout") || recv_ident == Some("xiom.alloc.Layout")) && fn_name == "new" {
                        if args.len() == 1 {
                            let (size_val, size_ty) = self.compile_expr(&args[0])?;
                            let size_i64 = self.val_to_i64(&size_val, &size_ty);
                            // Layout may be defined as bare "Layout" (from source)
                            // or qualified "xiom.alloc.Layout" (from builtin fallback).
                            // Use the type that actually has a struct definition emitted.
                            let layout_ty = if self.types.type_meta.contains_key(&"Layout".to_string()) {
                                self.llvm_type_for("Layout").unwrap_or_else(|_| "%struct.xiom.alloc.Layout".to_string())
                            } else {
                                self.llvm_type_for("xiom.alloc.Layout").unwrap_or_else(|_| "%struct.xiom.alloc.Layout".to_string())
                            };
                            let s0 = self.fresh_tmp();
                            self.emitln(&format!("  {s0} = insertvalue {layout_ty} undef, i64 {size_i64}, 0"));
                            let s1 = self.fresh_tmp();
                            self.emitln(&format!("  {s1} = insertvalue {layout_ty} {s0}, i64 8, 1"));
                            return Ok((s1, layout_ty));
                        }
                    }
                    if recv_ident == Some("Vec") && fn_name == "new" {
                            // Vec[UInt8] -> 1, Vec[Int16] -> 2, Vec[Int32] -> 4, default -> 8.
                            let elem_size: i64 = if let Some(type_arg) = type_arg {
                                if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() {
                                    eprintln!("[vecnew] type_arg={type_arg:?}");
                                }
                                let type_name = match type_arg {
                                    Expr::Ident(id) => id.name.clone(),
                                    Expr::Tuple(elems, _) => {
                                        let mut parts: Vec<String> = Vec::new();
                                        for e in elems.iter() {
                                            if let Expr::Ident(id) = e {
                                                parts.push(id.name.clone());
                                            } else {
                                                parts.clear();
                                                break;
                                            }
                                        }
                                        if parts.len() == elems.len() && !parts.is_empty() {
                                            format!("Tuple__{}", parts.join("__"))
                                        } else {
                                            "Int".to_string()
                                        }
                                    }
                                    _ => "Int".to_string(),
                                };
                                // BUG 23 #2 fix: NESTED generic args (`Vec[Vec[Int]]`,
                                // `Vec[Vec[Vec[Int]]]`) parse as Expr::Index chains, not
                                // Ident -- render them to "Vec[Vec[Int]]" so the
                                // struct-size lookup below returns 32 (a Vec element
                                // IS a %struct.Vec). Previously this fell to "Int" and
                                // inner Vecs were stored at 8 bytes each (truncated).
                                let type_name = if type_name == "Int" {
                                    Self::type_arg_to_name(type_arg)
                                } else {
                                    type_name
                                };
                                match type_name.as_str() {
                                    "UInt8" | "Int8" | "Char" | "Bool" => 1,
                                    "Int16" | "UInt16" => 2,
                                    "Int32" | "UInt32" | "Float32" => 4,
                                    _ => {
                                        // For struct types, compute the REAL layout
                                        // size (nested by-value struct fields count
                                        // fully -- 5c.30, field_countx8 truncated
                                        // JsonEntry-style elements). Container
                                        // fields count fully too (BUG 39:
                                        // struct_byte_size under-counted Vec fields
                                        // -> Vec[Vector] ctor buffer overflow).
                                        self.vec_elem_storage_size(&type_name)
                                    }
                                }
                            } else { 8 };
                            let initial_cap: i64 = 16;
                            let alloc_size = initial_cap * elem_size;
                            let struct_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {struct_alloca} = alloca %struct.Vec"));
                            let data_ptr = self.emit_alloc(&format!("{alloc_size}"));
                            // Null check on malloc -- trap on OOM
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
                    // 5c-R: Vec.with_capacity(n) -- same as Vec.new but with
                    // user-specified initial capacity (G-06).
                    if recv_ident == Some("Vec") && fn_name == "with_capacity" && args.len() == 1 {
                            let (cap_val, cap_ty) = self.compile_expr(&args[0])?;
                            let cap_i64 = self.val_to_i64(&cap_val, &cap_ty);
                            let elem_size: i64 = if let Some(type_arg) = type_arg {
                                let type_name = match type_arg {
                                    Expr::Ident(id) => id.name.clone(),
                                    Expr::Tuple(elems, _) => {
                                        let mut parts: Vec<String> = Vec::new();
                                        for e in elems.iter() {
                                            if let Expr::Ident(id) = e {
                                                parts.push(id.name.clone());
                                            } else {
                                                parts.clear();
                                                break;
                                            }
                                        }
                                        if parts.len() == elems.len() && !parts.is_empty() {
                                            format!("Tuple__{}", parts.join("__"))
                                        } else {
                                            "Int".to_string()
                                        }
                                    }
                                    _ => "Int".to_string(),
                                };
                                match type_name.as_str() {
                                    "UInt8" | "Int8" | "Char" | "Bool" => 1,
                                    "Int16" | "UInt16" => 2,
                                    "Int32" | "UInt32" | "Float32" => 4,
                                    _ => self.vec_elem_storage_size(&type_name),
                                }
                            } else { 8 };
                            let struct_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {struct_alloca} = alloca %struct.Vec"));
                            let alloc_size = self.fresh_tmp();
                            self.emitln(&format!("  {alloc_size} = mul i64 {elem_size}, {cap_i64}"));
                            let data_ptr = self.emit_alloc(&alloc_size);
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
                // Vec.push(vec, val) -- method call on Vec
                if fn_name == "push" && args.len() >= 1 {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        // Accept any Vec-typed receiver (Vec[Int], Vec[UInt8], a
                        // module-qualified `%struct.xiom.collections.Vec`, etc.),
                        // including i64 container-field handles (5c.29).
                        // BUG 34: an INDEXED element of a Vec (nested Vec[Vec[T]])
                        // compiles as i64/%struct.Vec -- accept it when the
                        // CONTAINER is Vec-typed (resolve_vec_receiver_ptr's
                        // element-address path handles the buffer GEP).
                        let is_indexed_vec_elem = matches!(receiver.as_ref(), Expr::Index(container, _, _) if {
                            let ct = self.infer_llvm_type(container);
                            Self::is_llvm_struct_named(&ct, "Vec")
                        });
                        let is_vec = Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || self.is_container_vec_field(receiver) || is_indexed_vec_elem;
                        if !is_vec {
                            // Not a Vec receiver --  fall through to general method dispatch
                        } else {
                        let (recv_val, recv_actual_ty) = self.compile_expr(receiver)?;
                        let (recv_vec, _) = self.resolve_vec_receiver(receiver, &recv_val, &recv_actual_ty);
                        // Convert array literals -> Vec structs for push arguments
                        let (mut val_raw, val_ty) = if let Expr::Array(elems, _) = &args[0] {
                            self.compile_array_as_vec(elems, "Int")?
                        } else {
                            self.compile_expr(&args[0])?
                        };
                        // B-007: pushing a bare fn-REFERENCE into a Vec[fn()] must
                        // wrap it in a closure ENV (the element is called env-first
                        // via the M20-A1 / index-call paths -- a raw code address
                        // would be deref'd as an env struct).
                        {
                            let elem_xiom = match &**receiver {
                                Expr::Ident(id) => self.local.local_vec_elem.get(&id.name).cloned(),
                                Expr::Field(..) => self.resolve_vec_container_elem_xiom(receiver),
                                _ => None,
                            };
                            let mut wrapped = None;
                            if elem_xiom.as_deref().map_or(false, |x| x.starts_with("fn(")) {
                                if let Some(ae) = args.first() {
                                    let is_fn_ref = matches!(ae, Expr::Ident(id)
                                        if (self.types.functions.contains_key(&id.name)
                                            || self.types.functions.keys().into_iter().any(|k| k.ends_with(&format!(".{}", id.name)))
                                            || self.mono.emitted_fns.contains(&id.name))
                                            && !self.local.closure_locals.contains(&id.name));
                                    if is_fn_ref {
                                        if let Expr::Ident(id) = ae {
                                            let params = self.types.functions.get(&id.name)
                                                .map(|(p, _)| p.clone())
                                                .or_else(|| {
                                                    let suffix = format!(".{}", id.name);
                                                    self.types.functions.entries().into_iter()
                                                        .find(|(k, _)| k.ends_with(&suffix))
                                                        .map(|(_, (p, _))| p.clone())
                                                })
                                                .unwrap_or_default();
                                            let ret = elem_xiom.as_deref().unwrap_or("Int").to_string();
                                            wrapped = Some(self.wrap_fn_ref_env(&id.name, &val_raw, &ret, params));
                                        }
                                    }
                                }
                            }
                            if let Some(w) = wrapped {
                                val_raw = w;
                            }
                        }
                        // G4: Float64->Float32 coercion for Vec[Float32] push.
                        // val_to_i64 bitcasts double->i64 preserving all 64 bits,
                        // but emit_elem_store truncates to i32 for 4-byte slots,
                        // discarding the upper 32 bits (exponent+sign). Must first
                        // fptrunc double->float so the float32 bit pattern is stored.
                        let (val_raw, val_ty) = if val_ty == "double" && self.vec_elem_float_type(receiver) == Some("float") {
                            let f32 = self.fresh_tmp();
                            self.emitln(&format!("  {f32} = fptrunc double {val_raw} to float"));
                            (f32, "float".to_string())
                        } else {
                            (val_raw, val_ty)
                        };
                        // R4 fix: Use the receiver's original alloca for in-place
                        // mutation instead of creating a per-call scratch alloca.
                        // The old `alloca %struct.Vec` leaked 32 bytes of stack per
                        // push iteration when called inside a loop, causing unbounded
                        // stack growth and ACCESS_VIOLATION (>100K iterations).
                        let (vec_alloca, needs_store_back) = self.resolve_vec_push_ptr(receiver)?;
                        // OPT-R5: When needs_store_back is false, the alloca IS the
                        // authoritative storage -- the SSA value was just loaded FROM
                        // this alloca. Skip the redundant extractvalue+store preamble
                        // (12 LLVM instructions per push: 4x extractvalue + 4x GEP +
                        // 4x store). In tight Vec push loops (e.g. t1-allocator init),
                        // this eliminates ~12M redundant instructions for 1M pushes.
                        if needs_store_back {
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
                        } // OPT-R5 end: skip redundant extractvalue+store for in-place alloca
                        // Load elem_size early -- needed to decide struct vs scalar path
                        let esz_gep = self.fresh_tmp();
                        let esz_val = {
                            let tn = val_ty.trim_start_matches("%struct.").trim_end_matches('*');
                            // BUG 42: enum elements (Vec[JsonValue]) are
                            // struct-likes too -- enum_variants, not types.
                            let is_struct_elem_here = val_ty.starts_with('%') && !val_ty.ends_with('*')
                                && self.is_struct_or_enum_type(&tn);
                            if is_struct_elem_here {
                                // BUG 34 (nested Vec[Vec[T]]): a STRUCT element
                                // (e.g. a Vec pushed into a Vec whose ctor
                                // defaulted elem_size to 8) needs the struct's
                                // REAL byte size for the slot math (grow + store
                                // offsets) - else a 32-byte element overwrites 4
                                // slots. Emit the size as a constant; the store
                                // block persists it into field 3 so later index
                                // reads use the same offsets.
                                // BUG 42: sizeof_struct under-counts ENUM
                                // fields (JsonValue in JsonEntry -> 8 instead
                                // of 16) and returns 0 for enum elements --
                                // vec_elem_storage_size knows the enum
                                // { i64 tag, i64 x slots } layout, container
                                // fields (Vec 32, Map/Set 64) and arrays.
                                let struct_size = self.vec_elem_storage_size(&tn);
                                let sz_tmp = self.fresh_tmp();
                                self.emitln(&format!("  {sz_tmp} = alloca i64"));
                                self.emitln(&format!("  store i64 {struct_size}, i64* {sz_tmp}"));
                                self.emitln(&format!("  {esz_gep} = load i64, i64* {sz_tmp}"));
                                esz_gep
                            } else {
                                let esz_reg = self.fresh_tmp();
                                self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
                                self.emitln(&format!("  {esz_reg} = load i64, i64* {esz_gep}"));
                                esz_reg
                            }
                        };
                        // For struct elements >8 bytes, skip val_to_i64 (which would
                        // heap-allocate) and use memcpy to store the struct inline.
                        let is_struct_elem = val_ty.starts_with('%') && {
                            let tn = val_ty.trim_start_matches("%struct.").trim_end_matches('*');
                            self.is_struct_or_enum_type(&tn)
                        };
                        let val = if !is_struct_elem {
                            self.val_to_i64(&val_raw, &val_ty)
                        } else {
                            // For structs, the raw value is preserved for memcpy.
                            // We emit a dummy i64; the store block will use val_raw directly.
                            val_raw.clone() // not used as i64 -- the store block checks is_struct_elem
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
                        // Capacity guard: trap if exceeding max (2^20 elements ~= 8MB)
                        let cap_ok_check = self.fresh_tmp();
                        let cap_ok_cont = self.fresh_block("vec_cap_ok");
                        let cap_trap_block = self.fresh_block("vec_cap_trap");
                            self.emitln(&format!("  {cap_ok_check} = icmp ule i64 {new_cap}, 16777216"));
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
                        let old_size_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {old_size_tmp} = mul i64 {cap_val}, {esz_val}"));
                        let new_data = self.emit_realloc(&grow_data_ptr, &old_size_tmp, &new_size);
                        // Null check on realloc -- trap on OOM
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
                        // BUG 34: persist the (possibly struct-sized) elem_size
                        // into field 3 so later index reads compute the same
                        // offsets (Vec.new() defaulted 8 for nested Vecs).
                        let esz_store_gep = self.fresh_tmp();
                        self.emitln(&format!("  {esz_store_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
                        self.emitln(&format!("  store i64 {esz_val}, i64* {esz_store_gep}"));
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
                        // R4 fix: Skip store-back when working directly on the
                        // receiver's original alloca (resolve_vec_push_ptr returned
                        // needs_store_back=false) -- the mutation is already in-place.
                        if needs_store_back {
                            self.store_back_to_receiver(receiver, &loaded, "%struct.Vec");
                        }
                        // BUG 34: record the NESTED element type on the pushed-to
                        // Vec so later INDEX reads (`bs[0]`, `bs[i].len()`) take
                        // the struct-element path (resolve_vec_elem_type). The
                        // ctor defaulted the type to Int, so without this the
                        // read inttoptr'd the element's first field as a header.
                        // BUG 37/36 follow-up (2026-08-17): only fire for
                        // NESTED-Vec receivers (Vec[Vec[T]]). For Vec[Struct]
                        // receivers the binding already recorded the struct
                        // element ("Item"); the arg0-based fallback here
                        // resolved the struct-literal arg to None -> "Int" and
                        // OVERWROTE it with "Vec[Int]", so `v[0]` memcpy'd the
                        // struct element into a %struct.Vec slot and field
                        // reads returned garbage (m21_vec_edge_012/027,
                        // m37_nested_vec, vec_of_struct, eco suites).
                        if is_struct_elem {
                            if let Expr::Ident(rid) = receiver.as_ref() {
                                let recv_elem = self.local.local_vec_elem.get(&rid.name).cloned();
                                // Record when the receiver's elem is a nested
                                // Vec OR UNRECORDED (`Vec[Vec[Int]].new()` --
                                // vec_ctor_elem_type can't resolve nested type
                                // args, so the ctor left it None and the push
                                // is the first chance to record "Vec[Int]").
                                // Skip only when a NON-Vec elem is recorded
                                // ("Item" -- the m21_vec_edge_012 overwrite bug).
                                let is_nested_vec = recv_elem.as_deref()
                                    .map_or(true, |e| e.starts_with("Vec["));
                                if is_nested_vec {
                                    if let Some(arg0) = args.first() {
                                        // The pushed value's OWN element type
                                        // (fr: Vec[Float64] -> "Float64");
                                        // resolve_vec_elem_type excludes
                                        // primitives so it alone fell back to
                                        // "Int" for Vec[Float64] args
                                        // (m37_nested_vec check 5).
                                        let inner = if let Expr::Ident(ai) = arg0 {
                                            self.local.local_vec_elem.get(&ai.name)
                                                .or_else(|| self.local.local_vec_handle.get(&ai.name))
                                                .cloned()
                                        } else {
                                            None
                                        };
                                        let inner = inner.unwrap_or_else(|| {
                                            self.resolve_vec_elem_type(arg0)
                                                .unwrap_or_else(|| "Int".to_string())
                                        });
                                        self.local.local_vec_elem.insert(rid.name.clone(), format!("Vec[{inner}]"));
                                    }
                                }
                            }
                        }
                        return Ok((loaded, "%struct.Vec".to_string()));
                        }
                    }
                }
                // Vec.clear() -- reset length to zero (inline builtin).
                if fn_name == "clear" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_vec = Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            let (hdr, needs_store_back) = self.resolve_vec_receiver_ptr(receiver)?;
                            let len_gep = self.fresh_tmp();
                            self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {hdr}, i32 0, i32 1"));
                            self.emitln(&format!("  store i64 0, i64* {len_gep}"));
                            if needs_store_back {
                                let loaded = self.emit_vec_load_fields(&hdr);
                                self.store_back_to_receiver(receiver, &loaded, "%struct.Vec");
                            }
                            return Ok(("0".to_string(), "void".to_string()));
                        }
                    }
                }
                // Vec.pop(vec) -- method call on Vec. Returns Option[T]: None when
                // empty (discriminant 0), else Some(last element) (discriminant 1,
                // value = element). A Vec is the builtin {i8*, i64, i64}; elements
                // are i64-wide slots. The pop reads the last live element; the
                // returned struct is a %struct.Option so `v.pop() == Some(x)` typechecks.
                // Vec.insert(idx, val) / Vec.remove(idx) -- inline builtins
                // (5c.29). The stdlib generic versions relied on general method
                // dispatch, which cannot resolve container-field receivers
                // (`tree.nodes[idx].keys.insert(...)` misdispatched to a stub).
                // Shifting uses llvm.memmove so it is element-size agnostic
                // (works for 32-byte struct elements too).
                if (fn_name == "insert" && args.len() == 2) || (fn_name == "remove" && args.len() == 1) {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_vec = Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            let is_insert = fn_name == "insert";
                            if !is_insert { self.types.used_builtins.insert("Option".to_string()); }
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
                                    self.types.types.contains_key(&tn.to_string())
                                        || self.types.types.keys().into_iter().any(|k| k.ends_with(&format!(".{tn}")))
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
                                let old_size2 = self.fresh_tmp();
                                self.emitln(&format!("  {old_size2} = mul i64 {cap}, {esz}"));
                                let new_data = self.emit_realloc(&gd_ptr, &old_size2, &new_size);
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
                        let is_vec = Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            self.types.used_builtins.insert("Option".to_string());
                            let (recv_val, recv_actual_ty) = self.compile_expr(receiver)?;
                            let (recv_vec, _) = self.resolve_vec_receiver(receiver, &recv_val, &recv_actual_ty);
                            let vec_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                            self.emit_vec_store_fields(&recv_vec, &vec_alloca);
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
                // BUG 55 facet-2 (2026-08-18): Vec.set(vec, idx, val) -- element
                // WRITE with elem_size scaling. Previously NOT inlined (get/
                // push/pop are) and NOT in generic_fn_decls (the stdlib's Vec
                // methods register only as erased signatures) -- the call
                // resolved to the bare "Vec.set" signature -> DIRECT call to
                // the never-emitted generic def -> zero-param auto-stub
                // (ret 0) -> the write silently dropped (cross_cmp_sort /
                // array.sort-by-swap family sorted nothing).
                if fn_name == "set" && args.len() == 2 {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_vec = Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            let (recv_val, recv_actual_ty) = self.compile_expr(receiver)?;
                            let (recv_vec, _) = self.resolve_vec_receiver(receiver, &recv_val, &recv_actual_ty);
                            let (idx_raw, idx_ty) = self.compile_expr(&args[0])?;
                            let idx = self.val_to_i64(&idx_raw, &idx_ty);
                            let (val_raw, val_ty) = self.compile_expr(&args[1])?;
                            let store_i64 = self.val_to_i64(&val_raw, &val_ty);
                            let vec_alloca = self.fresh_tmp();
                            self.emitln(&format!("  {vec_alloca} = alloca %struct.Vec"));
                            self.emit_vec_store_fields(&recv_vec, &vec_alloca);
                            let len_gep = self.fresh_tmp();
                            self.emitln(&format!("  {len_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 1"));
                            let len_val = self.fresh_tmp();
                            self.emitln(&format!("  {len_val} = load i64, i64* {len_gep}"));
                            let esz_gep = self.fresh_tmp();
                            let esz_val = self.fresh_tmp();
                            self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 3"));
                            self.emitln(&format!("  {esz_val} = load i64, i64* {esz_gep}"));
                            let data_gep = self.fresh_tmp();
                            let data_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {vec_alloca}, i32 0, i32 0"));
                            self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                            // Bounds check: skip the write when idx >= len
                            // (mirrors the stdlib body's `if index < 0 ||
                            // index >= len { return; }`).
                            let in_range = self.fresh_tmp();
                            self.emitln(&format!("  {in_range} = icmp ult i64 {idx}, {len_val}"));
                            let ok_block = self.fresh_block("vec_set_ok");
                            let done_block = self.fresh_block("vec_set_done");
                            self.emitln(&format!("  br i1 {in_range}, label %{ok_block}, label %{done_block}"));
                            self.emitln(&format!("\n{ok_block}:"));
                            let byte_off = self.fresh_tmp();
                            self.emitln(&format!("  {byte_off} = mul i64 {idx}, {esz_val}"));
                            let elem_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {data_ptr}, i64 {byte_off}"));
                            self.emit_elem_store(&store_i64, &elem_ptr, &esz_val);
                            // emit_elem_store's merge label needs a terminator
                            // before our done block (empty blocks are invalid).
                            self.emitln(&format!("  br label %{done_block}"));
                            self.emitln(&format!("\n{done_block}:"));
                            return Ok(("0".to_string(), "void".to_string()));
                        }
                    }
                }
                // 5c.39: Vec.sort() -- in-place insertion sort.
                if fn_name == "sort" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_vec = Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            self.emit_vec_sort(receiver, recv_ty)?;
                            return Ok(("0".to_string(), "void".to_string()));
                        }
                    }
                }
                // Vec.get(vec, idx) -- method call on Vec. Returns Option[T]: None
                // when idx is out of range (discriminant 0), else Some(data[idx])
                // (discriminant 1, value = element). Mirrors the pop lowering.
                if fn_name == "get" && args.len() == 1 {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_vec = Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || self.is_container_vec_field(receiver);
                        if is_vec {
                            self.types.used_builtins.insert("Option".to_string());
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
                            // in range iff (unsigned) idx < len -- also rejects idx<0.
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
                // Vec.len(vec) -- method call on Vec
                // G-36: Vec.clone() -- deep copy. New buffer (len*elem_size bytes),
                // memcpy the payload, fresh struct {newbuf, len, len, elem_size}.
                // Registered in the checker for Vec/Slice/Map/Set receivers.
                if fn_name == "clone" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        if Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || self.is_container_vec_field(receiver)
                        {
                            let (recv_raw, recv_raw_ty) = self.compile_expr(receiver)?;
                            let (recv_vec, vec_ty) = self.resolve_vec_receiver(receiver, &recv_raw, &recv_raw_ty);
                            let slot = self.fresh_tmp();
                            self.emitln(&format!("  {slot} = alloca {vec_ty}"));
                            self.emitln(&format!("  store {vec_ty} {recv_vec}, {vec_ty}* {slot}"));
                            // Load all 4 fields: data, len, cap, elem_size
                            let data_gep = self.fresh_tmp();
                            let data = self.fresh_tmp();
                            self.emitln(&format!("  {data_gep} = getelementptr {vec_ty}, {vec_ty}* {slot}, i32 0, i32 0"));
                            self.emitln(&format!("  {data} = load i8*, i8** {data_gep}"));
                            let len_gep = self.fresh_tmp();
                            let len = self.fresh_tmp();
                            self.emitln(&format!("  {len_gep} = getelementptr {vec_ty}, {vec_ty}* {slot}, i32 0, i32 1"));
                            self.emitln(&format!("  {len} = load i64, i64* {len_gep}"));
                            let es_gep = self.fresh_tmp();
                            let es = self.fresh_tmp();
                            self.emitln(&format!("  {es_gep} = getelementptr {vec_ty}, {vec_ty}* {slot}, i32 0, i32 3"));
                            self.emitln(&format!("  {es} = load i64, i64* {es_gep}"));
                            // bytes = len * elem_size; guard elem_size==0 -> treat as 8
                            let es_zero = self.fresh_tmp();
                            self.emitln(&format!("  {es_zero} = icmp eq i64 {es}, 0"));
                            let es_fixed = self.fresh_tmp();
                            self.emitln(&format!("  {es_fixed} = select i1 {es_zero}, i64 8, i64 {es}"));
                            let bytes = self.fresh_tmp();
                            self.emitln(&format!("  {bytes} = mul i64 {len}, {es_fixed}"));
                            // Allocate at least 1 byte so malloc(0) never returns null-ish edge
                            let bytes_zero = self.fresh_tmp();
                            self.emitln(&format!("  {bytes_zero} = icmp eq i64 {bytes}, 0"));
                            let alloc_bytes = self.fresh_tmp();
                            self.emitln(&format!("  {alloc_bytes} = select i1 {bytes_zero}, i64 1, i64 {bytes}"));
                            let newbuf = self.emit_alloc(&alloc_bytes);
                            self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {newbuf}, i8* {data}, i64 {bytes}, i1 false)"));
                            // Build the cloned struct: {newbuf, len, cap=len, elem_size}
                            let s0 = self.fresh_tmp();
                            self.emitln(&format!("  {s0} = insertvalue {vec_ty} undef, i8* {newbuf}, 0"));
                            let s1 = self.fresh_tmp();
                            self.emitln(&format!("  {s1} = insertvalue {vec_ty} {s0}, i64 {len}, 1"));
                            let s2 = self.fresh_tmp();
                            self.emitln(&format!("  {s2} = insertvalue {vec_ty} {s1}, i64 {len}, 2"));
                            let s3 = self.fresh_tmp();
                            self.emitln(&format!("  {s3} = insertvalue {vec_ty} {s2}, i64 {es_fixed}, 3"));
                            return Ok((s3, vec_ty));
                        }
                    }
                }
                // Vec.len(vec)
                if fn_name == "len" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_container = self.is_container_vec_field(receiver);
                        // M33: Detect Vec handles from Result.unwrap().
                        let is_unwrap_vec = !is_container && recv_ty == "i64"
                            && self.receiver_is_unwrap_of_vec(receiver);
                        // Also detect Vec/Slice field access where infer_llvm_type
                        // returns i64 (container handle) but compile_expr returns
                        // %struct.Vec -- avoid routing to Str.len() below.
                        let is_vec_field = is_container
                            || is_unwrap_vec
                            || Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || Self::is_llvm_struct_named(&recv_ty, "Slice")
                            || (recv_ty == "i64" && self.is_container_vec_field(receiver));
                        if !is_vec_field {
                            // Not a Vec/Slice receiver -- fall through to Str.len() below
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
                        return Ok((len_val, LLVM_I64.to_string()));
                        }
                    }
                }
                // Str.len(s) / Vec.len / Slice.len -- method call
                if fn_name == "len" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        // Skip this handler if the receiver is a Vec/Slice field --
                        // the Vec.len() handler above should have caught it.
                        if self.is_container_vec_field(receiver) {
                            // fall through to Vec/Slice path below (or to generic dispatch)
                        } else {
                        let recv_ty = self.infer_llvm_type(receiver);
                        // BUG 30 (utf8 ensure): a contract-ensure payload rebind
                        // (`result is Ok => result.len()`) binds a BOXED Vec
                        // HANDLE (i64) as the receiver. It must unbox to
                        // %struct.Vec and read field 1 - NOT go down the Str
                        // path (xiom_str_len on the boxed pointer -> AV) nor the
                        // generic dispatch (Map.len on the Result struct).
                        let is_vec_handle_local = if let Expr::Ident(id) = &**receiver {
                            self.local.local_xiom_types.get(&id.name)
                                .map_or(false, |t| t.contains("Vec[") || t.ends_with("]Vec") || t.ends_with(".Vec"))
                        } else { false };
                        // Check XIOM type: Str.len() should use xiom_str_len even when
                        // the receiver is i64 (Str pointers stored as i64 in ABI).
                        let is_str_type = if let Expr::Ident(id) = &**receiver {
                            self.local.local_xiom_types.get(&id.name)
                                .map_or(false, |t| t == "Str")
                        } else { false };
                        // 5c.30: Indexed Vec elements (e.g. outer[1] from Vec[Vec[Int]])
                        // return i64 but are NOT strings -- exclude them from the Str.len() path.
                        let is_vec_index = matches!(&**receiver, Expr::Index(..));
                        // Also skip module-level globals whose type is a named struct
                        // (like Map[K,V]) -- they're not strings.
                        let is_struct_global = if let Expr::Ident(id) = &**receiver {
                            self.local.module_globals.get(&id.name)
                                .map_or(false, |(_, ty)| ty.starts_with("%struct.") || ty.ends_with(".Map") || ty.ends_with(".Vec"))
                        } else { false };
                        if (recv_ty == "i8*" || recv_ty == "ptr" || (recv_ty == "i64" && !is_vec_index && !is_struct_global && !is_vec_handle_local) || is_str_type)
                            && !is_vec_index && !is_struct_global && !is_vec_handle_local
                        {
                            let (recv_val, _) = self.compile_expr(receiver)?;
                            let str_ptr = if recv_ty == "i64" {
                                let tmp = self.fresh_tmp();
                                self.emitln(&format!("  {tmp} = inttoptr i64 {recv_val} to i8*"));
                                tmp
                            } else { recv_val };
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = call i64 @xiom_str_len(i8* {str_ptr})"));
                            return Ok((tmp, LLVM_I64.to_string()));
                        }
                        // BUG 30: boxed Vec handle local (i64 slot, XIOM type Vec[..])
                        // - inttoptr to %struct.Vec*, load the header, read field 1.
                        if is_vec_handle_local && recv_ty == "i64" {
                            let (recv_val, _) = self.compile_expr(receiver)?;
                            let vp = self.fresh_tmp();
                            self.emitln(&format!("  {vp} = inttoptr i64 {recv_val} to %struct.Vec*"));
                            let vl = self.fresh_tmp();
                            self.emitln(&format!("  {vl} = load %struct.Vec, %struct.Vec* {vp}"));
                            let va = self.fresh_tmp();
                            self.emitln(&format!("  {va} = alloca %struct.Vec"));
                            self.emitln(&format!("  store %struct.Vec {vl}, %struct.Vec* {va}"));
                            let lg = self.fresh_tmp();
                            let lv = self.fresh_tmp();
                            self.emitln(&format!("  {lg} = getelementptr %struct.Vec, %struct.Vec* {va}, i32 0, i32 1"));
                            self.emitln(&format!("  {lv} = load i64, i64* {lg}"));
                            return Ok((lv, LLVM_I64.to_string()));
                        }
                        // Pointer-typed array references from monomorphised generics
                        // (e.g. &Slice[Int] -> i64*): length is at buf[0].
                        // round-9 (Set ABI): NOT when the pointee is a REGISTERED
                        // struct (&Set[Int] -> %struct.Set*): `s.len()` on a &Set
                        // param must dispatch to the Set.len generic method, not
                        // read the first field (the Vec's data pointer) as a length.
                        if recv_ty.ends_with('*') && recv_ty != "i8*" {
                            let pointee = &recv_ty[..recv_ty.len() - 1];
                            let pointee_clean = pointee.strip_prefix("%struct.").unwrap_or(pointee);
                            let is_struct_pointee = self.types.types.contains_key(&pointee_clean.to_string())
                                || self.types.type_meta.contains_key(&pointee_clean.to_string())
                                || self.types.type_meta.keys().into_iter().any(|k| k.ends_with(&format!(".{}", pointee_clean)));
                            if !is_struct_pointee {
                                let (recv_val, _) = self.compile_expr(receiver)?;
                                let tmp = self.fresh_tmp();
                                self.emitln(&format!("  {tmp} = load i64, {recv_ty} {recv_val}"));
                                return Ok((tmp, LLVM_I64.to_string()));
                            }
                        }
                        // Vec/Slice: length is field 1 of the {ptr, len, cap} struct.
                        if Self::is_llvm_struct_named(&recv_ty, "Vec")
                            || Self::is_llvm_struct_named(&recv_ty, "Slice")
                            || self.is_container_vec_field(receiver)
                            || (recv_ty == "i64" && matches!(&**receiver, Expr::Index(container, _, _)
                                if {
                                    let ct = self.infer_llvm_type(container);
                                    Self::is_llvm_struct_named(&ct, "Vec")
                                }))
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
                            return Ok((lenv, LLVM_I64.to_string()));
                        }
                        } // end else: not a container Vec field
                    }
                }
                // Str.c_str() -- identity on the string pointer (Str is already i8*).
                // Str.len() / Str.byte_len() -- return the string length.
                if (fn_name == "c_str" || fn_name == "byte_len" || fn_name == "len") && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_str = recv_ty == "i8*" || recv_ty.contains(".Str");
                        if fn_name == "c_str" {
                            let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                            let ptr = self.coerce_value(&recv_val, &recv_ty, "i8*");
                            return Ok((ptr, LLVM_STR_PTR.to_string()));
                        } else if is_str {
                            // .len() / .byte_len(): only for Str receivers
                            let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                            let ptr = self.coerce_value(&recv_val, &recv_ty, "i8*");
                            let len_tmp = self.fresh_tmp();
                            self.emitln(&format!("  {len_tmp} = call i64 @strlen(i8* {ptr})"));
                            return Ok((len_tmp, LLVM_I64.to_string()));
                        }
                    }
                }
                // Str.from_cstring(ptr) / from_c_str / from_utf8 -- reinterpret a
                // C string / byte buffer as a Str. A Str is `i8*` at the ABI and a
                // C string is already a NUL-terminated i8*, so this is an identity
                // on the pointer (coerced to i8*). Emitted inline since there is no
                // runtime function.
                // BUG 25 #3 fix: only fire when NO real fn with this name is
                // registered -- a user/module fn named `from_bytes` (or any of
                // these) was previously HIJACKED by the builtin intercept,
                // producing "invalid getelementptr indices" / wrong returns.
                let builtin_name_free = !self.types.functions.contains_key(&fn_name.to_string())
                    && !self.types.functions.entries().iter().any(|(k, _)| k.ends_with(&format!(".{fn_name}")));
                if builtin_name_free
                    && matches!(fn_name.as_str(), "from_cstring" | "from_c_str" | "from_utf8" | "from_bytes")
                    && !args.is_empty()
                {
                    let (arg_val, arg_ty) = self.compile_expr(&args[0])?;
                    // D1 hardening: Vec[UInt8] data is NOT NUL-terminated --
                    // copy to a terminated buffer (was returning the raw data
                    // pointer, causing reads past the buffer into adjacent
                    // memory: intermittent garbage suffixes in decoded strings).
                    if matches!(fn_name.as_str(), "from_utf8" | "from_bytes")
                        && arg_ty.starts_with("%struct.")
                    {
                        let vec_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {vec_tmp} = alloca {arg_ty}, align 16"));
                        self.emitln(&format!("  store {arg_ty} {arg_val}, {arg_ty}* {vec_tmp}, align 16"));
                        let data_gep = self.fresh_tmp();
                        self.emitln(&format!("  {data_gep} = getelementptr {arg_ty}, {arg_ty}* {vec_tmp}, i32 0, i32 0"));
                        let data_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                        let len_gep = self.fresh_tmp();
                        self.emitln(&format!("  {len_gep} = getelementptr {arg_ty}, {arg_ty}* {vec_tmp}, i32 0, i32 1"));
                        let len_val = self.fresh_tmp();
                        self.emitln(&format!("  {len_val} = load i64, i64* {len_gep}"));
                        let str_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {str_tmp} = call i8* @xiom_str_from_vec(i8* {data_ptr}, i64 {len_val})"));
                        return Ok((str_tmp, LLVM_STR_PTR.to_string()));
                    }
                    let as_ptr = self.coerce_value(&arg_val, &arg_ty, "i8*");
                    return Ok((as_ptr, LLVM_STR_PTR.to_string()));
                }
                // M12/P1: Str.slice(start, end) -- substring extraction.
                // Delegates to xiom.string.str_slice via normal function dispatch.
                // Handled here to short-circuit method resolution for the Str receiver.
                if fn_name == "slice" && args.len() == 2 {
                    if let Some(receiver) = receiver_expr {
                        if self.receiver_is_str(receiver) {
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        let recv_ptr = self.val_to_i8ptr(&recv_val, &recv_ty);
                        let (start_val, _) = self.compile_expr(&args[0])?;
                        let (end_val, _) = self.compile_expr(&args[1])?;
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i8* @xiom_str_slice(i8* {recv_ptr}, i64 {start_val}, i64 {end_val})"));
                        return Ok((tmp, LLVM_STR_PTR.to_string()));
                        }
                    }
                }
                // Round-6 fix (2026-08-19): Str.substr(start, end) -- the stdlib's
                // substring form (io.parent_path, path.file_name/extension use
                // `s.substr(0, i)`). It was registered as a builtin Str method but
                // had NO inline handler: the call fell through to normal method
                // dispatch, emitted `call i64 @Str.substr(...)` against a def that
                // never exists -- zero-param stub `ret i64 0` -- NULL string --
                // Option[Str] payloads of 0 -- 0xC0000005 in every parent_path /
                // file_name / extension user (the stdlib's "Option[Str] pointer-
                // payload construction mangles" report). Same lowering as slice.
                if fn_name == "substr" && args.len() == 2 {
                    if let Some(receiver) = receiver_expr {
                        if self.receiver_is_str(receiver) {
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        let recv_ptr = self.val_to_i8ptr(&recv_val, &recv_ty);
                        let (start_val, _) = self.compile_expr(&args[0])?;
                        let (end_val, _) = self.compile_expr(&args[1])?;
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i8* @xiom_str_slice(i8* {recv_ptr}, i64 {start_val}, i64 {end_val})"));
                        return Ok((tmp, LLVM_STR_PTR.to_string()));
                        }
                    }
                }
                // M12/P1: Str.starts_with(prefix) -- prefix check via string.xi.
                if fn_name == "starts_with" && args.len() == 1 {
                    if let Some(receiver) = receiver_expr {
                        if self.receiver_is_str(receiver) {
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        let recv_ptr = self.val_to_i8ptr(&recv_val, &recv_ty);
                        let (prefix_val, prefix_ty) = self.compile_expr(&args[0])?;
                        let prefix_ptr = self.val_to_i8ptr(&prefix_val, &prefix_ty);
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i1 @xiom_str_starts_with(i8* {recv_ptr}, i8* {prefix_ptr})"));
                        return Ok((tmp, "i1".to_string()));
                        }
                    }
                }
                // M12/P1: Str.ends_with(suffix) -- suffix check.
                if fn_name == "ends_with" && args.len() == 1 {
                    if let Some(receiver) = receiver_expr {
                        if self.receiver_is_str(receiver) {
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        let recv_ptr = self.val_to_i8ptr(&recv_val, &recv_ty);
                        let (suffix_val, suffix_ty) = self.compile_expr(&args[0])?;
                        let suffix_ptr = self.val_to_i8ptr(&suffix_val, &suffix_ty);
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i1 @xiom_str_ends_with(i8* {recv_ptr}, i8* {suffix_ptr})"));
                        return Ok((tmp, "i1".to_string()));
                        }
                    }
                }
                // M21: Str.concat(other) -- string concatenation via xiom_str_concat.
                if fn_name == "concat" && args.len() == 1 {
                    if let Some(receiver) = receiver_expr {
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        let recv_ptr = self.val_to_i8ptr(&recv_val, &recv_ty);
                        let (other_val, other_ty) = self.compile_expr(&args[0])?;
                        let other_ptr = self.val_to_i8ptr(&other_val, &other_ty);
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i8* @xiom_str_concat(i8* {recv_ptr}, i8* {other_ptr})"));
                        return Ok((tmp, LLVM_STR_PTR.to_string()));
                    }
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
                    return Ok(("0".to_string(), LLVM_I64.to_string()));
                }
                // BUG 27: debug intrinsics -- `dbg!(expr)`, `todo!()`,
                // `unimplemented!()`. Only when no user fn with the name is
                // registered (a user `dbg`/`todo` fn wins over the builtin).
                let dbg_builtin_free = !self.types.functions.contains_key(&fn_name)
                    && !self.types.functions.entries().iter().any(|(k, _)| k.ends_with(&format!(".{fn_name}")));
                if dbg_builtin_free && fn_name == "dbg" && compiled_args.len() == 1 {
                    let (arg_val, arg_ty) = (&compiled_args[0].0, &compiled_args[0].1);
                    // Security review (2026-08-13): release builds strip the
                    // dbg! print -- the EXPRESSION still evaluates and returns
                    // its value (dbg!(x) is an expression), only the output
                    // disappears (Rust debug_assert! policy).
                    if !self.config.strip_debug_checks {
                        // Format the value by type, prefix "[dbg] ", print.
                    let formatted = if arg_ty == "i64" {
                        let f = self.fresh_tmp();
                        self.emitln(&format!("  {f} = call i8* @xiom_int_to_string(i64 {arg_val})"));
                        f
                    } else if arg_ty == "double" || arg_ty == "float" || arg_ty == "fp128" {
                        let fv = if arg_ty == "float" {
                            let ext = self.fresh_tmp();
                            self.emitln(&format!("  {ext} = fpext float {arg_val} to double"));
                            ext
                        } else {
                            arg_val.clone()
                        };
                        let f = self.fresh_tmp();
                        self.emitln(&format!("  {f} = call i8* @xiom_double_to_string(double {fv})"));
                        f
                    } else if arg_ty == "i8*" {
                        arg_val.clone()
                    } else if arg_ty == "i1" {
                        let t = self.intern_cstring("true");
                        let f = self.intern_cstring("false");
                        let sel = self.fresh_tmp();
                        self.emitln(&format!("  {sel} = select i1 {arg_val}, i8* {t}, i8* {f}"));
                        sel
                    } else {
                        self.intern_cstring("<value>")
                    };
                    let prefix = self.intern_cstring("[dbg] ");
                    let out = self.fresh_tmp();
                    self.emitln(&format!("  {out} = call i8* @xiom_str_concat(i8* {prefix}, i8* {formatted})"));
                    self.emitln(&format!("  call i32 @puts(i8* {out})"));
                    }
                    return Ok((arg_val.clone(), arg_ty.clone()));
                }
                if dbg_builtin_free && (fn_name == "todo" || fn_name == "unimplemented") {
                    let loc = if let Expr::Ident(id) = func {
                        format!("{}!() at {}:{}", fn_name, id.span.line, id.span.col)
                    } else {
                        format!("{}!()", fn_name)
                    };
                    let msg = self.intern_cstring(&loc);
                    self.emitln(&format!("  call void @xiom_panic(i8* {msg})"));
                    self.emitln("  unreachable");
                    return Ok(("0".to_string(), LLVM_I64.to_string()));
                }
                // v0.56 I3: Mutex builtins for thread synchronization
                let is_mutex_fn = matches!(fn_name.as_str(), "Mutex.new" | "Mutex.lock" | "Mutex.unlock" | "Mutex.destroy");
                // v0.56: Numeric conversion builtins -- to_float (sitofp) and to_int (fptosi).
                // Only apply when the user has NOT defined their own to_float/to_int
                // function (e.g. `fn to_int(b: Bool) -> Int`). Otherwise the builtin
                // hijacks the user call and emits a malformed `fptosi double ...`.
                let user_defined_to_int = self.types.functions.keys().into_iter().any(|k| k == "to_int" || k.ends_with(".to_int"));
                let user_defined_to_float = self.types.functions.keys().into_iter().any(|k| k == "to_float" || k.ends_with(".to_float"));
                if fn_name == "to_float" && !user_defined_to_float && compiled_args.len() == 1 {
                    let (arg, arg_ty) = (&compiled_args[0].0, &compiled_args[0].1);
                    let arg_w = self.widen_to_i64(arg, arg_ty);
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = sitofp i64 {arg_w} to double"));
                    return Ok((tmp, "double".to_string()));
                }
                if fn_name == "to_int" && !user_defined_to_int && compiled_args.len() == 1 {
                    let (arg, _) = (&compiled_args[0].0, &compiled_args[0].1);
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = fptosi double {arg} to i64"));
                    return Ok((tmp, LLVM_I64.to_string()));
                }
                if fn_name == "to_int_from_char" && compiled_args.len() == 1 {
                    // Char -> Int: identity (already i64 in XIOM)
                    let (arg, _) = (&compiled_args[0].0, &compiled_args[0].1);
                    return Ok((arg.clone(), LLVM_I64.to_string()));
                }
                if fn_name == "to_char" && compiled_args.len() == 1 {
                    // Int -> Char: identity (already i64 in XIOM)
                    let (arg, _) = (&compiled_args[0].0, &compiled_args[0].1);
                    return Ok((arg.clone(), LLVM_I64.to_string()));
                }
                if is_mutex_fn && fn_name == "Mutex.new" {
                    let tmp = self.emit_alloc("64");
                    self.emitln(&format!("  call void @xiom_mutex_init(i8* {tmp})"));
                    return Ok((tmp, "i8*".to_string()));
                } else if is_mutex_fn {
                    let ptr = compiled_args.first().map(|(v, _)| v.clone()).unwrap_or_else(|| "null".to_string());
                    let fn_impl = match fn_name.as_str() {
                        "Mutex.lock" => "xiom_mutex_lock",
                        "Mutex.unlock" => "xiom_mutex_unlock",
                        _ => "xiom_mutex_destroy",
                    };
                    self.emitln(&format!("  call void @{fn_impl}(i8* {ptr})"));
                    return Ok((String::new(), "void".to_string()));
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
                    return Ok((tmp_int, LLVM_I64.to_string()));
                }
                if fn_name == "xiom_file_size" {
                    let tmp = self.fresh_tmp();
                    if let Some(path_arg) = args.first() {
                        let (path_ptr, _) = self.compile_expr(path_arg)?;
                        self.emitln(&format!("  {tmp} = call i64 @xiom_file_size(i8* {path_ptr})"));
                    } else {
                        self.emitln(&format!("  {tmp} = call i64 @xiom_file_size(i8* null)"));
                    }
                    return Ok((tmp, LLVM_I64.to_string()));
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
                    return Ok((tmp_ext, LLVM_I64.to_string()));
                }
                if fn_name == "xiom_str_len" && args.len() >= 1 {
                    let (src, src_ty) = self.compile_expr(&args[0])?;
                    let tmp = self.fresh_tmp();
                    let src_ptr = self.val_to_i8ptr(&src, &src_ty);
                    self.emitln(&format!("  {tmp} = call i64 @xiom_str_len(i8* {src_ptr})"));
                    return Ok((tmp, LLVM_I64.to_string()));
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
                        return Ok((len_val, LLVM_I64.to_string()));
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
                // Builtin offset(ptr, idx): pointer arithmetic.
                // Returns ptr + idx as a byte-offset pointer. Used by stdlib
                // io.read_file (*(ptr.offset(i))) to index into raw buffers.
                // Handles both real pointers (i8*) and ptrtoint'd i64 pointers.
                if fn_name == "offset" && args.len() >= 1 {
                    if let Some(receiver) = receiver_expr {
                        let (ptr_val, ptr_ty) = self.compile_expr(receiver)?;
                        let (idx_val, idx_ty) = self.compile_expr(&args[0])?;
                        let idx = self.val_to_i64(&idx_val, &idx_ty);
                        // Case 1: real pointer (i8* or T*)
                        if ptr_ty.ends_with('*') {
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = getelementptr i8, {ptr_ty} {ptr_val}, i64 {idx}"));
                            return Ok((tmp, "i8*".to_string()));
                        }
                        // Case 2: ptrtoint'd pointer (i64) -- add offset and return as i64
                        if ptr_ty == "i64" {
                            let tmp = self.fresh_tmp();
                            self.emitln(&format!("  {tmp} = add i64 {ptr_val}, {idx}"));
                            return Ok((tmp, "i64".to_string()));
                        }
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
                // Builtin size_of[T]() / sizeof[T](): return the LLVM size in bytes of type T.
                // size_of uses struct_byte_size (field-count x 8, XIOM-semantic).
                // sizeof uses sizeof_struct (precise LLVM widths, C-FFI-compatible).
                // align_of[T](): return alignment (8 for structs, else size).
                if fn_name == "size_of" || fn_name == "sizeof" || fn_name == "align_of" {
                    // 5e.3: type_arg may be None when idx_is_type fails or when
                    // the parser produces size_of(T) as a regular call instead of
                    // size_of[T]() as a generic instantiation. Try all sources.
                    let ta: Option<&Expr> = type_arg
                        .or_else(|| args.first())
                        .or_else(|| {
                            match func {
                                Expr::Index(_, idx, _) => Some(idx.as_ref()),
                                _ => None,
                            }
                        });
                    // 5e.3: parser drops nested generic type args like
                    // RcInner[T] in size_of[RcInner[T]](). When ta is None,
                    // check if we're inside a monomorphised Rc/RcInner context
                    // and compute the size of the base struct directly.
                    if ta.is_none() {
                        if let Some(ref current_fn) = self.fctx.current_fn {
                            if current_fn.contains("RcInner") || current_fn.contains("Rc.new_") || current_fn.contains("Rc.drop_") || current_fn.contains("Weak.drop_") {
                                let sz = if fn_name == "sizeof" {
                                    self.sizeof_struct("RcInner") as i64
                                } else {
                                    self.struct_byte_size("RcInner")
                                };
                                if sz > 0 {
                                    if fn_name == "align_of" { return Ok(("8".to_string(), LLVM_I64.to_string())); }
                                    return Ok((sz.to_string(), LLVM_I64.to_string()));
                                }
                            }
                        }
                    }
                    if let Some(ta) = ta {
                        let xiom_ty = match ta {
                            Expr::Ident(id) => id.name.clone(),
                            Expr::Field(_, f, _) => f.name.clone(),
                            // 5e.3: parameterized types like RcInner[T] parse as
                            // Expr::Index(Expr::Ident("RcInner"), Expr::Ident("T")).
                            // Extract the base type name so size_of/align_of can
                            // resolve it through type_meta (G-18, RC fix).
                            Expr::Index(base, _idx, _) => match base.as_ref() {
                                Expr::Ident(id) => id.name.clone(),
                                _ => String::new(),
                            },
                            _ => String::new(),
                        };
                        if !xiom_ty.is_empty() {
                            let llvm_ty = self.llvm_type_for(&xiom_ty)
                                .unwrap_or_else(|_| {
                                    Self::xiom_to_llvm_type(&xiom_ty).to_string()
                                });
                            let size = if llvm_ty.starts_with("%struct.") {
                                let type_name = llvm_ty[8..].to_string();
                                if fn_name == "sizeof" {
                                    // 5e.1 G-18: precise LLVM byte widths for C FFI.
                                    // i8=1, i16=2, i32=4, i64=8 -- matches C ABI sizes.
                                    self.sizeof_struct(&type_name) as i64
                                } else {
                                    self.struct_byte_size(&type_name)
                                }
                            } else {
                                match llvm_ty.as_str() {
                                    "i1" | "i8" => 1,
                                    "i16" => 2,
                                    "i32" | "float" => 4,
                                    "i64" | "double" | "i8*" | "ptr" => 8,
                                    _ => 8,
                                }
                            };
                            if fn_name == "align_of" {
                                let align = if llvm_ty.starts_with("%struct.") { 8 } else { size };
                                return Ok((align.to_string(), LLVM_I64.to_string()));
                            }
                            return Ok((size.to_string(), LLVM_I64.to_string()));
                        }
                    }
                    return Ok(("8".to_string(), LLVM_I64.to_string()));
                }
                // Enum variant constructor: `TypeName.Variant(args)`.
                // Detects when the call is constructing an enum variant and emits
                // the proper discriminant + payload struct.
                if let Some(receiver) = receiver_expr {
                    if let Expr::Ident(type_id) = &**receiver {
                        let enum_key = self.types.enum_variants.keys().into_iter()
    .find(|k| *k == type_id.name || k.ends_with(&format!(".{}", type_id.name)));
                        if let Some(ek) = enum_key {
                            if let Some(variants) = self.types.enum_variants.get(&ek) {
                                let var_info: Option<(usize, Vec<String>)> = variants.iter().enumerate()
                                    .find(|(_, (v, _))| v == &fn_name)
                                    .map(|(idx, (_, fields))| (idx, fields.clone()));
                                if let Some((var_idx, payload_fields)) = var_info {
                                    // Gather parent field layout BEFORE mutating self.
                                    let parent_field_names = self.types.types.get(&ek).unwrap_or_default();
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
                // Inline Option.is_some() / Option.is_none() / Result.is_ok() / Result.is_err()
                // for concrete types (Option__Point, Result__JsonValue__SerializeError etc.)
                // so the auto-stub generator doesn't create dead stubs returning 0 (B-001).
                if (fn_name == "is_some" || fn_name == "is_none"
                    || fn_name == "is_ok" || fn_name == "is_err") && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        // Use the compiled type (not infer_llvm_type) so concrete
                        // types like %struct.Result__X__Y are recognized, not just
                        // the generic %struct.Result / %struct.Option.
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        if recv_ty.contains("Option__") || recv_ty.contains("Result__")
                            || recv_ty.contains("Option.") || recv_ty.contains("Result.") {
                            let struct_ty = recv_ty.clone();
                            let alloca = self.fresh_tmp();
                            self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                            self.emitln(&format!("  store {struct_ty} {recv_val}, {struct_ty}* {alloca}"));
                            let disc_gep = self.fresh_tmp();
                            self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                            let disc = self.fresh_tmp();
                            self.emitln(&format!("  {disc} = load i64, i64* {disc_gep}"));
                            if fn_name == "is_none" || fn_name == "is_err" {
                                let neg = self.fresh_tmp();
                                self.emitln(&format!("  {neg} = xor i64 {disc}, 1"));
                                return Ok((neg, LLVM_I64.to_string()));
                            }
                            return Ok((disc, LLVM_I64.to_string()));
                        }
                    }
                }

                // Inline Option.unwrap() / Result.unwrap() / Result.unwrap_err()
                // and Option.unwrap_or() / Result.unwrap_or()
                // when called as a method on a known Option/Result value.
                let is_unwrap_like = (fn_name == "unwrap" || fn_name == "unwrap_err") && args.is_empty();
                let is_unwrap_or = fn_name == "unwrap_or" && args.len() == 1;
                if is_unwrap_like || is_unwrap_or {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        let is_option = recv_ty == "%struct.Option"
                            || recv_ty.ends_with(".Option")
                            || recv_ty.contains("Option__"); // concrete Option__Point etc.
                        let is_result = recv_ty == "%struct.Result"
                            || recv_ty.ends_with(".Result")
                            || recv_ty.contains("Result__"); // concrete Result__X__Y etc.
                        if is_option || is_result {
                            if is_option { self.types.used_builtins.insert("Option".to_string()); }
                            else { self.types.used_builtins.insert("Result".to_string()); }
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
                            if fn_name == "unwrap_err" || is_option || fn_name == "unwrap_or" {
                                // unwrap: expect disc != 0 (Some/Ok); unwrap_err: expect disc == 0 (Err)
                                // unwrap_or: same as unwrap (disc != 0) but with phi-fallback
                                let ok_cond = if fn_name == "unwrap_err" { "eq" } else { "ne" };
                                let ok = self.fresh_tmp();
                                self.emitln(&format!("  {ok} = icmp {ok_cond} i64 {disc}, 0"));
                                let ok_block = self.fresh_block("unwrap_ok");
                                let fail_block = self.fresh_block("unwrap_fail");
                                self.emitln(&format!("  br i1 {ok}, label %{ok_block}, label %{fail_block}"));
                                self.emitln(&format!("\n{fail_block}:"));
                                if fn_name == "unwrap_or" {
                                    // Return the default arg (unwrap_or fallback)
                                    let (default_val, default_ty) = self.compile_expr(&args[0])?;
                                    let done_label = format!("{}_done", fail_block);
                                    self.emitln(&format!("  br label %{done_label}"));
                                    self.emitln(&format!("\n{ok_block}:"));
                                    let val_field = 1;
                                    let type_name = struct_ty.trim_start_matches("%struct.");
                                    let field_ty = self.field_llvm_type(type_name, val_field);
                                    let val_gep = self.fresh_tmp();
                                    self.emitln(&format!("  {val_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {val_field}"));
                                    let payload = self.fresh_tmp();
                                    self.emitln(&format!("  {payload} = load {field_ty}, {field_ty}* {val_gep}"));
                                    // Coerce default_val to match the payload type if needed
                                    let default_val = if default_ty != field_ty && (default_ty == "i64" || field_ty == "i64") {
                                        let coerced = self.fresh_tmp();
                                        if field_ty == "i64" { 
                                            self.emitln(&format!("  {coerced} = ptrtoint {default_ty} {default_val} to i64"));
                                        } else {
                                            self.emitln(&format!("  {coerced} = inttoptr i64 {default_val} to {field_ty}"));
                                        }
                                        coerced
                                    } else { default_val };
                                    self.emitln(&format!("  br label %{done_label}"));
                                    self.emitln(&format!("\n{done_label}:"));
                                    let phi = self.fresh_tmp();
                                    self.emitln(&format!("  {phi} = phi {field_ty} [ {default_val}, %{fail_block} ], [ {payload}, %{ok_block} ]"));
                                    return Ok((phi, field_ty.to_string()));
                                }
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
                            // 5d: Typed payload extraction from tracked declared types.
                            // `let r = f()` where f -> Result[T, E] records T in
                            // local_opt_payload and E in local_err_payload. Use them
                            // so `.unwrap()`/`.unwrap_err()` return properly typed
                            // values instead of raw i64 (fixes msg.len() -> @len
                            // miscompile on Str payloads).
                            if field_ty == "i64" {
                                let declared: Option<String> = if let Expr::Ident(rid) = receiver.as_ref() {
                                    if fn_name == "unwrap_err" {
                                        self.local.local_err_payload.get(&rid.name).cloned()
                                    } else {
                                        self.local.local_opt_payload.get(&rid.name).cloned()
                                    }
                                } else { None };
                                if let Some(decl_ty) = declared {
                                    if decl_ty == "Str" {
                                        let sptr = self.fresh_tmp();
                                        self.emitln(&format!("  {sptr} = inttoptr i64 {val} to i8*"));
                                        return Ok((sptr, LLVM_STR_PTR.to_string()));
                                    }
                                    if decl_ty == "Float64" {
                                        let f = self.fresh_tmp();
                                        self.emitln(&format!("  {f} = bitcast i64 {val} to double"));
                                        return Ok((f, "double".to_string()));
                                    }
                                }
                            }
                            // M19: For non-struct, non-i64 field types (e.g., i8* for
                            // Str, double for Float64, float for Float32), return the
                            // value directly with its actual LLVM type. Previously these
                            // fell through to val_to_i64 which corrupted the pointer
                            // (ptrtoint round-trip), causing io.read_file().unwrap() to
                            // return an empty string (is_ok=true but unwrap=empty).
                            if field_ty != "i64" {
                                return Ok((val, field_ty.to_string()));
                            }
                            // When field_ty is i64, the payload may be a heap pointer
                            // from val_to_i64 for struct payloads.  Determine the actual
                            // struct type by resolving the generic return type of the
                            // concrete instantiation (e.g. `Option.unwrap[Point] -> Point`).
                            let struct_type_hint: Option<String> = {
                                let fn_key = if is_option { "Option.unwrap" } else { "Result.unwrap" };
                                self.mono.generic_fn_decls.iter().find(|(k, _)| k == fn_key || k.ends_with(&format!(".{}", fn_name)))
                                    .and_then(|(_, fd)| fd.return_type.as_ref().map(|t| Self::type_from_ast(t)))
                            };
                            if let Some(ref hint) = struct_type_hint {
                                // hint is the XIOM type name (e.g. "Point" for T=Point).
                                // Convert to LLVM struct type.
                                let struct_llvm = if self.types.types.contains_key(&hint.to_string()) || self.types.type_meta.contains_key(&hint.to_string()) {
                                    format!("%struct.{hint}")
                                } else {
                                    // Check if it resolves via type_meta
                                    let full_key = self.types.type_meta.keys().into_iter().find(|k| k.ends_with(&format!(".{hint}")));
                                    match full_key {
                                        Some(k) => format!("%struct.{k}"),
                                        None => return Ok((self.val_to_i64(&val, &field_ty), LLVM_I64.to_string())),
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
                            return Ok((result, LLVM_I64.to_string()));
                        }
                    }
                }
                // Check if this is a call to a generic function and track instantiation
                let fn_key = if let Some(receiver) = receiver_expr {
                    // round-10 (Bounded tower): a GENERIC-PARAM receiver
                    // (`T.max_value()` inside a mono'd generic body) is NOT a
                    // value instance nor a registered type -- resolve the param
                    // to its concrete binding FIRST so the fn_key becomes
                    // "Int.max_value" (the bare "max_value" leaf resolved to a
                    // zero-param stub -> 0, so checked_add's overflow guards saw
                    // max_value = 0 and always returned None).
                    if let Expr::Ident(id) = &**receiver {
                        if let Some(concrete) = self.mono.current_type_map.get(&id.name)
                            .or_else(|| self.mono.param_concrete_types.get(&id.name))
                        {
                            format!("{}.{}", concrete, fn_name)
                        } else if let Some(recv_type) = self.infer_struct_type_name(receiver) {
                            format!("{}.{}", recv_type, fn_name)
                        } else if self.receiver_is_instance(receiver) {
                            // Scalar value instance receiver (e.g. value.hash(hasher)
                            // inside a generic monomorphised body).
                            let obj_var_name = id.name.clone();
                            if let Some(concrete) = self.mono.param_concrete_types.get(&obj_var_name) {
                                format!("{}.{}", concrete, fn_name)
                            } else if let Some((_, llvm_ty)) = self.lookup_local(&obj_var_name) {
                                let ty_name = Self::xiom_type_name_from_llvm(llvm_ty);
                                format!("{}.{}", ty_name, fn_name)
                            } else {
                                self.resolve_module_call(receiver, &fn_name)
                            }
                        } else {
                            // Receiver is a module name (not a struct type).
                            self.resolve_module_call(receiver, &fn_name)
                        }
                    } else if let Some(recv_type) = self.infer_struct_type_name(receiver) {
                        format!("{}.{}", recv_type, fn_name)
                    } else if self.receiver_is_instance(receiver) {
                        let obj_var_name = match &**receiver {
                            Expr::Ident(id) => id.name.clone(),
                            _ => String::new(),
                        };
                        if !obj_var_name.is_empty() {
                            if let Some(concrete) = self.mono.param_concrete_types.get(&obj_var_name) {
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
                        // Receiver is a module name (not a struct type).
                        self.resolve_module_call(receiver, &fn_name)
                    }
                } else {
                    fn_name.clone()
                };
                // Rewrite bare internal stdlib calls (e.g. `args()` inside
                // env.args_os) to the leaf-qualified key registered for injected
                // free fns. The injected decls carry qualified names
                // ("env.args") so the emitted symbol matches the definition;
                // bare keys resolve through the keep-first alias map.
                // BUG 16/18 fix (2026-08-11): a bare call must prefer the
                // CALLER's own module first -- `_slot(...)` inside skiplist.xi
                // must resolve to `skiplist._slot`, not the first-registered
                // module's `_slot` (keep-first aliasing picked trie._slot when
                // both modules were linked -> wrong function -> garbage index ->
                // stack-buffer-overrun fast-fail 0xC0000409 at exit).
                let fn_key = if !fn_key.contains('.') {
                    // BUG 16/18 fix (2026-08-11): a bare call must prefer the
                    // CALLER's own module when no bare definition exists --
                    // `_slot(...)` inside skiplist.xi resolves to `skiplist._slot`,
                    // not the first-registered module's `_slot` (keep-first
                    // aliasing picked trie._slot when both modules were linked ->
                    // wrong function -> garbage index -> 0xC0000409 at exit).
                    // CRITICAL: when a BARE definition IS registered (user-module
                    // fns emit bare symbols -- fn_symbol dedup), the call MUST use
                    // the bare key -- the leaf-qualified key only exists as a
                    // resolution alias and would hit emit_undefined_symbol_stubs
                    // (zero-param stub -> ABI mismatch -> crash).
                    if self.types.functions.contains_key(&fn_key) || self.mono.emitted_fns.contains(&fn_key) {
                        fn_key
                    } else {
                        let caller_module = self.fctx.current_fn.as_ref()
                            .and_then(|k| k.rsplit_once('.'))
                            .map(|(m, _)| m.to_string())
                            .or_else(|| self.local.current_module.clone());
                        let in_caller_module = caller_module
                            .map(|m| format!("{m}.{fn_key}"))
                            .filter(|k| self.types.functions.contains_key(k) || self.mono.emitted_fns.contains(k));
                        if let Some(qualified) = in_caller_module {
                            qualified
                        } else if let Some(qualified) = self.mono.bare_fn_aliases.get(&fn_key) {
                            qualified.clone()
                        } else if let Some(aliased) = self.mono.use_alias_map.get(&fn_key) {
                            // BUG 25 #2 fix: `use X.Y.f as alias;` -- the alias
                            // resolves through the MODULE-CALL machinery, which
                            // registers the callee's signature on demand (the fn
                            // may not be in types.functions yet at preassign).
                            let parts: Vec<&str> = aliased.split('.').collect();
                            if parts.len() >= 2 {
                                let module_ident = Ident::new(parts[0], Span::new(0, 0));
                                let resolved = self.resolve_module_call(&Expr::Ident(module_ident), parts[1]);
                                if !resolved.is_empty() {
                                    resolved
                                } else {
                                    aliased.clone()
                                }
                            } else {
                                aliased.clone()
                            }
                        } else {
                            fn_key
                        }
                    }
                } else {
                    // BUG 22 #11 fix (qualified-call side of the BUG 12-18
                    // fn-key fix): the module-qualified key may NOT be
                    // registered while the DEFINITION exists under the BARE
                    // leaf -- fn_symbol dedup emits the bare symbol for the
                    // first same-named fn. Resolve to the bare leaf when it
                    // is the registered/emitted definition; otherwise
                    // emit_undefined_symbol_stubs creates a zero-param
                    // @qualified stub and the call returns garbage (0).
                    // Safe: the fallback only fires when NO qualified
                    // registration exists, i.e. exactly one (bare) def.
                    let bare_leaf = fn_key.rsplit('.').next().unwrap_or(&fn_key).to_string();
                    if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() {
                        eprintln!("[fnkeyq] key={fn_key} bare={bare_leaf} reg_q={} reg_b={}",
                            self.types.functions.contains_key(&fn_key),
                            self.types.functions.contains_key(&bare_leaf));
                    }
                    if !(self.types.functions.contains_key(&fn_key) || self.mono.emitted_fns.contains(&fn_key))
                        && (self.types.functions.contains_key(&bare_leaf) || self.mono.emitted_fns.contains(&bare_leaf))
                    {
                        bare_leaf
                    } else if !(self.types.functions.contains_key(&fn_key) || self.mono.emitted_fns.contains(&fn_key))
                        && !fn_key.contains('[')
                        && !bare_leaf.is_empty()
                        && !self.types.functions.contains_key(&bare_leaf)
                        && !self.mono.emitted_fns.contains(&bare_leaf)
                    {
                        // round-10 (Bounded tower): the DEFINITION may be
                        // MODULE-QUALIFIED with the receiver suffix --
                        // `T.max_value()` inside a mono'd generic resolves
                        // fn_key "Int.max_value" while the impl registers as
                        // "precision.Int.max_value". Resolve via the
                        // ".{Recv}.{method}" suffix (unique when it exists);
                        // otherwise keep the key and let the stub machinery
                        // produce a diagnostic.
                        let suffix = format!(".{}", fn_key);
                        let hit = self.types.functions.keys().into_iter()
                            .find(|k| k.ends_with(&suffix) && !k.starts_with("Tuple__") && !k.starts_with("Option__") && !k.starts_with("Result__"))
                            .or_else(|| self.mono.emitted_fns.iter().find(|k| k.ends_with(&suffix)).cloned());
                        match hit {
                            Some(k) => k,
                            None => fn_key,
                        }
                    } else {
                        fn_key
                    }
                };
                // Interface dispatch fallback: when the receiver type is a known
                // interface (e.g. `Error.description`), search all registered
                // concrete functions for one that matches `*.method_name` (static
                // dispatch -- the first matching implementation wins).
                let fn_key = if !self.types.functions.contains_key(&fn_key)
                    && !self.mono.generic_fn_decls.iter().any(|(k, _)| k == &fn_key)
                {
                    // Extract interface name and method from fn_key ("Error.description").
                    if let Some(dot_pos) = fn_key.find('.') {
                        let iface_name = &fn_key[..dot_pos];
                        let method_name = &fn_key[dot_pos + 1..];
                        if self.types.interfaces.contains_key(&iface_name.to_string()) {
                            let suffix = format!(".{}", method_name);
                            self.types.functions.keys().into_iter()
    .find(|k| k.ends_with(&suffix) && !k.starts_with(iface_name))
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
                let is_generic = self.mono.generic_fn_decls.iter().any(|(k, _)| k == &fn_key)
                    || (!self.types.functions.contains_key(&fn_key)
                        && (self.mono.generic_fn_decls.iter().any(|(k, _)| k.ends_with(&format!(".{}", fn_key)))
                            // `iter.range(1, 4).collect()` resolves fn_key to
                            // "Range.collect", but the generic decl is registered
                            // as "Iterator[T].collect". Match on the LEAF method
                            // name so the call enters monomorphisation with the
                            // concrete receiver instead of calling the erased
                            // i64-receiver definition (ABI mismatch -> garbage).
                            //
                            // round-7 (ve2 regression): the leaf match must only
                            // fire when the fn_key's RECEIVER part is an ABSTRACT
                            // (unregistered) type. A registered receiver means the
                            // call is `{KnownType}.{method}` -- a "Graph.push" key
                            // (base type of a `g.edges.push(...)` FIELD receiver)
                            // must NOT be hijacked onto the injected "Vec.push"
                            // decl (mono'd @Graph.push_Int with a literal-0
                            // receiver -> invalid IR). find_generic_decl's
                            // abstract-receiver preference uses the same rule.
                            || (fn_key.split('.').count() == 2
                                && {
                                    let recv_part = fn_key.rsplit_once('.').map(|(r, _)| r).unwrap_or("");
                                    let recv_is_abstract = !recv_part.is_empty()
                                        && !self.types.types.contains_key(&recv_part.to_string())
                                        && !self.types.type_meta.contains_key(&recv_part.to_string())
                                        && !self.types.type_meta.keys().into_iter().any(|k| k.ends_with(&format!(".{}", recv_part)))
                                        && !self.types.generic_type_names.iter().any(|k| k == recv_part || k.ends_with(&format!(".{}", recv_part)));
                                    recv_is_abstract
                                        && self.mono.generic_fn_decls.iter().any(|(k, _)| {
                                            k.rsplit('.').next() == fn_key.rsplit('.').next()
                                        })
                                })));
                if is_generic {
                    // Infer concrete types from argument types
                    let mut concrete_types: Vec<String> = Vec::new();
                    let mut const_values: HashMap<String, i64> = HashMap::new();
                    // D1: explicit type args from `fn[TypeArgs](...)` -- captured
                    // in explicit_generic_types above. Map each generic param to
                    // its explicit type directly so `add2[Float32]` monomorphises
                    // as Float32, not Int.
                    let explicit_types: Vec<String> = explicit_generic_types.clone();
                    // Find the generic function declaration
                    if let Some((_, fd)) = self.find_generic_decl(&fn_key) {
                        let fd = fd.clone();
                        for gp in &fd.generics {
                            // D1: explicit type args win over inference.
                            // Sources: `fn[T](...)` GenericCall types, or the
                            // Index-form type_arg (`fn[T]` parsed as index).
                            let explicit_name: Option<String> = if let Some(idx) = fd.generics.iter().position(|g| g.name.name == gp.name.name) {
                                if idx < explicit_types.len() && !explicit_types[idx].is_empty() {
                                    Some(explicit_types[idx].clone())
                                } else if let Some(ta) = type_arg {
                                    match ta {
                                        Expr::Ident(id) if fd.generics.len() == 1 => Some(id.name.clone()),
                                        Expr::Tuple(elems, _) => elems.get(idx)
                                            .and_then(|e| if let Expr::Ident(id) = e { Some(id.name.clone()) } else { None }),
                                        _ => None,
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            if let Some(explicit) = explicit_name {
                                concrete_types.push(explicit);
                                continue;
                            }
                            // BUG 29 (Map.keys on module globals): infer a
                            // generic METHOD's type args from the RECEIVER's
                            // declared XIOM type when the receiver is a
                            // module-global (`_coverage: Map[Str, Bool]` ->
                            // `_coverage.keys()` must monomorphise
                            // Map.keys[Str, Bool], not Map.keys[Int, Int]).
                            // Local receivers already infer via
                            // resolve_local_xiom_type below; globals were
                            // only tracked by LLVM type (erased), so they
                            // fell through to the Int default.
                            if let Some(recv) = receiver_expr {
                                if let Expr::Ident(rid) = recv.as_ref() {
                                    if let Some(gxiom) = self.local.global_xiom_types.get(&rid.name).cloned() {
                                        let (base, args) = Self::parse_generic_type_string(&gxiom);
                                        if let Some(pos) = fd.generics.iter().position(|g| g.name.name == gp.name.name) {
                                            if let Some(arg) = args.get(pos) {
                                                let arg = arg.clone();
                                                // Recurse one level: `Map[Str, Bool]`
                                                // args may themselves be generic
                                                // (`Map[K, V]` -> K is "Str").
                                                concrete_types.push(arg);
                                                continue;
                                            }
                                        }
                                        let _ = base;
                                    }
                                }
                            }
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
                                            if let Some(size) = self.local.local_array_sizes.get(&id.name) {
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
                                        // BUG 52: enum variant constructors
                                        // (`MyVal.Text("x")`) as generic args must
                                        // infer the ENUM type -- the old `_ => "Int"`
                                        // arm sent Map.insert[K,V] to
                                        // insert_Str_Int (payload dropped -> AV).
                                        Expr::Call(f, _, _) | Expr::GenericCall(f, _, _, _) => {
                                            if let Expr::Field(base, _, _) = f.as_ref() {
                                                let base_name = match base.as_ref() {
                                                    Expr::Ident(bid) => bid.name.clone(),
                                                    Expr::Field(_, bf, _) => bf.name.clone(),
                                                    _ => String::new(),
                                                };
                                                if !base_name.is_empty()
                                                    && (self.types.enum_variants.contains_key(&base_name)
                                                        || self.types.enum_variants.keys().into_iter()
                                                            .any(|k| k.ends_with(&format!(".{base_name}")))
                                                        || self.types.types.contains_key(&base_name))
                                                {
                                                    base_name
                                                } else {
                                                    "Int".to_string()
                                                }
                                            } else {
                                                "Int".to_string()
                                            }
                                        }
                                        Expr::Ident(id) => {
                                            if let Some(concrete) = self.mono.param_concrete_types.get(&id.name) {
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
                                // Nested generic: e.g. `Option[T]` -> extract T from type args
                                let arg_names = Self::extract_type_arg_names(&param.ty);
                                if let Some(_pos) = arg_names.iter().position(|a| a == &gp.name.name) {
                                    let concrete_ty = "Int".to_string();
                                    concrete_types.push(concrete_ty);
                                    inferred = true;
                                    break;
                                }
                            }
                            if !inferred {
                                // BUG 52: infer a generic METHOD's type args from a
                                // LOCAL receiver's recorded container type
                                // (`var m = Map[Str, MyVal].new()` -> `m.get("b")`
                                // must monomorphise Map.get[Str, MyVal], not
                                // Map.get[Str, Str]). The local's LLVM slot type is
                                // erased (%struct.Map); local_xiom_types keeps the
                                // args thanks to infer_value_xiom_type's ctor arm.
                                // Mirrors the BUG 29 module-global path below.
                                if let Some(recv) = receiver_expr {
                                    if let Expr::Ident(rid) = recv.as_ref() {
                                        if let Some(lxiom) = self.local.local_xiom_types.get(&rid.name).cloned() {
                                            if lxiom.contains('[') {
                                                let (_base, args) = Self::parse_generic_type_string(&lxiom);
                                                if let Some(pos) = fd.generics.iter().position(|g| g.name.name == gp.name.name) {
                                                    if let Some(arg) = args.get(pos) {
                                                        if !arg.is_empty() {
                                                            concrete_types.push(arg.clone());
                                                            continue;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                // Use explicit type args from the call syntax
                                // (e.g. `Map[Str, JsonValue].new()` -> type_arg = Tuple([Str, JsonValue])).
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
                                        // D1: `X as Float32` -- use the CAST TARGET type
                                        // (the value type, not the source). Fixes
                                        // generic Float32 monomorphisation colliding
                                        // with Int (wrong call target, garbage).
                                        Expr::As(_, ty, _) => Self::type_from_ast(ty),
                                        Expr::Ident(id) => {
                                            if let Some(concrete) = self.mono.param_concrete_types.get(&id.name) {
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
                                if self.types.interfaces.contains_key(&param_name) {
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
                        let already_tracked = self.mono.generic_instantiations.iter()
                            .any(|(f, cts)| f == &fn_key && cts == &concrete_types);
                        if !already_tracked {
                            self.mono.generic_instantiations.push((fn_key.clone(), concrete_types.clone()));
                            if !const_values.is_empty() {
                                self.mono.const_value_map.insert(specialized_name.clone(), const_values.clone());
                            }
                            // Also insert even without const values to avoid repeated lookups
                            self.mono.const_value_map.entry(specialized_name.clone()).or_insert_with(|| const_values.clone());
                        }
                        // Call the specialized version
                        let (ret_ty, param_types) = if let Some((pts, rt)) = self.types.functions.get(&specialized_name) {
                            (rt.clone(), pts.clone())
                        } else {
                            let mut inferred_types: Vec<String> = Vec::new();
                            // Compute param types from the GENERIC DECL's param types
                            // (substituted), mirroring what compile_generic_monomorphisations
                            // will register. Guessing from the ARG types is wrong: `&[N]T`
                            // registers i64* (data ptr) while `&Vec[T]` registers
                            // %struct.Vec (by value) -- both arrive as &array_local.
                            let generic_fd = self.find_generic_decl(&fn_key);
                            for (i, a) in args.iter().enumerate() {
                                let t = if let Some((_, fd)) = generic_fd.as_ref() {
                                    if let Some(p) = fd.params.get(i) {
                                        match &p.ty {
                                            // &mut T / *T -> real pointer to the value type
                                            Type::MutRef(inner) | Type::Ptr(inner) => {
                                                let subst = Self::substitute_type(inner, inner, &self.mono.param_concrete_types);
                                                let name = Self::type_from_ast(&subst);
                                                let base = self.llvm_type_for(&name).unwrap_or_else(|_| "i64".to_string());
                                                format!("{base}*")
                                            }
                                            // &[N]T fixed-array ref -> Vec data pointer
                                            Type::Ref(inner) => match inner.as_ref() {
                                                Type::Array(_, _) => "i64*".to_string(),
                                                // &Slice[T] -> by-value Vec struct (the
                                                // mono def's subst lowers Slice to
                                                // %struct.Vec -- the body calls
                                                // .len()/[i] on the value).
                                                Type::Slice(_) => "%struct.Vec".to_string(),
                                                // BUG 52 follow-up (2026-08-18):
                                                // &Vec[T] -> POINTER (the def's generic
                                                // Ref arm lowers "Vec" -> %struct.Vec*
                                                // and GEPs through the param). Passing
                                                // the Vec BY VALUE made the callee
                                                // read the data pointer as the Vec
                                                // header -> AV in every generic fn
                                                // taking &Vec[T] (contains family).
                                                Type::Vec(_) => "%struct.Vec*".to_string(),
                                                // &T scalar ref -> by-value value type
                                                other => {
                                                    // BUG 52 (2026-08-18): substitute
                                                    // with THIS call's type args --
                                                    // param_concrete_types is only
                                                    // populated while compiling a mono
                                                    // BODY, so at call sites it is
                                                    // stale/empty and `&K` with K=Str
                                                    // degraded to i64* (call ABI
                                                    // mismatched the def's i8**).
                                                    let mut subst_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                                                    for (gi, gp) in fd.generics.iter().enumerate() {
                                                        if let Some(c) = concrete_types.get(gi) {
                                                            subst_map.insert(gp.name.name.clone(), c.clone());
                                                        }
                                                    }
                                                    let subst = Self::substitute_type(other, other, &subst_map);
                                                    let name = Self::type_from_ast(&subst);
                                                    // BUG 31: &T scalar ref -> POINTER to
                                                    // the value type (the mono def takes
                                                    // i64*; a by-value i64 param made
                                                    // `*key` deref the VALUE -> AV).
                                                    let base = self.llvm_type_for(&name).unwrap_or_else(|_| "i64".to_string());
                                                    format!("{base}*")
                                                }
                                            },
                                            _ => {
                                                // BUG 52 (2026-08-18): by-VALUE generic
                                                // params (`value: V`) must resolve the
                                                // CONCRETE type from the call's type
                                                // args -- infer_llvm_type of an enum
                                                // variant-ctor arg degrades to i64
                                                // (the tag), so
                                                // `m.insert("b", MyVal.Text("x"))`
                                                // called with `i64 tag` against the
                                                // mono def's `%struct.MyVal` param ->
                                                // payload corruption / AV.
                                                let pt = Self::type_from_ast(&p.ty);
                                                if fd.generics.iter().any(|g| g.name.name == pt) {
                                                    let mut subst_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                                                    for (gi, gp) in fd.generics.iter().enumerate() {
                                                        if let Some(c) = concrete_types.get(gi) {
                                                            subst_map.insert(gp.name.name.clone(), c.clone());
                                                        }
                                                    }
                                                    let subst = Self::substitute_type(&p.ty, &p.ty, &subst_map);
                                                    let name = Self::type_from_ast(&subst);
                                                    self.llvm_type_for(&name).unwrap_or_else(|_| "i64".to_string())
                                                } else {
                                                    self.infer_llvm_type(a)
                                                }
                                            }
                                        }
                                    } else {
                                        self.infer_llvm_type(a)
                                    }
                                } else {
                                    self.infer_llvm_type(a)
                                };
                                inferred_types.push(t);
                            }
                            // 5c.34: Check if the generic decl body uses `self`
                            // (by-value self method). If so, prepend the receiver
                            // struct pointer type so the call matches the monomorphised
                            // function's actual signature.
                            let generic_fd = self.find_generic_decl(&fn_key);
                            let body_uses_self = generic_fd.as_ref().map_or(false, |(_, fd)| {
                                fd.body.as_ref().map_or(false, |b| IrEmitter::block_uses_self_ident(b))
                            });
                            let has_self = generic_fd.as_ref().map_or(false, |(_, fd)| {
                                fd.receiver.is_some()
                                    && fd.params.iter().any(|p| p.name.name == "self")
                            });
                            let has_mut_self = generic_fd.as_ref().map_or(false, |(_, fd)| {
                                fd.receiver.is_some()
                                    && fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self)
                            });
                            // BUG 29 (Map.keys on module globals): THIS-BASED
                            // generic methods (receiver, no self param, body
                            // reads bare fields) also take a receiver pointer
                            // in the monomorphised def -- prepend it here too.
                            let is_this_based = generic_fd.as_ref().map_or(false, |(_, fd)| {
                                !has_self
                                    && !body_uses_self
                                    && fd.receiver.is_some()
                                    && self.body_uses_receiver_state(fd)
                            });
                            if has_self || body_uses_self || is_this_based {
                                // Prepend the receiver's pointer/struct type.
                                // BUG 30 (smoke_rc): mirror the monomorphised
                                // callee's self ABI exactly -- has_self_param
                                // means BY-VALUE unless the self is `&mut self`
                                // (pointer); only body_uses_self/this-based get
                                // a pointer. The old caller logic checked only
                                // is_mut_self, so `Rc.get[T](self)` (by-value
                                // self) received `%struct.Rc*` while the callee
                                // took `%struct.Rc` by value -> the callee read
                                // the alloca ADDRESS as the Rc struct -> every
                                // field garbage (pointer-sized values).
                                if let Some(recv_name) = generic_fd
                                    .as_ref()
                                    .and_then(|(_, fd)| fd.receiver.as_ref())
                                {
                                    // BUG 38b: a KNOWN concrete receiver resolves
                                    // through llvm_type_for; an UNDECLARED abstract
                                    // receiver ("Iterator") falls back to the
                                    // receiver part of the call key ("Range").
                                    let recv_ty = self.receiver_llvm_type(&recv_name.name, &fn_key);
                                    // BUG 31/38b: mutating self methods (self.field
                                    // or BARE receiver-field assignments) pass BY
                                    // POINTER -- mirror the callee's ABI decision.
                                    let fd_mutates = generic_fd.as_ref().map_or(false, |(_, fd)| {
                                        self.block_mutates_self(fd) || self.block_mutates_receiver_state(fd)
                                    });
                                    let recv_abi = if has_self && !has_mut_self && !fd_mutates {
                                        recv_ty
                                    } else if recv_ty.starts_with('%') {
                                        format!("{recv_ty}*")
                                    } else {
                                        recv_ty
                                    };
                                    inferred_types.insert(0, recv_abi);
                                }
                            }
                            let generic_ret = self.types.functions.get(&fn_key)
                                .map(|(_, rt)| rt.clone())
                                .unwrap_or_else(|| LLVM_I64.to_string());
                            // D1 (2026-08-08): the generic decl's REGISTERED return
                            // type may be the un-substituted default (i64). If the
                            // monomorphised return type is known (via the generic
                            // decl's declared return type substituted with the
                            // concrete type map), use THAT -- fixes generic
                            // Float64/float ops returning garbage (the call was
                            // emitted as i64 while the fn returns double).
                            let generic_ret = if let Some((_, fd)) = generic_fd {
                                if let Some(ret_ty) = fd.return_type.as_ref() {
                                    // D1: substitute T with the CONCRETE type from
                                    // the call's inferred type args (param_concrete_types
                                    // is empty at call time). Fixes generic Float64
                                    // ops: the call was emitted as i64 while the fn
                                    // returns double. ONLY for scalar results --
                                    // struct results (Result/Option/Vec payloads)
                                    // have their own monomorphisation path.
                                    let mut subst_map: std::collections::HashMap<String, String> =
                                        self.mono.param_concrete_types.clone();
                                    for (i, gp) in fd.generics.iter().enumerate() {
                                        if let Some(c) = concrete_types.get(i) {
                                            subst_map.insert(gp.name.name.clone(), c.clone());
                                        }
                                    }
                                    let subst = Self::substitute_type(ret_ty, ret_ty, &subst_map);
                                    // type_from_ast DROPS Named args
                                    // (Type::Named("Result", [Env, Str]) ->
                                    // "Result") -- render the full
                                    // "Result[Env, Str]" so BUG 41's
                                    // concrete_container_llvm can build the
                                    // Result__Env__Str key.
                                    let name = match &subst {
                                        Type::Named(id, args) if !args.is_empty() => {
                                            let parts: Vec<String> = args.iter()
                                                .map(|a| Self::type_from_ast(a))
                                                .collect();
                                            format!("{}[{}]", id.name, parts.join(", "))
                                        }
                                        // BUG 41: the parser produces
                                        // Type::Result/Option for `Result[..]`
                                        // returns -- render the FULL name;
                                        // type_from_ast drops the payloads.
                                        Type::Result(ok, err) => format!(
                                            "Result[{}, {}]",
                                            Self::type_from_ast(ok),
                                            Self::type_from_ast(err)
                                        ),
                                        Type::Option(inner) => format!(
                                            "Option[{}]",
                                            Self::type_from_ast(inner)
                                        ),
                                        _ => Self::type_from_ast(&subst),
                                    };
                                    // BUG 41 (2026-08-17): concrete Result/
                                    // Option payloads must resolve to the
                                    // monomorphised struct (Result__Env__Str),
                                    // mirroring the mono definition's
                                    // subst_type. The generic-base lookup
                                    // produced %struct.Result while the
                                    // callee emits %struct.Result__Env__Str
                                    // -> invalid IR (m34_y15/y20 compile
                                    // failures, deterministic after BUG 40).
                                    let llvm = match self.concrete_container_llvm(&name) {
                                        Some(concrete) => concrete,
                                        None => self.llvm_type_for(&name).unwrap_or_else(|_| generic_ret.clone()),
                                    };
                                    let is_struct = name.starts_with("Result") || name.starts_with("Option")
                                        || name.starts_with("Vec") || name.starts_with("Map") || name.starts_with("Set")
                                        || name.starts_with("Slice");
                                    // BUG 38b: container returns ("Vec[Int]") must
                                    // keep the container LLVM type (%struct.Vec) --
                                    // the erased i64 made the call/def ABI mismatch
                                    // (clang: defined %struct.Range but expected i64).
                                    if is_struct && llvm != "i64" {
                                        llvm
                                    } else if !is_struct && (llvm != "i64" || generic_ret == "i64") {
                                        llvm
                                    } else {
                                        generic_ret
                                    }
                                } else {
                                    generic_ret.clone()
                                }
                            } else {
                                generic_ret.clone()
                            };
                            (generic_ret, inferred_types)
                        };
                        // Include receiver argument only if it's an actual struct instance
                        // AND it's not already in the registered param_types
                        let mut all_args: Vec<String> = compiled_args.iter().map(|(v, _)| v.clone()).collect();
                        let mut all_arg_types: Vec<String> = compiled_args.iter().map(|(_, t)| t.clone()).collect();
                        let mut all_param_types = param_types.clone();
                        // B-007: hoisted -- the fn-typed param positions need to
                        // know whether the receiver occupies fd.params[0].
                        let receiver_in_params = !all_param_types.is_empty() && all_param_types.len() > all_args.len();
                        if let Some(receiver) = receiver_expr {
                            let is_instance = self.receiver_is_instance(receiver);
                            // Check if param_types already includes a receiver (from monomorphised registration)
                            let has_receiver_in_params = !all_param_types.is_empty() && all_param_types.len() > all_args.len();
                            // Check if the generic function declaration has a self param.
                            // Methods like Map.insert(key, value) have receiver type
                            // but no self param -- don't pass the receiver instance.
                            // For generic functions in generic_fn_decls, check if the
                            // declaration has a self param. For non-generic methods
                            // (not in generic_fn_decls), default to true -- the
                            // has_receiver_in_params check above handles the rest.
                            let generic_has_self = self.mono.generic_fn_decls.iter()
                                .find(|(k, _)| k == &fn_key)
                                .map(|(_, fd)| {
                                    if fd.params.iter().any(|p| p.name.name == "self") {
                                        return true;
                                    }
                                    // BUG 29 (Map.keys on module globals):
                                    // THIS-BASED generic methods have a receiver
                                    // but NO self param -- their bodies read bare
                                    // fields (`keys.len()` in Map.keys[K, V]).
                                    // The monomorphised def takes a receiver
                                    // pointer, so the call MUST pass it.
                                    fd.receiver.is_some() && self.body_uses_receiver_state(fd)
                                })
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
                                                // BUG 29 (Map.keys on module globals):
                                                // the receiver is a MODULE-GLOBAL --
                                                // pass the global's ADDRESS
                                                // (@_coverage), not the loaded value.
                                                } else if let Some((symbol, _)) = self.local.module_globals.get(&id.name).cloned() {
                                                    (format!("@{symbol}"), format!("{recv_llvm_ty}*"))
                                                } else { (recv_val, recv_llvm_ty) }
                                            } else if let Expr::Field(..) = &**receiver {
                                                // round-9 (Set ABI): a FIELD receiver
                                                // needing a pointer self (&mut self) --
                                                // pass the field's ADDRESS (the Ref
                                                // arm GEPs into the base struct), not
                                                // the loaded value:
                                                // `holder.s.insert(10)` must call
                                                // @Set.insert_Int(%struct.Set* <field>),
                                                // not cast the value to a pointer.
                                                let (addr, _addr_ty) = self.compile_expr(
                                                    &Expr::Ref(Box::new(receiver.as_ref().clone()), xiom_ast::Span::new(0, 0)))?;
                                                (addr, format!("{recv_llvm_ty}*"))
                                            } else { (recv_val, recv_llvm_ty) }
                                        } else { (recv_val, recv_llvm_ty) }
                                    } else { (recv_val, recv_llvm_ty) }
                                } else { (recv_val, recv_llvm_ty) };
                                if has_receiver_in_params {
                                    // Receiver type already in param_types, just need the value
                                    all_args.insert(0, recv_val);
                                    all_arg_types.insert(0, all_param_types.first().cloned().unwrap_or_else(|| LLVM_I64.to_string()));
                                } else {
                                    all_param_types.insert(0, recv_llvm_ty.clone());
                                    all_args.insert(0, recv_val);
                                    all_arg_types.insert(0, recv_llvm_ty);
                                }
                            } else if !is_instance && has_receiver_in_params {
                                // Type name or module name receiver -- remove extra param type
                                all_param_types.remove(0);
                            }
                            // else: is_instance && !generic_has_self -> no self param to pass
                        }
                        // Coerce each arg to the callee's declared param type (its real
                        // compiled type may differ, e.g. an enum-variant arg compiled to
                        // a struct while the callee expects that struct). A pointer
                        // param fed `&x`/`&mut x` receives the scalar's slot address.
                        let arg_offset = all_args.len().saturating_sub(args.len());
                        // B-007: the callee's fn-typed PARAM positions (Type::Fn)
                        // so raw fn-REFERENCE args get wrapped into closure envs.
                        // Precomputed OUTSIDE the args closure (borrow rules).
                        let callee_fd = self.mono.generic_fn_decls.iter()
                            .find(|(k, _)| k == &fn_key || k.ends_with(&format!(".{}", fn_key)))
                            .map(|(_, f)| f);
                        let fn_typed_args: Vec<Option<String>> = (0..args.len()).map(|j| {
                            let pidx = j + (if receiver_in_params { 1 } else { 0 });
                            callee_fd.and_then(|fd| fd.params.get(pidx)).and_then(|p| match &p.ty {
                                Type::Fn(_, ret) => Some(Self::type_from_ast(ret)),
                                _ => None,
                            })
                        }).collect();
                        if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() {
                            eprintln!("[fta] fn={fn_name} key={fn_key} nargs={} typed={:?} has_fd={}", args.len(), fn_typed_args, callee_fd.is_some());
                        }
                        let args_str = all_args.iter().enumerate()
                            .map(|(i, arg)| {
                                let pty = all_param_types.get(i).cloned()
                                    .unwrap_or_else(|| all_arg_types.get(i).cloned().unwrap_or_else(|| LLVM_I64.to_string()));
                                let from = all_arg_types.get(i).cloned().unwrap_or_else(|| pty.clone());
                                let mut coerced = if i >= arg_offset {
                                    match args.get(i - arg_offset) {
                                        Some(ae) => self.coerce_arg_for_param(ae, arg, &from, &pty),
                                        None => self.coerce_value(arg, &from, &pty),
                                    }
                                } else {
                                    self.coerce_value(arg, &from, &pty)
                                };
                                // round-11 (B-007): fn-typed params receive a
                                // closure ENV (field 0 = fn ptr). A bare
                                // fn-REFERENCE arg (cmp_int, is_even) compiles
                                // to the raw code address -- wrap it in a
                                // forwarding thunk env; closure literals and
                                // closure locals are already envs.
                                if i >= arg_offset {
                                if let Some(ret_xiom) = fn_typed_args.get(i - arg_offset).and_then(|o| o.clone()) {
                                    if let Some(ae) = args.get(i - arg_offset) {
                                        let is_fn_ref = matches!(ae, Expr::Ident(id)
                                            if (self.types.functions.contains_key(&id.name)
                                                || self.types.functions.keys().into_iter().any(|k| k.ends_with(&format!(".{}", id.name)))
                                                || self.mono.emitted_fns.contains(&id.name))
                                                // B-007: a fn-typed PARAM re-passed as an
                                                // arg is ALREADY an env -- wrapping it
                                                // again double-wraps (heap_sort_by ->
                                                // heap_sift_down_by's `compare` arg also
                                                // matched the registered Int.compare
                                                // suffix and got wrapped -> the inner
                                                // M20-A1 read field 0 = the inner env
                                                // pointer as a code pointer).
                                                && !self.local.closure_locals.contains(&id.name));
                                        if is_fn_ref {
                                            let (ref_name, ref_params) = match ae {
                                                Expr::Ident(id) => {
                                                    let p = self.types.functions.get(&id.name)
                                                        .map(|(p, _)| p.clone())
                                                        .or_else(|| {
                                                            let suffix = format!(".{}", id.name);
                                                            self.types.functions.entries().into_iter()
                                                                .find(|(k, _)| k.ends_with(&suffix))
                                                                .map(|(_, (p, _))| p.clone())
                                                        })
                                                        .unwrap_or_default();
                                                    (id.name.clone(), p)
                                                }
                                                _ => (String::new(), Vec::new()),
                                            };
                                            if !ref_name.is_empty() {
                                                coerced = self.wrap_fn_ref_env(&ref_name, &coerced, &ret_xiom, ref_params);
                                            }
                                        }
                                    }
                                }
                                }
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
                            Ok((tmp, ret_ty.clone()))
                        }
                    } else {
                        Ok(("0".to_string(), LLVM_I64.to_string()))
                    }
                } else {
                    // For method calls, resolve the fully qualified function name
                    let mut implicit_self_resolved = false;
                    let mut resolved_fn_key = if let Some(receiver) = receiver_expr {                        let recv_type = self.infer_struct_type_name(receiver);
                        if let Some(rt) = recv_type {
                            format!("{}.{}", rt, fn_name)
                        } else if !self.mono.current_type_map.is_empty() {
                            // Check if receiver is a generic param being monomorphised
                            let obj_var_name = match &**receiver {
                                Expr::Ident(id) => id.name.clone(),
                                _ => String::new(),
                            };
                            if !obj_var_name.is_empty() {
                                if let Some(concrete_type) = self.mono.param_concrete_types.get(&obj_var_name) {
                                    format!("{}.{}", concrete_type, fn_name)
                                } else {
                                    fn_key.clone()
                                }
                            } else {
                                fn_key.clone()
                            }
                        } else {
                            // Receiver is a module name -- use module-qualified resolution
                            self.resolve_module_call(receiver, &fn_name)
                        }
                    } else {
                        fn_key.clone()
                    };
                    // Fallback: when the resolved key is not a known function (e.g.
                    // "is_match" from an i64-typed receiver), search for any registered
                    // function whose name ends with ".method_name" (e.g. "Regex.is_match").
                    // IMPORTANT: for bare function calls (no "." in key AND no receiver),
                    // only match bare function names -- do NOT match instance methods.
                    // Method calls (receiver_expr is Some) may resolve through this path
                    // when the receiver type is i64 (not a named struct), so the bare-call
                    // skip must NOT apply.
                    if !self.types.functions.contains_key(&resolved_fn_key) {
                        // 5e.3: when resolved_fn_key is Type.method (e.g. "Layout.new"),
                        // try ".Type.method" suffix FIRST so alloc.Layout.new(..)
                        // resolves to xiom.alloc.Layout.new even when Rc.new confuses
                        // the generic ".new" suffix search.
                        let suffix = format!(".{fn_name}");
                        let mut found = String::new();
                        let is_bare_call = !resolved_fn_key.contains('.') && receiver_expr.is_none();
                        // Pass 1: specific suffix match (e.g. ".Layout.new")
                        if resolved_fn_key.contains('.') {
                            let specific = format!(".{}", resolved_fn_key);
                            for key in self.types.functions.keys() {
                                if key.ends_with(&specific) {
                                    if found.is_empty() { found = key.clone(); }
                                    else if found != *key { found.clear(); break; }
                                }
                            }
                        }
                        // Pass 2: generic suffix match (e.g. ".new")
                        if found.is_empty() {
                            for key in self.types.functions.keys() {
                                if key.ends_with(&suffix) {
                                    if is_bare_call && key.contains('.') { continue; }
                                    if found.is_empty() { found = key.clone(); }
                                    else if found != *key { found.clear(); break; }
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
                        // non-local) is NOT an instance -> no phantom receiver arg.
                        let is_instance = self.receiver_is_instance(receiver);
                        // Registered callee param types (used to coerce args to the
                        // exact types the callee declares).
                        let callee_pts = self.types.functions.get(&resolved_fn_key).map(|(p, _)| p.clone());
                        if is_instance {
                            // Value sink: use the receiver's real compiled LLVM type
                            // (from compile_expr) rather than a re-inference.
                            let (recv_val, recv_llvm_ty) = self.compile_expr(receiver)?;
                            // If the callee expects a pointer self param (&mut Struct),
                            // pass the receiver's alloca ADDRESS instead of the
                            // loaded value so mutations propagate to the caller.
                            let (recv_val, recv_llvm_ty) = if let Some(p0) = callee_pts.as_ref().and_then(|p| p.first()) {
                                if p0.ends_with('*') && !recv_llvm_ty.ends_with('*') {
                                    // Round-6 fix (2026-08-19): a Str receiver
                                    // (`let name = io.file_name(p)?; name.byte_at(i)`)
                                    // holds a string HANDLE in an i64 slot; the
                                    // callee's `i8*` param wants the handle VALUE
                                    // (Str == i8*), NOT the slot address. The
                                    // address-pass heuristic below is for `&mut T`
                                    // / `&T` self params (mutation propagation) --
                                    // firing on Str receivers made byte_at read
                                    // the string bytes from the slot -> garbage ->
                                    // io.extension/path.extension returned None.
                                    let recv_is_str = matches!(receiver.as_ref(), Expr::Ident(id)
                                        if self.xiom_type_of_local(&id.name).as_deref() == Some("Str"));
                                    if recv_is_str {
                                        let coerced = self.coerce_value(&recv_val, &recv_llvm_ty, p0);
                                        (coerced, p0.clone())
                                    } else {
                                    // Unwrap &x / &mut x to find the underlying lvalue.
                                    let inner_ident: Option<&Expr> = match &**receiver {
                                        Expr::Ref(i, _) | Expr::MutRef(i, _) => Some(i.as_ref()),
                                        Expr::Unary(UnaryOp::Ref, i, _) | Expr::Unary(UnaryOp::MutRef, i, _) => Some(i.as_ref()),
                                        e => Some(e),
                                    };
                                    if let Some(Expr::Ident(id)) = inner_ident {
                                        if let Some((slot, _slot_ty)) = self.lookup_local(&id.name).cloned() {
                                            (slot, format!("{recv_llvm_ty}*"))
                                        } else {
                                            (recv_val, recv_llvm_ty)
                                        }
                                    } else if let Some(Expr::Index(..)) = inner_ident {
                                        // BUG 34: `bs[i].push(x)` -- a MUTATING
                                        // method on a nested-Vec ELEMENT. The
                                        // element must be passed BY ADDRESS (GEP
                                        // into the outer data buffer) -- the old
                                        // path passed the loaded COPY, so the
                                        // inner push mutated a discarded header
                                        // (bs[0][0] read 0). Reuse the Ref arm's
                                        // element-address machinery, then
                                        // inttoptr the ptrtoint'd address to the
                                        // callee's declared pointer type.
                                        let ref_expr = Expr::Ref(Box::new((**receiver).clone()), Span::new(0, 0));
                                        let (addr_val, _) = self.compile_expr(&ref_expr)?;
                                        let ptr_reg = self.fresh_tmp();
                                        self.emitln(&format!("  {ptr_reg} = inttoptr i64 {addr_val} to {p0}"));
                                        (ptr_reg, p0.clone())
                                    } else {
                                        (recv_val, recv_llvm_ty)
                                    }
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
                                        // the struct -- inttoptr directly, never
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
                            let fta: Vec<Option<String>> = (0..compiled_args.len()).map(|j| {
                                self.mono.fn_typed_params.get(&resolved_fn_key)
                                    .and_then(|v| v.iter().find(|(idx, _)| *idx == j + 1).map(|(_, r)| r.clone()))
                            }).collect();
                            let rest_str: Vec<String> = compiled_args.iter().enumerate()
                                .map(|(i, (arg_val, arg_ty))| {
                                    let pty = callee_pts.as_ref()
                                        .and_then(|p| p.get(i + 1).cloned())
                                        .unwrap_or_else(|| arg_ty.clone());
                                    let mut coerced = match args.get(i) {
                                        Some(ae) => self.coerce_arg_for_param(ae, arg_val, arg_ty, &pty),
                                        None => self.coerce_value(arg_val, arg_ty, &pty),
                                    };
                                    // B-007: wrap raw fn-REFERENCE args for fn-typed params.
                                    if let Some(ret_xiom) = fta.get(i).and_then(|o| o.clone()) {
                                        if let Some(ae) = args.get(i) {
                                            let is_fn_ref = matches!(ae, Expr::Ident(id)
                                                if self.types.functions.contains_key(&id.name)
                                                    || self.types.functions.keys().into_iter().any(|k| k.ends_with(&format!(".{}", id.name)))
                                                    || self.mono.emitted_fns.contains(&id.name));
                                            if is_fn_ref {
                                                let (ref_name, ref_params) = match ae {
                                                    Expr::Ident(id) => {
                                                        let p = self.types.functions.get(&id.name)
                                                            .map(|(p, _)| p.clone())
                                                            .or_else(|| {
                                                                let suffix = format!(".{}", id.name);
                                                                self.types.functions.entries().into_iter()
                                                                    .find(|(k, _)| k.ends_with(&suffix))
                                                                    .map(|(_, (p, _))| p.clone())
                                                            })
                                                            .unwrap_or_default();
                                                        (id.name.clone(), p)
                                                    }
                                                    _ => (String::new(), Vec::new()),
                                                };
                                                if !ref_name.is_empty() {
                                                    coerced = self.wrap_fn_ref_env(&ref_name, &coerced, &ret_xiom, ref_params);
                                                }
                                            }
                                        }
                                    }
                                    format!("{pty} {coerced}")
                                })
                                .collect();
                            if rest_str.is_empty() {
                                format!("{recv_llvm_ty} {recv_val}")
                            } else {
                                format!("{recv_llvm_ty} {recv_val}, {}", rest_str.join(", "))
                            }
                        } else {
                            // Type name or module name -- no receiver argument. Coerce
                            // each arg to the callee's declared param type (its real
                            // compiled type may differ, e.g. an enum-variant arg
                            // compiled to a struct while the callee expects it).
                            let fta2: Vec<Option<String>> = (0..compiled_args.len()).map(|j| {
                                self.mono.fn_typed_params.get(&resolved_fn_key)
                                    .and_then(|v| v.iter().find(|(idx, _)| *idx == j).map(|(_, r)| r.clone()))
                            }).collect();
                            compiled_args.iter().enumerate()
                                .map(|(i, (arg_val, arg_ty))| {
                                    let pty = callee_pts.as_ref()
                                        .and_then(|p| p.get(i).cloned())
                                        .unwrap_or_else(|| arg_ty.clone());
                                    let mut coerced = match args.get(i) {
                                        Some(ae) => self.coerce_arg_for_param(ae, arg_val, arg_ty, &pty),
                                        None => self.coerce_value(arg_val, arg_ty, &pty),
                                    };
                                    // B-007: wrap raw fn-REFERENCE args for fn-typed params.
                                    if let Some(ret_xiom) = fta2.get(i).and_then(|o| o.clone()) {
                                        if let Some(ae) = args.get(i) {
                                            let is_fn_ref = matches!(ae, Expr::Ident(id)
                                                if self.types.functions.contains_key(&id.name)
                                                    || self.types.functions.keys().into_iter().any(|k| k.ends_with(&format!(".{}", id.name)))
                                                    || self.mono.emitted_fns.contains(&id.name));
                                            if is_fn_ref {
                                                let (ref_name, ref_params) = match ae {
                                                    Expr::Ident(id) => {
                                                        let p = self.types.functions.get(&id.name)
                                                            .map(|(p, _)| p.clone())
                                                            .or_else(|| {
                                                                let suffix = format!(".{}", id.name);
                                                                self.types.functions.entries().into_iter()
                                                                    .find(|(k, _)| k.ends_with(&suffix))
                                                                    .map(|(_, (p, _))| p.clone())
                                                            })
                                                            .unwrap_or_default();
                                                        (id.name.clone(), p)
                                                    }
                                                    _ => (String::new(), Vec::new()),
                                                };
                                                if !ref_name.is_empty() {
                                                    coerced = self.wrap_fn_ref_env(&ref_name, &coerced, &ret_xiom, ref_params);
                                                }
                                            }
                                        }
                                    }
                                    format!("{pty} {coerced}")
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    } else {
                        // 5c.30: G-10 implicit-self -- override the fn key so
                        // bare `greet("Hi")` resolves to `Greeter.greet`.
                        // Self IS injected: sibling method calls (bare `name()`
                        // inside `Doctor.formal`) must receive the receiver.
                        if receiver_expr.is_none() {
                            if let Some(isk) = self.resolve_implicit_self_call(&fn_name) {
                                resolved_fn_key = isk;
                                implicit_self_resolved = true;
                            }
                        }
                        // Build args_str. When implicit-self resolved, prepend the
                        // receiver value (self) as the first argument.
                        let mut parts: Vec<String> = Vec::new();
                        if implicit_self_resolved {
                            // Only inject self when the resolved callee's FIRST
                            // registered param is the receiver struct (or pointer
                            // to it). Free functions sharing a suffix must not get
                            // an extra receiver argument.
                            if let Some((self_slot, self_ty)) = self.lookup_local("self").cloned() {
                                let callee_pts = self.types.functions.get(&resolved_fn_key).map(|(p, _)| p.clone());
                                let first_pt = callee_pts.as_ref().and_then(|p| p.first().cloned());
                                let self_base = self_ty.trim_end_matches('*').to_string();
                                let takes_self = first_pt.as_ref().map_or(false, |fp| {
                                    fp == &self_ty || fp == &self_base || fp.ends_with(&self_base)
                                });
                                if takes_self {
                                    let callee_self_ty = first_pt.unwrap_or(self_ty.clone());
                                    // Slot is an alloca (by-value receiver): load the struct.
                                    // Slot is a register (pointer receiver): pass directly.
                                    let self_val = if self_ty.starts_with("%struct.") && !self_ty.ends_with('*') {
                                        let loaded = self.fresh_tmp();
                                        self.emitln(&format!("  {loaded} = load {self_ty}, {self_ty}* {self_slot}"));
                                        loaded
                                    } else {
                                        self_slot.clone()
                                    };
                                    let coerced = self.coerce_value(&self_val, &self_ty, &callee_self_ty);
                                    parts.push(format!("{callee_self_ty} {coerced}"));
                                }
                            }
                        }
                        let use_registered = self.types.functions.get(&resolved_fn_key)
                            .map(|(pts, _)| pts.len() == compiled_args.len() + parts.len())
                            .unwrap_or(false);
                        if use_registered {
                            let pts = self.types.functions.get(&resolved_fn_key).unwrap().0.clone();
                            // Capture the pre-loop arg count (self-injected receiver,
                            // if any). pi must offset by the INITIAL parts length, not
                            // the live length which grows as we push.
                            let base = parts.len();
                            // B-007: fn-typed param positions for the raw fn-REF wrap.
                            let fta3: Vec<Option<String>> = (0..compiled_args.len()).map(|j| {
                                self.mono.fn_typed_params.get(&resolved_fn_key)
                                    .and_then(|v| v.iter().find(|(idx, _)| *idx == j + base).map(|(_, r)| r.clone()))
                            }).collect();
                            for (i, (arg_val, arg_ty)) in compiled_args.iter().enumerate() {
                                let pi = i + base;
                                if pi >= pts.len() { break; }
                                let pty = pts[pi].clone();
                                let mut coerced = match args.get(i) {
                                    Some(ae) => self.coerce_arg_for_param(ae, arg_val, arg_ty, &pty),
                                    None => self.coerce_value(arg_val, arg_ty, &pty),
                                };
                                // B-007: wrap raw fn-REFERENCE args into closure envs.
                                if let Some(ret_xiom) = fta3.get(i).and_then(|o| o.clone()) {
                                    if let Some(ae) = args.get(i) {
                                        let is_fn_ref = matches!(ae, Expr::Ident(id)
                                            if (self.types.functions.contains_key(&id.name)
                                                || self.types.functions.keys().into_iter().any(|k| k.ends_with(&format!(".{}", id.name)))
                                                || self.mono.emitted_fns.contains(&id.name))
                                                // B-007: a fn-typed PARAM re-passed as an
                                                // arg is ALREADY an env -- wrapping it
                                                // again double-wraps (heap_sort_by ->
                                                // heap_sift_down_by's `compare` arg also
                                                // matched the registered Int.compare
                                                // suffix and got wrapped -> the inner
                                                // M20-A1 read field 0 = the inner env
                                                // pointer as a code pointer).
                                                && !self.local.closure_locals.contains(&id.name));
                                        if is_fn_ref {
                                            let (ref_name, ref_params) = match ae {
                                                Expr::Ident(id) => {
                                                    let p = self.types.functions.get(&id.name)
                                                        .map(|(p, _)| p.clone())
                                                        .or_else(|| {
                                                            let suffix = format!(".{}", id.name);
                                                            self.types.functions.entries().into_iter()
                                                                .find(|(k, _)| k.ends_with(&suffix))
                                                                .map(|(_, (p, _))| p.clone())
                                                        })
                                                        .unwrap_or_default();
                                                    (id.name.clone(), p)
                                                }
                                                _ => (String::new(), Vec::new()),
                                            };
                                            if !ref_name.is_empty() {
                                                coerced = self.wrap_fn_ref_env(&ref_name, &coerced, &ret_xiom, ref_params);
                                            }
                                        }
                                    }
                                }
                                parts.push(format!("{pty} {coerced}"));
                            }
                            parts.join(", ")
                        } else {
                            for (arg_val, arg_ty) in compiled_args.iter() {
                                parts.push(format!("{arg_ty} {arg_val}"));
                            }
                            parts.join(", ")
                        }
                    };
                    let tmp = self.fresh_tmp();
                    let ret_ty = if let Some((_, rt)) = self.types.functions.get(&resolved_fn_key) {
                        rt.clone()
                    } else {
                        // Fallback: try current-module qualified name
                        let mut found = String::new();
                        if let Some(ref module) = self.local.current_module {
                            let qualified = format!("{module}.{resolved_fn_key}");
                            if let Some((_, rt)) = self.types.functions.get(&qualified) {
                                found = rt.clone();
                            }
                        }
                        // Fallback: search for any key ending with .resolved_fn_key
                        if found.is_empty() {
                            let suffix = format!(".{resolved_fn_key}");
                            for (k, (_, rt)) in self.types.functions.entries() {
                                if k.ends_with(&suffix) && rt.starts_with("%struct.") {
                                    found = rt.clone();
                                    break;
                                }
                            }
                        }
                        if found.is_empty() {
                            LLVM_I64.to_string()
                        } else {
                            found
                        }
                    };
                    // P0-4: Intercept math bitwise/shift builtins and emit native LLVM
                    // instructions. The stdlib math.xi provides per-bit software loops
                    // (O(n) with modulo/division per iteration); native LLVM and/or/xor/
                    // shl/ashr are single CPU instructions.
                    // BUG 15 fix (2026-08-11): this MUST only match the stdlib math
                    // module's helpers. The old `ends_with(".shr")` also intercepted
                    // USER helpers named shr/shl in any module (e.g. hash.xi's
                    // logical shift `fn shr` whose body computes a MASK) -- the
                    // intercept emitted a bare `ashr` and silently dropped the
                    // function body's mask statements. Qualified keys are restricted
                    // to xiom.math.*; BARE keys are intercepted only when no real
                    // function definition exists (soft/prelude names).
                    let bare_key = resolved_fn_key.rsplit('.').next().unwrap_or(&resolved_fn_key);
                    let is_named_builtin = matches!(bare_key, "shl" | "shr" | "bit_and" | "bit_or" | "bit_xor" | "bit_not");
                    let is_math_builtin = if !is_named_builtin {
                        false
                    } else if resolved_fn_key.contains('.') {
                        // Catalog keys: `math.shr` (the "xiom." prefix is
                        // stripped by the module registry). Only the stdlib
                        // math module's helpers are the software loops.
                        let is_math_module = resolved_fn_key.starts_with("math.")
                            || resolved_fn_key.starts_with("xiom.math.");
                        is_math_module
                    } else {
                        // Bare key with no registered definition -- soft name.
                        !self.types.functions.contains_key(&resolved_fn_key)
                    };
                    if is_math_builtin {
                        let lv = self.widen_to_i64(&compiled_args[0].0, &compiled_args[0].1);
                        let result = self.fresh_tmp();
                        if resolved_fn_key == "bit_not" || resolved_fn_key.ends_with(".bit_not") {
                            self.emitln(&format!("  {result} = xor i64 {lv}, -1"));
                        } else if compiled_args.len() >= 2 {
                            let rv = self.widen_to_i64(&compiled_args[1].0, &compiled_args[1].1);
                            let inst = if resolved_fn_key == "bit_and" || resolved_fn_key.ends_with(".bit_and") { "and" }
                                else if resolved_fn_key == "bit_or" || resolved_fn_key.ends_with(".bit_or") { "or" }
                                else if resolved_fn_key == "bit_xor" || resolved_fn_key.ends_with(".bit_xor") { "xor" }
                                else if resolved_fn_key == "shl" || resolved_fn_key.ends_with(".shl") { "shl" }
                                else { "ashr" };
                            self.emitln(&format!("  {result} = {inst} i64 {lv}, {rv}"));
                        } else {
                            // Insufficient args -- fall through to normal call
                            self.emitln(&format!("  {result} = add i64 {lv}, 0"));
                        }
                        return Ok((result, LLVM_I64.to_string()));
                    }
                    // BUG 49 (2026-08-18): a FN-TYPED PARAM local must win over a
                    // registered function with the same name. The impl-method
                    // registration aliases the bare method name ("Int.compare"
                    // also registers "compare"), so a param named `compare` in
                    // a fn-param fn resolved to @Int.compare and the call
                    // bypassed the passed fn pointer (wrong order / AV). The
                    // local SHADOWS the global for fn-typed params and closures.
                    let is_fn_typed_param = self.types.fn_ptr_return_types.contains_key(&fn_name);
                    let callee_is_fn_ptr = receiver_expr.is_none()
                        && self.lookup_local(&fn_name).is_some()
                        && (is_fn_typed_param
                            || self.local.closure_locals.contains(&fn_name)
                            || self.types.functions.get(&resolved_fn_key).is_none())
                        && ret_ty == "i64";
                    // BUG 29 (repro_fn_storage): call through a FN-TYPED FIELD of
                    // a MODULE-GLOBAL struct (`g.f(x)` where `g: FnBox` is a
                    // module-level `var`). The method dispatch resolves
                    // `g.f` to the key `FnBox.f` which is NOT a registered
                    // function -- without this the call landed on a zero-arg
                    // auto-stub (`ret 0`) -> "module-scope fn storage silently
                    // read-only". Detect: receiver is a module global whose
                    // struct type_meta declares a field named `f`; emit
                    // GEP -> load i64 fn ptr -> inttoptr -> call.
                    let global_fn_field: Option<(String, String, usize)> = if receiver_expr.is_some()
                        && self.types.functions.get(&resolved_fn_key).is_none()
                        && self.mono.generic_fn_decls.iter().all(|(k, _)| k != &resolved_fn_key)
                    {
                        if let Some(Expr::Ident(obj_id)) = receiver_expr.map(|r| r.as_ref()) {
                            if let Some((symbol, global_llvm_ty)) = self.local.module_globals.get(&obj_id.name).cloned() {
                                if let Some(struct_name) = global_llvm_ty.strip_prefix("%struct.").map(|s| s.trim_end_matches('*').to_string()) {
                                    let field_idx = self.types.type_meta.get(&struct_name)
                                        .and_then(|meta| meta.fields.iter().position(|(fname, _)| fname == &fn_name));
                                    field_idx.map(|idx| (symbol, struct_name, idx))
                                } else { None }
                            } else { None }
                        } else { None }
                    } else { None };
                    if let Some((symbol, struct_name, field_idx)) = global_fn_field {
                        // GEP into the global struct, load the fn pointer field,
                        // and call through it (closure-style dispatch).
                        let struct_llvm = format!("%struct.{struct_name}");
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr {struct_llvm}, {struct_llvm}* @{symbol}, i32 0, i32 {field_idx}"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load i64, i64* {gep}"));
                        let compiled_args: Vec<(String, String)> = args.iter()
                            .map(|a| self.compile_expr(a).map(|(v, t)| (v, t)))
                            .collect::<Result<Vec<_>, _>>()?;
                        let args_str = compiled_args.iter()
                            .map(|(v, t)| format!("{t} {v}"))
                            .collect::<Vec<_>>().join(", ");
                        let param_types: Vec<String> = compiled_args.iter().map(|(_, t)| t.clone()).collect();
                        let fn_ptr_ty = format!("i64 ({})*", param_types.join(", "));
                        let fn_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {fn_ptr} = inttoptr i64 {loaded} to {fn_ptr_ty}"));
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i64 {fn_ptr}({args_str})"));
                        return Ok((tmp, LLVM_I64.to_string()));
                    }
                    // BUG 22 #11 fix: emit the PRE-ASSIGNED symbol for the
                    // resolved key (bare or qualified) -- definitions and call
                    // sites agree even when the call compiles before its def;
                    // otherwise the stub pass creates a zero-param
                    // @qualified stub and the call returns garbage.
                    let call_symbol = self.mono.fn_symbol_map.get(&resolved_fn_key)
                        .cloned()
                        .unwrap_or_else(|| resolved_fn_key.clone());
                    if callee_is_fn_ptr {
                        let (alloca_reg, local_llvm_ty) = self.lookup_local(&fn_name).expect("fn_ptr target must be in locals").clone();
                        let fn_ptr_loaded = self.fresh_tmp();
                        self.emitln(&format!("  {fn_ptr_loaded} = load {local_llvm_ty}, {local_llvm_ty}* {alloca_reg}"));
                        let param_types: Vec<String> = args.iter().map(|a| self.infer_llvm_type(a)).collect();
                        let actual_ret_ty = if ret_ty == "i64" {
                            self.types.fn_ptr_return_types.get(&fn_name).unwrap_or_else(|| LLVM_I64.to_string())
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
                    } else if self.config.hot_reload
                        && (self.config.pub_functions.contains(&resolved_fn_key)
                            || resolved_fn_key.rsplitn(2, '.').next()
                                .map_or(false, |bare| self.config.pub_functions.contains(bare)))
                    {
                        // 5e.5a: hot reload -- redirect pub fn calls through thunks
                        let thunk_name = format!("xiom_hot_thunk_{}", resolved_fn_key);
                        if ret_ty == "void" {
                            self.emitln(&format!("  call void @{thunk_name}({args_str})"));
                            Ok((String::new(), "void".to_string()))
                    } else {
                        self.emitln(&format!("  {tmp} = call {ret_ty} @{thunk_name}({args_str})"));
                        // Store result back to receiver for by-value self methods
                        // (Counter.inc(self) pattern). &self (reference) methods
                        // mutate through the pointer and don't need store-back.
                        if let Some(receiver) = receiver_expr {
                            if ret_ty.starts_with("%struct.") && self.should_store_back_method(&resolved_fn_key) {
                                self.store_back_to_receiver(receiver, &tmp, &ret_ty);
                            }
                        }
                        Ok((tmp, ret_ty.clone()))
                    }
                    } else if ret_ty == "void" {
                        self.emitln(&format!("  call void @{call_symbol}({args_str})"));
                        Ok((String::new(), "void".to_string()))
                    } else {
                        self.emitln(&format!("  {tmp} = call {ret_ty} @{call_symbol}({args_str})"));
                        if let Some(receiver) = receiver_expr {
                            if ret_ty.starts_with("%struct.") && self.should_store_back_method(&resolved_fn_key) {
                                self.store_back_to_receiver(receiver, &tmp, &ret_ty);
                            }
                        }
                        Ok((tmp, ret_ty.clone()))
                    }
                }
            }

    /// D1 (2026-08-08): resolve a `Trait[Arg]` (or `Trait`) receiver to the
    /// implementing type registered by `impl Trait[Arg] { ... }`.
    /// Receiver shapes: Index(Ident(Trait), arg) and bare Ident(Trait).
    fn resolve_impl_receiver(&self, receiver: &Expr) -> Option<String> {
        let (trait_name, arg_name): (String, Option<String>) = match receiver {
            Expr::Index(base, idx, _) => {
                if let Expr::Ident(id) = base.as_ref() {
                    let arg = match idx.as_ref() {
                        Expr::Ident(i) => Some(i.name.clone()),
                        _ => None,
                    };
                    (id.name.clone(), arg)
                } else {
                    return None;
                }
            }
            Expr::Ident(id) => (id.name.clone(), None),
            _ => return None,
        };
        // `Trait[Arg]` -> the impl fn is `Arg.method` (expand_impl_blocks names
        // it from the first trait arg when the `for Type` form is absent).
        if let Some(arg) = arg_name {
            let is_interface = self.types.interfaces.contains_key(&trait_name)
                || self.types.interfaces.keys().into_iter().any(|k| k.ends_with(&format!(".{}", trait_name)));
            if is_interface {
                // D1/3c: the arg may be a GENERIC TYPE PARAMETER (e.g.
                // `Num[T].add` inside `fn sum2[T: Num]`). Resolve T to its
                // concrete binding in the current monomorphisation context so
                // the dispatch targets the concrete impl (Int.add, etc.).
                // current_type_map holds the T->concrete substitution during
                // monomorphised body emission; param_concrete_types maps
                // PARAM names (not generic params) to concrete types.
                let concrete = self.mono.current_type_map.get(&arg)
                    .cloned()
                    .or_else(|| self.mono.param_concrete_types.get(&arg).cloned())
                    .unwrap_or(arg);
                return Some(concrete);
            }
        }
        None
    }

    /// D1: emit a direct call to an impl method fn (`Type.method`), passing
    /// the args in order (no receiver -- impl methods are static).
    fn compile_impl_method_call(&mut self, impl_fn: &str, args: &[Expr]) -> Result<(String, String), String> {
        // Generic impl method: monomorphise with argument-inferred types.
        let is_generic = self.mono.generic_fn_decls.iter().any(|(k, _)| k == impl_fn);
        if is_generic {
            let concrete_types: Vec<String> = args.iter()
                .filter_map(|a| {
                    let ty = self.infer_llvm_type(a);
                    let xiom = Self::xiom_type_name_from_llvm(&ty);
                    Some(xiom)
                })
                .collect();
            let specialized = self.monomorphised_fn_name(impl_fn, &concrete_types);
            let already = self.mono.generic_instantiations.iter()
                .any(|(f, cts)| f == impl_fn && cts == &concrete_types);
            if !already {
                self.mono.generic_instantiations.push((impl_fn.to_string(), concrete_types.clone()));
            }
            let compiled_args: Vec<(String, String)> = args.iter()
                .map(|a| self.compile_expr(a))
                .collect::<Result<Vec<_>, _>>()?;
            let args_str = compiled_args.iter()
                .map(|(v, t)| format!("{t} {v}"))
                .collect::<Vec<_>>().join(", ");
            let ret_ty = self.types.functions.get(&specialized.to_string())
                .map(|(_, rt)| rt.clone())
                .unwrap_or_else(|| LLVM_I64.to_string());
            if ret_ty == "void" {
                self.emitln(&format!("  call void @{specialized}({args_str})"));
                return Ok((String::new(), "void".to_string()));
            }
            let tmp = self.fresh_tmp();
            self.emitln(&format!("  {tmp} = call {ret_ty} @{specialized}({args_str})"));
            return Ok((tmp, ret_ty));
        }
        // Non-generic impl method: direct call.
        let compiled_args: Vec<(String, String)> = args.iter()
            .map(|a| self.compile_expr(a))
            .collect::<Result<Vec<_>, _>>()?;
        let args_str = compiled_args.iter()
            .map(|(v, t)| format!("{t} {v}"))
            .collect::<Vec<_>>().join(", ");
        let ret_ty = self.types.functions.get(&impl_fn.to_string())
            .map(|(_, rt)| rt.clone())
            .unwrap_or_else(|| LLVM_I64.to_string());
        if ret_ty == "void" {
            self.emitln(&format!("  call void @{impl_fn}({args_str})"));
            return Ok((String::new(), "void".to_string()));
        }
        let tmp = self.fresh_tmp();
        self.emitln(&format!("  {tmp} = call {ret_ty} @{impl_fn}({args_str})"));
        Ok((tmp, ret_ty))
    }
}
