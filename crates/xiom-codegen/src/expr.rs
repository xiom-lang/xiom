use xiom_ast::*;
use crate::llvm_consts::*;
use crate::context::TypeMeta;

use super::IrEmitter;

/// Resolve a bare struct literal `{ field: value; }` (name = `_`) to a
/// registered type whose field names match. Returns the compiled struct value.
fn resolve_bare_struct(
    emitter: &mut IrEmitter,
    fields: &[(Ident, Expr)],
) -> Result<(String, String), String> {
    let field_names: Vec<String> = fields.iter().map(|(n,_)| n.name.clone()).collect();
    let resolved = emitter.types.types.entries().into_iter()
        .find(|(_, fnames)| fnames.len() == field_names.len()
            && fnames.iter().zip(&field_names).all(|(a, b)| a == b))
        .map(|(tn, _)| tn.clone());
    if let Some(type_name) = resolved {
        emitter.compile_struct_literal(&type_name, fields, false)
    } else {
        // Fallback: compile each field and return the last value (scalar).
        let mut last = ("0".to_string(), "i64".to_string());
        for (_, val) in fields.iter() {
            last = emitter.compile_expr(val)?;
        }
        Ok(last)
    }
}

impl IrEmitter {
    /// Compile a struct literal with a KNOWN type name. Used when the type
    /// was resolved from context (e.g. `Ok({ x: 1 })` where `Ok` expects `T`).
    pub(crate) fn compile_struct_literal(&mut self, type_name: &str, fields: &[(Ident, Expr)], _is_enum_variant: bool) -> Result<(String, String), String> {
        let struct_ty = self.llvm_type_for(type_name)?;
        let alloca = self.fresh_tmp();
        // D1: align 16 for structs with i128/fp128 fields (e.g. I128DivRem);
        // plain structs (Vec etc.) stay at default alignment.
        self.emitln(&format!("  {alloca} = alloca {struct_ty}{}", self.alloca_align(&struct_ty)));
        for (i, (_, val)) in fields.iter().enumerate() {
            let field_llvm_ty = self.field_llvm_type(type_name, i);
            // 5c.39: Empty array `[]` in a Vec-typed struct field -- compile as
            // a proper empty Vec (heap-allocated buffer) instead of a raw i8*
            // array buffer that would be inttoptr'd to a 32-byte Vec struct.
            let (field_val, field_val_ty) = if let Expr::Array(elems, _) = val {
                if elems.is_empty() && (field_llvm_ty == "%struct.Vec" || field_llvm_ty.ends_with(".Vec")) {
                    self.compile_empty_vec_for_field(type_name, i)?
                } else {
                    self.compile_expr(val)?
                }
            } else {
                self.compile_expr(val)?
            };
            let store_val = self.coerce_value(&field_val, &field_val_ty, &field_llvm_ty);
            let gep = self.fresh_tmp();
            self.emitln(&format!("  {gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 {i}"));
            self.emitln(&format!("  store {field_llvm_ty} {store_val}, {field_llvm_ty}* {gep}"));
        }
        let loaded = self.fresh_tmp();
        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {alloca}"));
        Ok((loaded, struct_ty))
    }

    /// 5c.39: Compile an empty Vec for struct field initialization. Returns a
    /// properly initialized Vec struct (insertvalue chain), not a raw i8*.
    fn compile_empty_vec_for_field(&mut self, type_name: &str, field_idx: usize) -> Result<(String, String), String> {
        let elem_size: i64 = self.resolve_vec_field_elem_size(type_name, field_idx).unwrap_or(8);
        let initial_cap: i64 = 16;
        let data_ptr = self.fresh_tmp();
        self.emitln(&format!("  {data_ptr} = call i8* @malloc(i64 {})", initial_cap * elem_size));
        let ok = self.fresh_block("empty_vec_ok");
        let tr = self.fresh_block("empty_vec_trap");
        let nc = self.fresh_tmp();
        self.emitln(&format!("  {nc} = icmp eq i8* {data_ptr}, null"));
        self.emitln(&format!("  br i1 {nc}, label %{tr}, label %{ok}"));
        self.emitln(&format!("\n{tr}:"));
        self.emitln("  call void @llvm.trap()");
        self.emitln("  unreachable");
        self.emitln(&format!("\n{ok}:"));
        let v1 = self.fresh_tmp();
        self.emitln(&format!("  {v1} = insertvalue %struct.Vec undef, i8* {data_ptr}, 0"));
        let v2 = self.fresh_tmp();
        self.emitln(&format!("  {v2} = insertvalue %struct.Vec {v1}, i64 0, 1"));
        let v3 = self.fresh_tmp();
        self.emitln(&format!("  {v3} = insertvalue %struct.Vec {v2}, i64 {initial_cap}, 2"));
        let v4 = self.fresh_tmp();
        self.emitln(&format!("  {v4} = insertvalue %struct.Vec {v3}, i64 {elem_size}, 3"));
        Ok((v4, "%struct.Vec".to_string()))
    }

    fn resolve_vec_field_elem_size(&self, type_name: &str, field_idx: usize) -> Option<i64> {
        let meta = self.types.type_meta.get(&type_name.to_string())
            .or_else(|| {
                self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{type_name}")))
                    .map(|(_, v)| v)
            })?;
        let ftype = meta.fields.get(field_idx).map(|(_, t)| t.as_str())?;
        let elem_name = ftype.strip_prefix("Vec[")?.strip_suffix(']')?;
        let struct_key = self.types.type_meta.keys().into_iter()
    .find(|k| k.ends_with(&format!(".{elem_name}")) || k.as_str() == elem_name)
            .unwrap_or(elem_name.to_string());
        let sz = self.struct_byte_size(&struct_key);
        if sz > 0 { Some(sz) } else { Some(8) }
    }

    pub(crate) fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        self.compile_stmt_impl(stmt)
    }


    /// Lower a single binary operation for the iterative fold path (deep-chain
    /// hardening). Takes pre-compiled operands and produces the folded result.
    /// Handles arithmetic (Add/Sub/Mul), bitwise (And/Or/Xor), and float coercions.
    fn compile_binop_fold(&mut self, l: &str, lt: &str, r: &str, rt: &str, op: &BinOp, l_expr: &Expr, r_expr: &Expr) -> Result<(String, String), String> {
        // String concatenation: Add with i8* operands must call xiom_str_concat,
        // not emit `add i64` on pointer values. The normal BinOp path checks this
        // first; replicate the check here for the iterative fold path. Integer
        // operands are formatted via @xiom_int_to_string (not inttoptr -- garbage
        // pointer + AV), decided by the XIOM-level type verdict.
                if matches!(op, BinOp::Add) && (lt == "i8*" || rt == "i8*")
                    && !((lt == "i8*" && self.expr_is_pointer(l_expr) && self.expr_is_integer(r_expr))
                        || (rt == "i8*" && self.expr_is_pointer(r_expr) && self.expr_is_integer(l_expr))) {
                    let lp = self.concat_val_to_i8ptr(&l, &lt, self.expr_is_integer(l_expr));
                    let rp = self.concat_val_to_i8ptr(&r, &rt, self.expr_is_integer(r_expr));
                    let res = self.fresh_tmp();
                    self.emitln(&format!("  {res} = call i8* @xiom_str_concat(i8* {lp}, i8* {rp})"));
                    return Ok((res, LLVM_STR_PTR.to_string()));
                }
                // Pointer arithmetic: `p + i` / `p - i` on a REAL pointer
                // (i64*/i32*/%struct.X*/double*... -- the i8* Str case was caught
                // by the concat intercept above) must GEP-scale by the ELEMENT
                // size, not integer-add the raw index to the pointer bits. The
                // old ptrtoint/add/inttoptr lowering also lost the pointee type,
                // so `*(p + 1)` loaded a BYTE instead of the element (round-7
                // ve2 follow-up: mono'd Vec.first/last/insert/remove bodies do
                // `*(data + len - 1)` -- byte offsets + byte loads on Vec[Int]).
                if matches!(op, BinOp::Add | BinOp::Sub) {
                    let (pval, pty, ival, ity) = if lt.ends_with('*') && !rt.ends_with('*') {
                        (l.to_string(), lt.to_string(), r.to_string(), rt.to_string())
                    } else if rt.ends_with('*') && !lt.ends_with('*') {
                        (r.to_string(), rt.to_string(), l.to_string(), lt.to_string())
                    } else {
                        ("".to_string(), String::new(), "".to_string(), String::new())
                    };
                    if !pty.is_empty() {
                        let pointee = pty.strip_suffix('*').unwrap_or(&pty).to_string();
                        let idx = if ity == "i64" { ival.clone() } else {
                            let w = self.fresh_tmp();
                            self.emitln(&format!("  {w} = sext {ity} {ival} to i64"));
                            w
                        };
                        let gep = self.fresh_tmp();
                        if matches!(op, BinOp::Sub) {
                            let neg = self.fresh_tmp();
                            self.emitln(&format!("  {neg} = sub i64 0, {idx}"));
                            self.emitln(&format!("  {gep} = getelementptr {pointee}, {pty} {pval}, i64 {neg}"));
                        } else {
                            self.emitln(&format!("  {gep} = getelementptr {pointee}, {pty} {pval}, i64 {idx}"));
                        }
                        return Ok((gep, pty));
                    }
                }
        let is_float = lt == "float" || lt == "double" || lt == "fp128" || rt == "float" || rt == "double" || rt == "fp128";
        let float_ty = if lt == "float" || rt == "float" { "float" } else if lt == "fp128" || rt == "fp128" { "fp128" } else { "double" };
        let is_add_sub_mul = matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul);
        // D1: i128 operands stay i128 (no i64 widening).
        let int_ty = if lt == "i128" || rt == "i128" { "i128" } else { "i64" };
        let ty = if is_float && is_add_sub_mul { float_ty } else { int_ty };
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
        if !is_float && int_ty == "i64" {
            lv = self.widen_to_i64(&lv, lt);
            rv = self.widen_to_i64(&rv, rt);
        } else if !is_float && int_ty == "i128" {
            // D1: widen narrow operands up to i128 (sext/zext).
            if lt != "i128" {
                let ext = self.fresh_tmp();
                let extop = if lt == "i8" || lt == "i1" { "zext" } else { "sext" };
                self.emitln(&format!("  {ext} = {extop} {lt} {lv} to i128"));
                lv = ext;
            }
            if rt != "i128" {
                let ext = self.fresh_tmp();
                let extop = if rt == "i8" || rt == "i1" { "zext" } else { "sext" };
                self.emitln(&format!("  {ext} = {extop} {rt} {rv} to i128"));
                rv = ext;
            }
        }
        // Struct operands in arithmetic context: extract the leading scalar.
        // Handles patterns like Some(x).unwrap() + 1 being folded.
        if lt.starts_with("%struct.") && int_ty == "i64" {
            lv = self.extract_scalar_field0(&lv, lt);
        }
        if rt.starts_with("%struct.") && int_ty == "i64" {
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

    /// CTFE Phase A: Evaluate a const-init expression at compile time.
    /// Handles:
    ///   - Literals (Int, Float, Bool)
    ///   - Unary ops (negation `-`, boolean not `!`, bitwise not `~`)
    ///   - Binary ops: arithmetic (+ - * / %), comparison (== != < > <= >=),
    ///     boolean (and, or)
    ///   - Const variable references -- resolve through self.local.constants
    ///   - `sizeof::<T>()`, `align_of::<T>()`, `type_id::<T>()`,
    ///     `field_offset::<T>(name)` builtins
    ///   - `const { expr }` blocks (via Expr::ConstBlock)
    ///   - Parenthesized expressions
    ///   - `if`/`match` on compile-time-known conditions
    /// Returns the original expression unchanged if evaluation fails.
    ///
    /// Security review (2026-08-13): bounded by CONST_EVAL_BUDGET recursion
    /// depth -- a pathological const expression must not be able to hang the
    /// compiler (stack overflow / OOM) via unbounded CTFE recursion. On
    /// exceeding the budget the expression is returned UNEVALUATED, which is
    /// always safe: the constant then materializes at runtime like any other
    /// non-foldable initializer.
    pub(crate) fn evaluate_const_init(&self, expr: &Expr) -> Expr {
        let depth = self.local.const_eval_depth.get();
        if depth >= Self::CONST_EVAL_BUDGET {
            return expr.clone();
        }
        self.local.const_eval_depth.set(depth + 1);
        let result = self.evaluate_const_init_inner(expr);
        self.local.const_eval_depth.set(depth);
        result
    }

    fn evaluate_const_init_inner(&self, expr: &Expr) -> Expr {
        match expr {
            // Literals -- already evaluated
            Expr::Int(..) | Expr::Float(..) | Expr::Bool(..) | Expr::Str(..) => expr.clone(),


            // Enum constructors -- evaluate inner expression
            Expr::Some(inner, span) => {
                let v = self.evaluate_const_init(inner);
                Expr::Some(Box::new(v), *span)
            }
            Expr::None(span) => Expr::None(*span),
            Expr::Ok(inner, span) => {
                let v = self.evaluate_const_init(inner);
                Expr::Ok(Box::new(v), *span)
            }
            Expr::Err(inner, span) => {
                let v = self.evaluate_const_init(inner);
                Expr::Err(Box::new(v), *span)
            }

            // Const variable reference -- substitute from self.local.constants
            Expr::Ident(ident) => {
                if let Some(val) = self.local.constants.get(&ident.name) {
                    // Cycle detection: prevent infinite recursion on `const A = B; const B = A;`
                    // by tracking which constants are currently being evaluated.
                    // Re-entering a constant we're already resolving means a cycle -- bail out.
                    if self.local.const_eval_stack.borrow().contains(&ident.name) {
                        return expr.clone();
                    }
                    self.local.const_eval_stack.borrow_mut().insert(ident.name.clone());
                    let result = self.evaluate_const_init(val);
                    self.local.const_eval_stack.borrow_mut().remove(&ident.name);
                    return result;
                }
                expr.clone()
            }

            // Unary negation: -X, !X (boolean not), ~X (bitwise not)
            Expr::Unary(op, inner, span) => {
                let inner = self.evaluate_const_init(inner);
                match op {
                    UnaryOp::Neg => {
                        if let Expr::Int(v, _) = inner {
                            // Expr::Int stores the i64 bit pattern as u64;
                            // negate in the SIGNED domain so -5 folds to its
                            // two's-complement bits. True i64 overflow
                            // (MIN) stays unevaluated (checked policy).
                            let sv = v as i64;
                            if let Some(neg) = sv.checked_neg() { return Expr::Int(neg as u64, *span); }
                            return expr.clone();
                        }
                        if let Expr::Float(v, _) = inner { return Expr::Float(-v, *span); }
                    }
                    UnaryOp::Not => {
                        // Boolean not: `!true` -> `false`, `!false` -> `true`
                        if let Expr::Bool(v, _) = inner { return Expr::Bool(!v, *span); }
                    }
                    UnaryOp::BitNot => {
                        // Bitwise not: `~expr` -- only for integer expressions
                        if let Expr::Int(v, _) = inner { return Expr::Int(!v, *span); }
                    }
                    _ => {}
                }
                expr.clone()
            }

            // Binary ops: arithmetic, comparison, boolean
            Expr::Binary(lhs, op, rhs, span) => {
                let l = self.evaluate_const_init(lhs);
                let r = self.evaluate_const_init(rhs);

                // --- Integer pairs ---
                // Expr::Int stores the i64 bit pattern as u64; all checked
                // arithmetic runs in the SIGNED domain (i64) and stores back
                // as bits. True i64 overflow leaves the expression
                // unevaluated so runtime semantics apply (checked policy --
                // was silent wrapping, audited).
                if let (Expr::Int(a, _), Expr::Int(b, _)) = (&l, &r) {
                    let (a, b) = (*a as i64, *b as i64);
                    let result: Expr = match op {
                        BinOp::Add => match a.checked_add(b) { Some(v) => Expr::Int(v as u64, *span), None => return expr.clone() },
                        BinOp::Sub => match a.checked_sub(b) { Some(v) => Expr::Int(v as u64, *span), None => return expr.clone() },
                        BinOp::Mul => match a.checked_mul(b) { Some(v) => Expr::Int(v as u64, *span), None => return expr.clone() },
                        BinOp::Div => if b != 0 && !(a == i64::MIN && b == -1) { Expr::Int((a / b) as u64, *span) } else { return expr.clone(); },
                        BinOp::Rem => if b != 0 && !(a == i64::MIN && b == -1) { Expr::Int((a % b) as u64, *span) } else { return expr.clone(); },
                        // Comparison (integer)
                        BinOp::Eq  => Expr::Bool(a == b, *span),
                        BinOp::Neq => Expr::Bool(a != b, *span),
                        BinOp::Lt  => Expr::Bool(a < b, *span),
                        BinOp::Gt  => Expr::Bool(a > b, *span),
                        BinOp::Le  => Expr::Bool(a <= b, *span),
                        BinOp::Ge  => Expr::Bool(a >= b, *span),
                        // Bitwise
                        BinOp::Shl => match a.checked_shl(b as u32).filter(|_| (0..64).contains(&b)) { Some(v) => Expr::Int(v as u64, *span), None => return expr.clone() },
                        BinOp::Shr => match a.checked_shr(b as u32).filter(|_| (0..64).contains(&b)) { Some(v) => Expr::Int(v as u64, *span), None => return expr.clone() },
                        BinOp::BitAnd => Expr::Int((a & b) as u64, *span),
                        BinOp::BitOr  => Expr::Int((a | b) as u64, *span),
                        BinOp::BitXor => Expr::Int((a ^ b) as u64, *span),
                        _ => return expr.clone(),
                    };
                    return result;
                }

                // --- Float pairs ---
                if let (Expr::Float(a, _), Expr::Float(b, _)) = (&l, &r) {
                    let result: Expr = match op {
                        BinOp::Add => Expr::Float(a + b, *span),
                        BinOp::Sub => Expr::Float(a - b, *span),
                        BinOp::Mul => Expr::Float(a * b, *span),
                        BinOp::Div => if *b != 0.0 { Expr::Float(a / b, *span) } else { return expr.clone(); },
                        BinOp::Eq  => Expr::Bool(a == b, *span),
                        BinOp::Neq => Expr::Bool(a != b, *span),
                        BinOp::Lt  => Expr::Bool(a < b, *span),
                        BinOp::Gt  => Expr::Bool(a > b, *span),
                        BinOp::Le  => Expr::Bool(a <= b, *span),
                        BinOp::Ge  => Expr::Bool(a >= b, *span),
                        _ => return expr.clone(),
                    };
                    return result;
                }

                // --- Bool pairs (and, or) ---
                if let (Expr::Bool(a, _), Expr::Bool(b, _)) = (&l, &r) {
                    let result: Expr = match op {
                        BinOp::And => Expr::Bool(*a && *b, *span),
                        BinOp::Or  => Expr::Bool(*a || *b, *span),
                        BinOp::Eq  => Expr::Bool(a == b, *span),
                        BinOp::Neq => Expr::Bool(a != b, *span),
                        _ => return expr.clone(),
                    };
                    return result;
                }

                // --- String comparisons ---
                if let (Expr::Str(a, _), Expr::Str(b, _)) = (&l, &r) {
                    match op {
                        BinOp::Eq  => return Expr::Bool(a == b, *span),
                        BinOp::Neq => return Expr::Bool(a != b, *span),
                        _ => return expr.clone(),
                    }
                }

                expr.clone()
            }

            // sizeof::<T>() / builtins via turbofish
            Expr::Call(func, _, _)
            | Expr::GenericCall(func, _, _, _) => {
                if let Expr::Field(base, field, _) = func.as_ref() {
                    if field.name == "sizeof" {
                        if let Expr::Ident(id) = base.as_ref() {
                            let ty_name = &id.name;
                            let size = self.struct_byte_size(ty_name) as u64;
                            return Expr::Int(size, Span::new(0, 0));
                        }
                    }
                }
                if let Expr::GenericCall(f, types, args, span) = expr {
                    if !types.is_empty() {
                        if let Expr::Ident(id) = f.as_ref() {
                            let type_name = crate::IrEmitter::type_from_ast(&types[0]);
                            if id.name == "sizeof" {
                                let size = self.struct_byte_size(&type_name) as u64;
                                return Expr::Int(size, *span);
                            }
                            if id.name == "is_signed" {
                                let signed = Self::is_signed_xiom_type(&type_name);
                                return Expr::Bool(signed, *span);
                            }
                            if id.name == "align_of" {
                                let align = self.align_of_type(&type_name);
                                return Expr::Int(align, *span);
                            }
                            if id.name == "type_id" {
                                let id_val = Self::type_id_of(&type_name);
                                return Expr::Int(id_val, *span);
                            }
                            if id.name == "field_offset" {
                                if let Some(arg) = args.first() {
                                    // Accept both string literal ("x") and unquoted ident (x)
                                    let field_name: String = match arg {
                                        Expr::Str(s, _) => s.clone(),
                                        Expr::Ident(id) => id.name.clone(),
                                        _ => String::new(),
                                    };
                                    if !field_name.is_empty() {
                                        let offset = self.field_offset_of(&type_name, &field_name);
                                        return Expr::Int(offset, *span);
                                    }
                                }
                            }
                        }
                    }
                }
                // v0.54 Phase B: CTFE function call evaluation
                if let Expr::Call(func, args, _) = expr {
                    if let Expr::Ident(fid) = func.as_ref() {
                        let mut arg_vals: Vec<xiom_ctfe::CtfeValue> = Vec::new();
                        let mut all_const = true;
                        for a in args.iter() {
                            let ev = self.evaluate_const_init(a);
                            match ev {
                                Expr::Int(n, _) => arg_vals.push(xiom_ctfe::CtfeValue::Int(n as i64)),
                                Expr::Float(f, _) => arg_vals.push(xiom_ctfe::CtfeValue::Float(f)),
                                Expr::Bool(b, _) => arg_vals.push(xiom_ctfe::CtfeValue::Bool(b)),
                                Expr::Str(s, _) => arg_vals.push(xiom_ctfe::CtfeValue::Str(s)),
                                _ => { all_const = false; break; }
                            }
                        }
                        if all_const && arg_vals.len() == args.len() {
                            if let Ok(result) = self.ctfe.borrow_mut().eval_function(
                                &fid.name, &arg_vals, 0,
                            ) {
                                // try_to_expr returns None for values with no
                                // faithful AST form (Ptr/Null/Range/custom
                                // variants). The audited Int(0) SENTINEL is
                                // gone: fall back to the UNEVALUATED call so
                                // the constant materializes at runtime.
                                if let Some(folded) = xiom_ctfe::CtfeEngine::try_to_expr(&result) {
                                    return folded;
                                }
                            }
                        }
                    }
                }
                expr.clone()
            }

            // Parenthesized expressions
            Expr::Paren(inner, _) => self.evaluate_const_init(inner),

            // if/else expression -- evaluate condition and pick branch
            Expr::If(cond, then_block, elifs, else_block, span) => {
                let cond_val = self.evaluate_const_init(cond);
                if let Expr::Bool(true, _) = cond_val {
                    if let Some(v) = self.eval_block_last(&then_block.stmts, *span) {
                        return v;
                    }
                    return expr.clone();
                }
                if let Expr::Bool(false, _) = cond_val {
                    for (elif_cond, elif_block) in elifs {
                        let ec = self.evaluate_const_init(elif_cond);
                        if let Expr::Bool(true, _) = ec {
                            if let Some(v) = self.eval_block_last(&elif_block.stmts, *span) {
                                return v;
                            }
                            return expr.clone();
                        }
                    }
                    if let Some(else_block) = else_block {
                        if let Some(v) = self.eval_block_last(&else_block.stmts, *span) {
                            return v;
                        }
                    }
                }
                expr.clone()
            }

            // match expression -- evaluate scrutinee, match against literal patterns
            Expr::Match(scrutinee, arms, span) => {
                let val = self.evaluate_const_init(scrutinee);
                for arm in arms {
                    let mut bindings: std::collections::HashMap<String, Expr> = std::collections::HashMap::new();
                    if self.pattern_matches_const_with_bindings(&arm.pattern, &val, &mut bindings) {
                        let folded = match &arm.body {
                            xiom_ast::MatchBody::Block(block) => {
                                self.eval_block_last(&block.stmts, *span)
                            }
                            xiom_ast::MatchBody::Expr(e) => {
                                Some(self.evaluate_const_init_with_bindings(e, &bindings))
                            }
                        };
                        // Non-foldable arm -> keep the WHOLE match unevaluated
                        // (the old Int(0) sentinel is gone).
                        if let Some(v) = folded {
                            return v;
                        }
                        return expr.clone();
                    }
                }
                expr.clone()
            }

            // Everything else: return unchanged (non-const-evaluable)
            _ => expr.clone(),
        }
    }

    /// Evaluate a block for const if/match folding and return its tail value.
    ///
    /// PRODUCTION FIX (readiness Stage 1): the old implementation walked the
    /// statements in REVERSE, evaluated only the final expression, and
    /// skipped every preceding statement -- `{ let a = 21; a * 2 }` folded
    /// with `a` unknown (or read a stale outer binding). Blocks now execute
    /// SEQUENTIALLY with a local constant environment. Returns None when the
    /// block is not const-foldable -- callers must keep the original
    /// expression unevaluated (the fabricated Int(0) fallback is gone).
    fn eval_block_last(
        &self,
        stmts: &[xiom_ast::StmtOrExpr],
        _span: Span,
    ) -> Option<Expr> {
        let mut locals: std::collections::HashMap<String, Expr> = std::collections::HashMap::new();
        for stmt in stmts.iter() {
            match stmt {
                xiom_ast::StmtOrExpr::Expr(e) => {
                    return Some(self.evaluate_const_init_with_bindings(e, &locals));
                }
                xiom_ast::StmtOrExpr::Stmt(s) => match s {
                    xiom_ast::Stmt::Expr(e, _) => {
                        return Some(self.evaluate_const_init_with_bindings(e, &locals));
                    }
                    xiom_ast::Stmt::Let(id, _, init, _) | xiom_ast::Stmt::Var(id, _, init, _) => {
                        let folded = self.evaluate_const_init_with_bindings(init, &locals);
                        if !Self::is_const_literal(&folded) {
                            // Depends on a runtime value: block is not
                            // compile-time computable.
                            return None;
                        }
                        locals.insert(id.name.clone(), folded);
                    }
                    xiom_ast::Stmt::Assign(target, rhs, _) => {
                        let name = match *target {
                            xiom_ast::Expr::Ident(ref id) => id.name.clone(),
                            _ => return None,
                        };
                        if !locals.contains_key(&name) {
                            // Assignment to a non-local: not a pure const block.
                            return None;
                        }
                        let folded = self.evaluate_const_init_with_bindings(rhs, &locals);
                        if !Self::is_const_literal(&folded) {
                            return None;
                        }
                        locals.insert(name, folded);
                    }
                    // Control flow / returns inside folded arms are not
                    // supported by this folder (the CTFE engine handles full
                    // function bodies).
                    _ => return None,
                },
            }
        }
        // Empty block has no value form.
        None
    }

    /// True when the expression is a fully-folded compile-time literal.
    fn is_const_literal(e: &xiom_ast::Expr) -> bool {
        match e {
            xiom_ast::Expr::Int(..)
            | xiom_ast::Expr::Float(..)
            | xiom_ast::Expr::Bool(..)
            | xiom_ast::Expr::Str(..)
            | xiom_ast::Expr::Char(..)
            | xiom_ast::Expr::None(_) => true,
            xiom_ast::Expr::Some(inner, _) | xiom_ast::Expr::Ok(inner, _)
            | xiom_ast::Expr::Err(inner, _) => Self::is_const_literal(inner),
            _ => false,
        }
    }

    /// Check whether a compile-time pattern matches a const-evaluated literal,
    /// collecting named bindings (e.g. `Some(v)` binds `v` to the inner value).
    /// Returns true if the pattern matches.
    fn pattern_matches_const_with_bindings(
        &self,
        pattern: &xiom_ast::Pattern,
        value: &Expr,
        bindings: &mut std::collections::HashMap<String, Expr>,
    ) -> bool {
        match pattern {
            xiom_ast::Pattern::Wildcard(_) => true,
            xiom_ast::Pattern::Ident(id) => {
                bindings.insert(id.name.clone(), value.clone());
                true
            }
            xiom_ast::Pattern::Lit(lit) => {
                match (lit, value) {
                    (xiom_ast::Literal::Bool(a, _), Expr::Bool(b, _)) => a == b,
                    (xiom_ast::Literal::Int(a, _), Expr::Int(b, _)) => a == b,
                    (xiom_ast::Literal::Float(a, _), Expr::Float(b, _)) => a == b,
                    (xiom_ast::Literal::Str(a, _), Expr::Str(b, _)) => a == b,
                    (xiom_ast::Literal::Char(a, _), Expr::Char(b, _)) => a == b,
                    _ => false,
                }
            }
            xiom_ast::Pattern::Some(inner, _) => {
                match value {
                    Expr::Some(v, _) => self.pattern_matches_const_with_bindings(inner, v, bindings),
                    _ => false,
                }
            }
            xiom_ast::Pattern::None(_) => matches!(value, Expr::None(_)),
            xiom_ast::Pattern::Ok(inner, _) => {
                match value {
                    Expr::Ok(v, _) => self.pattern_matches_const_with_bindings(inner, v, bindings),
                    _ => false,
                }
            }
            xiom_ast::Pattern::Err(inner, _) => {
                match value {
                    Expr::Err(v, _) => self.pattern_matches_const_with_bindings(inner, v, bindings),
                    _ => false,
                }
            }
            xiom_ast::Pattern::Struct(name, fields, _) => {
                match value {
                    Expr::Struct(s_name, s_fields, _, _) => {
                        if name.name != s_name.name { return false; }
                        for (field_name, field_pat) in fields {
                            let field_val = s_fields.iter()
                                .find(|(n, _)| &n.name == &field_name.name)
                                .map(|(_, v)| v);
                            if let Some(fv) = field_val {
                                if !self.pattern_matches_const_with_bindings(field_pat, fv, bindings) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                        true
                    }
                    _ => false,
                }
            }
            xiom_ast::Pattern::Tuple(elements, _) => {
                match value {
                    Expr::Tuple(items, _) => {
                        if elements.len() != items.len() { return false; }
                        for (elem, item) in elements.iter().zip(items.iter()) {
                            if !self.pattern_matches_const_with_bindings(elem, item, bindings) {
                                return false;
                            }
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Evaluate an expression with pattern binding substitutions.
    /// Replaces `Expr::Ident(name)` with the bound value if present.
    fn evaluate_const_init_with_bindings(
        &self,
        expr: &Expr,
        bindings: &std::collections::HashMap<String, Expr>,
    ) -> Expr {
        match expr {
            Expr::Ident(id) => {
                if let Some(val) = bindings.get(&id.name) {
                    return self.evaluate_const_init(val);
                }
                self.evaluate_const_init(expr)
            }
            _ => self.evaluate_const_init(expr),
        }
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
                    // D1: i128/fp128 loads need 16-byte alignment (x86-64).
                    self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* {ptr}{}", self.store_align(&llvm_ty)));
                    // Propagate array-value tracking through let-bound locals:
                    // if `ident` was bound from an Expr::Array, the loaded value
                    // also originates from an array buffer so val_to_struct can
                    // construct a proper Vec from it.
                    if self.local.array_locals.contains(&ident.name) {
                        self.local.array_value_regs.insert(tmp.clone());
                    }
                    // M17: Widen narrow integer loads to i64 immediately with correct
                    // sign extension (sext for signed Int8/Int16/Int32, zext for
                    // unsigned UInt8/UInt16/UInt32/Char/Bool). This ensures the
                    // arithmetic layer always operates on i64 values while preserving
                    // correct signedness semantics.
                    let (result_val, result_ty) = match llvm_ty.as_str() {
                        "i1" => {
                            let wide = self.fresh_tmp();
                            self.emitln(&format!("  {wide} = zext i1 {tmp} to i64"));
                            self.local.reg_signed.insert(wide.clone(), false);
                            (wide, LLVM_I64.to_string())
                        }
                        "i8" | "i16" | "i32" => {
                            let is_signed = self.is_signed_local(lookup_name);
                            let wide = self.widen_to_i64_signed(&tmp, &llvm_ty, is_signed);
                            self.local.reg_signed.insert(wide.clone(), is_signed);
                            (wide, LLVM_I64.to_string())
                        }
                        _ => (tmp, llvm_ty),
                    };
                    Ok((result_val, result_ty))
                } else if let Some((symbol, llvm_ty)) = self.local.module_globals.get(&ident.name).cloned() {
                    // Mutable module-level `var`: load the current value from the
                    // real global. Checked BEFORE enum-variant / constant fallbacks
                    // so a live global is never mistaken for a compile-time literal.
                    let tmp = self.fresh_tmp();
                    self.emitln(&format!("  {tmp} = load {llvm_ty}, {llvm_ty}* @{symbol}"));
                    Ok((tmp, llvm_ty))
                } else if let Some(enum_key) = self.types.enum_variants.entries().into_iter()
    .find(|(_, vars)| vars.iter().any(|(v, _)| v == &ident.name))
                    .map(|(ek, _)| ek)
                    .filter(|ek| self.types.types.contains_key(ek))
                {
                    if let Some(vars) = self.types.enum_variants.get(&enum_key) {
                        if let Some(var_idx) = vars.iter().position(|(v, _)| v == &ident.name) {
                            let struct_ty = self.llvm_type_for(&enum_key)?;
                            let alloca = self.fresh_tmp();
                            self.emitln(&format!("  {alloca} = alloca {struct_ty}"));
                            // BUG 30: zero-init unused payload slots (LLVM poison).
                            self.emitln(&format!("  store {struct_ty} zeroinitializer, {struct_ty}* {alloca}"));
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
                    // In a method body, a bare identifier might be a field
                    // of the implicit `self` receiver (e.g. `return Point{ x: x }`
                    // where `x` is self.x). Look up the field and GEP+load it.
                    if let Some(ref recv) = self.fctx.current_receiver {
                        let recv_clone = recv.clone();
                        // Try to find the field index in the receiver's struct type
                        let struct_key = self.types.types.keys().into_iter()
    .find(|k| k.ends_with(&format!(".{recv_clone}")) || k.as_str() == &recv_clone);
                        if let Some(ref sk) = struct_key {
                            let field_names = self.types.types.get(sk);
                            if let Some(field_names) = field_names {
                                if let Some(fi) = field_names.iter().position(|f| f == &ident.name) {
                                    // Look up `self` in locals
                                    if let Some((self_ptr, _)) = self.lookup_local("self").cloned() {
                                        let gep = self.fresh_tmp();
                                        let struct_ty_actual = self.llvm_type_for(sk).unwrap_or_else(|_| format!("%struct.{sk}"));
                                        self.emitln(&format!("  {gep} = getelementptr {struct_ty_actual}, {struct_ty_actual}* {self_ptr}, i32 0, i32 {fi}"));
                                        let loaded = self.fresh_tmp();
                                        let field_ty = self.field_llvm_type(sk, fi);
                                        self.emitln(&format!("  {loaded} = load {field_ty}, {field_ty}* {gep}"));
                                        return Ok((loaded, field_ty));
                                    }
                                }
                            }
                        }
                    }
                    // A bare reference to a module/global constant: substitute its
                    // literal value (constants aren't materialized as globals).
                    if let Some(cval) = self.local.constants.get(&ident.name).cloned() {
                        return self.compile_expr(&cval);
                    }
                    // Bare `null` in an expression is the NULL POINTER (0), not a
                    // function reference. Without this, the suffix search below
                    // matches `ptr.null` and emits `ptrtoint ... @null` (undefined
                    // symbol) -- e.g. contract checks like `result != null`.
                    if ident.name == "null" {
                        return Ok(("0".to_string(), LLVM_I64.to_string()));
                    }
                    // Function name used as value (e.g. v.push(add_one)):
                    // resolve to a function pointer via ptrtoint of the IR symbol.
                    // The functions map has both bare names and module-qualified names.
                    let fn_full: Option<(String, String, Vec<String>)> = {
                        let exact = self.types.functions.get(&ident.name).map(|(p, r)| (ident.name.clone(), r.clone(), p.clone()));
                        exact.or_else(|| {
                            self.types.functions.entries().into_iter().find(|(k, _)| k.ends_with(&format!(".{}", ident.name)))
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
                    // no `this` usage -> no %param_self). Emitting the generic `0`
                    // fallback here produced silent wrong values. Fail loudly with
                    // the fix.
                    if let Some(recv) = self.fctx.current_receiver.clone() {
                        let fields = self.types.type_meta.get(&recv)
                            .or_else(|| {
                                let suffix = format!(".{recv}");
                                self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&suffix))
                                    .map(|(_, v)| v)
                            });
                        if let Some(meta) = fields {
                            if meta.fields.iter().any(|(fname, _)| fname == &ident.name) {
                                return Err(format!(
                                    "receiver field '{0}' cannot be read bare in this method -- use 'this.{0}' (bare fields need a `self` param, a `&{1}` receiver-style first param, or a `this`-based body)",
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
            // D1: big literal (beyond u64) -- emits an i128 constant. The value
            // is a bit pattern; signedness is applied by the surrounding cast.
            Expr::BigInt(n, _) => {
                Ok((format!("{n}"), "i128".to_string()))
            }
            Expr::Float(f, _) => {
                // BUG 10 fix (2026-08-11): {:.6} truncated literals to 6
                // decimals (0.123456789 -> 0.123457). {:.17e} gives 17
                // significant digits -- exact f64 round-trip, and LLVM accepts
                // the exponent form (e.g. 1.23456789000000000e-1).
                Ok((format!("{f:.17e}"), "double".to_string()))
            }
            Expr::Bool(b, _) => {
                Ok((if *b { "1".to_string() } else { "0".to_string() }, LLVM_I64.to_string()))
            }
            Expr::Str(s, _) => {
                let tmp = self.intern_cstring(s);
                Ok((tmp, LLVM_STR_PTR.to_string()))
            }
            Expr::Char(c, _) => {
                // M17: Char is i32 (Unicode 32-bit), not i8
                Ok((format!("{}", *c as u32), "i32".to_string()))
            }
            Expr::Paren(inner, _) => self.compile_expr(inner),
            Expr::Tuple(items, _) => {
                if items.is_empty() {
                    Ok(("0".to_string(), "void".to_string()))
                } else {
                    // 5c.36: Register tuple type dynamically before inference
                    // so expression-level tuples like (x, y) get proper struct types.
                    let elem_types: Vec<String> = items.iter()
                        .map(|i| {
                            // BUG 23 #7 fix: prefer the REGISTERED XIOM type for
                            // idents -- infer_llvm_type erases Bool->i64, which named
                            // (Bool, Bool) tuples "Tuple__Int__Int" and broke
                            // cross-module Bool-tuple field access.
                            if let Expr::Ident(id) = i {
                                if let Some(xiom) = self.local.local_xiom_types.get(&id.name) {
                                    // BUG 52 follow-up (2026-08-18): container
                                    // bindings record ARG-bearing types
                                    // ("Vec[Int]", "Map[Str, MyVal]") for generic
                                    // method inference -- but TUPLE element names
                                    // must stay bare ("Vec", "Map") to match the
                                    // decl-side registration (infer_expr_type_name
                                    // / type_from_ast drop the args); a bracketed
                                    // element name produced invalid LLVM
                                    // identifiers ("Tuple__Vec[Int]__Vec[Int]" ->
                                    // clang "expected '=' after name") and broke
                                    // tuple types over containers (smoke_iter).
                                    return match xiom.find('[') {
                                        Some(b) => xiom[..b].to_string(),
                                        None => xiom.clone(),
                                    };
                                }
                            }
                            // BUG 29 (BUG 28 #6): name LITERALS by their XIOM type
                            // too. infer_llvm_type erases Bool->i64->"Int", so
                            // `(PathBuf, Bool)` returns built the expression
                            // "Tuple__PathBuf__Int" while the fn signature
                            // registered "Tuple__PathBuf__Bool" -- the expr-built
                            // type was never pre-registered, its definition
                            // emitted AFTER the alloca that used it, and clang
                            // rejected "Cannot allocate unsized type"
                            // (os/path.xi PathBuf.pop + os/file.xi combo).
                            match i {
                                Expr::Bool(..) => "Bool".to_string(),
                                Expr::Int(..) | Expr::BigInt(..) => "Int".to_string(),
                                Expr::Str(..) => "Str".to_string(),
                                Expr::Float(..) => "Float64".to_string(),
                                Expr::Char(..) => "Char".to_string(),
                                _ => {
                                    let t = self.infer_llvm_type(i);
                                    IrEmitter::xiom_type_name_from_llvm(&t)
                                }
                            }
                        })
                        .collect();
                    let name = format!("Tuple__{}", elem_types.join("__"));
                    if !self.types.type_meta.contains_key(&name) {
                        let field_names: Vec<String> = (0..elem_types.len()).map(|i| format!("_{i}")).collect();
                        let field_llvm: Vec<(String, String)> = elem_types.iter().enumerate()
                            .map(|(i, tn)| (format!("_{i}"), tn.clone()))
                            .collect();
                        self.types.types.insert(name.clone(), field_names);
                        self.types.type_meta.or_insert_with(name.clone(), || TypeMeta {
                            fields: field_llvm,
                            derives: vec![],
                            invariants: vec![],
                        });
                        // 5c.36: Emit struct definition at module level (deferred
                        // to the end of the module so it never appears inline inside
                        // a function body, which clang rejects). LLVM named types
                        // support forward references, so deferred emission is safe.
                        let field_llvm_types: Vec<String> = elem_types.iter()
                            .map(|tn| self.llvm_type_for(tn).unwrap_or_else(|_| "i64".to_string()))
                            .collect();
                        self.local.pending_module_type_defs.push(
                            format!("%struct.{name} = type {{ {} }}", field_llvm_types.join(", "))
                        );
                    }
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
                    // Parse field types from struct name: %struct.Tuple_Float32_Int ->
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
                // `&*p` / `&mut *p` is a REBORROW: it must yield p's VALUE
                // (the pointee address), not load through it. The Deref arm
                // loads the pointee, so compiling `&(*p)` as one unit
                // returned the loaded value -- Box.get's `return &*ptr`
                // returned the boxed 42 instead of the box address
                // (smoke_core_box dereferenced 42 -> AV at 0x2a). Peek BEFORE
                // compiling `inner` so no dead load is emitted.
                if matches!(op, UnaryOp::Ref | UnaryOp::MutRef) {
                    if let Expr::Unary(UnaryOp::Deref, ref_inner, _) = inner.as_ref() {
                        return self.compile_expr(ref_inner);
                    }
                }
                let (val, inner_ty) = self.compile_expr(inner)?;
                let tmp = self.fresh_tmp();
                match op {
                    UnaryOp::Neg => {
                        // BUG 31: fp128 (Float128) must also use `fneg` -- the
                        // integer `sub i64 0, %val` path made clang reject
                        // (`'%tmp' defined with type 'fp128' but expected 'i64'`).
                        if inner_ty == "double" || inner_ty == "float" || inner_ty == "fp128" {
                            self.emitln(&format!("  {tmp} = fneg {inner_ty} {val}"));
                            return Ok((tmp, inner_ty.clone()));
                        } else {
                            // B-006: Widen narrow int to i64 before negation.
                            // `-128 as Int8` produces an i8 value; must zext/sext to
                            // i64 before `sub i64 0, %val`.
                            let wide = self.widen_to_i64(&val, &inner_ty);
                            self.emitln(&format!("  {tmp} = sub i64 0, {wide}"));
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
                        // B-005: Widen narrow int to i64 before bitwise NOT.
                        // `~x` where x: Int32 produces an i32 value; must promote
                        // to i64 before `xor i64 %val, -1`.
                        let wide = self.widen_to_i64(&val, &inner_ty);
                        self.emitln(&format!("  {tmp} = xor i64 {wide}, -1"));
                        return Ok((tmp, LLVM_I64.to_string()));
                    }
                    UnaryOp::Deref => {
                        // `*p`: load through a real pointer. `inner_ty` is e.g. `i64*`
                        // (from a `*T` value). Load the pointee type.
                        if inner_ty.ends_with('*') {
                            // BUG 44: strip exactly ONE star. `trim_end_matches('*')`
                            // stripped ALL trailing stars, so deref of a `&Str`
                            // (an `i8**` -- pointer to the Str slot) emitted
                            // `load i8, i8**` (a byte) instead of `load i8*, i8**`.
                            let pointee = inner_ty.strip_suffix('*').unwrap_or(&inner_ty).to_string();
                            self.emitln(&format!("  {tmp} = load {pointee}, {inner_ty} {val}"));
                            return Ok((tmp, pointee));
                        }
                        // inner_ty is i64: the value might be a ptrtoint'd pointer
                        // (ptr.offset() byte pointer) or a reference to a scalar
                        // (&Int stored as i64-value). Try to determine the pointee
                        // type from the XIOM type system, falling back to i8 for
                        // byte-pointer compat.
                        if inner_ty == "i64" {
                            // Check if inner is a local with a &T XIOM type
                            let pointee_llvm = if let Expr::Ident(id) = inner.as_ref() {
                                // `&T` params carry the ADDRESS (i64); type_from_ast
                                // strips the &, so local_xiom_types holds "Int" for a
                                // `r: &Int` param -- that IS the pointee type. This
                                // fixes `*r` loading i8 instead of the declared width.
                                // BUG 44: ref-LOCALS (`var p = &s`, `var p: &Str = ..`)
                                // follow the same rule via ref_locals.
                                if self.local.param_locals.contains(&id.name) || self.local.ref_locals.contains(&id.name) {
                                    self.local.local_xiom_types.get(&id.name)
                                        .and_then(|xiom_ty| self.llvm_type_for(xiom_ty).ok())
                                } else {
                                    self.local.local_xiom_types.get(&id.name)
                                        .and_then(|xiom_ty| {
                                            if xiom_ty.starts_with('&') {
                                                let pointee = &xiom_ty[1..]; // strip &
                                                self.llvm_type_for(pointee).ok()
                                            } else { None }
                                        })
                                }
                            } else { None };
                            if let Some(pointee) = pointee_llvm {
                                // `&T` params/locals carry the ADDRESS (as i64) --
                                // inttoptr to the pointee type and load the value.
                                let ptr = self.fresh_tmp();
                                self.emitln(&format!("  {ptr} = inttoptr i64 {val} to {pointee}*"));
                                self.emitln(&format!("  {tmp} = load {pointee}, {pointee}* {ptr}"));
                                return Ok((tmp, pointee));
                            }
                            // M19: Legacy byte-pointer path (ptr.offset() compat)
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
                        let mut acc_expr = operands[0].clone();
                        // Iteratively compile and fold each remaining operand
                        for operand in &operands[1..] {
                            let (r_val, r_ty) = self.compile_expr(operand)?;
                            let (l, lt) = (acc_val, acc_ty);
                            let (r, rt) = (r_val, r_ty);
                            // Reuse the standard BinOp lowering for each pair
                            let folded = self.compile_binop_fold(&l, &lt, &r, &rt, op, &acc_expr, operand)?;
                            acc_val = folded.0;
                            acc_ty = folded.1;
                            acc_expr = (*operand).clone();
                        }
                        return Ok((acc_val, acc_ty));
                    }
                }
                // Normal path (short chain or non-foldable operator)
                // BUG 22 #1 fix: && / || MUST SHORT-CIRCUIT. The previous
                // lowering compiled BOTH operands then bitwise-and'ed them,
                // so a div-by-zero / fault in the RHS trapped even when the
                // LHS already decided the result. Branch on the LHS and
                // compile the RHS only in the block where it is needed.
                if matches!(op, BinOp::And | BinOp::Or) {
                    let is_or = matches!(op, BinOp::Or);
                    let (l, lt) = self.compile_expr(left)?;
                    let l_i1 = if lt == "i1" { l.clone() } else {
                        let ne = self.fresh_tmp();
                        self.emitln(&format!("  {ne} = icmp ne {lt} {l}, 0"));
                        ne
                    };
                    let rhs_block = self.fresh_block("logic_rhs");
                    let done_false = self.fresh_block("logic_done_false");
                    let done_block = self.fresh_block("logic_done");
                    // && : LHS false -> skip RHS, result 0.
                    // || : LHS true  -> skip RHS, result 1.
                    let skip_cond = if is_or {
                        l_i1.clone()
                    } else {
                        let not = self.fresh_tmp();
                        self.emitln(&format!("  {not} = xor i1 {l_i1}, true"));
                        not
                    };
                    self.emitln(&format!("  br i1 {skip_cond}, label %{done_false}, label %{rhs_block}"));
                    self.emitln(&format!("\n{rhs_block}:"));
                    let (r, rt) = self.compile_expr(right)?;
                    let r_i1 = if rt == "i1" { r.clone() } else {
                        let ne = self.fresh_tmp();
                        self.emitln(&format!("  {ne} = icmp ne {rt} {r}, 0"));
                        ne
                    };
                    let rhs_res = self.fresh_tmp();
                    self.emitln(&format!("  {rhs_res} = zext i1 {r_i1} to i64"));
                    // The RHS may have emitted its own blocks (elem-load
                    // switches, nested &&/||). Its LAST block must terminate
                    // with a branch to %done so the phi below has a valid
                    // predecessor. If the RHS's last block already terminated
                    // (rare -- only statement-like expressions), fall back to
                    // unconditional evaluation (correct result, no
                    // short-circuit) rather than emitting invalid IR.
                    if !self.current_block_terminated() {
                        let rhs_last = self.current_block_label().unwrap_or_else(|| rhs_block.clone());
                        self.emitln(&format!("  br label %{done_block}"));
                        self.emitln(&format!("\n{done_false}:"));
                        self.emitln(&format!("  br label %{done_block}"));
                        self.emitln(&format!("\n{done_block}:"));
                        let phi = self.fresh_tmp();
                        let skip_val = if is_or { "1" } else { "0" };
                        self.emitln(&format!("  {phi} = phi i64 [ {skip_val}, %{done_false} ], [ {rhs_res}, %{rhs_last} ]"));
                        return Ok((phi, LLVM_I64.to_string()));
                    }
                    // Fallback: RHS control flow already terminated -- evaluate
                    // both sides unconditionally (pre-BUG-22 semantics).
                    let lw = if lt == "i64" { l.clone() } else {
                        let ext = self.fresh_tmp();
                        self.emitln(&format!("  {ext} = zext {lt} {l} to i64"));
                        ext
                    };
                    let rw = if rt == "i64" { r.clone() } else {
                        let ext = self.fresh_tmp();
                        self.emitln(&format!("  {ext} = zext {rt} {r} to i64"));
                        ext
                    };
                    let op_name = if is_or { "or" } else { "and" };
                    let result = self.fresh_tmp();
                    self.emitln(&format!("  {result} = {op_name} i64 {lw}, {rw}"));
                    return Ok((result, LLVM_I64.to_string()));
                }
                let (mut l, mut lt) = self.compile_expr(left)?;
                let (mut r, mut rt) = self.compile_expr(right)?;
                let tmp = self.fresh_tmp();
                // Str + Str: concatenate at runtime, not `add i64` on pointers.
                // A Str is `i8*` at the ABI; `add` on two pointers is invalid IR
                // and semantically wrong. Lower to a call to the runtime concat.
                // Fires when EITHER operand is a Str pointer (the other side is
                // coerced to i8*), which also keeps IR valid where a Str-returning
                // callee was resolved to a fallback i64 signature.
                // round-7 (ve2): a *UInt8 byte buffer is ALSO i8* at the ABI but
                // must stay POINTER arithmetic -- expr_is_pointer gates it out
                // (only when the other operand is an integer; Str+Str and
                // Str+buffer shapes keep concatenation).
                if matches!(op, BinOp::Add) && (lt == "i8*" || rt == "i8*")
                    && !((lt == "i8*" && self.expr_is_pointer(left) && self.expr_is_integer(right))
                        || (rt == "i8*" && self.expr_is_pointer(right) && self.expr_is_integer(left))) {
                    let lp = self.concat_val_to_i8ptr(&l, &lt, self.expr_is_integer(left));
                    let rp = self.concat_val_to_i8ptr(&r, &rt, self.expr_is_integer(right));
                    let res = self.fresh_tmp();
                    self.emitln(&format!("  {res} = call i8* @xiom_str_concat(i8* {lp}, i8* {rp})"));
                    return Ok((res, LLVM_STR_PTR.to_string()));
                }
                // Pointer arithmetic: `p + i` / `p - i` on a REAL pointer
                // (i64*/i32*/%struct.X*/double*/i8*-byte-buffer...) must
                // GEP-scale by the ELEMENT size, not integer-add the raw index
                // to the pointer bits. The old ptrtoint/add/inttoptr lowering
                // also lost the pointee type, so `*(p + 1)` loaded a BYTE
                // instead of the element (round-7 ve2 follow-up: mono'd
                // Vec.first/last/insert/remove bodies do `*(data + len - 1)` --
                // byte offsets + byte loads on Vec[Int]).
                if matches!(op, BinOp::Add | BinOp::Sub) {
                    let (pval, pty, ival, ity) = if lt.ends_with('*') && !rt.ends_with('*') {
                        (l.clone(), lt.clone(), r.clone(), rt.clone())
                    } else if rt.ends_with('*') && !lt.ends_with('*') {
                        (r.clone(), rt.clone(), l.clone(), lt.clone())
                    } else {
                        ("".to_string(), String::new(), "".to_string(), String::new())
                    };
                    if !pty.is_empty() {
                        let pointee = pty.strip_suffix('*').unwrap_or(&pty).to_string();
                        let idx = if ity == "i64" { ival.clone() } else {
                            let w = self.fresh_tmp();
                            self.emitln(&format!("  {w} = sext {ity} {ival} to i64"));
                            w
                        };
                        let gep = self.fresh_tmp();
                        if matches!(op, BinOp::Sub) {
                            let neg = self.fresh_tmp();
                            self.emitln(&format!("  {neg} = sub i64 0, {idx}"));
                            self.emitln(&format!("  {gep} = getelementptr {pointee}, {pty} {pval}, i64 {neg}"));
                        } else {
                            self.emitln(&format!("  {gep} = getelementptr {pointee}, {pty} {pval}, i64 {idx}"));
                        }
                        return Ok((gep, pty));
                    }
                }
                let is_float = self.is_float_expr(left) || self.is_float_expr(right)
                    || lt == "float" || lt == "double" || lt == "fp128" || rt == "float" || rt == "double" || rt == "fp128";
                // Determine the actual float type from the operands.
                // If either operand is `float` (Float32), use `float` for the
                // comparison; otherwise default to `double` (Float64).
                let float_ty = if lt == "float" || rt == "float" { "float" }
                    else if lt == "fp128" || rt == "fp128" { "fp128" }
                    else { "double" };
                // D1 (2026-08-08): 128-bit integer detection -- i128 operands
                // must NOT be widened to i64. This drives the integer binop
                // type choice below (i128 vs i64).
                let int128_ty = if lt == "i128" || rt == "i128" { "i128" } else { "i64" };
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
                        } else if lt_is_struct && rt_is_struct && lt == rt {
                            // BUG 24 fix: no derived `.eq` registered -- compare
                            // STRUCTURALLY over ALL fields instead of only field 0
                            // (the old fallback compared `sign` for BigFloat ==
                            // BigFloat -- silent miscompare). Mirrors compile_eq_impl:
                            // scalars icmp/oeq, nested structs via their derived eq.
                            let l_a = self.fresh_tmp();
                            let r_a = self.fresh_tmp();
                            self.emitln(&format!("  {l_a} = alloca {lt}"));
                            self.emitln(&format!("  store {lt} {l}, {lt}* {l_a}"));
                            self.emitln(&format!("  {r_a} = alloca {rt}"));
                            self.emitln(&format!("  store {rt} {r}, {rt}* {r_a}"));
                            let mut acc: Option<String> = None;
                            if let Some(meta) = self.types.type_meta.get(&struct_name.to_string()) {
                                for (i, (_fname, _)) in meta.fields.iter().enumerate() {
                                    let field_llvm = self.field_llvm_type(struct_name, i);
                                    let lg = self.fresh_tmp(); let lv = self.fresh_tmp();
                                    let rg = self.fresh_tmp(); let rv = self.fresh_tmp();
                                    self.emitln(&format!("  {lg} = getelementptr {lt}, {lt}* {l_a}, i32 0, i32 {i}"));
                                    self.emitln(&format!("  {lv} = load {field_llvm}, {field_llvm}* {lg}"));
                                    self.emitln(&format!("  {rg} = getelementptr {rt}, {rt}* {r_a}, i32 0, i32 {i}"));
                                    self.emitln(&format!("  {rv} = load {field_llvm}, {field_llvm}* {rg}"));
                                    let mut cmp = self.fresh_tmp();
                                    if field_llvm.starts_with("%struct.") {
                                        let eq_fn = format!("{}.eq", &field_llvm[8..]);
                                        if self.types.functions.contains_key(&eq_fn) {
                                            self.emitln(&format!("  {cmp} = call i64 @{eq_fn}({field_llvm} {lv}, {field_llvm} {rv})"));
                                        } else {
                                            let pi = self.fresh_tmp();
                                            self.emitln(&format!("  {pi} = icmp eq {field_llvm} {lv}, {rv}"));
                                            let ze = self.fresh_tmp();
                                            self.emitln(&format!("  {ze} = zext i1 {pi} to i64"));
                                            cmp = ze;
                                        }
                                    } else if matches!(field_llvm.as_str(), "double" | "float" | "fp128") {
                                        let fc = self.fresh_tmp();
                                        self.emitln(&format!("  {fc} = fcmp oeq {field_llvm} {lv}, {rv}"));
                                        let ze = self.fresh_tmp();
                                        self.emitln(&format!("  {ze} = zext i1 {fc} to i64"));
                                        cmp = ze;
                                    } else {
                                        let ic = self.fresh_tmp();
                                        self.emitln(&format!("  {ic} = icmp eq {field_llvm} {lv}, {rv}"));
                                        let ze = self.fresh_tmp();
                                        self.emitln(&format!("  {ze} = zext i1 {ic} to i64"));
                                        cmp = ze;
                                    }
                                    acc = Some(match acc {
                                        None => cmp,
                                        Some(prev) => {
                                            let a = self.fresh_tmp();
                                            self.emitln(&format!("  {a} = and i64 {prev}, {cmp}"));
                                            a
                                        }
                                    });
                                }
                            }
                            // Empty structs are trivially equal.
                            let eq_result = match acc {
                                Some(a) => a,
                                None => {
                                    let one = self.fresh_tmp();
                                    self.emitln(&format!("  {one} = add i64 0, 1"));
                                    one
                                }
                            };
                            if matches!(op, BinOp::Neq) {
                                let negated = self.fresh_tmp();
                                self.emitln(&format!("  {negated} = xor i64 {eq_result}, 1"));
                                return Ok((negated, LLVM_I64.to_string()));
                            }
                            return Ok((eq_result, LLVM_I64.to_string()));
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
                    // round-8 (rw1): a reference-typed operand (&Str -- from
                    // `Some(&items[i])` payloads or `&local`) holds the T
                    // SLOT ADDRESS (i64/i8**/i8* forms); load through it FIRST
                    // so the condition sees the deref'd type and the content
                    // compare reads the string, not the address bytes.
                    // ASSIGN to the outer mut l/lt (a shadow would be lost
                    // after this block -- the generic icmp at the tail uses
                    // the outer bindings).
                    let (l2, lt2) = self.auto_deref_ref(left, &l, &lt);
                    l = l2;
                    lt = lt2;
                    let (r2, rt2) = self.auto_deref_ref(right, &r, &rt);
                    r = r2;
                    rt = rt2;
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
                // M17: Extract scalar field 0 from struct operands BEFORE the
                // widen_to_i64 block below, which resets lt/rt to "i64". When a
                // struct value (e.g. %struct.Result from a contract's `result`)
                // is compared with an integer, the struct's discriminant must be
                // extracted first so the `icmp` operates on a scalar type.
                // Skip pointer types (%struct.X*) -- they should be compared as
                // pointers, not have their fields extracted.
                if lt.starts_with("%struct.") && !lt.ends_with('*') && !rt.starts_with("%struct.") {
                    l = self.extract_scalar_field0(&l, &lt);
                }
                if rt.starts_with("%struct.") && !rt.ends_with('*') && !lt.starts_with("%struct.") {
                    r = self.extract_scalar_field0(&r, &rt);
                }
                // Widen narrow integer operands (i1/i8/i16/i32) to i64 before
                // emitting arithmetic, bitwise, shift, or comparison operations.
                // This prevents LLVM type mismatches when Int8/Int16/Int32 values
                // flow into binary ops that expect i64 operands. (B-004, B-005, B-006)
                // D1: i128 operands are already wide -- never widen them.
                if !is_float && !lt.contains('*') && !rt.contains('*') {
                    if int128_ty == "i128" {
                        // Widen narrow operands UP to i128 (sext/zext) so both
                        // sides share the i128 type for the op.
                        if lt != "i128" {
                            let ext = self.fresh_tmp();
                            let extop = if lt == "i8" || lt == "i1" { "zext" } else { "sext" };
                            self.emitln(&format!("  {ext} = {extop} {lt} {l} to i128"));
                            l = ext;
                            lt = "i128".to_string();
                        }
                        if rt != "i128" {
                            let ext = self.fresh_tmp();
                            let extop = if rt == "i8" || rt == "i1" { "zext" } else { "sext" };
                            self.emitln(&format!("  {ext} = {extop} {rt} {r} to i128"));
                            r = ext;
                            rt = "i128".to_string();
                        }
                    } else {
                        l = self.widen_to_i64(&l, &lt);
                        r = self.widen_to_i64(&r, &rt);
                        lt = "i64".to_string();
                        rt = "i64".to_string();
                    }
                }
                let int_ty = if int128_ty == "i128" { "i128" } else { "i64" };
                let (ty, inst) = match op {
                    BinOp::Add => (if is_float { float_ty } else { int_ty }, if is_float { "fadd" } else { "add" }),
                    BinOp::Sub => (if is_float { float_ty } else { int_ty }, if is_float { "fsub" } else { "sub" }),
                    BinOp::Mul => (if is_float { float_ty } else { int_ty }, if is_float { "fmul" } else { "mul" }),
                    BinOp::Div => (if is_float { float_ty } else { int_ty }, if is_float { "fdiv" } else { "sdiv" }),
                    BinOp::Rem => (if is_float { float_ty } else { int_ty }, if is_float { "frem" } else { "srem" }),
                    BinOp::BitXor => (int_ty, "xor"),
                    BinOp::BitAnd => (int_ty, "and"),
                    BinOp::BitOr => (int_ty, "or"),
                    BinOp::Shl => (int_ty, "shl"),
                    // BUG 14 fix (2026-08-11): UInt128 (and any unsigned
                    // integer) right-shift must use LSHR -- ashr sign-extends
                    // and corrupts values with the top bit set.
                    BinOp::Shr => {
                        let unsigned = self.expr_is_unsigned(left);
                        (int_ty, if unsigned { "lshr" } else { "ashr" })
                    }
                    BinOp::Eq => (if is_float { float_ty } else { int_ty }, if is_float { "fcmp oeq" } else { "icmp eq" }),
                    // BUG 19 fix (2026-08-11): float `!=` must lower to fcmp UNE
                    // (unordered-or-not-equal), not `one` -- for NaN operands `one`
                    // is FALSE, so `x != x` returned false and is_nan was impossible;
                    // IEEE requires NaN != NaN to be TRUE (`une`).
                    BinOp::Neq => (if is_float { float_ty } else { int_ty }, if is_float { "fcmp une" } else { "icmp ne" }),
                    BinOp::Lt => (if is_float { float_ty } else { int_ty }, if is_float { "fcmp olt" } else { "icmp slt" }),
                    BinOp::Gt => (if is_float { float_ty } else { int_ty }, if is_float { "fcmp ogt" } else { "icmp sgt" }),
                    BinOp::Le => (if is_float { float_ty } else { int_ty }, if is_float { "fcmp ole" } else { "icmp sle" }),
                    BinOp::Ge => (if is_float { float_ty } else { int_ty }, if is_float { "fcmp oge" } else { "icmp sge" }),
                    BinOp::Assign => {
                        // Compile the RHS value first
                        let (r_val, r_ty) = (r.clone(), rt.clone());
                        // Compile the LHS as an lvalue (pointer to the storage location)
                        if let Some((l_ptr, l_ptr_ty, l_elem_ty)) = self.compile_lvalue(left) {
                            let store_val = self.coerce_value(&r_val, &r_ty, &l_elem_ty);
                            self.emitln(&format!("  store {l_elem_ty} {store_val}, {l_ptr_ty} {l_ptr}{}", self.store_align(&l_elem_ty)));
                            return Ok((store_val, l_elem_ty));
                        }
                        // Fallback: return RHS (simple variable assignment handled by let/var)
                        return Ok((r_val, r_ty));
                    }
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
                    if lt.starts_with("%struct.") && !lt.ends_with('*') {
                        l = self.extract_scalar_field0(&l, &lt);
                    }
                    if rt.starts_with("%struct.") && !rt.ends_with('*') {
                        r = self.extract_scalar_field0(&r, &rt);
                    }
                    if lt == "i64" || (lt.starts_with("%struct.") && !lt.ends_with('*')) {
                        let conv = self.fresh_tmp();
                        // 5c.29: `opt.unwrap()` returns the float payload as RAW
                        // BITS in an i64 (Some(x) stores via bitcast) -- so the
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
                    // Narrow double -> float when the operation uses float
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
                    if lt.starts_with("%struct.") && !lt.ends_with('*') && !rt.starts_with("%struct.") {
                        l = self.extract_scalar_field0(&l, &lt);
                    }
                    if rt.starts_with("%struct.") && !rt.ends_with('*') && !lt.starts_with("%struct.") {
                        r = self.extract_scalar_field0(&r, &rt);
                    }
                    l = self.widen_to_i64(&l, &lt);
                    r = self.widen_to_i64(&r, &rt);
                }
                let div_cont = if !is_float && matches!(op, BinOp::Div | BinOp::Rem) {
                    let zero_check = self.fresh_tmp();
                    // D1: the zero-check must use the operand's actual type
                    // (i128 for Int128/UInt128, i64 otherwise).
                    let cmp_ty = if rt == "i128" { "i128" } else { "i64" };
                    self.emitln(&format!("  {zero_check} = icmp eq {cmp_ty} {r}, 0"));
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
                let (val, inner_ty) = self.compile_expr(inner)?;
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
                                if let Some(field_names) = self.types.types.get(&type_name.to_string()) {
                                    opt_like = field_names.len() <= 2;
                                }
                            }
                        }
                        if opt_like { self.types.used_builtins.insert("Option".to_string()); }
                        else { self.types.used_builtins.insert("Result".to_string()); }
                        opt_like
                    }
                    Expr::Call(func, _, _)
                    | Expr::GenericCall(func, _, _, _) => {
                        // Check return type from function signatures
                        let fn_name = match &**func {
                            Expr::Ident(name) => Some(name.name.clone()),
                            Expr::Field(_, field, _) => Some(field.name.clone()),
                            _ => None,
                        };
                        if let Some(name) = fn_name {
                            // If function is defined and its return type is a struct with <=2 fields
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
                    let opt_ty = if inner_ty.starts_with("%struct.") { inner_ty.as_str() } else { "%struct.Option" };
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
                    let payload_llvm_ty = if opt_ty.starts_with("%struct.") {
                        self.field_llvm_type(&opt_ty[8..], 1)
                    } else {
                        "i64".to_string()
                    };
                    self.emitln(&format!("  {some_val} = load {payload_llvm_ty}, {payload_llvm_ty}* {val_gep}"));
                    Ok((some_val, payload_llvm_ty))
                } else {
                    let result_ty = if inner_ty.starts_with("%struct.") { inner_ty.as_str() } else { "%struct.Result" };
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
                        let err_llvm_ty = if result_ty.starts_with("%struct.") {
                            self.field_llvm_type(&result_ty[8..], 2)
                        } else {
                            "i64".to_string()
                        };
                        self.emitln(&format!("  {err_val} = load {err_llvm_ty}, {err_llvm_ty}* {err_gep}"));
                        let ev = self.coerce_value(&err_val, &err_llvm_ty, &ret_ty);
                        self.emitln(&format!("  ret {ret_ty} {ev}"));
                    }
                    self.emitln(&format!("\n{ok_block}:"));
                    let val_gep = self.fresh_tmp();
                    let ok_val = self.fresh_tmp();
                    self.emitln(&format!("  {val_gep} = getelementptr {result_ty}, {result_ty}* {result_alloca}, i32 0, i32 1"));
                    let ok_llvm_ty = if result_ty.starts_with("%struct.") {
                        self.field_llvm_type(&result_ty[8..], 1)
                    } else {
                        "i64".to_string()
                    };
                    self.emitln(&format!("  {ok_val} = load {ok_llvm_ty}, {ok_llvm_ty}* {val_gep}"));
                    Ok((ok_val, ok_llvm_ty))
                }
            }
            Expr::Imply(left, right, _) => {
                // BUG 30 fix: scope the Imply so a bare `is Some`/`is Ok`/
                // `is Err` rebind of the scrutinee ident (contract ensures
                // like `result is Some => result.len() > 0`) is discarded
                // after the consequence compiles. Without the scope, the
                // rebind persisted in the locals map and every LATER
                // contract check at another return site resolved `result`
                // to the hoisted i64 payload slot -> the i64-form Is()
                // inttoptr+load dereferenced the payload value as a
                // pointer (uninitialized on Some-return paths) -> AV.
                self.push_scope();
                // BUG 38: the bare-scrutinee payload rebind in Is()
                // (`x is Some` rebinds x to the payload slot) is a
                // contract-ensures convenience (BUG 29: `result is Some
                // => result.len()`) and must fire ONLY inside the Imply
                // left side. In if/while conditions it leaked into the
                // enclosing scope and poisoned the subsequent `match x`
                // (payload read as i64, Some(v) arm bound 0).
                let saved_imply_lhs = self.local.in_imply_lhs;
                self.local.in_imply_lhs = true;
                let (l, lt) = self.compile_expr(left)?;
                self.local.in_imply_lhs = saved_imply_lhs;
                // bi4 fix (2026-08-19): SHORT-CIRCUIT the consequence.
                // The old code compiled the right side unconditionally and
                // masked it with `or (!l, r)` -- for an Err result the
                // consequence's payload UNBOX (`result is Ok => result.len()`
                // inttoptrs the payload slot and loads %struct.Vec) executed
                // anyway -> load from NULL (payload 0) -> UB -> the Err return
                // value got corrupted and gzip_decompress returned Ok for
                // garbage input. The consequence now runs only when the left
                // side is true; the false path contributes literal 1.
                let l_i1 = if lt == "i1" {
                    l.clone()
                } else {
                    let t = self.fresh_tmp();
                    self.emitln(&format!("  {t} = icmp ne {lt} {l}, 0"));
                    t
                };
                let imply_alloca = self.fresh_tmp();
                self.emitln(&format!("  {imply_alloca} = alloca i64"));
                // Default TRUE before the branch: the false path skips the
                // consequence entirely and must read 1 (vacuous implication).
                self.emitln(&format!("  store i64 1, i64* {imply_alloca}"));
                let conseq_block = self.fresh_block("imply_conseq");
                let done_block = self.fresh_block("imply_done");
                self.emitln(&format!("  br i1 {l_i1}, label %{conseq_block}, label %{done_block}"));
                self.emitln(&format!("\n{conseq_block}:"));
                let (r, _rt) = self.compile_expr(right)?;
                self.emitln(&format!("  store i64 {r}, i64* {imply_alloca}"));
                self.emitln(&format!("  br label %{done_block}"));
                self.emitln(&format!("\n{done_block}:"));
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load i64, i64* {imply_alloca}"));
                self.pop_scope();
                Ok((loaded, LLVM_I64.to_string()))
            }
            Expr::Is(expr, pattern, _) => {
                let (val, ty) = self.compile_expr(expr)?;
                // Determine which variant we're checking (common to all paths)
                let variant_name = match &pattern {
                    xiom_ast::Pattern::Some(..) => "Some",
                    xiom_ast::Pattern::None(..) => "None",
                    xiom_ast::Pattern::Ok(..) => "Ok",
                    xiom_ast::Pattern::Err(..) => "Err",
                    _ => { return Ok(("1".to_string(), LLVM_I64.to_string())); }
                };
                if ty.starts_with("%struct.") {
                    let type_name = &ty[8..];
                    if let Some(variants) = self.types.enum_variants.get(&type_name.to_string()) {
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
                            // M18: Bind pattern variable (e.g. Some(n) -> bind n).
                            // BUG 30: also rebind a BARE `is Some/Ok/Err` scrutinee
                            // ident (contract ensures `result is Ok => ...`) to the
                            // payload slot -- the consequence's method calls then
                            // dispatch on the payload (Vec.len unboxing), not the
                            // Result struct (which fell to generic Map.len).
                            if let xiom_ast::Pattern::Some(inner, _)
                                | xiom_ast::Pattern::Ok(inner, _)
                                | xiom_ast::Pattern::Err(inner, _) = &pattern
                            {
                                let bind_ident: Option<&Ident> = match inner.as_ref() {
                                    xiom_ast::Pattern::Ident(id) => Some(id),
                                    // BUG 38: the bare `is Some` scrutinee-name
                                    // rebind is a contract-ensures convenience
                                    // (BUG 29) -- gate it to Imply-left contexts.
                                    _ => match expr.as_ref() {
                                        Expr::Ident(sid) if self.local.in_imply_lhs => Some(sid),
                                        _ => None,
                                    },
                                };
                                if let Some(id) = bind_ident {
                                    let payload_gep = self.fresh_tmp();
                                    self.emitln(&format!("  {payload_gep} = getelementptr {ty}, {ty}* {alloca}, i32 0, i32 1"));
                                    let payload_loaded = self.fresh_tmp();
                                    self.emitln(&format!("  {payload_loaded} = load i64, i64* {payload_gep}"));
                                    let inner_alloca = self.fresh_tmp();
                                    // BUG 30: hoist -- `&&` guard chains bind in
                                    // one block, read in a later one (dominance).
                                    self.local.hoisted_allocas.push((inner_alloca.clone(), "i64".to_string()));
                                    self.emitln(&format!("  store i64 {payload_loaded}, i64* {inner_alloca}"));
                                    self.add_local(&id.name, inner_alloca, "i64");
                                    // BUG 29: record the payload's XIOM type so
                                    // method calls on the bound var dispatch
                                    // correctly (`result is Some => result.len()`
                                    // must be Str.len, not Map.len).
                                    self.bind_is_payload_xiom(expr, &pattern, id);
                                }
                            }
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
                    // M18: Bind pattern variable before the return
                    let in_imply_lhs = self.local.in_imply_lhs;
                    let bind_payload = |emitter: &mut Self, pat: &xiom_ast::Pattern, struct_alloca: &str, struct_ty: &str| {
                        if let xiom_ast::Pattern::Some(inner, _)
                            | xiom_ast::Pattern::Ok(inner, _)
                            | xiom_ast::Pattern::Err(inner, _) = pat
                        {
                            // BUG 29 (contract Some-payload ensures): a BARE
                            // `is Some` / `is Ok` / `is Err` pattern (no inner
                            // binding var) in a contract expression binds the
                            // payload to the SCRUTINEE's own name for the
                            // consequence -- `result is Some => result.len() >
                            // 0` must call Str.len on the payload, not Map.len
                            // on the Option. Rebind the scrutinee ident to the
                            // payload slot.
                            let bind_name: Option<String> = if let xiom_ast::Pattern::Ident(id) = inner.as_ref() {
                                Some(id.name.clone())
                            } else if let Expr::Ident(sid) = expr.as_ref() {
                                // BUG 38: gate the bare-scrutinee rebind to
                                // Imply-left (contract ensures) contexts.
                                if in_imply_lhs { Some(sid.name.clone()) } else { None }
                            } else {
                                None
                            };
                            if let Some(bname) = bind_name {
                                let payload_gep = emitter.fresh_tmp();
                                emitter.emitln(&format!("  {payload_gep} = getelementptr {struct_ty}, {struct_ty}* {struct_alloca}, i32 0, i32 1"));
                                let payload_loaded = emitter.fresh_tmp();
                                emitter.emitln(&format!("  {payload_loaded} = load i64, i64* {payload_gep}"));
                                let inner_alloca = emitter.fresh_tmp();
                                // BUG 29: hoist the payload slot to fn entry so
                                // contract-expression uses in LATER blocks
                                // dominate (the `is` check branches before the
                                // consequence reads the payload).
                                emitter.local.hoisted_allocas.push((inner_alloca.clone(), "i64".to_string()));
                                emitter.emitln(&format!("  store i64 {payload_loaded}, i64* {inner_alloca}"));
                                emitter.add_local(&bname, inner_alloca, "i64");
                                // BUG 29: record the payload's XIOM type for
                                // correct method dispatch on the bound var.
                                emitter.bind_is_payload_xiom(expr, pat, &Ident { name: bname.clone(), span: xiom_ast::Span::new(0, 0) });
                            }
                        }
                    };
                    bind_payload(self, &pattern, &alloca, &ty);
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
                // M18: Handle `is` on i64 values (nested is-expressions, match-bound payloads).
                // When `inner` was bound from `x is Some(inner)`, it's stored as i64.
                // Interpret it as a pointer to a 2-field struct (disc, payload) and
                // check the discriminant directly.
                if ty == "i64" && matches!(variant_name, "Some" | "None" | "Ok" | "Err") {
                    let ptr = self.fresh_tmp();
                    self.emitln(&format!("  {ptr} = inttoptr i64 {val} to i64*"));
                    let disc = self.fresh_tmp();
                    self.emitln(&format!("  {disc} = load i64, i64* {ptr}"));
                    // Bind payload if pattern has variable
                    if let xiom_ast::Pattern::Some(inner, _)
                        | xiom_ast::Pattern::Ok(inner, _)
                        | xiom_ast::Pattern::Err(inner, _) = &pattern
                    {
                        if let xiom_ast::Pattern::Ident(id) = inner.as_ref() {
                            let payload_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {payload_ptr} = getelementptr i64, i64* {ptr}, i64 1"));
                            let payload = self.fresh_tmp();
                            self.emitln(&format!("  {payload} = load i64, i64* {payload_ptr}"));
                            let inner_alloca = self.fresh_tmp();
                            // BUG 30: hoist the payload slot to fn entry. The
                            // `&&` short-circuit chain evaluates `x is Some(v)`
                            // in one block but reads `v` in a LATER block that
                            // is also reachable via the false edge (e.g.
                            // `x if x is Some(inner) && inner is Some(v) && v > 0`)
                            // -- an inline alloca there does not dominate the
                            // use (LLVM: "Instruction does not dominate all
                            // uses"). Same treatment as the struct-form bind.
                            self.local.hoisted_allocas.push((inner_alloca.clone(), "i64".to_string()));
                            self.emitln(&format!("  store i64 {payload}, i64* {inner_alloca}"));
                            self.add_local(&id.name, inner_alloca, "i64");
                            // BUG 29: record payload XIOM type for dispatch.
                            self.bind_is_payload_xiom(expr, &pattern, id);
                        }
                    }
                    if variant_name == "Some" || variant_name == "Ok" {
                        let cmp = self.fresh_tmp();
                        self.emitln(&format!("  {cmp} = icmp ne i64 {disc}, 0"));
                        let ext = self.fresh_tmp();
                        self.emitln(&format!("  {ext} = zext i1 {cmp} to i64"));
                        return Ok((ext, LLVM_I64.to_string()));
                    } else {
                        let cmp = self.fresh_tmp();
                        self.emitln(&format!("  {cmp} = icmp eq i64 {disc}, 0"));
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
                            if let Some(field_names) = self.types.types.get(&type_name.to_string())
                                .or_else(|| {
                                    let suffix = format!(".{type_name}");
                                    self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                        .and_then(|k|self.types.types.get(&k))
                                })
                            {
                                if let Some(field_idx) = IrEmitter::resolve_field_index(&field_names, &field.name) {
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
                                        self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix))
                                            .and_then(|k|self.types.types.get(&k))
                                    })
                                    
                                {
                                    // BUG 29 (BUG 27 #12): use resolve_field_index so
                                    // BOTH `pair._1` AND `pair.1` (numeric tuple field
                                    // syntax, as in crypto.xi's `&pair.0`/`&pair.1`
                                    // AES-GCM smokes) resolve -- the raw position() only
                                    // matched the legacy `_N` form, so `pair.1` fell
                                    // through to the Str.len handler (inttoptr 0 ->
                                    // garbage lengths -> heap corruption in the gcm
                                    // smoke's decrypt roundtrip).
                                    if let Some(fi) = IrEmitter::resolve_field_index(&field_names, &field.name) {
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
                            if let Some(field_names) = self.types.types.get(&type_name.to_string())
                                .or_else(|| {
                                    let suffix = format!(".{type_name}");
                                    self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                        .and_then(|k|self.types.types.get(&k))
                                })
                                 {
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
                            let type_name = &llvm_ty[8..];
                            if let Some(field_names) = self.types.types.get(&type_name.to_string())
                                .or_else(|| {
                                    let suffix = format!(".{type_name}");
                                    self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                        .and_then(|k|self.types.types.get(&k))
                                })
                            {
                                if let Some(field_idx) = field_names.iter().position(|f| f == &field.name) {
                                    // BUG 25 #5 fix: Option/Result `.value`/`.error`
                                    // field reads must use the ACTUAL payload type --
                                    // the static slot type is i64, so Str/Vec/Float
                                    // payloads read back as raw bits (pointer-as-
                                    // number, wrong len/bit pattern). Match
                                    // extraction was already payload-aware. Only
                                    // the OVERRIDE path reinterprets the i64 slot;
                                    // regular fields load with their static type.
                                    let is_result = type_name.ends_with("Result") || type_name.contains(".Result") || type_name.starts_with("Result__");
                                    let is_option = type_name.ends_with("Option") || type_name.contains(".Option") || type_name.starts_with("Option__");
                                    let mut field_llvm_ty = self.field_llvm_type(type_name, field_idx);
                                    let mut payload_reinterpret = false;
                                    let mut payload_boxed = false;
                                    // gzip-DECOMPRESS fix (2026-08-19): payload
                                    // FIELDS (Option.value / Result.value /
                                    // Result.error) resolve the payload type via
                                    // field_payload_xiom (local_opt_payload /
                                    // local_opt_payload_xiom / local_err_payload /
                                    // declared local type) -- NOT only the scalar
                                    // local_opt_payload_xiom registry. Without it
                                    // `let decompressed = decoded.value;` bound the
                                    // raw BOXED POINTER as an i64 (and never
                                    // unboxed the heap Vec), so &decompressed
                                    // passed the i64 SLOT address as %struct.Vec*
                                    // -> crc32 read stack garbage as len/elem_size
                                    // -> 8-byte element load -> 0xC0000005.
                                    let is_payload_field = (is_option || is_result)
                                        && (field.name == "value" || field.name == "error");
                                    // R7 (2026-09-10): the payload override exists to
                                    // REINTERPRET an ERASED i64 slot. In a CONCRETE
                                    // container (Result__Uri__Str) the field already has
                                    // the full payload type (%struct.Uri, inline) -- the
                                    // static read is correct. Applying the box unbox
                                    // (inttoptr the first 8 bytes as a struct pointer)
                                    // read the scheme string POINTER as a box address
                                    // (uri_normalize -> _lower -> str_len AV at
                                    // 0x50544854).
                                    let static_payload_i64 = field_llvm_ty == "i64";
                                    if is_payload_field && static_payload_i64 {
                                        if let Some(px) = self.field_payload_xiom(obj, &field.name) {
                                            payload_reinterpret = true;
                                            match px.as_str() {
                                                "Str" => field_llvm_ty = "i8*".to_string(),
                                                "Float64" | "Float" => field_llvm_ty = "double".to_string(),
                                                "Float32" => field_llvm_ty = "float".to_string(),
                                                "Bool" | "Char" | "Int" | "Int8" | "Int16" | "Int32" | "UInt8" | "UInt16" | "UInt32" | "Int64" | "UInt" | "UInt64" | "UInt128" | "Int128" => payload_reinterpret = false,
                                                _ => {
                                                    // Boxed struct/container payload:
                                                    // inttoptr the slot to the
                                                    // struct pointer + load (unboxes
                                                    // the heap box / Vec handle).
                                                    if let Ok(st) = self.llvm_type_for(&px) {
                                                        if st.starts_with("%struct.") {
                                                            field_llvm_ty = st;
                                                            payload_boxed = true;
                                                        } else {
                                                            payload_reinterpret = false;
                                                        }
                                                    } else {
                                                        payload_reinterpret = false;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    let struct_val = self.fresh_tmp();
                                    self.emitln(&format!("  {struct_val} = load {llvm_ty}, {llvm_ty}* {ptr}"));
                let struct_alloca = self.fresh_tmp();
                self.emitln(&format!("  {struct_alloca} = alloca {llvm_ty}, align 16"));
                self.emitln(&format!("  store {llvm_ty} {struct_val}, {llvm_ty}* {struct_alloca}, align 16"));
                                    let gep = self.fresh_tmp();
                                    self.emitln(&format!("  {gep} = getelementptr {llvm_ty}, {llvm_ty}* {struct_alloca}, i32 0, i32 {field_idx}"));
                                    let loaded = if payload_reinterpret {
                                        // Payload override: the slot holds raw bits --
                                        // load i64 and reinterpret to the payload type.
                                        let raw_loaded = self.fresh_tmp();
                                        self.emitln(&format!("  {raw_loaded} = load i64, i64* {gep}"));
                                        if payload_boxed {
                                            let sp = self.fresh_tmp();
                                            self.emitln(&format!("  {sp} = inttoptr i64 {raw_loaded} to {field_llvm_ty}*"));
                                            let sv = self.fresh_tmp();
                                            self.emitln(&format!("  {sv} = load {field_llvm_ty}, {field_llvm_ty}* {sp}"));
                                            sv
                                        } else if field_llvm_ty.ends_with('*') {
                                            let ip = self.fresh_tmp();
                                            self.emitln(&format!("  {ip} = inttoptr i64 {raw_loaded} to {field_llvm_ty}"));
                                            ip
                                        } else if field_llvm_ty == "float" {
                                            let t32 = self.fresh_tmp();
                                            self.emitln(&format!("  {t32} = trunc i64 {raw_loaded} to i32"));
                                            let bc = self.fresh_tmp();
                                            self.emitln(&format!("  {bc} = bitcast i32 {t32} to {field_llvm_ty}"));
                                            bc
                                        } else {
                                            let bc = self.fresh_tmp();
                                            self.emitln(&format!("  {bc} = bitcast i64 {raw_loaded} to {field_llvm_ty}"));
                                            bc
                                        }
                                    } else {
                                        // Regular field: load with the static type.
                                        let loaded = self.fresh_tmp();
                                        self.emitln(&format!("  {loaded} = load {field_llvm_ty}, {field_llvm_ty}* {gep}"));
                                        loaded
                                    };
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
                                // BUG 30: zero-init unused payload slots (LLVM poison).
                                self.emitln(&format!("  store {struct_ty} zeroinitializer, {struct_ty}* {alloca}"));
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
                // `data.get(i).value` or `items[i].val`). The object
                // isn't a bound local, so compile it and GEP the field by index.
                // Without this, such accesses fell through to the `0` default,
                // silently discarding Option payloads passed as call arguments.
                {
                    let (mut obj_val, mut ov_ty) = self.compile_expr(obj)?;
                    // M33: When the base is an array/vec index that
                    // returns an i64 handle (pointer to boxed struct),
                    // inttoptr+load the struct before field access.
                    // E.g. `items[i].val` where items is a Vec of
                    // Container structs stored as heap pointers.
                    if ov_ty == "i64" {
                        if let Some(elem_type_name) = self.resolve_vec_elem_type_for_index(obj) {
                            let sty = format!("%struct.{elem_type_name}");
                            if sty.starts_with('%') {
                                let sp = self.fresh_tmp();
                                self.emitln(&format!("  {sp} = inttoptr i64 {obj_val} to {sty}*"));
                                let sload = self.fresh_tmp();
                                self.emitln(&format!("  {sload} = load {sty}, {sty}* {sp}"));
                                obj_val = sload;
                                ov_ty = sty;
                            }
                        }
                    }
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
                        if let Some(field_names) = self.types.types.get(&type_name.to_string())
                            .or_else(|| {
                                let suffix = format!(".{type_name}");
                                self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix) || k.ends_with(type_name))
                                    .and_then(|k|self.types.types.get(&k))
                            })
                            
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
            Expr::GenericCall(func, types, args, _) => {
                // D1: pass explicit type args so generic monomorphisation maps
                // T->concrete correctly (e.g. `add2[Float32]` -> Float32).
                self.compile_call_with_types(func, args, Some(types))
            }
            Expr::Call(func, args, _) => self.compile_call(func, args),
                        Expr::Index(container, index, _) => {
                // Index into a Vec (builtin {i8*, i64, i64}) or a Str (i8*).
                // Fixed-size arrays [N x T] (from Expr::Array literals or stack
                // arrays) are handled by the `[N x T]` GEP path below.
                let (cont_val, cont_ty) = self.compile_expr(container)?;
                let (idx_raw, idx_ty) = self.compile_expr(index)?;
                let idx = self.val_to_i64(&idx_raw, &idx_ty);
                // round-15 (&[N]T mono params): the mono param lowers to a
                // bare ELEMENT pointer -- "i8*" for [N]Int8. i8* is ambiguous
                // with Str, so check the recorded array-param element type
                // FIRST (the mono param binding registers local_array_elem for
                // Ref-Array params; the caller's array locals are cleared at
                // mono body start). Read data[idx] with the elem type -- the
                // Str path (xiom_char_at) and the array-buffer path (offset+1)
                // both misread narrow-element arrays (probe_arr8: array.first
                // on [1 as Int8, 2, 3] returned 0 instead of 1).
                if cont_ty == "i8*" && self.is_array_elem_param(container) {
                    let elem_ty = self.local.local_array_elem.get(
                        match container.as_ref() { Expr::Ident(id) => &id.name, _ => return Ok(("0".to_string(), LLVM_I64.to_string())) }
                    ).cloned().unwrap_or_else(|| "i8".to_string());
                    let elem_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {elem_ptr} = getelementptr {elem_ty}, {elem_ty}* {cont_val}, i64 {idx}"));
                    let elem = self.fresh_tmp();
                    self.emitln(&format!("  {elem} = load {elem_ty}, {elem_ty}* {elem_ptr}"));
                    // Widen immediately with the param's registered signedness --
                    // UInt8 elements must ZERO-extend (200 -> 200, not -56); the
                    // default val_to_i64 sext corrupted unsigned narrow arrays
                    // (probe_arr8b u0=-56).
                    if elem_ty != "i64" {
                        let xiom_name = match container.as_ref() {
                            Expr::Ident(id) => self.local.local_xiom_types.get(&id.name).cloned(),
                            _ => None,
                        };
                        let signed = xiom_name.as_deref()
                            .map(Self::is_signed_xiom_type)
                            .unwrap_or(true);
                        let ext = if signed { "sext" } else { "zext" };
                        let wide = self.fresh_tmp();
                        self.emitln(&format!("  {wide} = {ext} {elem_ty} {elem} to i64"));
                        return Ok((wide, LLVM_I64.to_string()));
                    }
                    return Ok((elem, elem_ty));
                }
                // Index into an Expr::Array literal buffer (i8* with length at [0]).
                // The buffer layout is: [length: i64][elem0: i64][elem1: i64]...
                // Skip past the leading length slot and read the element at index+1.
                // Also handles local variables bound from array literals (let arr = [...];
                // arr[i]) -- detected by cont_ty == i8* and the ident resolves to a
                // buffer that wasn't interned as a C string.
                let is_array_buf = cont_ty == "i8*" && (
                    matches!(container.as_ref(), Expr::Array(..))
                    || (if let Expr::Ident(ident) = container.as_ref() {
                        if std::env::var_os("XIOM_TRACE_ARRIDX").is_some() {
                            eprintln!("[arridx] {} cont_ty={} array_locals={} consts={}", ident.name, cont_ty, self.local.array_locals.contains(&ident.name), self.local.constants.get(&ident.name).map_or(false, |v| matches!(v, Expr::Array(..))));
                        }
                        self.local.array_locals.contains(&ident.name)
                            // BUG 25 #10 (crypto): CONST fixed arrays
                            // (`const _AES_SBOX: [256]UInt8 = [...]`) substitute
                            // to an Expr::Array buffer at read sites but are NOT
                            // in array_locals -- without this, `_AES_SBOX[i]`
                            // read the LENGTH slot (buf[0]) as the first element
                            // and the whole AES S-box lookup returned garbage.
                            || self.local.constants.get(&ident.name).map_or(false, |v| matches!(v, Expr::Array(..)))
                    } else { false })
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
                    // round-14 (BUG 26 #7): the runtime returns the UTF-8
                    // CODEPOINT as i64 -- no i8 zext (the old i8 returned a
                    // raw byte; multibyte chars broke len_utf8/str_chars).
                    self.emitln(&format!("  {ch} = call i64 @xiom_char_at(i8* {cont_val}, i64 {idx})"));
                    return Ok((ch, LLVM_I64.to_string()));
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
let is_vec = Self::is_llvm_struct_named(&vec_ty, "Vec")
|| Self::is_llvm_struct_named(&vec_ty, "Slice");
                if is_vec {
                    // OPT-R7: Use extractvalue directly from the SSA struct value
                    // instead of creating a fresh alloca+memset+store+GEP+load cycle.
                    let data_ptr = self.fresh_tmp();
                    let len_tmp = self.fresh_tmp();
                    let esz_val = self.fresh_tmp();
                    self.emitln(&format!("  {data_ptr} = extractvalue %struct.Vec {vec_val}, 0"));
                    self.emitln(&format!("  {len_tmp} = extractvalue %struct.Vec {vec_val}, 1"));
                    self.emitln(&format!("  {esz_val} = extractvalue %struct.Vec {vec_val}, 3"));
                    // S1: Bounds check -- trap on out-of-bounds Vec indexing
                    // when overflow checks are enabled.
                    if self.config.overflow_checks {
                        let idx_ge0 = self.fresh_tmp();
                        self.emitln(&format!("  {idx_ge0} = icmp sge i64 {idx}, 0"));
                        let idx_lt_len = self.fresh_tmp();
                        self.emitln(&format!("  {idx_lt_len} = icmp slt i64 {idx}, {len_tmp}"));
                        let in_bounds = self.fresh_tmp();
                        self.emitln(&format!("  {in_bounds} = and i1 {idx_ge0}, {idx_lt_len}"));
                        let ok_block = self.fresh_block("bounds_ok");
                        let trap_block = self.fresh_block("bounds_trap");
                        self.emitln(&format!("  br i1 {in_bounds}, label %{ok_block}, label %{trap_block}"));
                        self.emitln(&format!("\n{trap_block}:"));
                        self.emitln("  call void @llvm.trap()");
                        self.emitln("  unreachable");
                        self.emitln(&format!("\n{ok_block}:"));
                    }
                    let byte_off = self.fresh_tmp();
                    self.emitln(&format!("  {byte_off} = mul i64 {idx}, {esz_val}"));
                    let elem_ptr = self.fresh_tmp();
                    self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {data_ptr}, i64 {byte_off}"));
                    // AUDIT BUG 57 FIX: chained indexing -- the map stores the
                    // VALUE TYPE each index-expression yields (keyed by that
                    // expression); our element type is that, stripped one
                    // Vec layer.
                    let mapped_elem = self.indexed_elem_types
                        .get(&Self::expr_key(container))
                        .cloned()
                        .and_then(|vt| {
                            if vt.starts_with("Vec[") && vt.ends_with(']') {
                                Some(vt[4..vt.len() - 1].to_string())
                            } else {
                                None
                            }
                        });
                    // BUG 37/36 follow-up: Vec[Str] elements are STRING
                    // HANDLES (i8* in 8-byte slots). Load the handle and
                    // inttoptr it back to i8* -- the generic scalar path
                    // returned a bare i64 which downstream Str consumers
                    // (println, Str params) mis-coerced into a single-byte
                    // temp (smoke_serialize yaml_emit_sequence garbage).
                    if self.vec_elem_is_str(container) {
                        let signed = self.vec_elem_signed(container);
                        let elem = self.emit_elem_load(&elem_ptr, &esz_val, signed);
                        let sp = self.fresh_tmp();
                        self.emitln(&format!("  {sp} = inttoptr i64 {elem} to i8*"));
                        return Ok((sp, "i8*".to_string()));
                    }
                    // For struct elements with a known element type, load the
                    // struct directly from Vec data via memcpy, bypassing the
                    // ptrtoint/inttoptr chain of emit_elem_load+val_to_struct.
                    let resolved_elem = self.resolve_vec_elem_type(container)
                        .or_else(|| mapped_elem.clone());
                    if let Some(elem_type_name) = resolved_elem {
                        // Record for CHAINED indexes: this Index expression
                        // yields elements of type elem_type_name.
                        self.indexed_elem_types
                            .insert(Self::expr_key(expr), elem_type_name.clone());
                        // FLOAT ELEMENTS: the scalar loader loads i64 bits and
                        // callers sitofp -- raw float bits reinterpreted as an
                        // integer produce garbage (BUG 57: -3.0 -> -4.6e18).
                        // Load typed instead.
                        if matches!(elem_type_name.as_str(), "Float64" | "Float32") {
                            if std::env::var_os("XIOM_B57_DEBUG").is_some() {
                                let kk = Self::expr_key(container);
                                eprintln!("[b57f] FLOAT path key={}", &kk[..kk.len().min(70)]);
                            }
                            let llvm_f = if elem_type_name == "Float64" { "double" } else { "float" };
                            let fp = self.fresh_tmp();
                            let fv = self.fresh_tmp();
                            self.emitln(&format!("  {fp} = bitcast i8* {elem_ptr} to {llvm_f}*"));
                            self.emitln(&format!("  {fv} = load {llvm_f}, {llvm_f}* {fp}"));
                            return Ok((fv, llvm_f.to_string()));
                        }
                        // PRIMITIVE elements keep the scalar loader below --
                        // the struct-memcpy path must only fire for real
                        // struct/nested-Vec elements (alloca %struct.Int was
                        // an unsized-type compile error).
                        let is_primitive_elem = matches!(elem_type_name.as_str(),
                            "Int" | "Bool" | "Str" | "UInt8" | "Int8" | "Int16"
                            | "Int32" | "UInt16" | "UInt32" | "Char");
                        if !is_primitive_elem {
                        // BUG 23 #2 fix: NESTED Vec[Vec[T]] -- the element IS a
                        // generic %struct.Vec (32 bytes); memcpy it like any
                        // struct element so `m[i][j]` / `m[i].len()` work.
                        let struct_ty = if elem_type_name.starts_with("Vec[") {
                            "%struct.Vec".to_string()
                        } else if elem_type_name.starts_with("Map[") {
                            "%struct.Map".to_string()
                        } else if elem_type_name.starts_with("Set[") {
                            "%struct.Set".to_string()
                        } else if elem_type_name.starts_with("Option[") {
                            // R8/regex fix: concrete when registered, else the
                            // erased two-field base (both 16 bytes for scalars).
                            let inner = &elem_type_name[7..elem_type_name.len() - 1];
                            let concrete = format!("Option__{}", Self::sanitize_container_arg(inner));
                            if self.types.type_meta.contains_key(&concrete) {
                                format!("%struct.{concrete}")
                            } else {
                                "%struct.Option".to_string()
                            }
                        } else if elem_type_name.starts_with("Result[") {
                            let (_b, args) = Self::parse_generic_type_string(&elem_type_name);
                            let concrete = if args.len() == 2 {
                                format!(
                                    "Result__{}__{}",
                                    Self::sanitize_container_arg(&args[0]),
                                    Self::sanitize_container_arg(&args[1])
                                )
                            } else {
                                String::new()
                            };
                            if !concrete.is_empty() && self.types.type_meta.contains_key(&concrete) {
                                format!("%struct.{concrete}")
                            } else {
                                "%struct.Result".to_string()
                            }
                        } else {
                            format!("%struct.{elem_type_name}")
                        };
                        let struct_alloca = self.fresh_tmp();
                        self.emitln(&format!("  {struct_alloca} = alloca {struct_ty}"));
                        let dst_i8 = self.fresh_tmp();
                        self.emitln(&format!("  {dst_i8} = bitcast {struct_ty}* {struct_alloca} to i8*"));
                        self.emitln(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* {dst_i8}, i8* {elem_ptr}, i64 {esz_val}, i1 false)"));
                        let loaded = self.fresh_tmp();
                        self.emitln(&format!("  {loaded} = load {struct_ty}, {struct_ty}* {struct_alloca}"));
                        return Ok((loaded, struct_ty));
                        }
                    }
                    // Fallback: use emit_elem_load for unknown element types.
                    if std::env::var_os("XIOM_B57_DEBUG").is_some() {
                        let kk = Self::expr_key(container);
                        eprintln!("[b57miss] FALLBACK key={} resolve={:?}",
                            &kk[..kk.len().min(70)], self.resolve_vec_elem_type(container));
                    }
                    let signed = self.vec_elem_signed(container);
                    let elem = self.emit_elem_load(&elem_ptr, &esz_val, signed);
                    // 5c.29: float elements round-trip as raw bits -- reinterpret
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
                // BUG 53 (2026-08-18): EXCLUDE pointer-typed forms -- a
                // `&[N]T` param slot is `[5 x i64]*` (starts with '[' but is a
                // POINTER); the old condition GEP'd the SLOT as the array
                // (`getelementptr [5 x i64]*, [5 x i64]** %slot, i64 0, i64 0`
                // -- clang "invalid getelementptr indices"). Pointer-typed
                // arrays fall through to the array-ref branch below.
                if cont_ty.starts_with('[') && cont_ty.contains(" x ") && !cont_ty.ends_with('*') {
                    let (arr_ptr, arr_ptr_ty) = if let Expr::Ident(id) = &**container {
                        if let Some((slot, _slot_ty)) = self.lookup_local(&id.name) {
                            // Use the existing alloca pointer directly -- avoids
                            // creating a fresh alloca on every loop iteration.
                            (slot.clone(), format!("{cont_ty}*"))
                        } else if let Some((symbol, _gty)) = self.local.module_globals.get(&id.name).cloned() {
                            // Stdlib finding 3b-2 #8 (module-level arrays): GEP the
                            // REAL global directly. The old fallback spilled the
                            // loaded [N x T] VALUE into a fresh stack alloca and
                            // indexed the copy -- reads saw a stale snapshot and
                            // writes were silently lost (module crc tables read
                            // back all zeros; undersized-backing AV family).
                            (format!("@{symbol}"), format!("{cont_ty}*"))
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
                // NOTE: `&[N]T` params receive the array's DATA pointer (Vec data is
                // headerless -- array literals compile to %struct.Vec with pure
                // elements). The old "+1" assumed a raw buffer with a length header
                // at [0], which misindexed every element by one and read one past
                // the end (array smoke: contains() returned false / crashed).
                if cont_ty.ends_with('*') && cont_ty != "i8*" {
                    let elem_ty = cont_ty.trim_end_matches('*');
                    // BUG 53 (2026-08-18): `[N x T]*` pointers (non-generic
                    // `&[N]T` params lower to the typed array pointer) need a
                    // TWO-INDEX GEP (`i64 0, i64 idx` -- one index would scale
                    // by the whole array) and an ELEMENT-typed load; the old
                    // single-index GEP returned the array pointer and loaded
                    // the whole array (invalid/garbage).
                    let elem_ptr = self.fresh_tmp();
                    if elem_ty.starts_with('[') && elem_ty.contains(" x ") {
                        let inner_ty = Self::extract_array_elem_ty(&elem_ty);
                        self.emitln(&format!("  {elem_ptr} = getelementptr {elem_ty}, {cont_ty} {cont_val}, i64 0, i64 {idx}"));
                        let elem = self.fresh_tmp();
                        self.emitln(&format!("  {elem} = load {inner_ty}, {inner_ty}* {elem_ptr}"));
                        let result = self.val_to_i64(&elem, &inner_ty);
                        return Ok((result, LLVM_I64.to_string()));
                    }
                    self.emitln(&format!("  {elem_ptr} = getelementptr {elem_ty}, {cont_ty} {cont_val}, i64 {idx}"));
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
                // `&*p` / `&mut *p` is a REBORROW: it must yield p's VALUE
                // (the pointee address), not load through it. Box.get's
                // `return &*ptr` returned the boxed 42 instead of the box
                // address -> smoke_core_box dereferenced 42 (AV at 0x2a).
                if let Expr::Unary(UnaryOp::Deref, ref_inner, _) = inner.as_ref() {
                    return self.compile_expr(ref_inner);
                }
                if let Expr::Paren(p, _) = inner.as_ref() {
                    if let Expr::Unary(UnaryOp::Deref, ref_inner, _) = p.as_ref() {
                        return self.compile_expr(ref_inner);
                    }
                }
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
                            // 5c.30: `&local.field` -- emit a REAL GEP into the
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
                                        self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix))
                                            .and_then(|k|self.types.types.get(&k))
                                    })
                                    
                                {
                                    if let Some(fi) = field_names.iter().position(|f| f == &field_name_expr.name) {
                                        let field_llvm_ty = self.field_llvm_type(&type_name, fi);
                                        // Struct-typed fields: return GEP pointer.
                                        if field_llvm_ty.starts_with("%struct.") && !field_llvm_ty.ends_with('*') {
                                            let gep = self.fresh_tmp();
                                            self.emitln(&format!("  {gep} = getelementptr {base_ty}, {base_ty}* {base_ptr}, i32 0, i32 {fi}"));
                                            return Ok((gep, format!("{field_llvm_ty}*")));
                                        }
                                        // 5c.31: Scalar-typed fields (i64, i8, etc.)
                                        // must also return the field ADDRESS, not the
                                        // value. Emit GEP + ptrtoint to i64 so the
                                        // caller can inttoptr + load through the
                                        // pointer (matching &ident semantics for
                                        // scalars). Fixes ACCESS_VIOLATION on &d.val.
                                        let gep = self.fresh_tmp();
                                        self.emitln(&format!("  {gep} = getelementptr {base_ty}, {base_ty}* {base_ptr}, i32 0, i32 {fi}"));
                                        let ptr_val = self.fresh_tmp();
                                        self.emitln(&format!("  {ptr_val} = ptrtoint {field_llvm_ty}* {gep} to i64"));
                                        return Ok((ptr_val, LLVM_I64.to_string()));
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
                    // 5c-E: Empty array (n == 0) -- construct a zeroed Vec without malloc.
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
                // 5c.31: `&v[i]` on a Vec -- return the ADDRESS of element i
                // within the Vec's data buffer, not the element VALUE.
                // Computes data_ptr + i * elem_size and ptrtoint to i64.
                if let Expr::Index(container, index, _) = inner.as_ref() {
                    let (cont_val, cont_ty) = self.compile_expr(container)?;
                    let (vec_val, vec_ty) = self.resolve_vec_receiver(container, &cont_val, &cont_ty);
                    let is_vec = vec_ty == "%struct.Vec" || vec_ty.ends_with(".Vec")
                        || vec_ty.contains("struct.Vec")
                        || vec_ty == "%struct.Slice" || vec_ty.contains("struct.Slice");
                    if is_vec {
                        let (idx_raw, idx_ty) = self.compile_expr(index)?;
                        let idx = self.val_to_i64(&idx_raw, &idx_ty);
                        let vslot = self.fresh_tmp();
                        self.emitln(&format!("  {vslot} = alloca %struct.Vec"));
                        self.emitln(&format!("  {vslot}_i8 = bitcast %struct.Vec* {vslot} to i8*"));
                        self.emitln(&format!("  call void @llvm.memset.p0i8.i64(i8* {vslot}_i8, i8 0, i64 32, i1 false)"));
                        self.emit_vec_store_fields(&vec_val, &vslot);
                        let esz_gep = self.fresh_tmp();
                        let esz_val = self.fresh_tmp();
                        self.emitln(&format!("  {esz_gep} = getelementptr %struct.Vec, %struct.Vec* {vslot}, i32 0, i32 3"));
                        self.emitln(&format!("  {esz_val} = load i64, i64* {esz_gep}"));
                        let data_gep = self.fresh_tmp();
                        let data_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {data_gep} = getelementptr %struct.Vec, %struct.Vec* {vslot}, i32 0, i32 0"));
                        self.emitln(&format!("  {data_ptr} = load i8*, i8** {data_gep}"));
                        let byte_off = self.fresh_tmp();
                        self.emitln(&format!("  {byte_off} = mul i64 {idx}, {esz_val}"));
                        let elem_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {elem_ptr} = getelementptr i8, i8* {data_ptr}, i64 {byte_off}"));
                        let ptr_val = self.fresh_tmp();
                        self.emitln(&format!("  {ptr_val} = ptrtoint i8* {elem_ptr} to i64"));
                        return Ok((ptr_val, LLVM_I64.to_string()));
                    }
                }
                // BUG 25 #10 (crypto AES-NI follow-up): `&arr[i]` on a FIXED
                // ARRAY local (`[N]T` -- slot type `[16 x i8]`) must return the
                // element ADDRESS (GEP), not the loaded element value. The
                // generic fallback compiled the Index as a VALUE, which the
                // `as *T` cast then inttoptr'd (the byte value 0 became the
                // NULL ciphertext pointer -> 0xC0000005 in crypto's AES-NI FFI
                // call). Mirrors the Vec element-address path above.
                if let Expr::Index(container, index, _) = inner.as_ref() {
                    if let Expr::Ident(id) = container.as_ref() {
                        if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                            if slot_ty.starts_with('[') && !slot_ty.ends_with('*') {
                                let (idx_raw, idx_ty) = self.compile_expr(index)?;
                                let idx = self.val_to_i64(&idx_raw, &idx_ty);
                                let gep = self.fresh_tmp();
                                self.emitln(&format!(
                                    "  {gep} = getelementptr {slot_ty}, {slot_ty}* {slot}, i64 0, i64 {idx}"
                                ));
                                // GEP result type is the ELEMENT pointer:
                                // `[16 x i8]` -> `i8*`.
                                let elem_ty = slot_ty
                                    .rsplit_once(" x ")
                                    .map(|(_, t)| t.trim_end_matches(']'))
                                    .unwrap_or("i8")
                                    .to_string();
                                let ptr_val = self.fresh_tmp();
                                self.emitln(&format!("  {ptr_val} = ptrtoint {elem_ty}* {gep} to i64"));
                                return Ok((ptr_val, LLVM_I64.to_string()));
                            }
                        }
                    }
                }
                // CRT-layout / sort_by fix (2026-09-10): `&arr[i]` where arr is
                // a POINTER-typed fixed-array local (`&mut [N]T` param -- its
                // slot holds the data base address as `i64*`, not a by-value
                // `[N x T]` alloca). The by-value branch above only fires for
                // `[N x T]` slots; for the param shape the fallback loaded the
                // element VALUE and passed it as the address -- the comparator
                // thunk derefed small ints -> 0xC0000005 in smoke_array_sort_by
                // (same family as the closure-env-size CRT-layout bug).
                if let Expr::Index(container, index, _) = inner.as_ref() {
                    if let Expr::Ident(id) = container.as_ref() {
                        if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                            if self.is_array_elem_param(container)
                                && (slot_ty.ends_with('*') || slot_ty == "i64")
                            {
                                if let Some(elem_llvm) = self.local.local_array_elem.get(&id.name).cloned() {
                                    let (idx_raw, idx_ty) = self.compile_expr(index)?;
                                    let idx = self.val_to_i64(&idx_raw, &idx_ty);
                                    let base = if slot_ty.ends_with('*') {
                                        let b = self.fresh_tmp();
                                        self.emitln(&format!("  {b} = load {slot_ty}, {slot_ty}* {slot}"));
                                        b
                                    } else {
                                        let b = self.fresh_tmp();
                                        self.emitln(&format!("  {b} = inttoptr i64 {slot} to i8*"));
                                        b
                                    };
                                    let gep = self.fresh_tmp();
                                    self.emitln(&format!("  {gep} = getelementptr {elem_llvm}, {slot_ty} {base}, i64 {idx}"));
                                    let ptr_val = self.fresh_tmp();
                                    self.emitln(&format!("  {ptr_val} = ptrtoint {elem_llvm}* {gep} to i64"));
                                    return Ok((ptr_val, LLVM_I64.to_string()));
                                }
                            }
                        }
                    }
                }
                // &x: return a pointer to x's storage.
                // For struct-typed idents, return the alloca pointer directly
                // (this-based methods receive a proper pointer receiver).
                // For scalar/handle idents, ptrtoint the alloca to i64 so
                // *r can inttoptr back and load through the pointer.
                // This fixes ACCESS_VIOLATION on &Int -> *Int deref patterns.
                if let Expr::Ident(id) = inner.as_ref() {
                    if let Some((slot, slot_ty)) = self.lookup_local(&id.name).cloned() {
                        // &array_local -- the local is a Vec (array literal). A
                        // `&[N]T` parameter wants the DATA pointer (i64*), not
                        // the Vec struct or its alloca. Emit field-0 (data ptr).
                        if self.local.array_locals.contains(&id.name) && slot_ty == "%struct.Vec" {
                            let gep = self.fresh_tmp();
                            self.emitln(&format!("  {gep} = getelementptr %struct.Vec, %struct.Vec* {slot}, i32 0, i32 0"));
                            let data_ptr = self.fresh_tmp();
                            self.emitln(&format!("  {data_ptr} = load i8*, i8** {gep}"));
                            let ptr_val = self.fresh_tmp();
                            self.emitln(&format!("  {ptr_val} = ptrtoint i8* {data_ptr} to i64"));
                            return Ok((ptr_val, "i64".to_string()));
                        }
                        if slot_ty.starts_with("%struct.") {
                            // BUG 24 fix: `&x` where x is ALREADY a reference
                            // (pointer-typed local -- a `&Vec[Float64]`/`&BigFloat`
                            // param) is a DOUBLE-ADDRESS: the callee would read
                            // the pointer SLOT as the struct (garbage -> wrong
                            // values / AVs, per-program-shape). Reject it so the
                            // typo is a compile error, not silent corruption.
                            if slot_ty.ends_with('*') {
                                return Err(format!(
                                    "cannot take a reference to '{}': it is already a reference (remove the leading '&')",
                                    id.name
                                ));
                            }
                            return Ok((slot, format!("{slot_ty}*")));
                        }
                        // Only ptrtoint when the ident refers to a non-self local.
                        // Method receivers and `this` use the struct path above.
                        if id.name != "self" && id.name != "this" {
                            let ptr_val = self.fresh_tmp();
                            self.emitln(&format!("  {ptr_val} = ptrtoint {slot_ty}* {slot} to i64"));
                            return Ok((ptr_val, "i64".to_string()));
                        }
                    }
                }
                // &literal (e.g. &30, &true): materialise a temp slot holding the
                // value and return its ADDRESS. The old fallback returned the raw
                // value which the caller inttoptr'd -- turning the VALUE into its
                // own address (inttoptr i64 30 to i64*), so the callee loaded from
                // address 0x1E instead of comparing with 30.
                match inner.as_ref() {
                    Expr::Int(_, _) | Expr::Float(_, _) | Expr::Bool(_, _) | Expr::Char(_, _) => {
                        let (v, v_ty) = self.compile_expr(inner)?;
                        let slot = self.fresh_tmp();
                        self.emitln(&format!("  {slot} = alloca {v_ty}"));
                        self.emitln(&format!("  store {v_ty} {v}, {v_ty}* {slot}"));
                        let ptr_val = self.fresh_tmp();
                        self.emitln(&format!("  {ptr_val} = ptrtoint {v_ty}* {slot} to i64"));
                        return Ok((ptr_val, "i64".to_string()));
                    }
                    _ => {}
                }
                self.compile_expr(inner)
            }
            Expr::Some(inner, _) => {
                self.types.used_builtins.insert("Option".to_string());
                // If the inner expression is an array literal, convert it to a Vec
                // struct so the Option payload is a proper Vec, not a raw buffer.
                // Also resolve bare struct literal `{ field: value; }` from
                // field-name-based type lookup.
                let (val, inner_ty) = if let Expr::Array(elems, _) = inner.as_ref() {
                    self.compile_array_as_vec(elems, "Int")?
                } else if let Expr::Struct(name, fields, _, _) = inner.as_ref() {
                    if name.name == "_" {
                        resolve_bare_struct(self, fields)?
                    } else {
                        self.compile_expr(inner)?
                    }
                } else {
                    self.compile_expr(inner)?
                };
                // Use the function's return type for concrete monomorphised
                // Option types (Option__Point). Only applies when the return
                // type actually IS an Option variant; for non-Option returns
                // (e.g. a struct wrapping Option fields), use the default.
                // 5c.35: Check that current_return_type is an Option-like struct.
                // BUG 55: inside an unsafe-block fn, current_return_type is the
                // block's i64 ABI -- use the ENCLOSING fn's declared return so
                // the ctor builds the CONCRETE container (Option__Rc), not the
                // generic %struct.Option (payload-slot mismatch -> corruption).
                let ctor_ret = self.fctx.enclosing_return_type.clone()
                    .unwrap_or_else(|| self.fctx.current_return_type.clone());
                let ret_is_option = ctor_ret.contains("Option");
                let mut opt_ty = if ret_is_option && ctor_ret.starts_with("%struct.") {
                    ctor_ret
                } else {
                    "%struct.Option".to_string()
                };
                let mut struct_name = opt_ty.trim_start_matches("%struct.").to_string();
                let mut field_type_1 = self.types.type_meta.get(&struct_name)
                    .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let mut field_llvm_1 = self.field_llvm_ty(&field_type_1);
                // R8/regex fix (2026-09-11): a STRUCT payload must use the
                // concrete Option__T -- the surrounding container (Vec element
                // size, match binding, return signature under M18) is concrete.
                // `groups.push(Some(match_obj))` inside a fn returning
                // Option[Captures] built an Option__Captures holding a Match
                // (clang: struct.Match vs struct.Captures); the opaque return
                // case store a boxed handle into a 24-byte concrete slot.
                if !inner_ty.is_empty() && field_llvm_1 != inner_ty {
                    if let Some(base) = inner_ty.strip_prefix("%struct.") {
                        let leaf = base.rsplit('.').next().unwrap_or(base);
                        let concrete = format!("Option__{}", Self::sanitize_container_arg(leaf));
                        // Only adopt a concrete Option__T that is ALREADY
                        // registered (a Vec[Option[T]] ctor or a concrete
                        // return signature registered it). Creating it here
                        // would mismatch consumers whose signature stayed
                        // opaque (net_address Option[Tuple...]).
                        if self.types.type_meta.contains_key(&concrete) {
                            opt_ty = format!("%struct.{concrete}");
                            struct_name = opt_ty.trim_start_matches("%struct.").to_string();
                            field_type_1 = self.types.type_meta.get(&struct_name)
                                .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                                .unwrap_or_else(|| "Int".to_string());
                            field_llvm_1 = self.field_llvm_ty(&field_type_1);
                        }
                    }
                }
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
                    // Struct-typed value field -- store the struct directly
                    let store_val = self.coerce_value(&val, &inner_ty, &field_llvm_1);
                    self.emitln(&format!("  store {field_llvm_1} {store_val}, {field_llvm_1}* {gep1}"));
                }
                let loaded = self.fresh_tmp();
                self.emitln(&format!("  {loaded} = load {opt_ty}, {opt_ty}* {alloca}"));
                Ok((loaded, opt_ty.to_string()))
            }
            Expr::None(_) => {
                self.types.used_builtins.insert("Option".to_string());
                // 5c.35: Check that current_return_type is an Option-like struct.
                // Without this guard, a function returning a non-Option struct
                // (e.g. Node { val:Int, next:Option[Int] }) would compile None as
                // %struct.Node instead of %struct.Option, producing type-mismatched IR.
                // BUG 55: unsafe-block fns consult the ENCLOSING return type.
                let ctor_ret = self.fctx.enclosing_return_type.clone()
                    .unwrap_or_else(|| self.fctx.current_return_type.clone());
                let ret_is_option = ctor_ret.contains("Option");
                let opt_ty = if ret_is_option && ctor_ret.starts_with("%struct.") {
                    ctor_ret
                } else {
                    "%struct.Option".to_string()
                };
                let struct_name = opt_ty.trim_start_matches("%struct.");
                let field_type_1 = self.types.type_meta.get(&struct_name.to_string())
                    .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_1 = self.field_llvm_ty(&field_type_1);
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
                // 5c.35: Check that current_return_type is a Result-like struct.
                // BUG 55: unsafe-block fns consult the ENCLOSING return type.
                let ctor_ret = self.fctx.enclosing_return_type.clone()
                    .unwrap_or_else(|| self.fctx.current_return_type.clone());
                let ret_is_result = ctor_ret.contains("Result");
                let result_ty = if ret_is_result && ctor_ret.starts_with("%struct.") {
                    ctor_ret
                } else {
                    "%struct.Result".to_string()
                };
                let struct_name = result_ty.trim_start_matches("%struct.");
                let field_type_1 = self.types.type_meta.get(&struct_name.to_string())
                    .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let (val, inner_ty) = if let Expr::Struct(ref name, ref fields, _, _) = **inner {
                    if name.name == "_" {
                        resolve_bare_struct(self, fields)?
                    } else {
                        self.compile_expr(inner)?
                    }
                } else {
                    self.compile_expr(inner)?
                };
                let field_llvm_1 = self.field_llvm_ty(&field_type_1);
                let field_type_2 = self.types.type_meta.get(&struct_name.to_string())
                    .and_then(|m| m.fields.get(2).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_2 = self.field_llvm_ty(&field_type_2);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {result_ty}"));
                // BUG 30: zero-init unused slots (Err payload for Ok, Ok payload
                // for Err) -- structural eq/Is reads must not see LLVM poison.
                self.emitln(&format!("  store {result_ty} zeroinitializer, {result_ty}* {alloca}"));
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
                let (val, inner_ty) = if let Expr::Struct(ref name, ref fields, _, _) = **inner {
                    if name.name == "_" {
                        resolve_bare_struct(self, fields)?
                    } else {
                        self.compile_expr(inner)?
                    }
                } else {
                    self.compile_expr(inner)?
                };
                // 5c.35: Check that current_return_type is a Result-like struct.
                // BUG 55: unsafe-block fns consult the ENCLOSING return type.
                let ctor_ret = self.fctx.enclosing_return_type.clone()
                    .unwrap_or_else(|| self.fctx.current_return_type.clone());
                let ret_is_result = ctor_ret.contains("Result");
                let result_ty = if ret_is_result && ctor_ret.starts_with("%struct.") {
                    ctor_ret
                } else {
                    "%struct.Result".to_string()
                };
                let struct_name = result_ty.trim_start_matches("%struct.");
                let field_type_1 = self.types.type_meta.get(&struct_name.to_string())
                    .and_then(|m| m.fields.get(1).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_1 = self.field_llvm_ty(&field_type_1);
                let field_type_2 = self.types.type_meta.get(&struct_name.to_string())
                    .and_then(|m| m.fields.get(2).map(|(_, t)| t.clone()))
                    .unwrap_or_else(|| "Int".to_string());
                let field_llvm_2 = self.field_llvm_ty(&field_type_2);
                let alloca = self.fresh_tmp();
                self.emitln(&format!("  {alloca} = alloca {result_ty}"));
                // BUG 30: zero-init unused slots (Ok payload for Err).
                self.emitln(&format!("  store {result_ty} zeroinitializer, {result_ty}* {alloca}"));
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
                if std::env::var_os("XIOM_TRACE_RETXIOM").is_some() && (name.name == "Result" || name.name.contains("Result")) {
                    eprintln!("[struct] name={} ret_ty={}", name.name, self.fctx.current_return_type);
                }
                // If `name` is an enum variant (e.g., `Circle` or `Shape.Circle`),
                // resolve to parent enum type. Qualified variant names need splitting.
                let (base_name, leaf_variant) = match name.name.rfind('.') {
                    Some(dot) => {
                        let enum_name = &name.name[..dot];
                        let variant = &name.name[dot + 1..];
                        (Some(enum_name.to_string()), variant.to_string())
                    }
                    None => (None, name.name.clone()),
                };
                let parent_enum = if let Some(ref ek) = base_name {
                    // Qualified: look up the enum by name
                    if self.types.enum_variants.contains_key(ek) {
                        Some(ek.clone())
                    } else {
                        // Try module-qualified
                        self.types.enum_variants.keys().into_iter()
    .find(|k| k.ends_with(&format!(".{ek}")))
                    }
                } else {
                    // Bare variant: search all enums -- BUT only when the name
                    // is NOT a known struct type. BUG 31 (bench_math native):
                    // `Node{ value: ...; children: ... }` (the bench_memory
                    // STRUCT) was hijacked by `enum BST[T] { Node(...) }`'s
                    // Node VARIANT -- the literal compiled as %struct.BST with
                    // the enum's payload slots (store %struct.BST %vecval at
                    // field 2 -> invalid IR).
                    if !self.types.types.contains_key(&name.name)
                        && !self.types.type_meta.contains_key(&name.name)
                        && !self.types.generic_type_names.iter().any(|k| k == &name.name || k.ends_with(&format!(".{}", name.name)))
                    {
                        self.types.enum_variants.entries().into_iter()
                            .find(|(_, vars)| vars.iter().any(|(v, _)| v == &leaf_variant))
                            .map(|(ek, _)| ek.clone())
                    } else {
                        None
                    }
                };
                // Verify the variant exists in the resolved enum
                let parent_enum = parent_enum.and_then(|ek| {
                    if self.types.enum_variants.get(&ek)
                        .map_or(false, |vars| vars.iter().any(|(v, _)| v == &leaf_variant))
                    {
                        Some(ek)
                    } else {
                        None
                    }
                });
                let struct_ty = if let Some(ref ek) = parent_enum {
                    self.llvm_type_for(ek)?
                } else if name.name == "_" {
                    // Bare struct literal `{ field: value; }` -- resolve from
                    // return type context, or via field-name-based type lookup.
                    if !self.fctx.current_return_type.is_empty()
                        && self.fctx.current_return_type.starts_with('%')
                    {
                        self.fctx.current_return_type.clone()
                    } else {
                        // 5c.31: Try resolve_bare_struct to match field names
                        // against registered types (e.g. `{ v: 42 }` -> L1,
                        // `{ l6: { ... } }` -> L7). This fixes deep struct
                        // literal chains where the inner struct is a bare `_`
                        // literal whose type cannot be inferred from context.
                        match resolve_bare_struct(self, fields) {
                            Ok((val, ty)) if ty.starts_with('%') => {
                                return Ok((val, ty));
                            }
                            _ => {
                                "i64".to_string()
                            }
                        }
                    }
                } else {
                    let mut ty = self.resolve_literal_struct_ty(&name.name);
                    // BUG 31: `Result[Unit, FmtError] { is_ok: ...; value: ();
                    // error: ... }` -- the parser drops the generic args, so the
                    // literal resolves to the GENERIC template (%struct.Result,
                    // void Unit-field stores, ABI mismatch vs the fn signature).
                    // When the fn's return type is a CONCRETE instantiation of
                    // the same base, adopt it.
                    if (ty == "%struct.Result" || ty == "%struct.Option")
                        && self.fctx.current_return_type.starts_with(&ty[..ty.len() - 1])
                        && self.fctx.current_return_type != ty
                    {
                        ty = self.fctx.current_return_type.clone();
                    }
                    ty
                };
                if !struct_ty.starts_with('%') {
                    // Scalar type -- struct literal was resolved to a non-struct
                    // type (e.g. `_` -> i64). Return the last field value.
                    let mut last = ("0".to_string(), "i64".to_string());
                    for (_, val) in fields.iter() {
                        last = self.compile_expr(val)?;
                    }
                    return Ok(last);
                }
                let alloca = self.fresh_tmp();
                // D1: align 16 only for structs with i128/fp128 fields.
                self.emitln(&format!("  {alloca} = alloca {struct_ty}{}", self.alloca_align(&struct_ty)));
                if let Some(ref enum_key) = parent_enum {
                    // BUG 30: zero-initialize the WHOLE enum literal before
                    // writing the discriminant/payloads. Unused payload slots
                    // of other variants were left uninitialized -> LLVM poison
                    // -> structural `==` (no derived eq) compared poison fields
                    // -> `br i1 poison` is UB and clang -O2 deterministically
                    // miscompiled it into an access violation (m35_z10/z29).
                    // Zeroed slots make the all-slots compare well-defined:
                    // same-variant payloads differ, other slots are 0 == 0.
                    self.emitln(&format!("  store {struct_ty} zeroinitializer, {struct_ty}* {alloca}"));
                    // Set discriminant (field 0) to the variant index
                    let var_idx = self.types.enum_variants.get(enum_key)
                        .and_then(|vars| vars.iter().position(|(v, _)| v == &leaf_variant))
                        .unwrap_or(0);
                    let disc_gep = self.fresh_tmp();
                    self.emitln(&format!("  {disc_gep} = getelementptr {struct_ty}, {struct_ty}* {alloca}, i32 0, i32 0"));
                    self.emitln(&format!("  store i64 {var_idx}, i64* {disc_gep}"));
                    // Map variant fields to their parent enum offsets (after discriminant)
                    // The parent enum stores field names uniquely across all variants,
                    // so we need to look up the actual field index in the parent's field list.
                    let parent_field_names = self.types.types.get(enum_key).unwrap_or_default();
                    let variant_fields = self.types.enum_variants.get(enum_key)
                        .and_then(|vars| vars.into_iter().find(|(v, _)| v == &leaf_variant))
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
                    // 5c.39: Use resolved struct type name for field lookups,
                    // not the bare `_` name from the struct literal.
                    let resolved_name = struct_ty.trim_start_matches("%struct.");
                    for (i, (_, val)) in fields.iter().enumerate() {
                        // 5c.39: Empty array `[]` in Vec-typed field -> compile as
                        // proper empty Vec, not raw i8* array buffer.
                        let (mut field_val, mut field_val_ty) = if let Expr::Array(elems, _) = val {
                            let fllvm = self.field_llvm_type(resolved_name, i);
                            if elems.is_empty() && (fllvm == "%struct.Vec" || fllvm.ends_with(".Vec")) {
                                self.compile_empty_vec_for_field(resolved_name, i)?
                            } else {
                                self.compile_expr(val)?
                            }
                        } else {
                            self.compile_expr(val)?
                        };
                        let mut field_llvm_ty = self.field_llvm_type(resolved_name, i);
                        // 5c.29: Generic container fields (Vec[Int], ...) are i64
                        // HANDLES (5c.28h). A by-value container header must be
                        // BOXED on the heap and the pointer stored as the handle;
                        // storing the 32-byte %struct.Vec into the 8-byte i64 slot
                        // corrupted the stack and broke every handle reader.
                        let is_generic_container_field = self
                            .field_xiom_type(resolved_name, i)
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
                            // BUG 31: NEVER adopt "void" -- a Unit value (`value: ()`)
                            // must keep the i64 slot (`store void 0, void*` was
                            // invalid IR); the struct DEF degrades Unit fields to i64.
                            if field_val_ty != "i64" && field_val_ty != "void" {
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
                // ACTUAL element LLVM type (Float64 -> double, Int -> i64, struct ->
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
                // M20-A1: Full closure implementation -- capturing + non-capturing.
                // round-13: closure ids come from the GLOBAL closure_counter
                // (never reset per fn) -- the defs are emitted at module level
                // and tmp_counter resets per function, so identical capture
                // shapes in different mono fns collided on the same
                // %struct.__closure_env_N (clang redefinition).
                let closure_id = self.closure_counter;
                self.closure_counter += 1;
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
                    // BUG 22 #6: isolate the hoisted-alloca list so the closure
                    // splices only its own loop-body allocas.
                    let saved_hoisted = std::mem::take(&mut self.local.hoisted_allocas);
                    self.tmp_counter = closure_id * 1000;
                    self.block_counter = closure_id * 1000;
                    self.push_scope();
                    
                    let param_str: Vec<String> = params.iter().enumerate()
                        .map(|(i, p)| format!("%{}_{}", p.name, i))
                        .collect();
                    let header = format!("define i64 @{fn_name}(i64 %__env, {}) {{\nentry:\n",
                        param_str.iter().enumerate()
                            .map(|(_i, s)| format!("i64 {}", s))
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
                // BUG 22 #6: splice loop-body-hoisted allocas into the block
                // fn's entry block (before the buffer is taken).
                self.finish_hoisted_allocas();
                self.pop_scope();
                    
                    let closure_ir = std::mem::take(&mut self.output);
                    self.local.deferred_closure_defs.push(closure_ir);
                    self.output = saved_output;
                    self.tmp_counter = saved_tmp;
                    self.block_counter = saved_block;
                    self.local.hoisted_allocas = saved_hoisted;
                    
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
                
                // Build the env struct type -- defer to before function body
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
                
                // Bind captured variables DIRECTLY to their env-struct field
                // GEPs (round-13: the local-copy approach lost mutations --
                // writes to the copy never reached the env, so captured
                // iterator state never advanced between calls).
                for (field_idx, (cap_name, cap_llvm_ty)) in captures.iter().enumerate() {
                    let gep = self.fresh_tmp();
                    let llvm_field_idx = field_idx + 1; // field 0 is fn_ptr
                    self.emitln(&format!("  {gep} = getelementptr %struct.{env_name}, %struct.{env_name}* %__env_ptr, i32 0, i32 {llvm_field_idx}"));
                    self.add_local(cap_name, gep, cap_llvm_ty);
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
                // CRT-layout fix (2026-09-09): the env size was 8 bytes PER
                // CAPTURE -- struct captures (%struct.Range = 16B, %struct.Vec
                // = 32B, ...) overflowed the malloc'd tail by (struct_size-8),
                // corrupting the heap: the whole clang -O2/MSVC-CRT layout
                // family (startup AVs in smoke_iter_collect / smoke_array_sort_by,
                // m34_y15/y20; "flips with unrelated stdlib code" = adjacent
                // allocation shifts). Size from the REAL LLVM field types.
                let env_size: usize = 8 + captures
                    .iter()
                    .map(|(_name, ty)| Self::llvm_type_byte_size(ty, &self.types.type_meta))
                    .sum::<usize>();
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
                // round-13: global closure_counter (see PipeClosure above).
                let closure_id = self.closure_counter;
                self.closure_counter += 1;
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
                // round-14 (aggregate closure params): params keep their REAL
                // LLVM types when they cannot marshal through an i64 register --
                // struct/tuple values pass BY VALUE (%struct.Point %p_0 -- the
                // call site splits a 16-byte struct across two registers, the
                // i64 thunk param read only the first: p.x garbage) and
                // struct-pointee refs are real %struct.X* pointers. Scalars and
                // scalar refs keep the uniform i64 convention.
                let param_llvm_types: Vec<String> = params.iter().map(|p| {
                    match &p.ty {
                        Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) => {
                            let inner_llvm = self.llvm_type_for(&Self::type_from_ast(inner)).unwrap_or_else(|_| "i64".to_string());
                            if inner_llvm.starts_with("%struct.") {
                                format!("{inner_llvm}*")
                            } else {
                                "i64".to_string()
                            }
                        }
                        _ => {
                            let llvm = self.llvm_type_for(&Self::type_from_ast(&p.ty)).unwrap_or_else(|_| "i64".to_string());
                            // round-15: float/double params keep their REAL
                            // types -- the M20-A1 call site passes them in XMM
                            // registers (the old i64 declaration read RDX
                            // garbage: fn(Float64) -> Float64 closures returned
                            // 0/wrong values).
                            if llvm.starts_with("%struct.") || llvm == "float" || llvm == "double" || llvm == "fp128" { llvm } else { "i64".to_string() }
                        }
                    }
                }).collect();
                let all_params = std::iter::once("i64 %__env".to_string())
                    .chain(params.iter().enumerate()
                        .map(|(i, p)| format!("{} %{}_{}", param_llvm_types[i], p.name.name, i)))
                    .collect::<Vec<_>>().join(", ");
                self.output.push_str(&format!("define {ret_llvm} @{fn_name}({all_params}) {{\nentry:\n"));
                
                // Load captured variables from env if any.
                // round-13 (iter adapters): bind each capture DIRECTLY to its
                // env-struct field GEP (no local copy alloca). The old code
                // copied the capture into a local at entry -- mutations inside
                // the closure body (r.next()'s implicit-self write, whole-value
                // reassignment) only touched the copy, so the env's copy never
                // advanced (Range.count/fold/max/min hung forever). Reads load
                // through the field; writes store through it -- persistent
                // across closure invocations.
                if !captures.is_empty() {
                    self.emitln(&format!("  %__env_ptr = inttoptr i64 %__env to %struct.{env_name}*"));
                    for (field_idx, (cap_name, cap_llvm_ty)) in captures.iter().enumerate() {
                        let gep = self.fresh_tmp();
                        let llvm_idx = field_idx + 1;
                        self.emitln(&format!("  {gep} = getelementptr %struct.{env_name}, %struct.{env_name}* %__env_ptr, i32 0, i32 {llvm_idx}"));
                        self.add_local(cap_name, gep, cap_llvm_ty);
                    }
                }
                
                // Store params as locals
                for (i, p) in params.iter().enumerate() {
                    let preg = format!("%{}_{}", p.name.name, i);
                    let a = format!("%{}_{}_alloca", p.name.name, i);
                    let p_llvm = param_llvm_types.get(i).cloned().unwrap_or_else(|| "i64".to_string());
                    self.emitln(&format!("  {a} = alloca {p_llvm}"));
                    self.emitln(&format!("  store {p_llvm} {preg}, {p_llvm}* {a}"));
                    self.add_local(&p.name.name, a, &p_llvm);
                    // round-12 (rm1/cb2): closure params arrive as i64 (the
                    // uniform thunk ABI) but their DECLARED XIOM types must be
                    // tracked so value coercions inside the body (Str -> i8*,
                    // &T derefs) don't degrade the bits -- a Str handle was
                    // truncated to a single byte (corrupt map_err payloads).
                    let xiom_ty_name = Self::ref_preserving_name(&p.ty)
                        .unwrap_or_else(|| Self::type_from_ast(&p.ty));
                    self.local.local_xiom_types.insert(p.name.name.clone(), xiom_ty_name.clone());
                    self.local.param_locals.insert(p.name.name.clone());
                    if let Type::Ref(inner) = &p.ty {
                        let inner_llvm = self.llvm_type_for(&Self::type_from_ast(inner)).unwrap_or_else(|_| "i64".to_string());
                        if !inner_llvm.starts_with("%struct.") {
                            self.local.ref_params.insert(p.name.name.clone());
                        }
                    }
                    if Self::is_signed_xiom_type(&xiom_ty_name) {
                        self.local.signed_locals.insert(p.name.name.clone());
                    } else {
                        self.local.signed_locals.remove(&p.name.name);
                    }
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
                // CRT-layout fix (2026-09-09): real field sizes, not 8B/capture
                // (struct captures overflowed the malloc'd tail -- see the
                // PipeClosure arm above).
                let env_size: usize = 8 + captures
                    .iter()
                    .map(|(_name, ty)| Self::llvm_type_byte_size(ty, &self.types.type_meta))
                    .sum::<usize>();
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
                // G-44 / M17: `&out as *mut UInt8` and similar pointer casts.
                // With correct parser precedence (M17 fix), the AST is
                // `As(Ref(ident), *Type)` where `inner` is `Expr::Ref`.
                // Legacy code (pre-M17 parser) produces `Ref(As(ident, *Type))`
                // with `inner` as bare `Expr::Ident`. Handle both forms.
                let target_llvm_ty = self.llvm_type_for_fallback(&Self::type_from_ast(ty));
                // Extract the local ident from Ref(ident) / MutRef(ident) / bare Ident.
                // BUG 32: the REF forms mean "address of the local" (&x as *T);
                // the BARE-Ident form means "the local's VALUE as a pointer"
                // (`h as *UInt8` where h holds ptrtoint bits) -- it must
                // inttoptr the loaded value, never bitcast the slot address.
                let is_ref_form = matches!(inner.as_ref(),
                    Expr::Ref(_, _) | Expr::MutRef(_, _)
                    | Expr::Unary(UnaryOp::Ref, _, _) | Expr::Unary(UnaryOp::MutRef, _, _));
                let ident_opt: Option<&str> = match inner.as_ref() {
                    Expr::Ref(id_expr, _) | Expr::MutRef(id_expr, _) => {
                        if let Expr::Ident(id) = id_expr.as_ref() {
                            Some(id.name.as_str())
                        } else { None }
                    }
                    Expr::Unary(UnaryOp::Ref, id_expr, _) | Expr::Unary(UnaryOp::MutRef, id_expr, _) => {
                        if let Expr::Ident(id) = id_expr.as_ref() {
                            Some(id.name.as_str())
                        } else { None }
                    }
                    Expr::Ident(id) => Some(id.name.as_str()),
                    _ => None,
                };
                if target_llvm_ty.ends_with('*') {
                    if let Some(name) = ident_opt {
                        if let Some((slot, slot_ty)) = self.lookup_local(name).cloned() {
                            let ptr_reg = self.fresh_tmp();
                            if is_ref_form {
                                // `&x as *T`: the ADDRESS of the local's slot.
                                self.emitln(&format!("  {ptr_reg} = bitcast {slot_ty}* {slot} to {target_llvm_ty}"));
                            } else if slot_ty.ends_with('*') {
                                // Pointer-typed local: load the pointer, bitcast.
                                let loaded = self.fresh_tmp();
                                self.emitln(&format!("  {loaded} = load {slot_ty}, {slot_ty}* {slot}"));
                                self.emitln(&format!("  {ptr_reg} = bitcast {slot_ty} {loaded} to {target_llvm_ty}"));
                            } else {
                                // BUG 32: an INT-typed local holding pointer bits
                                // (`var h = buf as Int; h as *UInt8`) must
                                // inttoptr the loaded VALUE -- the old code
                                // bitcast the slot ADDRESS (q == buf was false,
                                // q[0] read stack bytes).
                                let loaded = self.fresh_tmp();
                                self.emitln(&format!("  {loaded} = load {slot_ty}, {slot_ty}* {slot}"));
                                self.emitln(&format!("  {ptr_reg} = inttoptr {slot_ty} {loaded} to {target_llvm_ty}"));
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
                        "i128" => Some(128),
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
                    // M39: Float <-> narrow int / float width conversions
                    ("double", "i32") | ("float", "i32") => {
                        self.emitln(&format!("  {tmp} = fptosi {inner_llvm_ty} {val} to i32"));
                        Ok((tmp, "i32".to_string()))
                    }
                    ("double", "i8") | ("float", "i8") | ("double", "i16") | ("float", "i16") => {
                        // Float -> narrow int: fptosi to i64 then trunc to target width
                        let mid = self.fresh_tmp();
                        self.emitln(&format!("  {mid} = fptosi {inner_llvm_ty} {val} to i64"));
                        self.emitln(&format!("  {tmp} = trunc i64 {mid} to {target_llvm_ty}"));
                        Ok((tmp, target_llvm_ty.to_string()))
                    }
                    ("i8", "double") | ("i16", "double") => {
                        // Narrow int -> float: sext to i64 then sitofp
                        let mid = self.fresh_tmp();
                        self.emitln(&format!("  {mid} = sext {inner_llvm_ty} {val} to i64"));
                        self.emitln(&format!("  {tmp} = sitofp i64 {mid} to double"));
                        Ok((tmp, "double".to_string()))
                    }
                    ("i8", "float") | ("i16", "float") => {
                        let mid = self.fresh_tmp();
                        self.emitln(&format!("  {mid} = sext {inner_llvm_ty} {val} to i64"));
                        let mid2 = self.fresh_tmp();
                        self.emitln(&format!("  {mid2} = sitofp i64 {mid} to double"));
                        self.emitln(&format!("  {tmp} = fptrunc double {mid2} to float"));
                        Ok((tmp, "float".to_string()))
                    }
                    ("i32", "double") => {
                        self.emitln(&format!("  {tmp} = sitofp i32 {val} to double"));
                        Ok((tmp, "double".to_string()))
                    }
                    ("i32", "float") | ("i64", "float") => {
                        let intermediate = if inner_llvm_ty == "i64" {
                            let mid = self.fresh_tmp();
                            self.emitln(&format!("  {mid} = trunc i64 {val} to i32"));
                            mid
                        } else { val.clone() };
                        self.emitln(&format!("  {tmp} = sitofp i32 {intermediate} to float"));
                        Ok((tmp, "float".to_string()))
                    }
                    ("double", "float") => {
                        self.emitln(&format!("  {tmp} = fptrunc double {val} to float"));
                        Ok((tmp, "float".to_string()))
                    }
                    ("float", "double") => {
                        self.emitln(&format!("  {tmp} = fpext float {val} to double"));
                        Ok((tmp, "double".to_string()))
                    }
                    ("float", "i64") => {
                        self.emitln(&format!("  {tmp} = fptosi float {val} to i64"));
                        Ok((tmp, LLVM_I64.to_string()))
                    }
                    (a, b) if a == b => Ok((val, target_llvm_ty.clone())),
                    // D1: Float64 -> Float128 (fpext). fpext of a double constant
                    // yields a valid fp128 SSA value; the STORE then uses the
                    // register (clang rejects bare decimal fp128 literals in
                    // stores, so we never pass the constant through untyped).
                    ("double", "fp128") => {
                        self.emitln(&format!("  {tmp} = fpext double {val} to fp128"));
                        Ok((tmp, "fp128".to_string()))
                    }
                    // D1: Float32 -> Float128 (fpext).
                    ("float", "fp128") => {
                        self.emitln(&format!("  {tmp} = fpext float {val} to fp128"));
                        Ok((tmp, "fp128".to_string()))
                    }
                    // D1: Float128 -> Float64 (fptrunc) / Float128 -> Float32.
                    ("fp128", "double") => {
                        self.emitln(&format!("  {tmp} = fptrunc fp128 {val} to double"));
                        Ok((tmp, "double".to_string()))
                    }
                    ("fp128", "float") => {
                        let mid = self.fresh_tmp();
                        self.emitln(&format!("  {mid} = fptrunc fp128 {val} to double"));
                        self.emitln(&format!("  {tmp} = fptrunc double {mid} to float"));
                        Ok((tmp, "float".to_string()))
                    }
                    // D1: Int <-> Int128 conversions (sext/trunc handled by the
                    // generic integer-width arm below via int_width; fp128
                    // integer conversions go through i64 then widen).
                    ("fp128", "i64") => {
                        self.emitln(&format!("  {tmp} = fptosi fp128 {val} to i64"));
                        Ok((tmp, LLVM_I64.to_string()))
                    }
                    ("i64", "fp128") => {
                        self.emitln(&format!("  {tmp} = sitofp i64 {val} to fp128"));
                        Ok((tmp, "fp128".to_string()))
                    }
                    // BUG 37/36 follow-up (2026-08-17): Int128 <-> Float128 casts
                    // fell through to the generic coerce path and emitted a
                    // mis-typed store (i128 value stored as fp128 -> clang
                    // rejected; the LLVM libcalls __floattitf/__fixtfti now
                    // exist in fp128_helpers.c with matching shims).
                    ("i128", "fp128") => {
                        self.emitln(&format!("  {tmp} = sitofp i128 {val} to fp128"));
                        Ok((tmp, "fp128".to_string()))
                    }
                    ("fp128", "i128") => {
                        self.emitln(&format!("  {tmp} = fptosi fp128 {val} to i128"));
                        Ok((tmp, "i128".to_string()))
                    }
                    // 5e.2 G-34: fn-ptr <-> Int casts.
                    (inner_ty, target_fn_ptr) if target_fn_ptr.contains('(')
                        && target_fn_ptr.contains(')')
                        && target_fn_ptr.ends_with('*')
                        && int_width(&inner_ty).is_some() =>
                    {
                        let ptr_reg = self.fresh_tmp();
                        self.emitln(&format!("  {ptr_reg} = inttoptr {inner_ty} {val} to {target_fn_ptr}"));
                        return Ok((ptr_reg, target_fn_ptr.to_string()));
                    }
                    // Reverse: fn-ptr -> Int (ptrtoint)
                    (src_fn_ptr, target_ty) if src_fn_ptr.contains('(')
                        && src_fn_ptr.contains(')')
                        && src_fn_ptr.ends_with('*')
                        && int_width(&target_ty).is_some() =>
                    {
                        let int_reg = self.fresh_tmp();
                        self.emitln(&format!("  {int_reg} = ptrtoint {src_fn_ptr} {val} to {target_ty}"));
                        return Ok((int_reg, target_ty.to_string()));
                    }
                    // 5c-E G2: &local as Int -- emit ADDRESS not VALUE
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
                        // Fallback: use fptosi for float->int, sitofp for int->float,
                        // sext/trunc for integer width changes.
                        let op = if (a == "double" || a == "float") && int_width(b).is_some() {
                            "fptosi"
                        } else if int_width(a).is_some() && (b == "double" || b == "float") {
                            "sitofp"
                        } else if int_width(a).is_some() && int_width(b).is_some() {
                            let aw = int_width(a).unwrap_or(64);
                            let bw = int_width(b).unwrap_or(64);
                            if bw < aw { "trunc" } else { "sext" }
                        } else {
                            "sext"
                        };
                        self.emitln(&format!("  {tmp} = {op} {a} {val} to {b}"));
                        Ok((tmp, target_llvm_ty.clone()))
                    }
                    // D1: big literal semantics -- `9223372036854775808 as Int128`
                    // means the VALUE 2^63, not i64::MIN sign-extended. When the
                    // source is a plain Int literal whose u64 bit pattern is
                    // > i64::MAX and the target is i128, interpret the literal
                    // as an unsigned u64 value and ZERO-extend.
                    ("i64", "i128") if matches!(inner.as_ref(), Expr::Int(v, _) if *v > i64::MAX as u64) => {
                        let z = self.fresh_tmp();
                        self.emitln(&format!("  {z} = zext i64 {val} to i128"));
                        Ok((z, "i128".to_string()))
                    }
                    // Integer <-> integer width conversions (e.g. Int<->Char, Int<->Int8/16/32).
                    // Char is i8 and Int is i64, so Int->Char truncs and Char->Int sign-extends.
                    (a, b) if int_width(a).is_some() && int_width(b).is_some() => {
                        let aw = int_width(a).expect("int_width(a) is Some (guarded above)");
                        let bw = int_width(b).expect("int_width(b) is Some (guarded above)");
                        if bw < aw {
                            self.emitln(&format!("  {tmp} = trunc {a} {val} to {b}"));
                        } else {
                            // BUG 14 fix: unsigned sources must ZERO-extend when
                            // widening (UInt64->UInt128, UInt8->Int128). Resolve the
                            // source's REGISTERED XIOM type (xiom_type_of_local
                            // prefers local_xiom_types -- the LLVM-slot-derived
                            // name loses signedness); unknown sources default to
                            // sext (historical behavior).
                            let src_signed = match inner.as_ref() {
                                Expr::Ident(id) => self.xiom_type_of_local(&id.name)
                                    .map(|xiom_ty| Self::is_signed_xiom_type(&xiom_ty))
                                    .unwrap_or(true),
                                // round-14 (BUG 26 #7): UInt*-RETURNING CALLS
                                // widen zext -- `byte_at(s, 1) as Int` on a
                                // UInt8 byte (0xCE = 206) was sext'd to -50.
                                Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
                                    self.callee_return_xiom(func)
                                        .map(|xiom_ty| Self::is_signed_xiom_type(&xiom_ty))
                                        .unwrap_or(true)
                                }
                                _ => true,
                            };
                            let extop = if src_signed { "sext" } else { "zext" };
                            self.emitln(&format!("  {tmp} = {extop} {a} {val} to {b}"));
                        }
                        // M17: Track signedness of the As result based on target XIOM type.
                        let target_xiom = Self::type_from_ast(ty);
                        self.local.reg_signed.insert(tmp.clone(), Self::is_signed_xiom_type(&target_xiom));
                        Ok((tmp, target_llvm_ty.clone()))
                    }
                    // Fallback: coerce the value to the declared target type so the
                    // As expression's reported type always matches the value.
                    _ => {
                        let coerced = self.coerce_value(&val, &inner_llvm_ty, &target_llvm_ty);
                        // M17: Track signedness of coerced result based on target XIOM type.
                        let target_xiom = Self::type_from_ast(ty);
                        self.local.reg_signed.insert(coerced.clone(), Self::is_signed_xiom_type(&target_xiom));
                        Ok((coerced, target_llvm_ty.clone()))
                    }
                }
            }
            Expr::Await(inner, _) => self.compile_expr(inner),
            Expr::Comptime(inner, _) => self.compile_expr(inner),
            Expr::Unsafe(block, _) => {
                // D2.1 (Phase 5, requirement f -- REVISED 2026-08-10): canonical
                // TRAP LOWERING. The inline VEH approach (xiom_trap_enter with
                // RtlCaptureContext/RtlRestoreContext) is BROKEN: the captured
                // context's RSP points into xiom_trap_enter's OWN frame, which
                // is popped and reused before a fault deep in the block fires.
                // RtlRestoreContext then restores RSP into that dead region, the
                // epilogue `ret` pops a stale address, and control jumps back
                // into the faulting block -> infinite AV->restore->AV loop.
                //
                // Canonical fix (plan S2.7): lower the block to a STANDALONE
                // function `int64_t __unsafe_block_N(uint8_t* ctx)` (captures =
                // free variables packed in a ctx struct) and run it through the
                // pre-compiled SEH trampoline xiom_trampoline_call, whose
                // `__try/__except` checkpoint frame stays ALIVE across
                // block_fn(ctx). On fault, __except returns a code 1-6 (no
                // register-restore). Branch on the fault code.
                //
                // D2.1 (Phase 5, S2.13): a NESTED unsafe block (already inside an
                // unsafe-block fn) must NOT create a second trampoline -- the outer
                // SEH checkpoint already covers it. Compile it as a plain block
                // (allocations still route to the guard arena via guard_heap_depth,
                // and captured-variable access works directly since we are in the
                // same fn scope). This avoids nested trampolines corrupting the
                // guard-arena/TLS state (observed: str_concat's unsafe block inside
                // str_pad_left's unsafe block crashed).
                if self.in_unsafe_block_fn {
                    // Emit guard enter/arm (idempotent with the outer block's --
                    // the outer block already entered; nested re-enter bumps depth,
                    // re-exit decrements -- so the arena stays active throughout).
                    self.emitln("  call void @xiom_guard_heap_enter()");
                    self.emitln("  call void @xiom_guard_page_arm()");
                    self.guard_heap_depth += 1;
                    // A `return` inside a nested unsafe block must NOT signal a
                    // return-from-the-enclosing-fn (that is the OUTER block fn's
                    // job). It returns from the current block fn normally, so the
                    // nested block's `return X` yields X as the outer block fn's
                    // value. Temporarily clear the in-block-fn flag so Stmt::Return
                    // emits a plain `ret`.
                    let saved_nested_in_block = self.in_unsafe_block_fn;
                    self.in_unsafe_block_fn = false;
                    let mut last = String::new();
                    let mut last_ty = String::new();
                    let n = block.stmts.len();
                    for (i, item) in block.stmts.iter().enumerate() {
                        let is_last = i + 1 == n;
                        match item {
                            xiom_ast::StmtOrExpr::Expr(e) => {
                                let (v, vt) = self.compile_expr(e)?;
                                if is_last { last = v; last_ty = vt; }
                            }
                            xiom_ast::StmtOrExpr::Stmt(s) => {
                                if is_last {
                                    if let Stmt::Expr(e, ..) = s {
                                        let (v, vt) = self.compile_expr(e)?;
                                        last = v; last_ty = vt;
                                    } else {
                                        self.compile_stmt(s)?;
                                    }
                                } else {
                                    self.compile_stmt(s)?;
                                }
                            }
                        }
                    }
                    self.in_unsafe_block_fn = saved_nested_in_block;
                    if last.is_empty() {
                        last = "0".to_string();
                        last_ty = "void".to_string();
                    }
                    self.guard_heap_depth -= 1;
                    // Copy-Out a Str tail (the outer block's arena reset would
                    // otherwise UAF it).
                    if last_ty == LLVM_STR_PTR {
                        let copy_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {copy_tmp} = call i8* @xiom_guard_copy_str(i8* {last})"));
                        let not_null = self.fresh_tmp();
                        self.emitln(&format!("  {not_null} = icmp ne i8* {copy_tmp}, null"));
                        let sel = self.fresh_tmp();
                        self.emitln(&format!("  {sel} = select i1 {not_null}, i8* {copy_tmp}, i8* {last}"));
                        last = sel;
                    }
                    self.emitln("  call void @xiom_guard_heap_exit()");
                    self.emitln("  call void @xiom_guard_page_disarm()");
                    return Ok((last, last_ty));
                }
                // D2.1 (Phase 7): `#[unsafe_direct]` -- trusted escape hatch.
                // The fn's unsafe blocks run as PLAIN unsafe blocks (no
                // trampoline, no arena, no guard page): today's transparent
                // lowering. Intended for stdlib/selfhost hot paths. Counted
                // against the audited cap.
                if self.fctx.unsafe_direct {
                    self.unsafe_direct_count += 1;
                    if self.unsafe_direct_count > self.config.unsafe_direct_cap {
                        return Err(format!(
                            "unsafe_direct cap exceeded: {} `#[unsafe_direct]` blocks (limit {})",
                            self.unsafe_direct_count, self.config.unsafe_direct_cap
                        ));
                    }
                    // Compile as a plain block (no guard heap / page -- trusted).
                    let mut last = String::new();
                    let mut last_ty = String::new();
                    let n = block.stmts.len();
                    for (i, item) in block.stmts.iter().enumerate() {
                        let is_last = i + 1 == n;
                        match item {
                            xiom_ast::StmtOrExpr::Expr(e) => {
                                let (v, vt) = self.compile_expr(e)?;
                                if is_last { last = v; last_ty = vt; }
                            }
                            xiom_ast::StmtOrExpr::Stmt(s) => {
                                if is_last {
                                    if let Stmt::Expr(e, ..) = s {
                                        let (v, vt) = self.compile_expr(e)?;
                                        last = v; last_ty = vt;
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
                        last = "0".to_string();
                        last_ty = "void".to_string();
                    }
                    return Ok((last, last_ty));
                }
                let unsafe_id = self.unsafe_block_counter;
                self.unsafe_block_counter += 1;
                let fn_name = format!("__unsafe_block_{unsafe_id}");
                let ctx_name = format!("__unsafe_ctx_{unsafe_id}");

                // Free variables referenced by the block (from enclosing scope).
                // Captures are passed BY POINTER (not by value) so that reads AND
                // writes to captured variables inside the block fn propagate back
                // to the enclosing scope (e.g. `pad_str = str_concat(...)` in a
                // confined block must update the caller's `pad_str`). Each ctx
                // field holds the ADDRESS of the enclosing alloca.
                let captures = self.collect_block_free_vars(block, &[]);

                // ---- Emit the ctx struct type (before the enclosing define) ----
                if !captures.is_empty() {
                    let ctx_fields: Vec<String> = captures.iter().map(|(_, t)| format!("{t}*")).collect();
                    let ctx_def = format!("%struct.{ctx_name} = type {{ {} }}\n", ctx_fields.join(", "));
                    if let Some(pos) = self.output.find("define ") {
                        self.output.insert_str(pos, &ctx_def);
                    } else {
                        self.output.push_str(&ctx_def);
                    }
                }

                // ---- Emit the standalone block function (deferred) ----
                let saved_output = std::mem::take(&mut self.output);
                let saved_tmp = self.tmp_counter;
                let saved_block = self.block_counter;
                let saved_ret = self.fctx.current_return_type.clone();
                let saved_result_ptr = self.fctx.result_ptr.take();
                let saved_match_ptr = self.fctx.match_result_ptr.take();
                let saved_match_ty = self.fctx.match_result_ty.take();
                let saved_ensures = std::mem::take(&mut self.fctx.current_ensures);
                let saved_in_block_fn = self.in_unsafe_block_fn;
                // BUG 22 #6: the block fn and the enclosing fn share
                // self.local -- isolate the hoisted-alloca list so each fn
                // splices only its own loop-body allocas into its own entry.
                let saved_hoisted = std::mem::take(&mut self.local.hoisted_allocas);
                self.in_unsafe_block_fn = true;
                self.tmp_counter = unsafe_id * 1000;
                self.block_counter = unsafe_id * 1000;
                // BUG 55: keep the ENCLOSING fn's declared return type for the
                // Some/None/Ok/Err ctor decision (concrete Option__Rc) -- the
                // block-fn ABI stays i64 via current_return_type below.
                self.fctx.enclosing_return_type = Some(saved_ret.clone());
                self.fctx.current_return_type = LLVM_I64.to_string();
                self.push_scope();

                self.output.push_str(&format!("define i64 @{fn_name}(i8* %ctx_raw) {{\nentry:\n"));
                // Load capture POINTERS from the ctx struct; register them as the
                // locals directly (no fresh alloca) so loads AND stores go through
                // the enclosing alloca's address (write-back semantics).
                if !captures.is_empty() {
                    self.emitln(&format!("  %__ctx_ptr = bitcast i8* %ctx_raw to %struct.{ctx_name}*"));
                    for (i, (cap_name, cap_ty)) in captures.iter().enumerate() {
                        let gep = self.fresh_tmp();
                        let cap_ptr = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr %struct.{ctx_name}, %struct.{ctx_name}* %__ctx_ptr, i32 0, i32 {i}"));
                        self.emitln(&format!("  {cap_ptr} = load {cap_ty}*, {cap_ty}** {gep}"));
                        self.add_local(cap_name, cap_ptr, cap_ty);
                    }
                }
                // Guard heap + guard page inside the confined block fn.
                self.emitln("  call void @xiom_guard_heap_enter()");
                self.emitln("  call void @xiom_guard_page_arm()");
                self.guard_heap_depth += 1;
                // Compile every statement; the block's value is its tail.
                let mut last = String::new();
                let mut last_ty = String::new();
                let mut block_ends_in_return = false;
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
                                    if matches!(s, Stmt::Return(..)) { block_ends_in_return = true; }
                                    self.compile_stmt(s)?;
                                }
                            } else {
                                self.compile_stmt(s)?;
                            }
                        }
                    }
                }
                if last.is_empty() {
                    // A block whose tail is a `return` returns the enclosing
                    // fn's value type (coerced to i64 for the block-fn ABI);
                    // the value flows through the trampoline's result slot.
                    last = "0".to_string();
                    if block_ends_in_return && !saved_ret.is_empty() && saved_ret != "void" {
                        last_ty = saved_ret.clone();
                    } else {
                        last_ty = "void".to_string();
                    }
                }
                self.guard_heap_depth -= 1;
                // If the block's tail was a `return` statement, Stmt::Return has
                // ALREADY emitted the copy-out + guard exit + `ret` (with the
                // enclosing fn's value type, coerced to i64 for the block-fn
                // ABI). Do NOT emit a second dead tail here.
                if !self.current_block_terminated() {
                    // Copy-Out (requirement i, UAF fix): promote a Str tail to
                    // the main heap BEFORE the arena resets.
                    if last_ty == LLVM_STR_PTR {
                        let copy_tmp = self.fresh_tmp();
                        self.emitln(&format!("  {copy_tmp} = call i8* @xiom_guard_copy_str(i8* {last})"));
                        let not_null = self.fresh_tmp();
                        self.emitln(&format!("  {not_null} = icmp ne i8* {copy_tmp}, null"));
                        let sel = self.fresh_tmp();
                        self.emitln(&format!("  {sel} = select i1 {not_null}, i8* {copy_tmp}, i8* {last}"));
                        last = sel;
                    }
                    self.emitln("  call void @xiom_guard_heap_exit()");
                    self.emitln("  call void @xiom_guard_page_disarm()");
                    // Return the block's value as i64 (the trampoline ABI is
                    // int64_t (*)(uint8_t*)); the trampoline stores it in the
                    // TLS result slot on success.
                    let ret_i64 = self.val_to_i64(&last, &last_ty);
                    self.emitln(&format!("  ret i64 {ret_i64}"));
                }
                self.emitln("}");
                // BUG 22 #6: splice loop-body-hoisted allocas into the block
                // fn's entry block (before the buffer is taken).
                self.finish_hoisted_allocas();
                self.pop_scope();

                let block_ir = std::mem::take(&mut self.output);
                self.local.deferred_closure_defs.push(block_ir);
                self.output = saved_output;
                self.tmp_counter = saved_tmp;
                self.block_counter = saved_block;
                self.fctx.current_return_type = saved_ret;
                self.fctx.result_ptr = saved_result_ptr;
                self.fctx.match_result_ptr = saved_match_ptr;
                self.fctx.match_result_ty = saved_match_ty;
                self.fctx.current_ensures = saved_ensures;
                self.in_unsafe_block_fn = saved_in_block_fn;
                self.local.hoisted_allocas = saved_hoisted;
                // BUG 55: the block fn is done -- the enclosing return type
                // context is no longer needed.
                self.fctx.enclosing_return_type = None;

                // ---- At the block site: build ctx, call trampoline, branch ----
                // Allocate + populate the ctx struct (stack), then call the
                // trampoline which wraps block_fn in SEH __try/__except.
                let ctx_i8 = if captures.is_empty() {
                    "null".to_string()
                } else {
                    let ctx_slot = self.fresh_tmp();
                    self.emitln(&format!("  {ctx_slot} = alloca %struct.{ctx_name}"));
                    for (i, (cap_name, _cap_ty)) in captures.iter().enumerate() {
                        let gep = self.fresh_tmp();
                        self.emitln(&format!("  {gep} = getelementptr %struct.{ctx_name}, %struct.{ctx_name}* {ctx_slot}, i32 0, i32 {i}"));
                        if let Some((slot, ty)) = self.lookup_local(cap_name).cloned() {
                            // BUG 22 #6: capture the ADDRESS of the enclosing
                            // alloca directly (pointer capture -- the block fn
                            // loads AND stores through it, so mutations write
                            // back). Loop-body binding allocas are HOISTED to
                            // the fn entry by the binding codegen, so every
                            // captured alloca dominates this site.
                            self.emitln(&format!("  store {ty}* {slot}, {ty}** {gep}"));
                        }
                    }
                    let bc = self.fresh_tmp();
                    self.emitln(&format!("  {bc} = bitcast %struct.{ctx_name}* {ctx_slot} to i8*"));
                    bc
                };
                let fn_i64 = self.fresh_tmp();
                self.emitln(&format!("  {fn_i64} = ptrtoint ptr @{fn_name} to i64"));
                // D2.1 (Phase 6, requirement h): apply the enclosing fn's retry
                // policy. `#[unsafe_no_retry]` (fctx.unsafe_allow_retry=false)
                // disables the once-only transient retry.
                let retry_policy = if self.fctx.unsafe_allow_retry { "1" } else { "0" };
                self.emitln(&format!("  call void @xiom_trampoline_set_allow_retry(i64 {retry_policy})"));
                let fault_flag = self.fresh_tmp();
                self.emitln(&format!("  {fault_flag} = call i64 @xiom_trampoline_call(i64 {fn_i64}, i8* {ctx_i8})"));
                let fault_is_set = self.fresh_tmp();
                self.emitln(&format!("  {fault_is_set} = icmp ne i64 {fault_flag}, 0"));
                let fault_path = self.fresh_block("confined_fault");
                let normal_path = self.fresh_block("confined_normal");
                self.emitln(&format!("  br i1 {fault_is_set}, label %{fault_path}, label %{normal_path}"));

                // Fault path: discard the arena + disarm, return a type-correct
                // zero for the enclosing fn's return type (recoverable indicator).
                self.emitln(&format!("\n{fault_path}:"));
                self.emitln("  call void @xiom_guard_heap_exit()");
                self.emitln("  call void @xiom_guard_page_disarm()");
                let ret_ty = self.fctx.current_return_type.clone();
                if ret_ty.is_empty() || ret_ty == "void" {
                    self.emitln("  ret void");
                } else if ret_ty == LLVM_STR_PTR {
                    self.emitln("  ret i8* null");
                } else if ret_ty.starts_with("%struct.") {
                    let slot = self.fresh_tmp();
                    self.emitln(&format!("  {slot} = alloca {ret_ty}, align 16"));
                    let loaded = self.fresh_tmp();
                    self.emitln(&format!("  {loaded} = load {ret_ty}, {ret_ty}* {slot}, align 16"));
                    self.emitln(&format!("  ret {ret_ty} {loaded}"));
                } else if ret_ty.ends_with('*') {
                    // Pointer return type (e.g. *Int, *Node): the recoverable
                    // fault indicator is a NULL pointer, not an integer literal
                    // (clang rejects `ret i64* 0`).
                    self.emitln(&format!("  ret {ret_ty} null"));
                } else if ret_ty.starts_with("float") || ret_ty == "double" {
                    self.emitln(&format!("  ret {ret_ty} 0.0"));
                } else {
                    self.emitln(&format!("  ret {ret_ty} 0"));
                }

                // Normal path: recover the block's value from the trampoline's
                // TLS result slot and convert it back to the block's tail type.
                self.emitln(&format!("\n{normal_path}:"));
                let res_i64 = self.fresh_tmp();
                self.emitln(&format!("  {res_i64} = call i64 @xiom_trampoline_get_result()"));
                let (last, last_ty) = self.unsafe_result_i64_to_val(&res_i64, &last_ty);
                // If the block fn executed a `return` statement, the block's
                // value must be RETURNED from the ENCLOSING fn (the block was
                // not used as an expression value). Emit the enclosing return
                // with the block's value.
                let was_returned = self.fresh_tmp();
                self.emitln(&format!("  {was_returned} = call i64 @xiom_trampoline_was_returned()"));
                let returned_cond = self.fresh_tmp();
                self.emitln(&format!("  {returned_cond} = icmp ne i64 {was_returned}, 0"));
                let normal_tail = self.fresh_block("confined_value");
                let ret_from_enclosing = self.fresh_block("confined_return");
                self.emitln(&format!("  br i1 {returned_cond}, label %{ret_from_enclosing}, label %{normal_tail}"));
                self.emitln(&format!("\n{ret_from_enclosing}:"));
                let enc_ret_ty = self.fctx.current_return_type.clone();
                if enc_ret_ty.is_empty() || enc_ret_ty == "void" {
                    self.emitln("  ret void");
                } else {
                    // Use a SEPARATE register for the enclosing-fn coercion: the
                    // block's `last` value must stay untouched for the normal_tail
                    // path below (e.g. a pointer tail stays a pointer -- coercing
                    // it to the enclosing fn's i64 here previously leaked the
                    // i64 into `icmp eq ptr, i64` at the block's use site).
                    let coerced_last = self.coerce_value(&last, &last_ty, &enc_ret_ty);
                    if let Some(res_ptr) = self.fctx.result_ptr.as_ref() {
                        let enc_ret_ty = self.fctx.current_return_type.clone();
                        self.emitln(&format!("  store {enc_ret_ty} {coerced_last}, {enc_ret_ty}* {res_ptr}"));
                    }
                    if !self.fctx.current_ensures.is_empty() {
                        self.compile_ensures_checks();
                    }
                    let depth_dec = self.fresh_tmp();
                    self.emitln(&format!("  {depth_dec} = load i64, i64* @xiom_recursion_counter"));
                    let new_depth = self.fresh_tmp();
                    self.emitln(&format!("  {new_depth} = sub i64 {depth_dec}, 1"));
                    self.emitln(&format!("  store i64 {new_depth}, i64* @xiom_recursion_counter"));
                    self.emitln(&format!("  ret {enc_ret_ty} {coerced_last}"));
                }
                self.emitln(&format!("\n{normal_tail}:"));
                Ok((last, last_ty))
            }
            Expr::BlockExpr(block, _) => {
                // A plain block expression: compile every statement; the value
                // is its tail. No guard-heap wrapping (only `unsafe` blocks are
                // confined).
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
                    Ok(("0".to_string(), "void".to_string()))
                } else {
                    Ok((last, last_ty))
                }
            }
            Expr::If(cond, then_block, elifs, else_block, _) => {
                // Value-producing if-expression (e.g. `let x = if c { 1 } else { 0 }`).
                // Emit conditional branches a la Stmt::If, but have each arm store its
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

                // gzip fix (2026-08-19): `let compressed = if c { rle_encode(data) }
                // else { _store_encode(data) };` -- the result alloca must keep the
                // arms' STRUCT type (%struct.Vec). The old hardcoded i64 coerced
                // the Vec VALUE to field-0-as-i64 (the data pointer), so
                // `compressed.len()` degraded to xiom_str_len and `compressed[i]`
                // compiled to a literal 0 (payload of all zeros -> wrong decode).
                // Infer the result type from the arm tail expressions like
                // infer_match_llvm_type does (struct > pointer > i64).
                let mut arm_tys: Vec<String> = Vec::new();
                let arm_blocks = std::iter::once(then_block)
                    .chain(elifs.iter().map(|(_, b)| b))
                    .chain(else_block.iter());
                for b in arm_blocks {
                    if let Some(last) = b.stmts.last() {
                        if let StmtOrExpr::Expr(e) = last {
                            let t = self.infer_llvm_type(e);
                            if !t.is_empty() {
                                arm_tys.push(t);
                            }
                        }
                    }
                }
                let result_ty = if let Some(st) = arm_tys.iter().find(|t| t.starts_with("%struct.")) {
                    // Only use the struct type when ALL struct arms agree -- mixed
                    // struct types would emit invalid stores into one slot.
                    if arm_tys.iter().filter(|t| t.starts_with("%struct.")).all(|t| t == st) {
                        st.clone()
                    } else {
                        LLVM_I64.to_string()
                    }
                } else if let Some(pt) = arm_tys.iter().find(|t| t.ends_with('*')) {
                    pt.clone()
                } else if arm_tys.iter().any(|t| t == "double") && arm_tys.iter().all(|t| t == "double") {
                    // Float64 arms keep the double bits (was bitcast-to-i64 +
                    // sitofp on the return -- wrong values).
                    "double".to_string()
                } else if arm_tys.iter().any(|t| t == "float") && arm_tys.iter().all(|t| t == "float") {
                    "float".to_string()
                } else {
                    LLVM_I64.to_string()
                };
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
                    // No elifs, no else -- the original else_label IS the merge_label
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
            Expr::ConstBlock(inner, _) => {
                let evaluated = self.evaluate_const_init(inner);
                self.compile_expr(&evaluated)
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
                let (_scrutinee_val, scrutinee_ty) = self.compile_expr(scrutinee)?;
                let result_ty = self.infer_match_llvm_type(arms, &scrutinee_ty);
                let result_alloca = self.fresh_tmp();
                self.emitln(&format!("  {result_alloca} = alloca {result_ty}"));
                // 5c.37: Initialize match result slot to prevent uninitialized
                // reads when all arms return/exit (no fallthrough store).
                if result_ty.starts_with("%struct.") {
                    self.emitln(&format!("  store {result_ty} zeroinitializer, {result_ty}* {result_alloca}"));
                } else {
                    self.emitln(&format!("  store {result_ty} 0, {result_ty}* {result_alloca}"));
                }
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

    /// BUG 29 (contract Some-payload ensures): after binding a pattern
    /// variable from `expr is Some(x)` / `Ok(x)` / `Err(x)`, record the
    /// payload's XIOM type in local_xiom_types so method calls on the
    /// bound var dispatch correctly -- `result is Some => result.len() > 0`
    /// was emitting Map.len (first generic match) instead of Str.len
    /// because the payload was an untracked i64.
    fn bind_is_payload_xiom(&mut self, scrutinee: &Expr, pat: &xiom_ast::Pattern, id: &Ident) {
        let scrut_xiom = match scrutinee {
            Expr::Ident(sid) => self.local.local_xiom_types.get(&sid.name).cloned(),
            _ => None,
        };
        let payload_ty = scrut_xiom.and_then(|t| {
            let (base, args) = Self::parse_generic_type_string(&t);
            match (base.as_str(), pat) {
                ("Option", _) => args.first().cloned(),
                ("Result", xiom_ast::Pattern::Err(..)) => args.get(1).cloned(),
                ("Result", _) => args.first().cloned(),
                _ => None,
            }
        });
        if let Some(pt) = payload_ty {
            self.local.local_xiom_types.insert(id.name.clone(), pt);
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
            // are not locals -- not instances.
            Expr::Ident(ident) => {
                self.lookup_local(&ident.name).is_some()
                    // BUG 29 (Map.keys on module globals): a MODULE-GLOBAL var
                    // (`var _coverage: Map[Str, Bool]`) is a VALUE -- generic
                    // method calls on it must pass the receiver instance.
                    || self.local.module_globals.contains_key(&ident.name)
            }
            // `a.b`: instance iff its base chain is rooted in a value (local/self),
            // e.g. `obj.field`. A module path like `xiom.char` is rooted in `xiom`
            // (not a local) -> NOT an instance. Also an instance if the whole
            // expression has a concrete struct type.
            Expr::Field(base, field, _) => {
                // `module.Type` static path (e.g. `alloc.Layout`) is NOT an instance:
                // the base is a module (not a value) and the leaf names a known type.
                if !self.receiver_is_instance(base)
                    && (self.types.types.contains_key(&field.name)
                        || self.types.type_meta.contains_key(&field.name)
                        || self.types.type_meta.keys().into_iter().any(|k| k.ends_with(&format!(".{}", field.name))))
                {
                    return false;
                }
                self.receiver_is_instance(base) || self.infer_struct_type_name(receiver).is_some()
            }
            // Calls / indexing / parens evaluate to values.
            Expr::Call(..) | Expr::GenericCall(..) | Expr::Paren(..) => true,
            // D1: `Trait[Arg].method(...)` -- an Index whose base is a known
            // interface name is a STATIC impl-dispatch receiver (e.g.
            // `Num[Int].add(a, b)`), NOT an instance. The codegen resolves the
            // call to the impl's `Type.method` freestanding fn directly.
            Expr::Index(base, _, _) => {
                if let Expr::Ident(id) = base.as_ref() {
                    if self.types.interfaces.contains_key(&id.name)
                        || self.types.interfaces.keys().into_iter().any(|k| k.ends_with(&format!(".{}", id.name)))
                    {
                        return false;
                    }
                }
                true
            }
            // Any other receiver form evaluates to a value -- preserve the prior
            // "complex receiver is an instance" behavior (only the Ident type-name
            // and Field module-path shapes above are treated as non-instances).
            _ => true,
        }
    }

    /// Round-6 fix (2026-08-19): true when the receiver is a real STRING value
    /// (Str == i8*, or an i64 slot holding a string handle via the `?`/payload
    /// bindings). Guards the Str builtin handlers (slice/substr/starts_with/
    /// ends_with) so `p.starts_with(b)` on a Path STRUCT falls through to the
    /// real method dispatch instead of the Str builtin (which BOXED the Path
    /// and passed the box address as a string -- always false / garbage).
    pub(crate) fn receiver_is_str(&self, receiver: &Expr) -> bool {
        if self.infer_llvm_type(receiver) == "i8*" {
            return true;
        }
        if let Expr::Ident(id) = receiver {
            if self.xiom_type_of_local(&id.name).as_deref() == Some("Str") {
                return true;
            }
        }
        // round-14 (Vec[Str] elements): `v[0].len()` / `h.names[i].starts_with(..)`
        // -- the elem-load switch yields i64 so infer_llvm_type can't see the
        // string. The container's element type is Str (from local_vec_elem /
        // local_xiom_types / struct-field type_meta).
        if self.is_vec_str_elem_receiver(receiver) {
            return true;
        }
        false
    }

    /// round-14 (Vec[Str] elements): true when the receiver is an INDEX into a
    /// Vec whose element type is Str (`v[0]`, `h.names[i]`, `&Vec[Str]` params).
    pub(crate) fn is_vec_str_elem_receiver(&self, receiver: &Expr) -> bool {
        let Expr::Index(container, _, _) = receiver else { return false; };
        self.resolve_vec_elem_xiom(container).as_deref() == Some("Str")
    }

    /// round-15: true when `container` is an IDENT bound from a mono'd
    /// `&[N]T` PARAM (registered in local_array_elem by the mono param
    /// binding) -- the elem pointer must index data[idx] directly, NOT via
    /// the array-buffer (+1) or Str (xiom_char_at) conventions. Excludes
    /// array-LITERAL locals (also in local_array_elem): those use the
    /// is_array_buf buffer layout.
    pub(crate) fn is_array_elem_param(&self, container: &Expr) -> bool {
        let Expr::Ident(id) = container else { return false; };
        self.local.param_locals.contains(&id.name)
            && self.local.local_array_elem.contains_key(&id.name)
    }

    /// BUG 31: LLVM type for a STRUCT FIELD -- degrades Unit to i64. The
    /// struct-def path emits Unit fields as i64, so the ctor stores must
    /// match (`store void 0, void*` was invalid IR; Ok(()) on
    /// Result[Unit, FmtError] -> "void type only allowed for function
    /// results").
    pub(crate) fn field_llvm_ty(&self, xiom_ty: &str) -> String {
        let t = self.llvm_type_for(xiom_ty).unwrap_or_else(|_| "i64".to_string());
        if t == "void" { "i64".to_string() } else { t }
    }

    /// BUG 31: resolve a struct-literal type NAME that may carry generic
    /// args (`Result[Unit, FmtError]`) to the CONCRETE registered type
    /// (`%struct.Result__Unit__FmtError`). Without this, the literal built
    /// the GENERIC template (%struct.Result) while the fn signature used the
    /// concrete type -- the Unit field store emitted `store void 0, void*`
    /// (invalid IR) and the return ABI mismatched.
    fn resolve_literal_struct_ty(&self, name: &str) -> String {
        if name.contains('[') {
            let mangled = name.replace('[', "__")
                .replace(", ", "__")
                .replace(',', "__")
                .replace(']', "");
            let t = self.llvm_type_for_fallback(&mangled);
            if t.starts_with('%') {
                return t;
            }
        }
        self.llvm_type_for_fallback(name)
    }

}

