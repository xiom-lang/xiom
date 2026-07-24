// XIOM Codegen — Function/method call compilation (extracted from expr.rs, M4.2)
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

use xiom_ast::*;
use std::collections::HashMap;

use super::IrEmitter;

impl IrEmitter {
    pub(crate) fn compile_call(&mut self, func: &Expr, args: &[Expr]) -> Result<(String, String), String> {
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
                        _ => false, // integer literal, binary expr, etc. — always a value index
                    }
                };
let (func_unwrapped, mut type_arg): (&Expr, Option<&Expr>) = match func {
                    Expr::Index(base, idx, _) if idx_is_type(&idx) => {
                        (base.as_ref(), Some(idx.as_ref()))
                    }
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
                    Some(ref n) => {
                        n.clone()
                    }
                    None => {
                        // The callee is not a simple Ident or Field — it may be an
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
                        if self.types.enum_variants.contains_key(&variant_key)
                            || self.types.enum_variants.get(&recv_name)
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
                    let has_user_fn = self.mono.emitted_fns.contains(fn_name.as_str())
                        || self.types.functions.contains_key(fn_name.as_str())
                        || self.mono.emitted_fns.iter().any(|k| k.ends_with(&format!(".{}", fn_name)))
                        || self.types.functions.keys().any(|k| k.ends_with(&format!(".{}", fn_name)));
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
                        _ => unreachable!("contains method with unexpected argument count"),
                    }
                    return Ok((tmp, "i64".to_string()));
                }
                } // if !has_user_fn ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â contract builtin guard
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
                        // `x.hash()` or `42.hash()`) ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â module-qualified calls like
                        // `hash.hash(42)` must fall through to generic dispatch.
                        if !is_value_instance {
                            // module-qualified call ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â fall through to generic dispatch
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
                            return Ok((recv_val, "i8*".to_string()));
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
                                _ => unreachable!("unknown comparison operator"),
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
                            self.types.functions.contains_key(&key)
                                || self.types.functions.keys().any(|k| k.ends_with(&format!(".{key}")))
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
                        // 5e.3: module-qualified type paths like alloc.Layout
                        // produce Expr::Field receivers whose leaf is the type.
                        Expr::Field(_, field, _) => Some(field.name.as_str()),
                        _ => None,
                    };
                    // 5e.3: Layout.new(size) — inline struct constructor { size, align: 8 }.
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
                            let layout_ty = if self.types.type_meta.contains_key("Layout") {
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
                        // G4: Float64->Float32 coercion for Vec[Float32] push.
                        // val_to_i64 bitcasts double→i64 preserving all 64 bits,
                        // but emit_elem_store truncates to i32 for 4-byte slots,
                        // discarding the upper 32 bits (exponent+sign). Must first
                        // fptrunc double→float so the float32 bit pattern is stored.
                        let (val_raw, val_ty) = if val_ty == "double" && self.vec_elem_float_type(receiver) == Some("float") {
                            let f32 = self.fresh_tmp();
                            self.emitln(&format!("  {f32} = fptrunc double {val_raw} to float"));
                            (f32, "float".to_string())
                        } else {
                            (val_raw, val_ty)
                        };
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
                            self.types.types.contains_key(tn)
                                || self.types.types.keys().any(|k| k.ends_with(&format!(".{tn}")))
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
                                    self.types.types.contains_key(tn)
                                        || self.types.types.keys().any(|k| k.ends_with(&format!(".{tn}")))
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
                            self.types.used_builtins.insert("Option".to_string());
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
                // G-36: Vec.clone() — deep copy. New buffer (len*elem_size bytes),
                // memcpy the payload, fresh struct {newbuf, len, len, elem_size}.
                // Registered in the checker for Vec/Slice/Map/Set receivers.
                if fn_name == "clone" && args.is_empty() {
                    if let Some(receiver) = receiver_expr {
                        let recv_ty = self.infer_llvm_type(receiver);
                        if recv_ty == "%struct.Vec" || recv_ty.contains("struct.Vec")
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
                            // bytes = len * elem_size; guard elem_size==0 → treat as 8
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
                            let newbuf = self.fresh_tmp();
                            self.emitln(&format!("  {newbuf} = call i8* @malloc(i64 {alloc_bytes})"));
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
                // M12/P1: Str.slice(start, end) — substring extraction.
                // Delegates to xiom.string.str_slice via normal function dispatch.
                // Handled here to short-circuit method resolution for the Str receiver.
                if fn_name == "slice" && args.len() == 2 {
                    if let Some(receiver) = receiver_expr {
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        let recv_ptr = self.val_to_i8ptr(&recv_val, &recv_ty);
                        let (start_val, _) = self.compile_expr(&args[0])?;
                        let (end_val, _) = self.compile_expr(&args[1])?;
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i8* @xiom_str_slice(i8* {recv_ptr}, i64 {start_val}, i64 {end_val})"));
                        return Ok((tmp, "i8*".to_string()));
                    }
                }
                // M12/P1: Str.starts_with(prefix) — prefix check via string.xi.
                if fn_name == "starts_with" && args.len() == 1 {
                    if let Some(receiver) = receiver_expr {
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        let recv_ptr = self.val_to_i8ptr(&recv_val, &recv_ty);
                        let (prefix_val, prefix_ty) = self.compile_expr(&args[0])?;
                        let prefix_ptr = self.val_to_i8ptr(&prefix_val, &prefix_ty);
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i1 @xiom_str_starts_with(i8* {recv_ptr}, i8* {prefix_ptr})"));
                        return Ok((tmp, "i1".to_string()));
                    }
                }
                // M12/P1: Str.ends_with(suffix) — suffix check.
                if fn_name == "ends_with" && args.len() == 1 {
                    if let Some(receiver) = receiver_expr {
                        let (recv_val, recv_ty) = self.compile_expr(receiver)?;
                        let recv_ptr = self.val_to_i8ptr(&recv_val, &recv_ty);
                        let (suffix_val, suffix_ty) = self.compile_expr(&args[0])?;
                        let suffix_ptr = self.val_to_i8ptr(&suffix_val, &suffix_ty);
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = call i1 @xiom_str_ends_with(i8* {recv_ptr}, i8* {suffix_ptr})"));
                        return Ok((tmp, "i1".to_string()));
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
                // Builtin size_of[T]() / sizeof[T](): return the LLVM size in bytes of type T.
                // size_of uses struct_byte_size (field-count × 8, XIOM-semantic).
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
                                    if fn_name == "align_of" { return Ok(("8".to_string(), "i64".to_string())); }
                                    return Ok((sz.to_string(), "i64".to_string()));
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
                                    // i8=1, i16=2, i32=4, i64=8 — matches C ABI sizes.
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
                        let enum_key = self.types.enum_variants.keys()
                            .find(|k| **k == type_id.name || k.ends_with(&format!(".{}", type_id.name)))
                            .cloned();
                        if let Some(ek) = enum_key {
                            if let Some(variants) = self.types.enum_variants.get(&ek) {
                                let var_info: Option<(usize, Vec<String>)> = variants.iter().enumerate()
                                    .find(|(_, (v, _))| v == &fn_name)
                                    .map(|(idx, (_, fields))| (idx, fields.clone()));
                                if let Some((var_idx, payload_fields)) = var_info {
                                    // Gather parent field layout BEFORE mutating self.
                                    let parent_field_names = self.types.types.get(&ek).cloned().unwrap_or_default();
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
                            // 5d: Typed payload extraction from tracked declared types.
                            // `let r = f()` where f -> Result[T, E] records T in
                            // local_opt_payload and E in local_err_payload. Use them
                            // so `.unwrap()`/`.unwrap_err()` return properly typed
                            // values instead of raw i64 (fixes msg.len() → @len
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
                                        return Ok((sptr, "i8*".to_string()));
                                    }
                                    if decl_ty == "Float64" {
                                        let f = self.fresh_tmp();
                                        self.emitln(&format!("  {f} = bitcast i64 {val} to double"));
                                        return Ok((f, "double".to_string()));
                                    }
                                }
                            }
                            // When field_ty is i64, the payload may be a heap pointer
                            // from val_to_i64 for struct payloads.  Determine the actual
                            // struct type by resolving the generic return type of the
                            // concrete instantiation (e.g. `Option.unwrap[Point] ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ Point`).
                            let struct_type_hint: Option<String> = {
                                let fn_key = if is_option { "Option.unwrap" } else { "Result.unwrap" };
                                self.mono.generic_fn_decls.iter().find(|(k, _)| k == fn_key || k.ends_with(&format!(".{}", fn_name)))
                                    .and_then(|(_, fd)| fd.return_type.as_ref().map(|t| Self::type_from_ast(t)))
                            };
                            if let Some(ref hint) = struct_type_hint {
                                // hint is the XIOM type name (e.g. "Point" for T=Point).
                                // Convert to LLVM struct type.
                                let struct_llvm = if self.types.types.contains_key(hint) || self.types.type_meta.contains_key(hint) {
                                    format!("%struct.{hint}")
                                } else {
                                    // Check if it resolves via type_meta
                                    let full_key = self.types.type_meta.keys().find(|k| k.ends_with(&format!(".{hint}"))).cloned();
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
                let fn_key = if !self.types.functions.contains_key(&fn_key)
                    && !self.mono.generic_fn_decls.iter().any(|(k, _)| k == &fn_key)
                {
                    // Extract interface name and method from fn_key ("Error.description").
                    if let Some(dot_pos) = fn_key.find('.') {
                        let iface_name = &fn_key[..dot_pos];
                        let method_name = &fn_key[dot_pos + 1..];
                        if self.types.interfaces.contains_key(iface_name) {
                            let suffix = format!(".{}", method_name);
                            self.types.functions.keys()
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
                let is_generic = self.mono.generic_fn_decls.iter().any(|(k, _)| k == &fn_key)
                    || (!self.types.functions.contains_key(&fn_key)
                        && self.mono.generic_fn_decls.iter().any(|(k, _)| k.ends_with(&format!(".{}", fn_key))));
                if is_generic {
                    // Infer concrete types from argument types
                    let mut concrete_types: Vec<String> = Vec::new();
                    let mut const_values: HashMap<String, i64> = HashMap::new();
                    // Find the generic function declaration
                    if let Some((_, fd)) = self.mono.generic_fn_decls.iter().find(|(k, _)| k == &fn_key)
                        .or_else(|| self.mono.generic_fn_decls.iter().find(|(k_2, _)| k_2.ends_with(&format!(".{}", fn_key)))) {
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
                            // Not yet registered - use the generic signature with
                            // argument-inferred param types, prepending the receiver
                            // type if the generic decl has a self parameter.
                            let mut inferred_types: Vec<String> = args.iter()
                                .map(|a| self.infer_llvm_type(a))
                                .collect();
                            // Check if the generic decl has a self param (receiver)
                            let has_self = self.mono.generic_fn_decls.iter()
                                .find(|(k, _)| k == &fn_key)
                                .map(|(_, fd)| fd.receiver.is_some()
                                    && fd.params.iter().any(|p| p.name.name == "self" && p.is_mut_self))
                                .unwrap_or(false);
                            if has_self {
                                // Prepend the receiver's pointer type
                                if let Some(recv_name) = self.mono.generic_fn_decls.iter()
                                    .find(|(k, _)| k == &fn_key)
                                    .and_then(|(_, fd)| fd.receiver.as_ref())
                                {
                                    let recv_ty = self.llvm_type_for(&recv_name.name)
                                        .unwrap_or_else(|_| "i64".to_string());
                                    let recv_ptr = if recv_ty.starts_with('%') { format!("{recv_ty}*") } else { recv_ty };
                                    inferred_types.insert(0, recv_ptr);
                                }
                            }
                            let generic_ret = self.types.functions.get(&fn_key)
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
                            let generic_has_self = self.mono.generic_fn_decls.iter()
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
                        // non-local) is NOT an instance ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢ no phantom receiver arg.
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
                        let use_registered = self.types.functions.get(&resolved_fn_key)
                            .map(|(pts, _)| pts.len() == compiled_args.len())
                            .unwrap_or(false);
                        if use_registered {
                            let pts = self.types.functions[&resolved_fn_key].0.clone();
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
                            for (k, (_, rt)) in &self.types.functions {
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
                        && self.types.functions.get(&resolved_fn_key).is_none()
                        && ret_ty == "i64";
                    if callee_is_fn_ptr {
                        let (alloca_reg, local_llvm_ty) = self.lookup_local(&fn_name).cloned().expect("fn_ptr target must be in locals");
                        let fn_ptr_loaded = self.fresh_tmp();
                        self.emitln(&format!("  {fn_ptr_loaded} = load {local_llvm_ty}, {local_llvm_ty}* {alloca_reg}"));
                        let param_types: Vec<String> = args.iter().map(|a| self.infer_llvm_type(a)).collect();
                        let actual_ret_ty = if ret_ty == "i64" {
                            self.types.fn_ptr_return_types.get(&fn_name).cloned().unwrap_or_else(|| "i64".to_string())
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
                        // 5e.5a: hot reload — redirect pub fn calls through thunks
                        let thunk_name = format!("xiom_hot_thunk_{}", resolved_fn_key);
                        if ret_ty == "void" {
                            self.emitln(&format!("  call void @{thunk_name}({args_str})"));
                            Ok((String::new(), "void".to_string()))
                        } else {
                            self.emitln(&format!("  {tmp} = call {ret_ty} @{thunk_name}({args_str})"));
                            if let Some(receiver) = receiver_expr {
                                if ret_ty.starts_with("%struct.") {
                                    self.store_back_to_receiver(receiver, &tmp, &ret_ty);
                                }
                            }
                            Ok((tmp, ret_ty.clone()))
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
}
