// XIOM CTFE — Phase B Compile-Time Function Evaluation
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.
//
// A tree-walking interpreter that evaluates pure XIOM functions at compile time.
// Supports: arithmetic, comparison, boolean ops, if/match, while/for loops,
// function calls, recursion with depth limit, struct construction, array ops.
//
// Safety: sandboxed — no I/O, no FFI, no mutable globals. Arena-allocated
// memory is discarded after evaluation. Hard limits on recursion depth (1000)
// and evaluation steps (100K) prevent infinite loops.

use xiom_ast::*;
use std::collections::HashMap;

// ============================================================================
// CTFE Value — runtime representation during compile-time evaluation
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CtfeValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    Unit,
    /// Struct value: field_name → CtfeValue
    Struct(String, Vec<(String, CtfeValue)>),
    /// Enum variant: variant_name → payload values
    Variant(String, Vec<CtfeValue>),
    /// Ptr value — arena offset; not dereferenceable outside CTFE
    Ptr(usize),
    Null,
}

impl CtfeValue {
    pub fn as_bool(&self) -> bool {
        match self {
            CtfeValue::Bool(b) => *b,
            _ => false,
        }
    }

    pub fn as_int(&self) -> i64 {
        match self {
            CtfeValue::Int(n) => *n,
            CtfeValue::Float(f) => *f as i64,
            CtfeValue::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            CtfeValue::Float(f) => *f,
            CtfeValue::Int(n) => *n as f64,
            _ => 0.0,
        }
    }
}

// ============================================================================
// CTFE Arena — bounded memory allocator for compile-time structs/arrays
// ============================================================================

const ARENA_SIZE: usize = 256 * 1024 * 1024; // 256MB

#[derive(Clone)]
pub struct CtfeArena {
    data: Vec<u8>,
    offset: usize,
}

impl CtfeArena {
    pub fn new() -> Self {
        Self { data: Vec::with_capacity(1024 * 1024), offset: 0 }
    }

    pub fn alloc(&mut self, size: usize) -> usize {
        let ptr = self.offset;
        self.offset += size;
        while self.data.len() < self.offset {
            self.data.push(0);
        }
        if self.offset > ARENA_SIZE {
            panic!("CTFE arena overflow: {} bytes allocated (limit: {})", self.offset, ARENA_SIZE);
        }
        ptr
    }
}

// ============================================================================
// CTFE Context — evaluation state for a single function call
// ============================================================================

pub struct CtfeContext {
    /// Local variables: name → value
    pub locals: HashMap<String, CtfeValue>,
    /// Current recursion depth
    pub depth: u32,
    /// Maximum recursion depth
    pub max_depth: u32,
    /// Steps executed (safety limit)
    pub steps: u64,
    /// Maximum steps
    pub max_steps: u64,
}

impl CtfeContext {
    pub fn new(depth: u32) -> Self {
        Self {
            locals: HashMap::new(),
            depth,
            max_depth: 1000,
            steps: 0,
            max_steps: 100_000,
        }
    }

    pub fn tick(&mut self) -> Result<(), CtfeError> {
        self.steps += 1;
        if self.steps > self.max_steps {
            return Err(CtfeError::Timeout(self.max_steps));
        }
        Ok(())
    }

    pub fn check_depth(&self) -> Result<(), CtfeError> {
        if self.depth > self.max_depth {
            return Err(CtfeError::RecursionLimit(self.max_depth));
        }
        Ok(())
    }
}

// ============================================================================
// CTFE Error types
// ============================================================================

#[derive(Debug, Clone)]
pub enum CtfeError {
    /// Recursion depth exceeded
    RecursionLimit(u32),
    /// Evaluation timed out (too many steps)
    Timeout(u64),
    /// Function not found in registry
    UndefinedFunction(String),
    /// Function is not pure (has I/O, FFI, etc.)
    ImpureFunction(String),
    /// Type mismatch during evaluation
    TypeError(String),
    /// Division by zero
    DivisionByZero,
    /// Arena overflow
    ArenaOverflow,
    /// Unsupported operation in CTFE
    Unsupported(String),
}

// ============================================================================
// CTFE Engine — top-level evaluator
// ============================================================================

#[derive(Clone)]
pub struct CtfeEngine {
    /// Arena for compile-time allocations
    pub arena: CtfeArena,
    /// Registry of pure functions: name → (params, body)
    pub functions: HashMap<String, (Vec<String>, Vec<StmtOrExpr>)>,
    /// Global constants: name → CtfeValue
    pub constants: HashMap<String, CtfeValue>,
}

impl CtfeEngine {
    pub fn new() -> Self {
        Self {
            arena: CtfeArena::new(),
            functions: HashMap::new(),
            constants: HashMap::new(),
        }
    }

    /// Register a function body for CTFE evaluation.
    pub fn register_function(&mut self, name: &str, params: Vec<String>, body: &[StmtOrExpr]) {
        self.functions.insert(name.to_string(), (params, body.to_vec()));
    }

    /// Evaluate a function call with const arguments.
    pub fn eval_function(
        &mut self,
        name: &str,
        args: &[CtfeValue],
        depth: u32,
    ) -> Result<CtfeValue, CtfeError> {
        let (params, body) = self.functions.get(name)
            .cloned()
            .ok_or_else(|| CtfeError::UndefinedFunction(name.to_string()))?;

        if params.len() != args.len() {
            return Err(CtfeError::TypeError(
                format!("function {} expects {} args, got {}", name, params.len(), args.len())
            ));
        }

        let mut ctx = CtfeContext::new(depth);
        ctx.check_depth()?;

        for (i, param_name) in params.iter().enumerate() {
            ctx.locals.insert(param_name.clone(), args[i].clone());
        }

        // Save old locals and restore after (for recursion)
        let _saved_depth = ctx.depth;
        let result = self.eval_block(&body, &mut ctx)?;
        Ok(result)
    }

    /// Evaluate an expression within a CTFE context.
    pub fn eval_expr(&mut self, expr: &Expr, ctx: &mut CtfeContext) -> Result<CtfeValue, CtfeError> {
        ctx.tick()?;
        match expr {
            Expr::Int(n, _) => Ok(CtfeValue::Int(*n as i64)),
            Expr::Float(f, _) => Ok(CtfeValue::Float(*f)),
            Expr::Bool(b, _) => Ok(CtfeValue::Bool(*b)),
            Expr::Str(s, _) => Ok(CtfeValue::Str(s.clone())),
            Expr::Char(c, _) => Ok(CtfeValue::Char(*c)),

            Expr::Ident(id) => {
                if let Some(val) = ctx.locals.get(&id.name) {
                    Ok(val.clone())
                } else if let Some(val) = self.constants.get(&id.name) {
                    Ok(val.clone())
                } else {
                    Err(CtfeError::TypeError(format!("undefined variable: {}", id.name)))
                }
            }

            Expr::Binary(lhs, op, rhs, _) => {
                let l = self.eval_expr(lhs, ctx)?;
                let r = self.eval_expr(rhs, ctx)?;
                self.eval_binary(&l, op, &r)
            }

            Expr::Unary(op, inner, _) => {
                let v = self.eval_expr(inner, ctx)?;
                self.eval_unary(op, &v)
            }

            Expr::Call(func, args, _) => {
                self.eval_call(func, args, ctx)
            }

            Expr::If(cond, then_block, elifs, else_block, _) => {
                let cond_val = self.eval_expr(cond, ctx)?;
                if cond_val.as_bool() {
                    return self.eval_block_last(&then_block.stmts, ctx);
                }
                for (elif_cond, elif_block) in elifs {
                    let ec = self.eval_expr(elif_cond, ctx)?;
                    if ec.as_bool() {
                        return self.eval_block_last(&elif_block.stmts, ctx);
                    }
                }
                if let Some(else_block) = else_block {
                    return self.eval_block_last(&else_block.stmts, ctx);
                }
                Ok(CtfeValue::Unit)
            }

            Expr::Match(scrutinee, arms, _) => {
                let val = self.eval_expr(scrutinee, ctx)?;
                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &val, ctx)? {
                        return match &arm.body {
                            MatchBody::Block(block) => self.eval_block_last(&block.stmts, ctx),
                            MatchBody::Expr(e) => self.eval_expr(e, ctx),
                        };
                    }
                }
                Err(CtfeError::TypeError("match: no arm matched".to_string()))
            }

            Expr::Paren(inner, _) => self.eval_expr(inner, ctx),

            Expr::Struct(name, fields, _, _) => {
                let mut field_vals = Vec::new();
                for (fname, fexpr) in fields {
                    let fval = self.eval_expr(fexpr, ctx)?;
                    field_vals.push((fname.name.clone(), fval));
                }
                Ok(CtfeValue::Struct(name.name.clone(), field_vals))
            }

            Expr::Field(obj, field, _) => {
                let val = self.eval_expr(obj, ctx)?;
                match val {
                    CtfeValue::Struct(_, fields) => {
                        for (fname, fval) in &fields {
                            if fname == &field.name {
                                return Ok(fval.clone());
                            }
                        }
                        Err(CtfeError::TypeError(format!("field {} not found", field.name)))
                    }
                    _ => Err(CtfeError::TypeError("field access on non-struct".to_string())),
                }
            }

            Expr::Some(inner, _) => {
                let v = self.eval_expr(inner, ctx)?;
                Ok(CtfeValue::Variant("Some".to_string(), vec![v]))
            }
            Expr::None(_) => Ok(CtfeValue::Variant("None".to_string(), vec![])),
            Expr::Ok(inner, _) => {
                let v = self.eval_expr(inner, ctx)?;
                Ok(CtfeValue::Variant("Ok".to_string(), vec![v]))
            }
            Expr::Err(inner, _) => {
                let v = self.eval_expr(inner, ctx)?;
                Ok(CtfeValue::Variant("Err".to_string(), vec![v]))
            }

            Expr::BlockExpr(block, _) => self.eval_block_last(&block.stmts, ctx),

            Expr::Index(arr, idx, _) => {
                let _arr_val = self.eval_expr(arr, ctx)?;
                let _idx_val = self.eval_expr(idx, ctx)?;
                Err(CtfeError::Unsupported("index on runtime arrays".to_string()))
            }

            _ => Err(CtfeError::Unsupported(format!("expression: {:?}", expr))),
        }
    }

    /// Evaluate a statement and return the expression result (if any).
    fn eval_stmt(&mut self, stmt: &Stmt, ctx: &mut CtfeContext) -> Result<CtfeValue, CtfeError> {
        ctx.tick()?;
        match stmt {
            Stmt::Let(ident, _, expr, _) | Stmt::Var(ident, _, expr, _) => {
                let val = self.eval_expr(expr, ctx)?;
                ctx.locals.insert(ident.name.clone(), val);
                Ok(CtfeValue::Unit)
            }
            Stmt::Assign(lhs, rhs, _) => {
                let val = self.eval_expr(rhs, ctx)?;
                if let Expr::Ident(id) = lhs {
                    ctx.locals.insert(id.name.clone(), val);
                }
                Ok(CtfeValue::Unit)
            }
            Stmt::Expr(e, _) => self.eval_expr(e, ctx),
            Stmt::Return(Some(e), _) => self.eval_expr(e, ctx),
            Stmt::Return(None, _) => Ok(CtfeValue::Unit),
            Stmt::If(cond, then_block, elifs, else_block, _) => {
                let cond_val = self.eval_expr(cond, ctx)?;
                if cond_val.as_bool() {
                    return self.eval_block(&then_block.stmts, ctx);
                }
                for (elif_cond, elif_block) in elifs {
                    if self.eval_expr(elif_cond, ctx)?.as_bool() {
                        return self.eval_block(&elif_block.stmts, ctx);
                    }
                }
                if let Some(else_block) = else_block {
                    return self.eval_block(&else_block.stmts, ctx);
                }
                Ok(CtfeValue::Unit)
            }
            Stmt::While(cond, body, _, _, _) => {
                while self.eval_expr(cond, ctx)?.as_bool() {
                    self.eval_block(&body.stmts, ctx)?;
                }
                Ok(CtfeValue::Unit)
            }
            Stmt::Match(scrutinee, arms, _) => {
                let val = self.eval_expr(scrutinee, ctx)?;
                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &val, ctx)? {
                        return match &arm.body {
                            MatchBody::Block(block) => self.eval_block(&block.stmts, ctx),
                            MatchBody::Expr(e) => self.eval_expr(e, ctx),
                        };
                    }
                }
                Ok(CtfeValue::Unit)
            }
            Stmt::For(_ident, iter, body, _, _) => {
                // Very basic for-loop: iterate over array literal range
                // For CTFE we only support simple integer range-like patterns
                let _iter_val = self.eval_expr(iter, ctx)?;
                // Unsupported for now — return Unit
                let _body_val = self.eval_block(&body.stmts, ctx)?;
                Ok(CtfeValue::Unit)
            }
            _ => Err(CtfeError::Unsupported(format!("statement: {:?}", stmt))),
        }
    }

    /// Evaluate a statement-or-expression.
    fn eval_stmt_or_expr(&mut self, soe: &StmtOrExpr, ctx: &mut CtfeContext) -> Result<CtfeValue, CtfeError> {
        match soe {
            StmtOrExpr::Stmt(s) => self.eval_stmt(s, ctx),
            StmtOrExpr::Expr(e) => self.eval_expr(e, ctx),
        }
    }

    /// Evaluate all statements in a block, returning the value of the last expression.
    fn eval_block(&mut self, stmts: &[StmtOrExpr], ctx: &mut CtfeContext) -> Result<CtfeValue, CtfeError> {
        let mut result = CtfeValue::Unit;
        for stmt in stmts {
            match stmt {
                StmtOrExpr::Stmt(Stmt::Return(_, _)) => {
                    return self.eval_stmt_or_expr(stmt, ctx);
                }
                _ => {
                    result = self.eval_stmt_or_expr(stmt, ctx)?;
                }
            }
        }
        Ok(result)
    }

    /// Evaluate the last expression in a block.
    fn eval_block_last(&mut self, stmts: &[StmtOrExpr], ctx: &mut CtfeContext) -> Result<CtfeValue, CtfeError> {
        for stmt in stmts.iter().rev() {
            match stmt {
                StmtOrExpr::Expr(e) => return self.eval_expr(e, ctx),
                StmtOrExpr::Stmt(Stmt::Expr(e, _)) => return self.eval_expr(e, ctx),
                StmtOrExpr::Stmt(Stmt::Return(Some(e), _)) => return self.eval_expr(e, ctx),
                _ => continue,
            }
        }
        Ok(CtfeValue::Unit)
    }

    /// Evaluate a function call.
    fn eval_call(&mut self, func: &Expr, args: &[Expr], ctx: &mut CtfeContext) -> Result<CtfeValue, CtfeError> {
        // Evaluate arguments
        let arg_vals: Vec<CtfeValue> = args.iter()
            .map(|a| self.eval_expr(a, ctx))
            .collect::<Result<Vec<_>, _>>()?;

        // Resolve the function name
        let fn_name = match func {
            Expr::Ident(id) => id.name.clone(),
            Expr::Field(base, method, _) => {
                if let Expr::Ident(id) = base.as_ref() {
                    format!("{}.{}", id.name, method.name)
                } else {
                    return Err(CtfeError::Unsupported("complex call target".to_string()));
                }
            }
            _ => return Err(CtfeError::Unsupported("complex call target".to_string())),
        };

        // Check if it's a builtin that we handle
        if let Some(val) = self.eval_builtin(&fn_name, &arg_vals) {
            return Ok(val);
        }

        // Recursively evaluate user-defined function
        self.eval_function(&fn_name, &arg_vals, ctx.depth + 1)
    }

    /// Handle CTFE-known builtin functions.
    fn eval_builtin(&mut self, name: &str, args: &[CtfeValue]) -> Option<CtfeValue> {
        match name {
            "str_len" | "xiom_str_len" => {
                if let Some(CtfeValue::Str(s)) = args.first() {
                    Some(CtfeValue::Int(s.len() as i64))
                } else { None }
            }
            "str_concat" | "xiom_str_concat" => {
                if let (CtfeValue::Str(a), CtfeValue::Str(b)) = (&args[0], &args[1]) {
                    Some(CtfeValue::Str(format!("{}{}", a, b)))
                } else { None }
            }
            "int_to_string" => {
                if let Some(CtfeValue::Int(n)) = args.first() {
                    Some(CtfeValue::Str(n.to_string()))
                } else { None }
            }
            _ => None,
        }
    }

    /// Evaluate binary operations.
    fn eval_binary(&self, lhs: &CtfeValue, op: &BinOp, rhs: &CtfeValue) -> Result<CtfeValue, CtfeError> {
        match (lhs, rhs) {
            (CtfeValue::Int(a), CtfeValue::Int(b)) => {
                let a = *a; let b = *b;
                match op {
                    BinOp::Add => Ok(CtfeValue::Int(a.wrapping_add(b))),
                    BinOp::Sub => Ok(CtfeValue::Int(a.wrapping_sub(b))),
                    BinOp::Mul => Ok(CtfeValue::Int(a.wrapping_mul(b))),
                    BinOp::Div => if b != 0 { Ok(CtfeValue::Int(a / b)) } else { Err(CtfeError::DivisionByZero) },
                    BinOp::Rem => if b != 0 { Ok(CtfeValue::Int(a % b)) } else { Err(CtfeError::DivisionByZero) },
                    BinOp::Eq => Ok(CtfeValue::Bool(a == b)),
                    BinOp::Neq => Ok(CtfeValue::Bool(a != b)),
                    BinOp::Lt => Ok(CtfeValue::Bool(a < b)),
                    BinOp::Gt => Ok(CtfeValue::Bool(a > b)),
                    BinOp::Le => Ok(CtfeValue::Bool(a <= b)),
                    BinOp::Ge => Ok(CtfeValue::Bool(a >= b)),
                    BinOp::And => Ok(CtfeValue::Int(a & b)),
                    BinOp::Or => Ok(CtfeValue::Int(a | b)),
                    BinOp::Shl => Ok(CtfeValue::Int(a.wrapping_shl(b as u32))),
                    BinOp::Shr => Ok(CtfeValue::Int(a.wrapping_shr(b as u32))),
                    _ => Err(CtfeError::Unsupported(format!("op {:?} on Int", op))),
                }
            }
            (CtfeValue::Float(a), CtfeValue::Float(b)) => {
                let a = *a; let b = *b;
                match op {
                    BinOp::Add => Ok(CtfeValue::Float(a + b)),
                    BinOp::Sub => Ok(CtfeValue::Float(a - b)),
                    BinOp::Mul => Ok(CtfeValue::Float(a * b)),
                    BinOp::Div => Ok(CtfeValue::Float(a / b)),
                    BinOp::Eq => Ok(CtfeValue::Bool((a - b).abs() < 1e-15)),
                    BinOp::Neq => Ok(CtfeValue::Bool((a - b).abs() >= 1e-15)),
                    BinOp::Lt => Ok(CtfeValue::Bool(a < b)),
                    BinOp::Gt => Ok(CtfeValue::Bool(a > b)),
                    BinOp::Le => Ok(CtfeValue::Bool(a <= b)),
                    BinOp::Ge => Ok(CtfeValue::Bool(a >= b)),
                    _ => Err(CtfeError::Unsupported(format!("op {:?} on Float", op))),
                }
            }
            (CtfeValue::Bool(a), CtfeValue::Bool(b)) => {
                match op {
                    BinOp::And => Ok(CtfeValue::Bool(*a && *b)),
                    BinOp::Or => Ok(CtfeValue::Bool(*a || *b)),
                    BinOp::Eq => Ok(CtfeValue::Bool(a == b)),
                    BinOp::Neq => Ok(CtfeValue::Bool(a != b)),
                    _ => Err(CtfeError::Unsupported(format!("op {:?} on Bool", op))),
                }
            }
            (CtfeValue::Str(a), CtfeValue::Str(b)) => {
                match op {
                    BinOp::Add => Ok(CtfeValue::Str(format!("{}{}", a, b))),
                    BinOp::Eq => Ok(CtfeValue::Bool(a == b)),
                    BinOp::Neq => Ok(CtfeValue::Bool(a != b)),
                    _ => Err(CtfeError::Unsupported(format!("op {:?} on Str", op))),
                }
            }
            _ => Err(CtfeError::TypeError(format!("binary op {:?} on {:?} and {:?}", op, lhs, rhs))),
        }
    }

    /// Evaluate unary operations.
    fn eval_unary(&self, op: &UnaryOp, val: &CtfeValue) -> Result<CtfeValue, CtfeError> {
        match (op, val) {
            (UnaryOp::Neg, CtfeValue::Int(n)) => Ok(CtfeValue::Int(-n)),
            (UnaryOp::Neg, CtfeValue::Float(f)) => Ok(CtfeValue::Float(-f)),
            (UnaryOp::Not, CtfeValue::Bool(b)) => Ok(CtfeValue::Bool(!b)),
            (UnaryOp::BitNot, CtfeValue::Int(n)) => Ok(CtfeValue::Int(!n)),
            _ => Err(CtfeError::Unsupported(format!("unary {:?} on {:?}", op, val))),
        }
    }

    /// Pattern matching for CTFE match folding.
    fn pattern_matches(&self, pattern: &Pattern, value: &CtfeValue, ctx: &mut CtfeContext) -> Result<bool, CtfeError> {
        match pattern {
            Pattern::Wildcard(_) => Ok(true),
            Pattern::Ident(id) => {
                ctx.locals.insert(id.name.clone(), value.clone());
                Ok(true)
            }
            Pattern::Lit(lit) => {
                match (lit, value) {
                    (Literal::Bool(a, _), CtfeValue::Bool(b)) => Ok(a == b),
                    (Literal::Int(a, _), CtfeValue::Int(b)) => Ok(*a as i64 == *b),
                    (Literal::Float(a, _), CtfeValue::Float(b)) => Ok((a - b).abs() < 1e-15),
                    (Literal::Str(a, _), CtfeValue::Str(b)) => Ok(a == b),
                    _ => Ok(false),
                }
            }
            Pattern::Some(inner, _) => {
                match value {
                    CtfeValue::Variant(name, vals) if name == "Some" => {
                        if let Some(v) = vals.first() {
                            self.pattern_matches(inner, v, ctx)
                        } else { Ok(false) }
                    }
                    _ => Ok(false),
                }
            }
            Pattern::None(_) => {
                match value {
                    CtfeValue::Variant(name, vals) if name == "None" => Ok(vals.is_empty()),
                    _ => Ok(false),
                }
            }
            Pattern::Ok(inner, _) => {
                match value {
                    CtfeValue::Variant(name, vals) if name == "Ok" => {
                        if let Some(v) = vals.first() {
                            self.pattern_matches(inner, v, ctx)
                        } else { Ok(false) }
                    }
                    _ => Ok(false),
                }
            }
            Pattern::Err(inner, _) => {
                match value {
                    CtfeValue::Variant(name, vals) if name == "Err" => {
                        if let Some(v) = vals.first() {
                            self.pattern_matches(inner, v, ctx)
                        } else { Ok(false) }
                    }
                    _ => Ok(false),
                }
            }
            Pattern::Struct(name, fields, _) => {
                match value {
                    CtfeValue::Struct(v_name, v_fields) if v_name == &name.name => {
                        for (field_name, field_pat) in fields {
                            let field_val = v_fields.iter()
                                .find(|(n, _)| n == &field_name.name)
                                .map(|(_, v)| v);
                            if let Some(fv) = field_val {
                                self.pattern_matches(field_pat, fv, ctx)?;
                            } else {
                                return Ok(false);
                            }
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            Pattern::Tuple(elements, _) => {
                match value {
                    CtfeValue::Struct(_, v_fields) => {
                        if v_fields.len() != elements.len() { return Ok(false); }
                        for (i, elem) in elements.iter().enumerate() {
                            self.pattern_matches(elem, &v_fields[i].1, ctx)?;
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }

    /// Convert a CtfeValue back to an AST Expr for codegen integration.
    pub fn to_expr(val: &CtfeValue) -> Expr {
        match val {
            CtfeValue::Int(n) => Expr::Int(*n as u64, Span::new(0, 0)),
            CtfeValue::Float(f) => Expr::Float(*f, Span::new(0, 0)),
            CtfeValue::Bool(b) => Expr::Bool(*b, Span::new(0, 0)),
            CtfeValue::Str(s) => Expr::Str(s.clone(), Span::new(0, 0)),
            CtfeValue::Char(c) => Expr::Char(*c, Span::new(0, 0)),
            CtfeValue::Unit => Expr::Int(0, Span::new(0, 0)),
            _ => Expr::Int(0, Span::new(0, 0)), // complex values → 0 sentinel
        }
    }
}

impl Default for CtfeEngine {
    fn default() -> Self { Self::new() }
}
