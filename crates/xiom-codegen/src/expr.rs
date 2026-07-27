use xiom_ast::*;
use crate::llvm_consts::*;

use super::IrEmitter;

impl IrEmitter {
    pub(crate) fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        self.compile_stmt_impl(stmt)
    }


    /// Lower a single binary operation for the iterative fold path (deep-chain
    /// hardening). Takes pre-compiled operands and produces the folded result.
    /// Handles arithmetic (Add/Sub/Mul), bitwise (And/Or/Xor), and float coercions.
    fn compile_binop_fold(&mut self, l: &str, lt: &str, r: &str, rt: &str, op: &BinOp) -> Result<(String, String), String> {
        // String concatenation: Add with i8* operands must call xiom_str_concat,
        // not emit `add i64` on pointer values. The normal BinOp path checks this
        // first; replicate the check here for the iterative fold path.
        if matches!(op, BinOp::Add) && (lt == "i8*" || rt == "i8*") {
            let lp = self.val_to_i8ptr(l, lt);
            let rp = self.val_to_i8ptr(r, rt);
            let res = self.fresh_tmp();
            self.emitln(&format!("  {res} = call i8* @xiom_str_concat(i8* {lp}, i8* {rp})"));
            return Ok((res, LLVM_STR_PTR.to_string()));
        }
        let is_float = lt == "float" || lt == "double" || rt == "float" || rt == "double";
        let float_ty = if lt == "float" || rt == "float" { "float" } else { "double" };
        let is_add_sub_mul = matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul);
        let ty = if is_float && is_add_sub_mul { float_ty } else { "i64" };
        let llvm_op = match op {
            BinOp::Add => if is_float { "fadd" } else { "add" },
            BinOp::Sub => if is_float { "fsub" } else { "sub" },
            BinOp::Mul => if is_float { "fmul" } else { "mul" },
            BinOp::BitAnd => "and",
            BinOp::BitOr => "or",
            BinOp::BitXor => "xor",
            _ => return Err(format!("compile_binop_fold: unsupported op {op:?}")),
        };
        // Coerce to float if needed
        let (mut lv, mut rv) = (l.to_string(), r.to_string());
        if is_float && is_add_sub_mul {
            if lt == "i64" {
                let conv = self.fresh_tmp();
                self.emitln(&format!("  {conv} = sitofp i64 {l} to {float_ty}"));
                lv = conv;
            }
            if rt == "i64" {
                let conv = self.fresh_tmp();
                self.emitln(&format!("  {conv} = sitofp i64 {r} to {float_ty}"));
                rv = conv;
            }
            if float_ty == "float" {
                if lt == "double" {
                    let conv = self.fresh_tmp();
                    self.emitln(&format!("  {conv} = fptrunc double {l} to float"));
                    lv = conv;
                }
                if rt == "double" {
                    let conv = self.fresh_tmp();
                    self.emitln(&format!("  {conv} = fptrunc double {r} to float"));
                    rv = conv;
                }
            }
        }
        // Widen narrow integers for non-float ops.
        // Extract scalar fields from struct operands (e.g. Option::unwrap()
        // returns a struct value used in arithmetic).
        if !is_float && ty == "i64" {
            lv = self.widen_to_i64(&lv, lt);
            rv = self.widen_to_i64(&rv, rt);
        }
        // Struct operands in arithmetic context: extract the leading scalar.
        // Handles patterns like Some(x).unwrap() + 1 being folded.
        if lt.starts_with("%struct.") && ty == "i64" {
            lv = self.extract_scalar_field0(&lv, lt);
        }
        if rt.starts_with("%struct.") && ty == "i64" {
            rv = self.extract_scalar_field0(&rv, rt);
        }
        let tmp = self.fresh_tmp();
        // M18: Integer overflow protection for Add/Sub/Mul on i64.
        // Use LLVM @llvm.sadd/sub/mul.with.overflow.i64 to trap on overflow.
        if self.config.overflow_checks && is_add_sub_mul && ty == "i64" {
            let intrinsic = match op {
                BinOp::Add => "llvm.sadd.with.overflow.i64",
                BinOp::Sub => "llvm.ssub.with.overflow.i64",
                BinOp::Mul => "llvm.smul.with.overflow.i64",
                _ => unreachable!(),
            };
            let ov_struct = self.fresh_tmp();
            self.emitln(&format!("  {ov_struct} = call {{i64, i1}} @{intrinsic}(i64 {lv}, i64 {rv})"));
            let result_val = self.fresh_tmp();
            self.emitln(&format!("  {result_val} = extractvalue {{i64, i1}} {ov_struct}, 0"));
            let overflow_flag = self.fresh_tmp();
            self.emitln(&format!("  {overflow_flag} = extractvalue {{i64, i1}} {ov_struct}, 1"));
            let ok_block = self.fresh_block("ov_ok");
            let trap_block = self.fresh_block("ov_trap");
            self.emitln(&format!("  br i1 {overflow_flag}, label %{trap_block}, label %{ok_block}"));
            self.emitln(&format!("\n{trap_block}:"));
            self.emitln("  call void @llvm.trap()");
            self.emitln("  unreachable");
            self.emitln(&format!("\n{ok_block}:"));
            return Ok((result_val, ty.to_string()));
        }
        self.emitln(&format!("  {tmp} = {llvm_op} {ty} {lv}, {rv}"));
        Ok((tmp, ty.to_string()))
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
                if let Some(val) = self.mono.current_const_map.get(&ident.name) {
                    return Ok((val.to_string(), LLVM_I64.to_string()));
                }
                let lookup_name: &str = if ident.name == "this" { "self" } else { &ident.name };
                if let Some((ptr, llvm_ty)) = self.lookup_local(lookup_name).cloned() {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                    // Propagate array-value tracking through let-bound locals:
                    // if `ident` was bound from an Expr::Array, the loaded value
                    // also originates from an array buffer so val_to_struct can
                    // construct a proper Vec from it.
                    if self.local.array_locals.contains(&ident.name) {
                        self.local.array_value_regs.insert(tmp.clone());
                    }
                    Ok((tmp, llvm_ty))
                } else if let Some((symbol, llvm_ty)) = self.local.module_globals.get(&ident.name).cloned() {
                    // Mutable module-level `var`: load the current value from the
                    // real global. Checked BEFORE enum-variant / constant fallbacks
                    // so a live global is never mistaken for a compile-time literal.
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* @{symbol}"));
                    Ok((tmp, llvm_ty))
                } else if let Some(enum_key) = self.types.enum_variants.iter()
                    .find(|(_, vars)| vars.iter().any(|(v, _)| v == &ident.name))
                    .map(|(ek, _)| ek)
                    .filter(|ek| self.types.types.contains_key(*ek))
                {
                    if let Some(vars) = self.types.enum_variants.get(enum_key) {
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
                        Ok(("0".to_string(), LLVM_I64.to_string()))
                    }
                } else {
                        Ok(("0".to_string(), LLVM_I64.to_string()))
                    }
                } else {
                    // A bare reference to a module/global constant: substitute its
                    // literal value (constants aren't materialized as globals).
                    if let Some(cval) = self.local.constants.get(&ident.name).cloned() {
                        return self.compile_expr(&cval);
                    }
                    // Function name used as value (e.g. v.push(add_one)):
                    // resolve to a function pointer via ptrtoint of the IR symbol.
                    // The functions map has both bare names and module-qualified names.
                    let fn_full: Option<(String, String, Vec<String>)> = {
                        let exact = self.types.functions.get(&ident.name).map(|(p, r)| (ident.name.clone(), r.clone(), p.clone()));
                        exact.or_else(|| {
                            self.types.functions.iter().find(|(k, _)| k.ends_with(&format!(".{}", ident.name)))
                                .map(|(k, (p, r))| (k.clone(), r.clone(), p.clone()))
                        })
                    };
                    if let Some((fn_name, ret_ty, param_tys)) = fn_full {
                        let fpty = format!("{ret_ty} ({})*", param_tys.join(", "));
                        let fp = self.fresh_tmp();
                        self.emitln(&format!("  {fp} = ptrtoint {fpty} @{fn_name} to i64"));
                        return Ok((fp, LLVM_I64.to_string()));
                    }
                    // G-20: a bare receiver-FIELD reference in a method body with
                    // no receiver slot (no self param, no &T receiver-style param,
                    // no `this` usage → no %param_self). Emitting the generic `0`
                    // fallback here produced silent wrong values. Fail loudly with
                    // the fix.
                    if let Some(recv) = self.fctx.current_receiver.clone() {
                        let fields = self.types.type_meta.get(&recv)
                            .or_else(|| {
                                let suffix = format!(".{recv}");
                                self.types.type_meta.iter()
                                    .find(|(k, _)| k.ends_with(&suffix))
                                    .map(|(_, v)| v)
                            });
                        if let Some(meta) = fields {
                            if meta.fields.iter().any(|(fname, _)| fname == &ident.name) {
                                return Err(format!(
                                    "receiver field '{0}' cannot be read bare in this method — use 'this.{0}' (bare fields need a `self` param, a `&{1}` receiver-style first param, or a `this`-based body)",
                                    ident.name, recv
                                ));
                            }
                        }
                    }
                    Ok(("0".to_string(), LLVM_I64.to_string()))
                }
            }
            Expr::Int(n, _) => {
                Ok((format!("{n}"), LLVM_I64.to_string()))
            }
            Expr::Float(f, _) => {
                Ok((format!("{f:.6}"), "double".to_string()))
            }
            Expr::Bool(b, _) => {
                Ok((if *b { "1".to_string() } else { "0".to_string() }, LLVM_I64.to_string()))
            }
            Expr::Str(s, _) => {
                let tmp = self.intern_cstring(s);
                Ok((tmp, LLVM_STR_PTR.to_string()))
            }
            Expr::Char(c, _) => {
                Ok((format!("{}", *c as u32), "i8".to_string()))
            }
            Expr::Paren(inner, _) => self.compile_expr(inner),
            Expr::Tuple(items, _) => {
                if items.is_empty() {
                    Ok(("0".to_string(), "void".to_string()))
                } else {
                    let mut struct_ty = self.infer_llvm_type(expr);
                    // Fix: If inference falls back to i64 (because element types
                    // don't match registered struct), try the function return type.
                    if !struct_ty.starts_with("%struct.") {
                        if self.fctx.current_return_type.starts_with("%struct.") {
                            struct_ty = self.fctx.current_return_type.clone();
                        } else {
                            return Ok(("0".to_string(), LLVM_I64.to_string()));
                        }
                    }
                    // Parse field types from struct name: %struct.Tuple_Float32_Int →
                    // field LLVM types: [float, i64]
                    let field_types = self.parse_struct_field_types(&struct_ty);
                    let alloca = self.fresh_tmp();
                    self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                    for (i, item) in items.iter().enumerate() {
                        let (mut item_val, mut item_ty) = self.compile_expr(item)?;
                        // Coerce float literals to match struct field width.
                        // e.g., 0.0 defaults to double, but Float32 field needs float.
                        if let Some(needed) = field_types.get(i) {
                            if *needed != item_ty {
                                item_val = self.coerce_value(&item_val, &item_ty, needed);
                                item_ty = needed.clone();
                            }
                        }
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
                            return Ok((tmp, LLVM_I64.to_string()));
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
                        return Ok((tmp, LLVM_I64.to_string()));
                    }
                    UnaryOp::BitNot => {
                        self.emitln(&format!("  {tmp} = xor i64 {val}, -1"));
                        return Ok((tmp, LLVM_I64.to_string()));
                    }
                    UnaryOp::Deref => {
                        // `*p`: load through a real pointer. `inner_ty` is e.g. `i64*`
                        // (from a `*T` value). Load the pointee type.
                        if inner_ty.ends_with('*') {
                            let pointee = inner_ty.trim_end_matches('*').to_string();
                            self.emitln(&format!("  {tmp} = load {pointee}, {inner_ty} {val}"));
                            return Ok((tmp, pointee));
                        }
                        // M19: If the operand is i64 (ptrtoint'd pointer from
                        // ptr.offset()), convert to i8* and load a byte. This
                        // fixes *(ptr.offset(i)) in stdlib io.read_file.
                        if inner_ty == "i64" {
                            let ptr = self.fresh_tmp();
                            self.emitln(&format!("  {ptr} = inttoptr i64 {val} to i8*"));
                            let loaded = self.fresh_tmp();
                            self.emitln(&format!("  {loaded} = load i8, i8* {ptr}"));
                            let ext = self.fresh_tmp();
                            self.emitln(&format!("  {ext} = zext i8 {loaded} to i64"));
                            return Ok((ext, "i64".to_string()));
                        }
                        // Legacy path: not a pointer, return unchanged.
                        return Ok((val, inner_ty));
                    }
                    UnaryOp::Ref | UnaryOp::MutRef => return Ok((val, inner_ty)),
                }
            }
            Expr::Binary(left, op, right, _) => {
                // Production hardening: deep left-associative chains of the same
                // arithmetic/bitwise operator can create ASTs thousands of levels
                // deep, overflowing the native stack via recursive compile_expr.
                // Detect this pattern and compile operands iteratively via an
                // explicit work list, avoiding stack overflow.
                let is_foldable = matches!(op,
                    BinOp::Add | BinOp::Sub | BinOp::Mul |
                    BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor
                );
                if is_foldable {
                    // Walk the left-associative chain collecting operands
                    let mut operands: Vec<Expr> = Vec::new();
                    let mut current: &Expr = left;
                    loop {
                        if let Expr::Binary(inner_left, inner_op, inner_right, _) = current {
                            if *inner_op == *op {
                                operands.push((**inner_right).clone());
                                current = inner_left;
                                continue;
                            }
                        }
                        operands.push(current.clone());
                        break;
                    }
                    operands.reverse();
                    operands.push((**right).clone());
                    if operands.len() > 2 {
                        // Compile the first operand
                        let (mut acc_val, mut acc_ty) = self.compile_expr(&operands[0])?;
                        // Iteratively compile and fold each remaining operand
                        for operand in &operands[1..] {
                            let (r_val, r_ty) = self.compile_expr(operand)?;
                            let (l, lt) = (acc_val, acc_ty);
                            let (r, rt) = (r_val, r_ty);
                            // Reuse the standard BinOp lowering for each pair
                            let folded = self.compile_binop_fold(&l, &lt, &r, &rt, op)?;
                            acc_val = folded.0;
                            acc_ty = folded.1;
                        }
                        return Ok((acc_val, acc_ty));
                    }
                }
                // Normal path (short chain or non-foldable operator)
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
                    return Ok((res, LLVM_STR_PTR.to_string()));
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
                    return Ok((result, LLVM_I64.to_string()));
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
                        if self.types.functions.contains_key(&eq_fn) {
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
                            return Ok((negated, LLVM_I64.to_string()));
                        }
                        return Ok((eq_result, LLVM_I64.to_string()));
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
                                return Ok((neg, LLVM_I64.to_string()));
                            }
                            return Ok((zext, LLVM_I64.to_string()));
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
                        return Ok((ext, LLVM_I64.to_string()));
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
                    _ => unreachable!("binary operation not lowered to LLVM IR"),
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
                // M18: Integer overflow protection for Add/Sub/Mul on i64.
                if self.config.overflow_checks && !is_float && matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul) && ty == "i64" {
                    let intrinsic = match op {
                        BinOp::Add => "llvm.sadd.with.overflow.i64",
                        BinOp::Sub => "llvm.ssub.with.overflow.i64",
                        BinOp::Mul => "llvm.smul.with.overflow.i64",
                        _ => unreachable!(),
                    };
                    let ov_struct = self.fresh_tmp();
                    self.emitln(&format!("  {ov_struct} = call {{i64, i1}} @{intrinsic}(i64 {l}, i64 {r})"));
                    let result_val = self.fresh_tmp();
                    self.emitln(&format!("  {result_val} = extractvalue {{i64, i1}} {ov_struct}, 0"));
                    let overflow_flag = self.fresh_tmp();
                    self.emitln(&format!("  {overflow_flag} = extractvalue {{i64, i1}} {ov_struct}, 1"));
                    let ok_block = self.fresh_block("ov_ok");
                    let trap_block = self.fresh_block("ov_trap");
                    self.emitln(&format!("  br i1 {overflow_flag}, label %{trap_block}, label %{ok_block}"));
                    self.emitln(&format!("\n{trap_block}:"));
                    self.emitln("  call void @llvm.trap()");
                    self.emitln("  unreachable");
                    self.emitln(&format!("\n{ok_block}:"));
                    if let Some(cont) = div_cont {
                        self.emitln(&format!("  br label %{cont}"));
                        self.emitln(&format!("\n{cont}:"));
                    }
                    return Ok((result_val, "i64".to_string()));
                }
                self.emitln(&format!("  {tmp} = {inst} {ty} {l}, {r}"));
                let (result, result_ty) = if inst.starts_with("icmp") || inst.starts_with("fcmp") {
                    let ext = self.fresh_tmp();
                    self.emitln(&format!("  {ext} = zext i1 {tmp} to i64"));
                    (ext, LLVM_I64.to_string())
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
                        self.types.used_builtins.insert("Option".to_string());
                        true
                    }
                    Expr::Ok(..) | Expr::Err(..) => {
                        self.types.used_builtins.insert("Result".to_string());
                        false
                    }
                    Expr::Ident(id) => {
                        // Check type from locals
                        let mut opt_like = true;
                        if let Some((_, llvm_ty)) = self.lookup_local(&id.name) {
                            if llvm_ty.starts_with("%struct.") {
                                let type_name = &llvm_ty[8..];
                                if let Some(field_names) = self.types.types.get(type_name) {
                                    opt_like = field_names.len() <= 2;
                                }
                            }
                        }
                        if opt_like { self.types.used_builtins.insert("Option".to_string()); }
                        else { self.types.used_builtins.insert("Result".to_string()); }
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
                            if let Some(field_names) = self.types.types.get(&name) {
                                let opt_like = field_names.len() <= 2;
                                if opt_like { self.types.used_builtins.insert("Option".to_string()); }
                                else { self.types.used_builtins.insert("Result".to_string()); }
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
                        self.types.used_builtins.insert("Result".to_string());
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
                    let ret_ty = self.fctx.current_return_type.clone();
                    let default_val = if ret_ty.starts_with('%') { "zeroinitializer".to_string() } else { "0".to_string() };
                    self.emitln(&format!("  ret {ret_ty} {default_val}"));
                    self.emitln(&format!("\n{some_block}:"));
                    let val_gep = self.fresh_tmp();
                    let some_val = self.fresh_tmp();
                    self.emitln(&format!("  {val_gep} = getelementptr {opt_ty}, {opt_ty}* {opt_alloca}, i32 0, i32 1"));
                    self.emitln(&format!("  {some_val} = load i64, i64* {val_gep}"));
                    Ok((some_val, LLVM_I64.to_string()))
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
                    let ret_ty = self.fctx.current_return_type.clone();
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
                    Ok((ok_val, LLVM_I64.to_string()))
                }
            }
            Expr::Imply(left, right, _) => {
                let (l, _lt) = self.compile_expr(left)?;
                let (r, _rt) = self.compile_expr(right)?;
                let tmp1 = self.fresh_tmp();
                let tmp2 = self.fresh_tmp();
                self.emitln(&format!("  {tmp1} = xor i64 {l}, 1"));
                self.emitln(&format!("  {tmp2} = or i64 {tmp1}, {r}"));
                Ok((tmp2, LLVM_I64.to_string()))
            }
            Expr::Is(expr, pattern, _) => {
                let (val, ty) = self.compile_expr(expr)?;
                if ty.starts_with("%struct.") {
                    let variant_name = match &pattern {
                        xiom_ast::Pattern::Some(..) => "Some",
                        xiom_ast::Pattern::None(..) => "None",
                        xiom_ast::Pattern::Ok(..) => "Ok",
                        xiom_ast::Pattern::Err(..) => "Err",
                        _ => { return Ok(("1".to_string(), LLVM_I64.to_string())); }
                    };
                    let type_name = &ty[8..];
                    if let Some(variants) = self.types.enum_variants.get(type_name) {
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
                            return Ok((ext, LLVM_I64.to_string()));
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
                        return Ok((ext, LLVM_I64.to_string()));
                    } else {
                        let cmp = self.fresh_tmp();
                        self.emitln(&format!("  {cmp} = icmp eq i64 {loaded}, 0"));
                        let ext = self.fresh_tmp();
                        self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
                        return Ok((ext, LLVM_I64.to_string()));
                    }
                }
                Ok(("1".to_string(), LLVM_I64.to_string()))
            }
            Expr::Field(obj, field, _) => {
                // Module-qualified constant, e.g. `simd.SIMD_SSE`: when the object is
                // NOT a value instance (a module path), and the leaf names a known
                // constant, substitute its literal value. Constants are keyed by their
                // bare name, so `simd.SIMD_SSE` resolves via `SIMD_SSE`.
                if !self.receiver_is_instance(obj) {
                    if let Some(cval) = self.local.constants.get(&field.name).cloned() {
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
                                if let Some(field_names) = self.types.types.get(type_name)
                                    .or_else(|| {
                                        let suffix = format!(".{type_name}");
                                        self.types.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                            .and_then(|k| self.types.types.get(k))
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
                            if let Some(tn) = self.local.local_boxed_struct.get(&obj_ident.name).cloned() {
                                if let Some(field_names) = self.types.types.get(&tn)
                                    .or_else(|| {
                                        let suffix = format!(".{tn}");
                                        self.types.types.keys().find(|k| k.ends_with(&suffix))
                                            .and_then(|k| self.types.types.get(k))
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
                            if let Some(field_names) = self.types.types.get(type_name)
                                .or_else(|| {
                                    let suffix = format!(".{type_name}");
                                    self.types.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                        .and_then(|k| self.types.types.get(k))
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
                            let is_result = type_name.ends_with("Result") || type_name.contains(".Result") || type_name.starts_with("Result__");
                            let is_option = type_name.ends_with("Option") || type_name.contains(".Option") || type_name.starts_with("Option__");
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
                                return Ok((result, LLVM_I64.to_string()));
                            }
                        }
                        // Check if it's a (by-value) struct type. Exclude pointer
                        // types (handled above) so `%struct.X*` never takes this path.
                        if llvm_ty.starts_with("%struct.") && !llvm_ty.ends_with('*') {
                            // Find field index
                            let type_name = &llvm_ty[8..];
                            if let Some(field_names) = self.types.types.get(type_name)
                                .or_else(|| {
                                    let suffix = format!(".{type_name}");
                                    self.types.types.keys().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                        .and_then(|k| self.types.types.get(k))
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
                    if let Some(vars) = self.types.enum_variants.get(&enum_key) {
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
                        let is_result = type_name.ends_with("Result") || type_name.contains(".Result") || type_name.starts_with("Result__");
                        let is_option = type_name.ends_with("Option") || type_name.contains(".Option") || type_name.starts_with("Option__");
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
                            return Ok((result, LLVM_I64.to_string()));
                        }
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
                Ok(("0".to_string(), LLVM_I64.to_string()))
            }
            Expr::Call(func, args, _) => self.compile_call(func, args),
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
                    || (if let Expr::Ident(ident) = container.as_ref() { self.local.array_locals.contains(&ident.name) } else { false })
                );
                if is_array_buf {
                    let base_ptr = self.fresh_tmp();
                    // 5c-R: use the declared element type for arrays (G-11).
                    // Default to i64 for backward-compat. Tracked via local_array_elem.
                    let arr_elem_ty = if let Expr::Ident(ident) = container.as_ref() {
                        self.local.local_array_elem.get(&ident.name).cloned().unwrap_or_else(|| LLVM_I64.to_string())
                    } else { LLVM_I64.to_string() };
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
                    return Ok((ext, LLVM_I64.to_string()));
                }
                // Str: char access via the raw runtime accessor, returned as i64.
                if cont_ty == "i8*" {
                    let ch = self.fresh_tmp();
                    self.emitln(&format!("  {ch} = call i8 @xiom_char_at(i8* {cont_val}, i64 {idx})"));
                    let ext = self.fresh_tmp();
                    self.emitln(&format!("  {ext} = zext i8 {ch} to i64"));
                    return Ok((ext, LLVM_I64.to_string()));
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
                    return Ok((elem, LLVM_I64.to_string()));
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
                    return Ok((result, LLVM_I64.to_string()));
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
                    return Ok((result, LLVM_I64.to_string()));
                }
                // safe default.
                Ok(("0".to_string(), LLVM_I64.to_string()))
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
                                if let Some(field_names) = self.types.types.get(&type_name)
                                    .or_else(|| {
                                        let suffix = format!(".{type_name}");
                                        self.types.types.keys().find(|k| k.ends_with(&suffix))
                                            .and_then(|k| self.types.types.get(k))
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
                    // 5c-E: Empty array (n == 0) — construct a zeroed Vec without malloc.
                    // malloc(0) returns NULL on many platforms, causing a trap.
                    if n == 0 {
                        let vec_alloca = self.fresh_tmp();
                        let struct_ty = "%struct.Vec";
                        self.emitln(&format!("  {vec_alloca} = alloca {struct_ty}"));
                        let g0 = self.fresh_tmp();
                        self.emitln(&format!("  {g0} = getelementptr {struct_ty}, {struct_ty}* {vec_alloca}, i32 0, i32 0"));
                        self.emitln(&format!("  store i8* null, i8** {g0}"));
                        let g1 = self.fresh_tmp();
                        self.emitln(&format!("  {g1} = getelementptr {struct_ty}, {struct_ty}* {vec_alloca}, i32 0, i32 1"));
                        self.emitln(&format!("  store i64 0, i64* {g1}"));
                        let g2 = self.fresh_tmp();
                        self.emitln(&format!("  {g2} = getelementptr {struct_ty}, {struct_ty}* {vec_alloca}, i32 0, i32 2"));
                        self.emitln(&format!("  store i64 0, i64* {g2}"));
                        let g3 = self.fresh_tmp();
                        self.emitln(&format!("  {g3} = getelementptr {struct_ty}, {struct_ty}* {vec_alloca}, i32 0, i32 3"));
                        self.emitln(&format!("  store i64 4, i64* {g3}"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {vec_alloca}"));
                        return Ok((loaded, struct_ty.to_string()));
                    }
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
                self.types.used_builtins.insert("Option".to_string());
                let (val, inner_ty) = self.compile_expr(inner)?;
                // Use the function's return type so concrete monomorphised
                // types (Option__Point) get the correct struct layout (B-001).
                let opt_ty = if self.fctx.current_return_type.starts_with("%struct.") {
                    self.fctx.current_return_type.clone()
                } else {
                    "%struct.Option".to_string()
                };
                let struct_name = opt_ty.trim_start_matches("%struct.");
                let field_type_1 = self.types.type_meta.get(struct_name)
                    .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_1 = self.llvm_type_for(&field_type_1)
                    .unwrap_or_else(|_| "i64".to_string());
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {opt_ty}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {opt_ty}, {opt_ty}* {alloca}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 1, i64* {gep0}"));
                let gep1 = self.fresh_tmp();
                self.emitln(&format!("  {gep1} = getelementptr {opt_ty}, {opt_ty}* {alloca}, i32 0, i32 1"));
                if field_llvm_1 == "i64" {
                    let store_val = self.val_to_i64(&val, &inner_ty);
                    self.emitln(&format!("  store i64 {store_val}, i64* {gep1}"));
                } else {
                    // Struct-typed value field — store the struct directly
                    let store_val = self.coerce_value(&val, &inner_ty, &field_llvm_1);
                    self.emitln(&format!("  store {field_llvm_1} {store_val}, {field_llvm_1}* {gep1}"));
                }
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {opt_ty}, {opt_ty}* {alloca}"));
                Ok((loaded, opt_ty.to_string()))
            }
            Expr::None(_) => {
                self.types.used_builtins.insert("Option".to_string());
                let opt_ty = if self.fctx.current_return_type.starts_with("%struct.") {
                    self.fctx.current_return_type.clone()
                } else {
                    "%struct.Option".to_string()
                };
                let struct_name = opt_ty.trim_start_matches("%struct.");
                let field_type_1 = self.types.type_meta.get(struct_name)
                    .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_1 = self.llvm_type_for(&field_type_1)
                    .unwrap_or_else(|_| "i64".to_string());
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {opt_ty}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {opt_ty}, {opt_ty}* {alloca}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 0, i64* {gep0}"));
                let gep1 = self.fresh_tmp();
                self.emitln(&format!("  {gep1} = getelementptr {opt_ty}, {opt_ty}* {alloca}, i32 0, i32 1"));
                if field_llvm_1 == "i64" {
                    self.emitln(&format!("  store i64 0, i64* {gep1}"));
                } else {
                    self.emitln(&format!("  store {field_llvm_1} zeroinitializer, {field_llvm_1}* {gep1}"));
                }
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {opt_ty}, {opt_ty}* {alloca}"));
                Ok((loaded, opt_ty.to_string()))
            }
            Expr::Ok(inner, _) => {
                self.types.used_builtins.insert("Result".to_string());
                let (val, inner_ty) = self.compile_expr(inner)?;
                let result_ty = if self.fctx.current_return_type.starts_with("%struct.") {
                    self.fctx.current_return_type.clone()
                } else {
                    "%struct.Result".to_string()
                };
                let struct_name = result_ty.trim_start_matches("%struct.");
                let field_type_1 = self.types.type_meta.get(struct_name)
                    .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_1 = self.llvm_type_for(&field_type_1)
                    .unwrap_or_else(|_| "i64".to_string());
                let field_type_2 = self.types.type_meta.get(struct_name)
                    .and_then(|m| m.fields.get(2).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_2 = self.llvm_type_for(&field_type_2)
                    .unwrap_or_else(|_| "i64".to_string());
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {result_ty}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 1, i64* {gep0}"));
                let gep1 = self.fresh_tmp();
                self.emitln(&format!("  {gep1} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 1"));
                if field_llvm_1 == "i64" {
                    let store_val = self.val_to_i64(&val, &inner_ty);
                    self.emitln(&format!("  store i64 {store_val}, i64* {gep1}"));
                } else {
                    let store_val = self.coerce_value(&val, &inner_ty, &field_llvm_1);
                    self.emitln(&format!("  store {field_llvm_1} {store_val}, {field_llvm_1}* {gep1}"));
                }
                let gep2 = self.fresh_tmp();
                self.emitln(&format!("  {gep2} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 2"));
                if field_llvm_2 == "i64" {
                    self.emitln(&format!("  store i64 0, i64* {gep2}"));
                } else {
                    self.emitln(&format!("  store {field_llvm_2} zeroinitializer, {field_llvm_2}* {gep2}"));
                }
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {result_ty}, {result_ty}* {alloca}"));
                Ok((loaded, result_ty.to_string()))
            }
            Expr::Err(inner, _) => {
                self.types.used_builtins.insert("Result".to_string());
                let (val, inner_ty) = self.compile_expr(inner)?;
                let result_ty = if self.fctx.current_return_type.starts_with("%struct.") {
                    self.fctx.current_return_type.clone()
                } else {
                    "%struct.Result".to_string()
                };
                let struct_name = result_ty.trim_start_matches("%struct.");
                let field_type_1 = self.types.type_meta.get(struct_name)
                    .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_1 = self.llvm_type_for(&field_type_1)
                    .unwrap_or_else(|_| "i64".to_string());
                let field_type_2 = self.types.type_meta.get(struct_name)
                    .and_then(|m| m.fields.get(2).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_2 = self.llvm_type_for(&field_type_2)
                    .unwrap_or_else(|_| "i64".to_string());
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {result_ty}"));
                let gep0 = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 0"));
                self.emitln(&format!("  store i64 0, i64* {gep0}"));
                let gep1 = self.fresh_tmp();
                self.emitln(&format!("  {gep1} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 1"));
                if field_llvm_1 == "i64" {
                    self.emitln(&format!("  store i64 0, i64* {gep1}"));
                } else {
                    self.emitln(&format!("  store {field_llvm_1} zeroinitializer, {field_llvm_1}* {gep1}"));
                }
                let gep2 = self.fresh_tmp();
                self.emitln(&format!("  {gep2} = getelementptr {result_ty}, {result_ty}* {alloca}, i32 0, i32 2"));
                if field_llvm_2 == "i64" {
                    let store_val = self.val_to_i64(&val, &inner_ty);
                    self.emitln(&format!("  store i64 {store_val}, i64* {gep2}"));
                } else {
                    let store_val = self.coerce_value(&val, &inner_ty, &field_llvm_2);
                    self.emitln(&format!("  store {field_llvm_2} {store_val}, {field_llvm_2}* {gep2}"));
                }
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {result_ty}, {result_ty}* {alloca}"));
                Ok((loaded, result_ty.to_string()))
            }
            Expr::Struct(name, fields, _spread, _span) => {
                // If `name` is an enum variant (e.g., `Single`), resolve to parent enum type
                let parent_enum = self.types.enum_variants.iter()
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
                    let var_idx = self.types.enum_variants.get(enum_key)
                        .and_then(|vars| vars.iter().position(|(v, _)| v == &name.name))
                        .unwrap_or(0);
                    let disc_gep = self.fresh_tmp();
                    self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                    self.emitln(&format!("  store i64 {var_idx}, i64* {disc_gep}"));
                    // Map variant fields to their parent enum offsets (after discriminant)
                    // The parent enum stores field names uniquely across all variants,
                    // so we need to look up the actual field index in the parent's field list.
                    let parent_field_names = self.types.types.get(enum_key).cloned().unwrap_or_default();
                    let variant_fields = self.types.enum_variants.get(enum_key)
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
                            field_val_ty = LLVM_I64.to_string();
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
                if self.config.check_contracts {
                    if let Some(meta) = self.types.type_meta.get(&name.name) {
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
                    if t == "double" || t == "float" || t.starts_with("%struct.") { t } else { LLVM_I64.to_string() }
                } else { LLVM_I64.to_string() };
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
                self.local.array_value_regs.insert(ptr.clone());
                Ok((ptr, LLVM_STR_PTR.to_string()))
            }
            Expr::PipeClosure(params, body, _) => {
                // M20-A1: Full closure implementation — capturing + non-capturing.
                let closure_id = self.tmp_counter;
                self.tmp_counter += 1;
                let fn_name = format!("__closure_{closure_id}");
                let env_name = format!("__closure_env_{closure_id}");
                
                // Detect captured (free) variables: identifiers in the body
                // that are in the local scope but NOT closure params.
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let captures = self.collect_free_vars(body, &param_names);
                
                if captures.is_empty() {
                    // === NON-CAPTURING: use uniform env struct rep ===
                    // Even non-capturing closures get a minimal env struct
                    // with just the fn_ptr, so the call site is uniform.
                    let saved_output = std::mem::take(&mut self.output);
                    let saved_tmp = self.tmp_counter;
                    let saved_block = self.block_counter;
                    self.tmp_counter = closure_id * 1000;
                    self.block_counter = closure_id * 1000;
                    self.push_scope();
                    
                    let param_str: Vec<String> = params.iter().enumerate()
                        .map(|(i, p)| format!("%{}_{}", p.name, i))
                        .collect();
                    let header = format!("define i64 @{fn_name}(i64 %__env, {}) {{\nentry:\n",
                        param_str.iter().enumerate()
                            .map(|(i, s)| format!("i64 {}", s))
                            .collect::<Vec<_>>().join(", "));
                    self.output.push_str(&header);
                    
                    for (i, p) in params.iter().enumerate() {
                        let preg = format!("%{}_{}", p.name, i);
                        let a = format!("%{}_{}_alloca", p.name, i);
                        self.emitln(&format!("  {a} = alloca i64"));
                        self.emitln(&format!("  store i64 {preg}, i64* {a}"));
                        self.add_local(&p.name, a, "i64");
                    }
                    
                    let (ret_val, ret_ty) = self.compile_expr(body)?;
                    let result = self.val_to_i64(&ret_val, &ret_ty);
                    self.emitln(&format!("  ret i64 {}", result));
                    self.emitln("}");
                    self.pop_scope();
                    
                    let closure_ir = std::mem::take(&mut self.output);
                    self.local.deferred_closure_defs.push(closure_ir);
                    self.output = saved_output;
                    self.tmp_counter = saved_tmp;
                    self.block_counter = saved_block;
                    
                    // Emit env struct: { fn_ptr }
                    let env_def = format!("%struct.{env_name} = type {{ i64 }}\n");
                    if let Some(define_pos) = self.output.find("define ") {
                        self.output.insert_str(define_pos, &env_def);
                    }
                    
                    // Malloc env, store fn_ptr, return ptr as closure value
                    let env_ptr = self.fresh_tmp();
                    let malloc_call = self.fresh_tmp();
                    self.emitln(&format!("  {malloc_call} = call i8* @malloc(i64 8)"));
                    self.emitln(&format!("  {env_ptr} = bitcast i8* {malloc_call} to %struct.{env_name}*"));
                    let gep0 = self.fresh_tmp();
                    let fn_ptr_i64 = self.fresh_tmp();
                    self.emitln(&format!("  {gep0} = getelementptr %struct.{env_name}, %struct.{env_name}* {env_ptr}, i32 0, i32 0"));
                    self.emitln(&format!("  {fn_ptr_i64} = ptrtoint ptr @{fn_name} to i64"));
                    self.emitln(&format!("  store i64 {fn_ptr_i64}, i64* {gep0}"));
                    let closure_val = self.fresh_tmp();
                    self.emitln(&format!("  {closure_val} = ptrtoint %struct.{env_name}* {env_ptr} to i64"));
                    return Ok((closure_val, LLVM_I64.to_string()));
                }
                
                // === CAPTURING CLOSURE ===
                // Strategy: heap-allocated env struct with { fn_ptr, captured_vals... }
                // The thunk takes (env_ptr, params...) and loads captures from env.
                
                // Build the env struct type — defer to before function body
                let mut env_fields = vec!["i64".to_string()]; // field 0: fn_ptr
                for (_cap_name, cap_llvm_ty) in &captures {
                    env_fields.push(cap_llvm_ty.clone());
                }
                // Register and emit the env struct type BEFORE the current function.
                // We emit it by inserting the definition right before the `define`
                // line of the current function in the output buffer.
                let env_struct_def = format!("%struct.{env_name} = type {{ {} }}\n", env_fields.join(", "));
                // Find the position of `define` in output (the current function's start)
                // and insert the struct definition before it.
                if let Some(define_pos) = self.output.find("define ") {
                    self.output.insert_str(define_pos, &env_struct_def);
                } else {
                    // Fallback: prepend to start of output
                    let mut s = env_struct_def;
                    s.push_str(&self.output);
                    self.output = s;
                }
                
                // Build thunk function: takes env_ptr + params
                let saved_output = std::mem::take(&mut self.output);
                let saved_tmp = self.tmp_counter;
                let saved_block = self.block_counter;
                self.tmp_counter = closure_id * 1000;
                self.block_counter = closure_id * 1000;
                self.push_scope();
                
                // Thunk params: env_ptr as i64 (uniform), then closure params
                let mut thunk_params = vec!["i64 %__env".to_string()];
                for (i, p) in params.iter().enumerate() {
                    thunk_params.push(format!("i64 %{}_{}", p.name, i));
                }
                let thunk_header = format!("define i64 @{fn_name}({}) {{\nentry:\n",
                    thunk_params.join(", "));
                self.output.push_str(&thunk_header);
                
                // Bitcast env from i64 to typed struct pointer
                self.emitln(&format!("  %__env_ptr = inttoptr i64 %__env to %struct.{env_name}*"));
                
                // Alloca env_ptr
                self.emitln(&format!("  %__env_alloca = alloca %struct.{env_name}*"));
                self.emitln(&format!("  store %struct.{env_name}* %__env_ptr, %struct.{env_name}** %__env_alloca"));
                
                // Load captured variables from env struct into locals
                for (field_idx, (cap_name, cap_llvm_ty)) in captures.iter().enumerate() {
                    let gep = self.fresh_tmp();
                    let loaded = self.fresh_tmp();
                    let alloca = self.fresh_tmp();
                    let llvm_field_idx = field_idx + 1; // field 0 is fn_ptr
                    self.emitln(&format!("  {gep} = getelementptr %struct.{env_name}, %struct.{env_name}* %__env_ptr, i32 0, i32 {llvm_field_idx}"));
                    self.emitln(&format!("  {loaded} = load {cap_llvm_ty}, {cap_llvm_ty}* {gep}"));
                    self.emitln(&format!("  {alloca} = alloca {cap_llvm_ty}"));
                    self.emitln(&format!("  store {cap_llvm_ty} {loaded}, {cap_llvm_ty}* {alloca}"));
                    self.add_local(cap_name, alloca, cap_llvm_ty);
                }
                
                // Store params as locals
                for (i, p) in params.iter().enumerate() {
                    let preg = format!("%{}_{}", p.name, i);
                    let a = format!("%{}_{}_alloca", p.name, i);
                    self.emitln(&format!("  {a} = alloca i64"));
                    self.emitln(&format!("  store i64 {preg}, i64* {a}"));
                    self.add_local(&p.name, a, "i64");
                }
                
                // Compile body
                let (ret_val, ret_ty) = self.compile_expr(body)?;
                let result = self.val_to_i64(&ret_val, &ret_ty);
                self.emitln(&format!("  ret i64 {}", result));
                self.emitln("}");
                self.pop_scope();
                
                let closure_ir = std::mem::take(&mut self.output);
                self.local.deferred_closure_defs.push(closure_ir);
                self.output = saved_output;
                self.tmp_counter = saved_tmp;
                self.block_counter = saved_block;
                
                // At the closure creation site: malloc env, store captures, return ptr
                let env_ptr = self.fresh_tmp();
                let env_size = 8 * (1 + captures.len()); // 8 bytes per field
                let malloc_call = self.fresh_tmp();
                self.emitln(&format!("  {malloc_call} = call i8* @malloc(i64 {env_size})"));
                self.emitln(&format!("  {env_ptr} = bitcast i8* {malloc_call} to %struct.{env_name}*"));
                
                // Store fn_ptr in field 0
                let fn_ptr_field = self.fresh_tmp();
                self.emitln(&format!("  {fn_ptr_field} = getelementptr %struct.{env_name}, %struct.{env_name}* {env_ptr}, i32 0, i32 0"));
                let fn_ptr_val = self.fresh_tmp();
                self.emitln(&format!("  {fn_ptr_val} = ptrtoint ptr @{fn_name} to i64"));
                self.emitln(&format!("  store i64 {fn_ptr_val}, i64* {fn_ptr_field}"));
                
                // Store captured values in subsequent fields
                for (field_idx, (cap_name, _cap_llvm_ty)) in captures.iter().enumerate() {
                    let llvm_field_idx = field_idx + 1;
                    let gep = self.fresh_tmp();
                    self.emitln(&format!("  {gep} = getelementptr %struct.{env_name}, %struct.{env_name}* {env_ptr}, i32 0, i32 {llvm_field_idx}"));
                    // Load the captured variable from its existing alloca
                    if let Some((cap_slot, cap_ty)) = self.lookup_local(cap_name).cloned() {
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {cap_ty}, {cap_ty}* {cap_slot}"));
                        self.emitln(&format!("  store {cap_ty} {loaded}, {cap_ty}* {gep}"));
                    }
                }
                
                // Return env_ptr as i8* (closure value)
                let closure_val = self.fresh_tmp();
                self.emitln(&format!("  {closure_val} = ptrtoint %struct.{env_name}* {env_ptr} to i64"));
                Ok((closure_val, LLVM_I64.to_string()))
            }
            Expr::Closure(params, ret_ty, body, _) => {
                // M20-A1: Block-style closure (fn(x) { body; return expr; })
                let closure_id = self.tmp_counter;
                self.tmp_counter += 1;
                let fn_name = format!("__closure_{closure_id}");
                let env_name = format!("__closure_env_{closure_id}");
                
                let ret_llvm = ret_ty.as_ref()
                    .map(|t| self.llvm_type_for(&Self::type_from_ast(t)).unwrap_or_else(|_| "i64".to_string()))
                    .unwrap_or_else(|| "i64".to_string());
                
                // Collect free variables from block body
                let param_names: Vec<String> = params.iter().map(|p| p.name.name.clone()).collect();
                let captures = self.collect_block_free_vars(body, &param_names);
                
                // Build thunk with env + params
                let saved_output = std::mem::take(&mut self.output);
                let saved_tmp = self.tmp_counter;
                let saved_block = self.block_counter;
                let saved_ret = self.fctx.current_return_type.clone();
                self.tmp_counter = closure_id * 1000;
                self.block_counter = closure_id * 1000;
                self.fctx.current_return_type = ret_llvm.clone();
                self.push_scope();
                
                // Build thunk header: fn(i64 %__env, params...)
                let param_str: Vec<String> = params.iter().enumerate()
                    .map(|(i, p)| format!("%{}_{}", p.name.name, i))
                    .collect();
                let all_params = std::iter::once("i64 %__env".to_string())
                    .chain(param_str.iter().enumerate()
                        .map(|(_, s)| format!("i64 {}", s)))
                    .collect::<Vec<_>>().join(", ");
                self.output.push_str(&format!("define {ret_llvm} @{fn_name}({all_params}) {{\nentry:\n"));
                
                // Load captured variables from env if any
                if !captures.is_empty() {
                    self.emitln(&format!("  %__env_ptr = inttoptr i64 %__env to %struct.{env_name}*"));
                    for (field_idx, (cap_name, cap_llvm_ty)) in captures.iter().enumerate() {
                        let gep = self.fresh_tmp(); let loaded = self.fresh_tmp();
                        let alloca = self.fresh_tmp();
                        let llvm_idx = field_idx + 1;
                        self.emitln(&format!("  {gep} = getelementptr %struct.{env_name}, %struct.{env_name}* %__env_ptr, i32 0, i32 {llvm_idx}"));
                        self.emitln(&format!("  {loaded} = load {cap_llvm_ty}, {cap_llvm_ty}* {gep}"));
                        self.emitln(&format!("  {alloca} = alloca {cap_llvm_ty}"));
                        self.emitln(&format!("  store {cap_llvm_ty} {loaded}, {cap_llvm_ty}* {alloca}"));
                        self.add_local(cap_name, alloca, cap_llvm_ty);
                    }
                }
                
                // Store params as locals
                for (i, p) in params.iter().enumerate() {
                    let preg = format!("%{}_{}", p.name.name, i);
                    let a = format!("%{}_{}_alloca", p.name.name, i);
                    self.emitln(&format!("  {a} = alloca i64"));
                    self.emitln(&format!("  store i64 {preg}, i64* {a}"));
                    self.add_local(&p.name.name, a, "i64");
                }
                
                // Compile block body
                self.compile_block(body, false)?;
                
                // If no return was emitted, add fallback
                if !self.current_block_terminated() {
                    self.emitln(&format!("  ret {ret_llvm} 0"));
                }
                self.emitln("}");
                self.pop_scope();
                
                let closure_ir = std::mem::take(&mut self.output);
                self.local.deferred_closure_defs.push(closure_ir);
                self.output = saved_output;
                self.tmp_counter = saved_tmp;
                self.block_counter = saved_block;
                self.fctx.current_return_type = saved_ret;
                
                // Create closure value: malloc env, store fn_ptr + captures, return ptr
                if captures.is_empty() {
                    // Non-capturing: minimal env with just fn_ptr
                    let env_def = format!("%struct.{env_name} = type {{ i64 }}\n");
                    if let Some(define_pos) = self.output.find("define ") {
                        self.output.insert_str(define_pos, &env_def);
                    } else {
                        self.output.push_str(&env_def);
                    }
                    let env_ptr = self.fresh_tmp(); let mc = self.fresh_tmp();
                    self.emitln(&format!("  {mc} = call i8* @malloc(i64 8)"));
                    self.emitln(&format!("  {env_ptr} = bitcast i8* {mc} to %struct.{env_name}*"));
                    let gep0 = self.fresh_tmp(); let fpi = self.fresh_tmp();
                    self.emitln(&format!("  {gep0} = getelementptr %struct.{env_name}, %struct.{env_name}* {env_ptr}, i32 0, i32 0"));
                    self.emitln(&format!("  {fpi} = ptrtoint ptr @{fn_name} to i64"));
                    self.emitln(&format!("  store i64 {fpi}, i64* {gep0}"));
                    let cv = self.fresh_tmp();
                    self.emitln(&format!("  {cv} = ptrtoint %struct.{env_name}* {env_ptr} to i64"));
                    return Ok((cv, LLVM_I64.to_string()));
                }
                
                // Capturing block closure
                let env_fields: Vec<String> = std::iter::once("i64".to_string())
                    .chain(captures.iter().map(|(_, t)| t.clone()))
                    .collect();
                let env_def = format!("%struct.{env_name} = type {{ {} }}\n", env_fields.join(", "));
                if let Some(define_pos) = self.output.find("define ") {
                    self.output.insert_str(define_pos, &env_def);
                } else {
                    self.output.push_str(&env_def);
                }
                let env_size = 8 * (1 + captures.len());
                let env_ptr = self.fresh_tmp(); let mc = self.fresh_tmp();
                self.emitln(&format!("  {mc} = call i8* @malloc(i64 {env_size})"));
                self.emitln(&format!("  {env_ptr} = bitcast i8* {mc} to %struct.{env_name}*"));
                let gep0 = self.fresh_tmp(); let fpi = self.fresh_tmp();
                self.emitln(&format!("  {gep0} = getelementptr %struct.{env_name}, %struct.{env_name}* {env_ptr}, i32 0, i32 0"));
                self.emitln(&format!("  {fpi} = ptrtoint ptr @{fn_name} to i64"));
                self.emitln(&format!("  store i64 {fpi}, i64* {gep0}"));
                for (field_idx, (cap_name, _cap_llvm_ty)) in captures.iter().enumerate() {
                    let gep = self.fresh_tmp();
                    self.emitln(&format!("  {gep} = getelementptr %struct.{env_name}, %struct.{env_name}* {env_ptr}, i32 0, i32 {}", field_idx + 1));
                    if let Some((cap_slot, cap_ty)) = self.lookup_local(cap_name).cloned() {
                        let ld = self.fresh_tmp();
                        self.emitln(&format!("  {ld} = load {cap_ty}, {cap_ty}* {cap_slot}"));
                        self.emitln(&format!("  store {cap_ty} {ld}, {cap_ty}* {gep}"));
                    }
                }
                let cv = self.fresh_tmp();
                self.emitln(&format!("  {cv} = ptrtoint %struct.{env_name}* {env_ptr} to i64"));
                Ok((cv, LLVM_I64.to_string()))
            }
            Expr::As(inner, ty, _) => {
                // G-44: `&out as *mut UInt8` — Xiom binds `&` with LOWER
                // precedence than `as`, so the AST is `&(out as *mut UInt8)`.
                // The `inner` of `As` is a bare `Expr::Ident("out")` — never
                // `Expr::Ref`. We must detect the local BEFORE compile_expr
                // loads the value and produce the alloca address as a typed
                // pointer. Previously `out` compiled to `load i64 = 7` and
                // `inttoptr i64 7 to i8*` caused AV on memset write-back.
                if let Expr::Ident(id) = inner.as_ref() {
                    let target_llvm_ty = self.llvm_type_for_fallback(&Self::type_from_ast(ty));
                    if target_llvm_ty.ends_with('*') {
                        if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                            // 5e.3: when the local is already a pointer type
                            // (e.g. raw: *UInt8 cast to *RcInner[T]), load the
                            // stored pointer value BEFORE bitcasting. Without this,
                            // bitcast(i8** -> %RcInner*) corrupts the stack by
                            // pointing to the alloca slot instead of the heap buf.
                            let ptr_reg = self.fresh_tmp();
                            if slot_ty.ends_with('*') {
                                let loaded = self.fresh_tmp();
                                self.emitln(&format!("  {loaded} = load {slot_ty}, {slot_ty}* {slot}"));
                                self.emitln(&format!("  {ptr_reg} = bitcast {slot_ty} {loaded} to {target_llvm_ty}"));
                            } else {
                                self.emitln(&format!("  {ptr_reg} = bitcast {slot_ty}* {slot} to {target_llvm_ty}"));
                            }
                            return Ok((ptr_reg, target_llvm_ty.clone()));
                        }
                    }
                }
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
                        if let Some(concrete) = self.mono.param_concrete_types.get(&id.name) {
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
                        Ok((tmp, LLVM_I64.to_string()))
                    }
                    (a, b) if a == b => Ok((val, target_llvm_ty.clone())),
                    // 5e.2 G-34: fn-ptr ↔ Int casts.
                    (inner_ty, target_fn_ptr) if target_fn_ptr.contains('(')
                        && target_fn_ptr.contains(')')
                        && target_fn_ptr.ends_with('*')
                        && int_width(&inner_ty).is_some() =>
                    {
                        let ptr_reg = self.fresh_tmp();
                        self.emitln(&format!("  {ptr_reg} = inttoptr {inner_ty} {val} to {target_fn_ptr}"));
                        return Ok((ptr_reg, target_fn_ptr.to_string()));
                    }
                    // Reverse: fn-ptr → Int (ptrtoint)
                    (src_fn_ptr, target_ty) if src_fn_ptr.contains('(')
                        && src_fn_ptr.contains(')')
                        && src_fn_ptr.ends_with('*')
                        && int_width(&target_ty).is_some() =>
                    {
                        let int_reg = self.fresh_tmp();
                        self.emitln(&format!("  {int_reg} = ptrtoint {src_fn_ptr} {val} to {target_ty}"));
                        return Ok((int_reg, target_ty.to_string()));
                    }
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
                        let aw = int_width(a).expect("int_width(a) is Some (guarded above)");
                        let bw = int_width(b).expect("int_width(b) is Some (guarded above)");
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
                } else if cond_ty.starts_with("%struct.") {
                    // M17: Struct-typed conditions can't be compared with icmp.
                    // Treat as always-true (the discriminant check already
                    // handled variant matching). This avoids invalid IR like
                    // `icmp ne %struct.Vec %val, 0`.
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = icmp ne i64 1, 0"));
                    tmp
                } else {
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = icmp ne {cond_ty} {cond_raw}, 0"));
                    tmp
                };

                let result_ty = LLVM_I64.to_string();
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
                    let ec_val = if ec_ty == "i1" { ec_raw } else if ec_ty.starts_with("%struct.") {
                        let tmp = self.fresh_tmp();
                        self.emitln(&format!("  {tmp} = icmp ne i64 1, 0"));
                        tmp
                    } else {
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
                Ok(("0".to_string(), LLVM_I64.to_string()))
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
                let saved_ptr = self.fctx.match_result_ptr.take();
                let saved_ty = self.fctx.match_result_ty.take();
                self.fctx.match_result_ptr = Some(result_alloca.clone());
                self.fctx.match_result_ty = Some(result_ty.clone());
                let stmt = Stmt::Match((**scrutinee).clone(), arms.clone(), *span);
                self.compile_stmt(&stmt)?;
                self.fctx.match_result_ptr = saved_ptr;
                self.fctx.match_result_ty = saved_ty;
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
                    && (self.types.types.contains_key(&field.name)
                        || self.types.type_meta.contains_key(&field.name)
                        || self.types.type_meta.keys().any(|k| k.ends_with(&format!(".{}", field.name))))
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

