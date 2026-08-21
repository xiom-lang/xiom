//! LLVM type system utilities for the XIOM code generator.
//!
//! Bridges the XIOM type system and the LLVM IR type system. Key functions:
//! - [`IrEmitter::type_from_ast`] -- XIOM AST types to LLVM type strings
//! - [`IrEmitter::zero_val_for`] -- LLVM zero initializers
//! - [`IrEmitter::xiom_to_llvm_type`] -- XIOM type names to LLVM primitives
//! - [`IrEmitter::infer_struct_type_name`] -- resolve struct types from expressions

use xiom_ast::*;
use crate::llvm_consts::*;

impl crate::IrEmitter {
    pub fn zero_val_for(&self, val: &str, llvm_ty: &str) -> String {
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
    pub fn default_const_for(llvm_ty: &str) -> String {
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
    /// materialized to their real value -- these are the initializers that
    /// currently-passing modules depend on (e.g. `_global_state = 12345`).
    /// Anything more complex (enum variants, struct/aggregate values,
    /// constructor calls like `Vec[T]::new()`) is zero-initialized: a valid,
    /// safe default. Such globals are always assigned before first meaningful
    /// read in practice.
    pub fn global_const_init(value: &Expr, llvm_ty: &str) -> String {
        eprintln!("CG02 DEBUG: value={value:?}, llvm_ty={llvm_ty}");
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
                format!("0x{:08X}", (*f as f32).to_bits())
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

    /// Widen a narrow integer value (`i1`/`i8`/`i16`/`i32`) to `i64` so it can
    /// participate in the emitter's i64 integer arithmetic/comparison model.
    pub fn widen_to_i64(&mut self, val: &str, ty: &str) -> String {
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

    /// Quick scan: returns true if the expression tree contains any `this` ident.
    /// 5c.29: true when `e` is an `x.unwrap()`-style call. Their i64 ABI
    /// result carries float payloads as RAW BITS (Some(x) boxes via bitcast),
    /// so float-context conversions must bit-reinterpret rather than sitofp.
    pub fn expr_is_unwrap_call(e: &Expr) -> bool {
        if let Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) = e {
            if let Expr::Field(_, f, _) = func.as_ref() {
                return matches!(f.name.as_str(), "unwrap" | "unwrap_or" | "unwrap_err");
            }
        }
        false
    }

    pub fn expr_uses_this(expr: &Expr) -> bool {        match expr {
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

    pub fn stmt_uses_this(stmt: &Stmt) -> bool {
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

    pub fn block_uses_this(block: &Block) -> bool {
        block.stmts.iter().any(|s| match s {
            StmtOrExpr::Stmt(stmt) => Self::stmt_uses_this(stmt),
            StmtOrExpr::Expr(expr) => Self::expr_uses_this(expr),
        })
    }

    /// G-20: does the method body reference receiver STATE -- either `this`
    /// or a BARE receiver-field ident (e.g. `val` in `fn Counter.inc() ->
    /// Int { return val + 1; }`)? Used by registration + definition to emit
    /// a %param_self slot so the prologue can bind bare fields via GEP.
    /// Param names shadow fields (a param `x` is never receiver state).
    pub fn body_uses_receiver_state(&self, fd: &FnDecl) -> bool {
        let Some(body) = fd.body.as_ref() else { return false };
        if Self::block_uses_this(body) {
            return true;
        }
        let Some(recv) = fd.receiver.as_ref() else { return false };
        let fields = self.types.types.get(&recv.name)
            .or_else(|| {
                let suffix = format!(".{}", recv.name);
                self.types.types.keys().into_iter().find(|k| k.ends_with(&suffix)).and_then(|k|self.types.types.get(&k))
            });
        let Some(fields) = fields else { return false };
        let param_names: std::collections::HashSet<&str> =
            fd.params.iter().map(|p| p.name.name.as_str()).collect();
        let candidates: Vec<&str> = fields.iter()
            .map(|f| f.as_str())
            .filter(|f| !param_names.contains(f))
            .collect();
        if candidates.is_empty() { return false; }
        Self::block_mentions_any_ident(body, &candidates)
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
            Expr::Binary(a, _, b, _) => Self::expr_mentions_any_ident(a, names) || Self::expr_mentions_any_ident(b, names),
            // obj.FIELD: the field NAME is not a bare ident -- only scan the object.
            Expr::Field(obj, _, _) => Self::expr_mentions_any_ident(obj, names),
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

    pub fn is_primitive_type_name(type_name: &str) -> bool {
        matches!(
            type_name,
            "Bool" | "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "Int128"
                | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64" | "UInt128"
                | "Float32" | "Float64" | "Float128" | "Char" | "Str"
        )
    }

    pub fn xiom_to_llvm_type(xiom_ty: &str) -> &'static str {
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
            "Unit" => "i64",
            "Vec" | "Map" | "Set" | "Option" | "Result" => "i64",
            _ => {
                // Generic params (T, K, V) and Self -- silent i64 defaults.
                if xiom_ty.len() == 1 && xiom_ty.chars().next().map_or(false, |c| c.is_uppercase()) {
                    return "i64";
                }
                if xiom_ty == "Self" { return "i64"; }
                // Bracket-preserving type names: strip to base and recurse.
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
                "i64"
            }
        }
    }

    /// Map an AST Type to its LLVM type string, handling pointer types (`*T` -> `<T>*`),
    /// ref types (`&T` -> `<T>*`), and named/builtin types.
    pub fn extern_type_to_llvm(&self, ty: &Type) -> String {
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
            Type::Tuple(_) => LLVM_I64.to_string(),
            _ => LLVM_I64.to_string(),
        }
    }

    pub fn xiom_type_name_from_llvm(llvm_ty: &str) -> String {
        // Check pointer types before stripping `*` -- `i8*` is Str, not Int8.
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
            "i128" => "Int128".to_string(),
            "double" => "Float64".to_string(),
            "float" => "Float32".to_string(),
            "fp128" => "Float128".to_string(),
            "i1" => "Bool".to_string(),
            _ => base.to_string(),
        }
    }

    /// Extract the name of each type argument from a Type AST node.
    /// For `Option[T]` returns `["T"]`, for `Map[K, V]` returns `["K", "V"]`.
    pub fn extract_type_arg_names(ty: &Type) -> Vec<String> {
        match ty {
            Type::Named(_, args) => args.iter().map(|a| Self::type_from_ast(a)).collect(),
            Type::Option(inner) => vec![Self::type_from_ast(inner)],
            Type::Result(ok, err) => vec![Self::type_from_ast(ok), Self::type_from_ast(err)],
            Type::Vec(inner) => vec![Self::type_from_ast(inner)],
            Type::Map(k, v) => vec![Self::type_from_ast(k), Self::type_from_ast(v)],
            Type::Set(inner) => vec![Self::type_from_ast(inner)],
            // Unwrap Ref/MutRef/Ptr to find type args nested inside (e.g. `&[N]T` -> N, T)
            Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) => Self::extract_type_arg_names(inner),
            // Array: extract const-generic size ident + element type args
            // (e.g. [N]T -> N, T; [3]Int -> Int)
            Type::Slice(elem) => Self::type_from_ast(elem), Type::Array(size_expr, elem) => {
                let mut names = vec![Self::type_from_ast(elem)];
                if let Expr::Ident(id) = size_expr.as_ref() {
                    names.insert(0, id.name.clone());
                }
                names
            }
            _ => vec![],
        }
    }

    /// Returns `true` when `name` is NOT a known primitive/scalar/container --
    /// i.e., it is a user-defined named struct that needs concrete monomorphisation
    /// inside Result/Option generic types (B-001).
    pub fn is_struct_type_name(name: &str) -> bool {
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
        if name.contains("__") { return false; }
        // Generic type parameters are single uppercase letters (T, K, V, E, etc.)
        if name.len() == 1 && name.chars().next().map_or(false, |c| c.is_uppercase()) {
            return false;
        }
        true
    }

    /// Convert a XIOM AST [`Type`] to a canonical LLVM type string.
    /// Handles named types, references, pointers, arrays, tuples, and containers.
    /// Used during codegen to determine struct layouts and function signatures.
    pub fn type_from_ast(ty: &Type) -> String {
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
            Type::Slice(elem) => Self::type_from_ast(elem), Type::Array(size_expr, elem) => {
                let elem_name = Self::type_from_ast(elem);
                match size_expr.as_ref() {
                    Expr::Int(n, _) => format!("[{n} x {elem_name}]"),
                    Expr::Ident(id) => format!("[{} x {elem_name}]", id.name),
                    _ => elem_name,
                }
            }
            _ => "Int".to_string(),
        }
    }

    /// Like `type_from_ast`, but preserves generic type arguments for Vec, Map, Set.
    /// Used for type_meta field registration so we can resolve element types at
    /// Vec index time (5c.21 Vec-of-struct fix).
    pub fn type_from_ast_with_args(ty: &Type) -> String {
        match ty {
            Type::Vec(inner) => format!("Vec[{}]", Self::type_from_ast_with_args(inner)),
            Type::Map(k, v) => format!("Map[{},{}]", Self::type_from_ast_with_args(k), Self::type_from_ast_with_args(v)),
            // B-007: keep a "fn(...)" MARKER for fn-typed fields/elements so
            // closure-valued container elements (Vec[fn()]) can be detected at
            // binding/call time -- the ABI still erases to i64.
            Type::Fn(params, ret) => format!("fn({}) -> {}", params.iter().map(Self::type_from_ast).collect::<Vec<_>>().join(", "), Self::type_from_ast(ret)),
            Type::Set(inner) => format!("Set[{}]", Self::type_from_ast_with_args(inner)),
            other => Self::type_from_ast(other),
        }
    }

    /// 5c.30: FULL type string including Option/Result payload args
    /// ("Result[Vec[Int], Str]"). Used ONLY for fn_return_xiom -- the field
    /// registration keeps type_from_ast_with_args so Option/Result struct
    /// fields keep their by-value layout.
    pub fn type_string_full(ty: &Type) -> String {
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
    /// -> "Vec[Int]").
    pub fn option_result_payload(s: &str) -> Option<String> {
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

    /// 5c.30: If `expr` is a `Vec[T].new()` / `Vec[T].with_capacity(..)` call,
    /// return the element type name `T` (from the explicit type argument).
    pub fn vec_ctor_elem_type(expr: &Expr) -> Option<String> {
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
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract the element type name from a `Vec[T]` type annotation.
    /// e.g. `Vec[Float64]` -> Some("Float64"), `Vec[Int]` -> Some("Int").
    pub fn vec_elem_from_type_annotation(ty: &Type) -> Option<String> {
        // Unwrap reference wrappers: catalog fns take `&Vec[T]`/`&mut Vec[T]`
        // params, which must STILL register the element type -- otherwise
        // `v[0]` on a `&Vec[Float64]` param falls back to the width-based i64
        // load + sitofp (the f64 bit pattern converted as an int, BUG 12) and
        // `Str` elements lose the strcmp equality lowering (BUG 17).
        let inner = match ty {
            Type::Ref(t) | Type::MutRef(t) | Type::Ptr(t) => t.as_ref(),
            other => other,
        };
        match inner {
            Type::Named(ident, type_args) if ident.name == "Vec" => {
                type_args.first().map(|t| Self::type_from_ast(t))
            }
            Type::Vec(inner) => Some(Self::type_from_ast(inner)),
            _ => None,
        }
    }

    /// 5c.29: If `container` is a struct-field access whose declared type is a
    /// float container (Vec[Float32] / Vec[Float64]), return the float LLVM
    /// type. Float Vec elements are stored as RAW BITS (val_to_i64 bitcast),
    /// so Index loads must bit-reinterpret instead of numerically converting.
    pub fn vec_elem_float_type(&self, container: &Expr) -> Option<&'static str> {
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

    pub fn resolve_vec_elem_type(&self, container: &Expr) -> Option<String> {
        // 5c.30: local Vec bindings (`var v = Vec[Point2D].new()`): the elem
        // type was recorded at the let/var binding. Only struct element types
        // are returned (primitives use the scalar elem_load path).
        if let Expr::Ident(id) = container {
            let elem = self.local.local_vec_elem.get(&id.name)
                .or_else(|| self.local.local_vec_handle.get(&id.name))?;
            if matches!(elem.as_str(), "Int" | "Bool" | "Str" | "Float64" | "Float32" | "UInt8" | "Int8" | "Int16" | "Int32" | "UInt16" | "UInt32" | "Char" | "Float") {
                return None;
            }
            return self.types.types.keys().into_iter()
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
                                        if let Some(qualified) = self.types.types.keys().into_iter()
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

    pub fn llvm_type_for(&self, type_name: &str) -> Result<String, String> {
        // Parse array types like [N x ElementType] -- used for fixed-size stack arrays.
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
                    // to resolve it as a struct name or builtin -- the caller will get
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
        if self.types.types.contains_key(&type_name.to_string()) || self.types.type_meta.contains_key(&type_name.to_string()) {
            return Ok(format!("%struct.{type_name}"));
        }
        // Search for any module-qualified variant ending with .type_name.
        // BUG 16-family fix (2026-08-11): skip GENERATED aggregate keys
        // (`Tuple__...`, `Option__...`, `Result__...`, `_Anon__...`) -- their
        // names end with `.Type` too (e.g. `Tuple__m.Big__m.Big` ends with
        // `.Big`), so a bare `Big` could resolve to the TUPLE key depending
        // on HashMap iteration order (regression: m37_tuple_struct emitted
        // `Tuple__Big__Big` whose fields were 3-element tuples -> llvm.trap).
        for (key, _) in self.types.type_meta.entries() {
            if key.ends_with(&format!(".{type_name}"))
                && !key.contains("Tuple__")
                && !key.starts_with("Option__")
                && !key.starts_with("Result__")
                && !key.starts_with("_Anon__")
            {
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
        // then suffix -- mirroring the struct lookup above.
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
            _ => Err(format!("unknown type '{}' -- not a registered struct, enum, or builtin", type_name)),
        }
    }

    /// Resolve type name to LLVM type, with suffix-search fallback for module-qualified types.
    /// Use this when the exact type registration is uncertain (e.g., type aliases from other modules).
    pub fn llvm_type_for_fallback(&self, type_name: &str) -> String {
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
                // Also check generic_type_names -- generic types may not
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
                LLVM_I64.to_string()
            }
        }
    }

    /// Compute the in-memory byte size of a struct using its declared field
    /// types (type_meta), recursing into nested structs. Generic-container
    /// fields contribute their own size -- the old `field_count x 8` math
    /// undercounted (JsonEntry { key: Str, value: JsonValue } is 24 bytes,
    /// not 16), truncating Vec elements on push/index.
    pub fn struct_byte_size(&self, type_name: &str) -> i64 {
        self.struct_byte_size_depth(type_name, 0)
    }

    pub fn struct_byte_size_depth(&self, type_name: &str, depth: u32) -> i64 {
        if depth > 8 {
            return 8;
        }
        let meta = self.types.type_meta.get(&type_name.to_string())
            .or_else(|| {
                self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{type_name}")))
                    .map(|(_, v)| v)
            });
        let Some(meta) = meta else { return 8 };
        let mut total = 0i64;
        for (_, fty) in meta.fields.iter() {
            // Generic containers are i64 handles (5c.28h).
            if fty.contains('[') && !fty.starts_with('[') {
                total += 8;
                continue;
            }
            // Fixed-size arrays `[N x T]`: N x 8.
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
            let llvm = self.llvm_type_for(fty).unwrap_or_else(|_| LLVM_I64.to_string());
            if llvm.starts_with("%struct.") && !llvm.ends_with('*') {
                let inner = llvm[8..].to_string();
                total += self.struct_byte_size_depth(&inner, depth + 1);
            } else {
                total += 8;
            }
        }
        if total == 0 { 8 } else { total }
    }

    pub fn field_llvm_type(&self, struct_name: &str, field_idx: usize) -> String {
        let meta = self.types.type_meta.get(&struct_name.to_string())
            .or_else(|| {
                // Try current module's qualified name first (deterministic)
                if let Some(ref module) = self.local.current_module {
                    let qualified = format!("{}.{}", module, struct_name);
                    self.types.type_meta.get(&qualified)
                } else {
        // Array literals: infer element type from first element.
        // e.g. [1.5, 2.5] -> Float64, [1, 2, 3] -> Int
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
            })
            .or_else(|| {
                // Fallback: search all qualified keys
                self.types.type_meta.entries().into_iter()
    .find(|(k, _)| k.ends_with(&format!(".{struct_name}")))
                    .map(|(_, v)| v)
            });
        if let Some(meta) = meta {
            if let Some((_, ty_name)) = meta.fields.get(field_idx) {
                // For generic types (Vec[Int], Map[Str,Int]), return i64
                // to avoid Win64 sret corruption (5c.28 NET crash fix).
                if ty_name.contains('[') {
                    return LLVM_I64.to_string();
                }
                return self.llvm_type_for(ty_name).unwrap_or_else(|_| LLVM_I64.to_string());
            }
        }
        LLVM_I64.to_string()
    }

    /// Returns the declared XIOM type name of field `field_idx` of `struct_name`
    /// from type_meta (e.g. "Vec[Int]"), using the same qualified-name fallbacks
    /// as `field_llvm_type`.
    pub fn field_xiom_type(&self, struct_name: &str, field_idx: usize) -> Option<String> {
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

    /// Returns true if `container` is a field access on a struct and the
    /// field's type in type_meta is a generic container (Vec[..], Map[..], etc.)
    pub fn is_container_vec_field(&self, container: &Expr) -> bool {
        // 5c.30: locals bound to an i64 container handle (match-arm payload
        // bindings like `JsonValue.Array(ref mut items)`).
        if let Expr::Ident(id) = container {
            return self.local.local_vec_handle.contains_key(&id.name);
        }
        if let Expr::Field(base, field_expr, _) = container {
            if let Some(base_ty) = self.infer_struct_type_name(base) {
                // round-7 (ve2 regression): two structs can share a leaf name --
                // search ALL suffix-matching keys (the old loop broke at the
                // first match, missing fields of the other type).
                for key in self.types.type_meta.keys() {
                    if key.ends_with(&base_ty) || key == &base_ty {
                        if let Some(meta) = self.types.type_meta.get(key) {
                            for (fname, ftype) in &meta.fields {
                                if fname == &field_expr.name {
                                    return ftype.contains('[');
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Returns `true` if `name` is the name of a variant of the enum currently
    /// being matched on. Used to decide whether a bare `Pattern::Ident` should
    /// be compiled as a runtime discriminant check (rather than a variable
    /// binding / wildcard).
    pub fn ident_is_enum_variant(&self, scrutinee_type: &Option<String>, name: &str) -> bool {
        if let Some(type_name) = scrutinee_type {
            // 5c.29: qualified variant idents (`SqliteValue.Null`) carry the
            // enum qualifier -- compare the LEAF segment. Also tolerate
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
    pub fn pattern_needs_check(&self, pattern: &Pattern, scrutinee_type: &Option<String>) -> bool {
        match pattern {
            Pattern::Lit(Literal::Int(..)) | Pattern::Lit(Literal::Float(..)) | Pattern::Lit(Literal::Bool(..))
            | Pattern::Lit(Literal::Str(..)) | Pattern::Lit(Literal::Char(..)) => true,
            Pattern::Variant(..) | Pattern::Struct(..) | Pattern::Tuple(..) => true,
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
    pub fn emit_variant_discriminant_check(
        &mut self,
        variant_name: &str,
        scrutinee_alloca_info: &Option<(String, String, String)>,
        val: &str,
        arm_label: &str,
        next: &str,
    ) {
        if let Some((alloca, type_name, struct_ty)) = scrutinee_alloca_info {
            // 5c.29: qualified variant patterns (`SqliteValue.Integer(v)`)
            // carry the enum qualifier in the name -- compare against the LEAF
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

    pub fn infer_struct_type_name(&self, expr: &Expr) -> Option<String> {
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
                // Fallback: search generic_type_names -- generic types may not
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
                // BASE type so the fn_key becomes "Map.new" -- previously the
                // bare-key fallback hijacked another module's generic `new`
                // (stub body returning 0 -> runtime crash in module-global
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
            Expr::Call(func, _, _) | Expr::GenericCall(func, _, _, _) => {
                // Infer type from the return type of a method/function call
                let fn_key = if let Expr::Field(obj, field, _) = func.as_ref() {
                    // Try module-qualified resolution first (e.g. iter.range -- xiom.iter.range)
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
            Expr::Index(base, _, _) => {
                // Strip Index wrapper for type-arg annotations like
                // `Map[Str, JsonValue].new()`. The base is the actual type name.
                self.infer_struct_type_name(base.as_ref())
            }
            _ => None,
        }
    }
}
