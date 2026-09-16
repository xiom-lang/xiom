// XIOM CTFE -- Compile-Time Function Evaluation (explicit-stack machine)
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// PRODUCTION REWRITE (2026-08-24, readiness plan Stage 1). Replaces the
// tree-walking recursive interpreter. Fixes the audited defect set:
//   - Native Rust recursion across user-function calls overflowed the host
//     stack BEFORE the depth check could fire (documented ICE). The evaluator
//     is a flat work-stack machine: user recursion grows a Vec of frames, so
//     RecursionLimit is reachable and yields a diagnostic, never a crash.
//   - Aggregate constants (structs/enums/arrays) were folded to an Int(0)
//     sentinel by to_expr -- silent wrong programs. try_to_expr reconstructs
//     the aggregate AST; Ptr/Null/Range have no faithful form and callers
//     fall back to UNEVALUATED (runtime materialization), never zero.
//   - `for` loops evaluated the body exactly once with the loop variable
//     unbound. Ranges (parser-desugared range/range_inclusive calls), array
//     literals and array values iterate correctly; unlabeled break/continue
//     supported (labels rejected with a clear error).
//   - Integer arithmetic wrapped silently. Const evaluation uses checked
//     arithmetic; overflow is a hard compile-time error (rustc precedent),
//     keeping one numeric story across checker/CTFE/runtime.
//   - Float equality used an epsilon (1e-15); now exact IEEE ==/!=.
//   - The step budget was per-call-frame (recursive chains multiplied it).
//     Fuel is session-wide on the engine.
//   - Block tail values were computed by evaluating ONLY the final
//     expression and skipping preceding statements. Blocks execute
//     sequentially; frame.last records each expression-statement value.
//
// Safety: sandboxed -- no I/O, no FFI, no filesystem/env access. Hard limits
// on frame depth (RecursionLimit) and session steps (Timeout).

use xiom_ast::*;
use std::collections::HashMap;

// ============================================================================
// CTFE Value -- runtime representation during compile-time evaluation
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum CtfeValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Char(char),
    Unit,
    /// Struct value: struct name + ordered (field_name, value) pairs
    Struct(String, Vec<(String, CtfeValue)>),
    /// Enum variant: variant name + payload values
    Variant(String, Vec<CtfeValue>),
    /// Fixed list of element values (array literals, `for` over arrays)
    Array(Vec<CtfeValue>),
    /// Half-open / inclusive integer range produced by range/range_inclusive
    Range(i64, i64, bool),
    /// Ptr value -- arena offset; not dereferenceable outside CTFE
    Ptr(usize),
    Null,
}

impl CtfeValue {
    pub fn as_bool(&self) -> bool {
        matches!(self, CtfeValue::Bool(true))
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

    fn type_name(&self) -> &'static str {
        match self {
            CtfeValue::Int(_) => "Int",
            CtfeValue::Float(_) => "Float",
            CtfeValue::Bool(_) => "Bool",
            CtfeValue::Str(_) => "Str",
            CtfeValue::Char(_) => "Char",
            CtfeValue::Unit => "Unit",
            CtfeValue::Struct(..) => "Struct",
            CtfeValue::Variant(..) => "Variant",
            CtfeValue::Array(..) => "Array",
            CtfeValue::Range(..) => "Range",
            CtfeValue::Ptr(_) => "Ptr",
            CtfeValue::Null => "Null",
        }
    }
}

// ============================================================================
// CTFE Arena -- bounded memory allocator for compile-time allocations
// ============================================================================

const ARENA_SIZE: usize = 256 * 1024 * 1024; // 256MB

#[derive(Clone)]
pub struct CtfeArena {
    pub(crate) data: Vec<u8>,
    offset: usize,
}

impl CtfeArena {
    pub fn new() -> Self {
        Self { data: Vec::with_capacity(1024 * 1024), offset: 0 }
    }

    /// Was a panic (audited): one hostile constant must not abort the
    /// compiler. Reported as a normal CTFE error instead.
    pub fn alloc(&mut self, size: usize) -> Result<usize, CtfeError> {
        let ptr = self.offset;
        self.offset += size;
        if self.offset > ARENA_SIZE {
            return Err(CtfeError::ArenaOverflow);
        }
        while self.data.len() < self.offset {
            self.data.push(0);
        }
        Ok(ptr)
    }
}

// ============================================================================
// CTFE Error types
// ============================================================================

#[derive(Debug, Clone)]
pub enum CtfeError {
    /// Frame depth exceeded (user recursion too deep at compile time)
    RecursionLimit(u32),
    /// Session-wide evaluation budget exhausted
    Timeout(u64),
    /// Integer arithmetic overflow (const overflow is a hard error)
    Overflow(String),
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
// Limits
// ============================================================================

/// max_depth is a REAL bound now (the machine does not grow the native stack
/// per user frame), so the full budget is safely reachable.
pub const DEFAULT_MAX_DEPTH: u32 = 1000;
pub const DEFAULT_MAX_STEPS: u64 = 1_000_000;

// ============================================================================
// CTFE Engine -- top-level evaluator
// ============================================================================

#[derive(Clone)]
pub struct CtfeEngine {
    /// Arena for compile-time allocations
    pub arena: CtfeArena,
    /// Registry of pure functions: name -> (param names, body)
    pub functions: HashMap<String, (Vec<String>, Vec<StmtOrExpr>)>,
    /// Global constants: name -> CtfeValue
    pub constants: HashMap<String, CtfeValue>,
    /// Session-wide fuel counter (shared across ALL frames of one eval)
    pub steps: u64,
    /// Session-wide fuel budget
    pub max_steps: u64,
    /// Maximum simultaneous call frames
    pub max_depth: u32,
}

impl CtfeEngine {
    pub fn new() -> Self {
        Self {
            arena: CtfeArena::new(),
            functions: HashMap::new(),
            constants: HashMap::new(),
            steps: 0,
            max_steps: DEFAULT_MAX_STEPS,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    /// Register a function body for CTFE evaluation.
    pub fn register_function(&mut self, name: &str, params: Vec<String>, body: &[StmtOrExpr]) {
        self.functions.insert(name.to_string(), (params, body.to_vec()));
    }

    /// Evaluate a function call with constant arguments.
    ///
    /// The `depth` parameter is retained for API compatibility with existing
    /// callers (codegen passes 0); frame growth is tracked internally against
    /// `max_depth`.
    pub fn eval_function(
        &mut self,
        name: &str,
        args: &[CtfeValue],
        depth: u32,
    ) -> Result<CtfeValue, CtfeError> {
        let _ = depth;
        let (params, body) = self.functions.get(name)
            .cloned()
            .ok_or_else(|| CtfeError::UndefinedFunction(name.to_string()))?;

        if params.len() != args.len() {
            return Err(CtfeError::TypeError(format!(
                "function {} expects {} args, got {}", name, params.len(), args.len())));
        }

        let mut m = Machine::new(
            &self.functions, &self.constants, self.steps, self.max_steps, self.max_depth);
        let b = m.intern(body);
        {
            let f0 = &mut m.frames[0];
            f0.name = name.to_string();
            for (p, a) in params.iter().zip(args.iter()) {
                f0.locals.insert(p.clone(), a.clone());
            }
        }
        // LIFO: Seq runs first; the boundary then delivers frame.last.
        m.work.push(Task::EndFrame(FrameExit::Function));
        m.work.push(Task::Seq { b, idx: 0, want_value: false });
        let outcome = m.run();
        self.steps = m.steps;
        outcome
    }

    /// Evaluate a bare expression (no user function frame).
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<CtfeValue, CtfeError> {
        let mut m = Machine::new(
            &self.functions, &self.constants, self.steps, self.max_steps, self.max_depth);
        let b = m.intern(vec![StmtOrExpr::Expr(expr.clone())]);
        m.work.push(Task::EndFrame(FrameExit::Function));
        m.work.push(Task::Seq { b, idx: 0, want_value: false });
        let outcome = m.run();
        self.steps = m.steps;
        outcome
    }

    /// Convert a CTFE value back to an AST expression for codegen.
    ///
    /// Returns None for values with no faithful AST representation (Ptr,
    /// Null, ranges). Callers MUST fall back to leaving the original
    /// expression unevaluated (runtime materialization) -- fabricating a
    /// zero here was the audited silent-corruption bug.
    pub fn try_to_expr(val: &CtfeValue) -> Option<Expr> {
        let sp = Span::new(0, 0);
        Some(match val {
            CtfeValue::Int(n) => Expr::Int(*n as u64, sp),
            CtfeValue::Float(f) => Expr::Float(*f, sp),
            CtfeValue::Bool(b) => Expr::Bool(*b, sp),
            CtfeValue::Str(s) => Expr::Str(s.clone(), sp),
            CtfeValue::Char(c) => Expr::Char(*c, sp),
            // Unit has no value form; Int(0) preserves the historic ABI slot.
            CtfeValue::Unit => Expr::Int(0, sp),
            CtfeValue::Struct(name, fields) => {
                let mut out = Vec::with_capacity(fields.len());
                for (fname, fval) in fields {
                    out.push((Ident::new(fname.clone(), sp), Self::try_to_expr(fval)?));
                }
                Expr::Struct(Ident::new(name.clone(), sp), out, None, sp)
            }
            CtfeValue::Variant(name, payloads) => match (name.as_str(), payloads.as_slice()) {
                ("Some", [v]) => Expr::Some(Box::new(Self::try_to_expr(v)?), sp),
                ("None", []) => Expr::None(sp),
                ("Ok", [v]) => Expr::Ok(Box::new(Self::try_to_expr(v)?), sp),
                ("Err", [v]) => Expr::Err(Box::new(Self::try_to_expr(v)?), sp),
                // Custom enum variants cannot be reconstructed faithfully:
                // the value model does not record constructor arity/types.
                _ => return None,
            },
            CtfeValue::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for it in items {
                    out.push(Self::try_to_expr(it)?);
                }
                Expr::Array(out, sp)
            }
            CtfeValue::Range(..) | CtfeValue::Ptr(_) | CtfeValue::Null => return None,
        })
    }
}

impl Default for CtfeEngine {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// The machine
// ============================================================================

type BlockId = usize;
type ExprId = usize;

struct Frame {
    #[allow(dead_code)]
    name: String,
    locals: HashMap<String, CtfeValue>,
    /// Value of the most recent expression-statement in this frame. Doubles
    /// as block-tail value AND explicit-return carrier.
    last: CtfeValue,
}

enum FrameExit {
    /// A call-frame boundary. Return unwinds here; the frame's tail becomes
    /// the callee result. Break/Continue may NOT cross it (hard error).
    Function,
    /// A loop boundary. Break/Continue unwind here; Return passes through.
    Loop(usize),
}

enum Signal {
    Return,
    Break,
    Continue,
}

enum Task {
    Lit(CtfeValue),
    Eval(ExprId),
    /// Pop rhs then lhs, apply a binary op (checked arithmetic).
    BinOp(BinOp),
    /// Pop lhs; if true evaluate rhs into the value, else push false.
    AndRhs(ExprId),
    /// Pop lhs; if false evaluate rhs into the value, else push true.
    OrRhs(ExprId),
    UnOp(UnaryOp),
    /// Pop a payload, wrap in Some/Ok/Err.
    WrapVariant(&'static str),
    WrapNone,
    /// Pop `count` field values (reverse order), assemble the struct.
    BuildStruct { name: String, names: Vec<String>, count: usize },
    /// Pop a struct value, extract the named field.
    FieldAccess(String),
    /// Pop idx then array; push the element.
    IndexOp,
    /// Pop `count` element values (reverse), assemble the array.
    BuildArray(usize),
    /// Invoke a call target: pop nargs args (reverse), dispatch
    /// builtins/range/user-fn.
    Invoke { func: Box<Expr>, nargs: usize },
    /// Run block #b sequentially; optionally yield its tail value.
    Seq { b: BlockId, idx: usize, want_value: bool },
    StoreLocal(String),
    AssignLocal(String),
    /// Copy frame.last onto the value stack (block tail expressions).
    TakeLast,
    /// Pop value, record it as frame.last, push it back.
    RecordLast,
    Discard,
    /// Pop condition, branch to then/elif-chain/else.
    Branch { then_b: BlockId, elifs: Box<Vec<(ExprId, BlockId)>>, else_b: Option<BlockId>, want_value: bool },
    /// Pop elif condition; on false continue down the chain.
    ElifNext { next_b: BlockId, elifs: Box<Vec<(ExprId, BlockId)>>, else_b: Option<BlockId>, want_value: bool },
    /// Pop scrutinee, select first matching arm, bind, execute its body.
    Arms { arms: Box<Vec<ArmSpec>>, want_value: bool },
    /// Register loop context #i (index precomputed by the issuer).
    LoopStart(usize, Box<ActiveLoop>),
    /// Pop while-condition; start an iteration or end the loop.
    WhileCheck(usize),
    /// Advance a for-loop (deferred-init variants consume their operands).
    LoopTick(usize),
    /// Iteration finished normally (or continue landed): advance.
    IterDone(usize),
    /// Loop finished: unregister context.
    LoopEnd(usize),
    /// Unwind boundary.
    EndFrame(FrameExit),
    /// Pop value, store as frame.last, raise Return.
    DoReturn,
}

struct ArmSpec {
    pattern: Pattern,
    body: BlockId,
}

#[derive(Clone)]
enum ActiveLoop {
    While { cond: ExprId, body: BlockId },
    ForRange { var: String, next: i64, end: i64, inclusive: bool, body: BlockId },
    ForArray { var: String, items: Vec<CtfeValue>, idx: usize, body: BlockId },
    /// Deferred registration: pops range end then start from the value stack.
    RangeInit { var: String, inclusive: bool, body: BlockId },
    /// Deferred registration: pops `count` element values.
    ArrayInit { var: String, count: usize, body: BlockId },
}

struct Machine<'e> {
    vals: Vec<CtfeValue>,
    work: Vec<Task>,
    frames: Vec<Frame>,
    loops: Vec<ActiveLoop>,
    signal: Option<Signal>,
    result: Option<CtfeValue>,
    blocks: Vec<Box<[StmtOrExpr]>>,
    exprs: Vec<Expr>,
    funcs: &'e HashMap<String, (Vec<String>, Vec<StmtOrExpr>)>,
    consts: &'e HashMap<String, CtfeValue>,
    steps: u64,
    max_steps: u64,
    max_depth: u32,
}

impl<'e> Machine<'e> {
    fn new(
        funcs: &'e HashMap<String, (Vec<String>, Vec<StmtOrExpr>)>,
        consts: &'e HashMap<String, CtfeValue>,
        steps: u64,
        max_steps: u64,
        max_depth: u32,
    ) -> Self {
        Self {
            vals: Vec::new(),
            work: Vec::new(),
            frames: vec![Frame { name: String::from("<entry>"), locals: HashMap::new(), last: CtfeValue::Unit }],
            loops: Vec::new(),
            signal: None,
            result: None,
            blocks: Vec::new(),
            exprs: Vec::new(),
            funcs,
            consts,
            steps,
            max_steps,
            max_depth,
        }
    }

    fn intern(&mut self, body: Vec<StmtOrExpr>) -> BlockId {
        self.blocks.push(body.into_boxed_slice());
        self.blocks.len() - 1
    }

    fn intern_expr(&mut self, e: &Expr) -> ExprId {
        self.exprs.push(e.clone());
        self.exprs.len() - 1
    }

    fn tick(&mut self) -> Result<(), CtfeError> {
        self.steps += 1;
        if self.steps > self.max_steps {
            return Err(CtfeError::Timeout(self.max_steps));
        }
        Ok(())
    }

    fn internal(&self, what: &str) -> CtfeError {
        CtfeError::TypeError(format!("CTFE internal error: {} (value stack empty)", what))
    }

    fn frame(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("machine always keeps one frame")
    }

    fn run(&mut self) -> Result<CtfeValue, CtfeError> {
        loop {
            self.tick()?;
            let Some(task) = self.work.pop() else { break };
            // While a control-flow signal is live, discard everything except
            // boundaries -- intermediate continuations must not observe a
            // half-unwound value stack.
            if self.signal.is_some() && !matches!(task, Task::EndFrame(_)) {
                continue;
            }
            if !self.step(task)? {
                break;
            }
        }
        self.result.take()
            .ok_or_else(|| CtfeError::TypeError("CTFE machine finished without a result".into()))
    }

    /// Execute one task. Returns false when the ENTRY frame exits.
    fn step(&mut self, task: Task) -> Result<bool, CtfeError> {
        match task {
            Task::Lit(v) => self.vals.push(v),

            Task::Eval(id) => {
                let e = self.exprs[id].clone();
                self.eval_dispatch(e)?;
            }

            Task::BinOp(op) => {
                let r = self.vals.pop().ok_or_else(|| self.internal("binop rhs"))?;
                let l = self.vals.pop().ok_or_else(|| self.internal("binop lhs"))?;
                self.vals.push(eval_binary_checked(l, op, r)?);
            }

            Task::AndRhs(rhs) => {
                let l = self.vals.pop().ok_or_else(|| self.internal("and lhs"))?;
                if l.as_bool() {
                    self.work.push(Task::RecordLast);
                    self.work.push(Task::Eval(rhs));
                } else {
                    self.vals.push(CtfeValue::Bool(false));
                    self.work.push(Task::RecordLast);
                }
            }

            Task::OrRhs(rhs) => {
                let l = self.vals.pop().ok_or_else(|| self.internal("or lhs"))?;
                if l.as_bool() {
                    self.vals.push(CtfeValue::Bool(true));
                    self.work.push(Task::RecordLast);
                } else {
                    self.work.push(Task::RecordLast);
                    self.work.push(Task::Eval(rhs));
                }
            }

            Task::UnOp(op) => {
                let v = self.vals.pop().ok_or_else(|| self.internal("unop operand"))?;
                self.vals.push(eval_unary_checked(op, v)?);
            }

            Task::WrapVariant(n) => {
                let v = self.vals.pop().ok_or_else(|| self.internal("variant payload"))?;
                self.vals.push(CtfeValue::Variant(n.to_string(), vec![v]));
            }

            Task::WrapNone => self.vals.push(CtfeValue::Variant("None".into(), vec![])),

            Task::BuildStruct { name, names, count } => {
                if self.vals.len() < count {
                    return Err(self.internal("struct assembly"));
                }
                let start = self.vals.len() - count;
                // Fields were evaluated left-to-right, so drain order IS
                // declaration order.
                let vals: Vec<CtfeValue> = self.vals.drain(start..).collect();
                let fields: Vec<(String, CtfeValue)> =
                    names.into_iter().zip(vals.into_iter()).collect();
                self.vals.push(CtfeValue::Struct(name, fields));
            }

            Task::FieldAccess(fname) => {
                let obj = self.vals.pop().ok_or_else(|| self.internal("field receiver"))?;
                match obj {
                    CtfeValue::Struct(_, fields) => match fields.into_iter().find(|(n, _)| n == &fname) {
                        Some((_, v)) => self.vals.push(v),
                        None => return Err(CtfeError::TypeError(format!("field {} not found", fname))),
                    },
                    other => return Err(CtfeError::TypeError(format!(
                        "field access on non-struct ({})", other.type_name()))),
                }
            }

            Task::IndexOp => {
                let idx_v = self.vals.pop().ok_or_else(|| self.internal("index"))?;
                let arr_v = self.vals.pop().ok_or_else(|| self.internal("indexed"))?;
                let i = match idx_v {
                    CtfeValue::Int(i) if i >= 0 => i as usize,
                    other => return Err(CtfeError::TypeError(format!(
                        "array index must be a non-negative Int, got {}", other.type_name()))),
                };
                match arr_v {
                    CtfeValue::Array(items) => {
                        let n = items.len();
                        items.into_iter().nth(i)
                            .map(|v| self.vals.push(v))
                            .ok_or_else(|| CtfeError::TypeError(format!(
                                "index {} out of bounds (len {})", i, n)))?;
                    }
                    other => return Err(CtfeError::TypeError(format!(
                        "indexing unsupported on {} in const evaluation", other.type_name()))),
                }
            }

            Task::BuildArray(count) => {
                if self.vals.len() < count {
                    return Err(self.internal("array assembly"));
                }
                let start = self.vals.len() - count;
                let items: Vec<CtfeValue> = self.vals.drain(start..).collect();
                self.vals.push(CtfeValue::Array(items));
            }

            Task::Invoke { func, nargs } => self.invoke(func, nargs)?,

            Task::StoreLocal(name) => {
                let v = self.vals.pop().ok_or_else(|| self.internal("store local"))?;
                self.frame().locals.insert(name, v);
            }

            Task::AssignLocal(name) => {
                let v = self.vals.pop().ok_or_else(|| self.internal("assign local"))?;
                match self.frame().locals.get_mut(&name) {
                    Some(slot) => *slot = v,
                    None => return Err(CtfeError::TypeError(format!(
                        "assignment to undefined variable: {}", name))),
                }
            }

            Task::TakeLast => {
                let v = self.frame().last.clone();
                self.vals.push(v);
            }

            Task::RecordLast => {
                let v = self.vals.pop().ok_or_else(|| self.internal("record last"))?;
                self.frame().last = v.clone();
                self.vals.push(v);
            }

            Task::Discard => { self.vals.pop(); }

            Task::Branch { then_b, elifs, else_b, want_value } => {
                let cond = self.vals.pop().ok_or_else(|| self.internal("if condition"))?;
                if cond.as_bool() {
                    self.work.push(Task::Seq { b: then_b, idx: 0, want_value });
                } else if let Some((ec, eb)) = elifs.first().cloned() {
                    let rest: Vec<(ExprId, BlockId)> = elifs.iter().skip(1).cloned().collect();
                    self.work.push(Task::ElifNext {
                        next_b: eb, elifs: Box::new(rest), else_b, want_value,
                    });
                    self.work.push(Task::Eval(ec));
                } else if let Some(eb) = else_b {
                    self.work.push(Task::Seq { b: eb, idx: 0, want_value });
                } else if want_value {
                    self.work.push(Task::RecordLast);
                    self.vals.push(CtfeValue::Unit);
                }
            }

            Task::ElifNext { next_b, elifs, else_b, want_value } => {
                let cond = self.vals.pop().ok_or_else(|| self.internal("elif condition"))?;
                if cond.as_bool() {
                    self.work.push(Task::Seq { b: next_b, idx: 0, want_value });
                } else if let Some((ec, eb)) = elifs.first().cloned() {
                    let rest: Vec<(ExprId, BlockId)> = elifs.iter().skip(1).cloned().collect();
                    self.work.push(Task::ElifNext {
                        next_b: eb, elifs: Box::new(rest), else_b, want_value,
                    });
                    self.work.push(Task::Eval(ec));
                } else if let Some(eb) = else_b {
                    self.work.push(Task::Seq { b: eb, idx: 0, want_value });
                } else if want_value {
                    self.work.push(Task::RecordLast);
                    self.vals.push(CtfeValue::Unit);
                }
            }

            Task::Arms { arms, want_value } => {
                let val = self.vals.pop().ok_or_else(|| self.internal("match scrutinee"))?;
                let mut binds: HashMap<String, CtfeValue> = HashMap::new();
                for arm in arms.iter() {
                    if pattern_matches_into(&arm.pattern, &val, &mut binds)? {
                        self.frame().locals.extend(binds);
                        self.work.push(Task::Seq { b: arm.body, idx: 0, want_value });
                        return Ok(true);
                    }
                    binds.clear();
                }
                return Err(CtfeError::TypeError("match: no arm matched".into()));
            }

            Task::LoopStart(i, active) => {
                if i != self.loops.len() {
                    return Err(self.internal("loop registration index mismatch"));
                }
                self.loops.push(*active);
                // While conditions are evaluated by the issuer (Eval +
                // WhileCheck already queued beneath us); for-loops tick here.
                if !matches!(self.loops[i], ActiveLoop::While { .. }) {
                    self.work.push(Task::LoopTick(i));
                }
            }

            Task::WhileCheck(i) => {
                if i >= self.loops.len() {
                    // Stranded advance from an already-ended cycle (see
                    // LoopEnd): neutralize silently.
                    return Ok(true);
                }
                let cond = self.vals.pop().ok_or_else(|| self.internal("while condition"))?;
                if cond.as_bool() {
                    self.start_iteration(i)?;
                } else {
                    self.work.push(Task::LoopEnd(i));
                }
            }

            Task::LoopTick(i) => {
                if i >= self.loops.len() {
                    // Stranded advance from an already-ended cycle.
                    return Ok(true);
                }
                // Clone the state to keep the loops borrow disjoint from
                // frame/work mutations below.
                let state = self.loops[i].clone();
                match state {
                    ActiveLoop::While { .. } => {
                        return Err(self.internal("bare tick on while loop"));
                    }
                    ActiveLoop::RangeInit { var, inclusive, body } => {
                        let b_end = self.vals.pop().ok_or_else(|| self.internal("range end"))?;
                        let b_start = self.vals.pop().ok_or_else(|| self.internal("range start"))?;
                        let (start, end) = match (int_of(&b_start), int_of(&b_end)) {
                            (Some(a), Some(b)) => (a, b),
                            _ => return Err(CtfeError::TypeError("range bounds must be Int".into())),
                        };
                        self.loops[i] = ActiveLoop::ForRange {
                            var, next: start, end, inclusive, body,
                        };
                        self.work.push(Task::LoopTick(i));
                    }
                    ActiveLoop::ArrayInit { var, count, body } => {
                        if self.vals.len() < count {
                            return Err(self.internal("for-array init"));
                        }
                        let start = self.vals.len() - count;
                        let items: Vec<CtfeValue> = self.vals.drain(start..).collect();
                        self.loops[i] = ActiveLoop::ForArray { var, items, idx: 0, body };
                        self.work.push(Task::LoopTick(i));
                    }
                    ActiveLoop::ForRange { var, next, end, inclusive, .. } => {
                        let done = if inclusive { next > end } else { next >= end };
                        if done {
                            self.work.push(Task::LoopEnd(i));
                        } else {
                            if let ActiveLoop::ForRange { next: n, .. } = &mut self.loops[i] {
                                *n = next + 1;
                            }
                            self.frame().locals.insert(var, CtfeValue::Int(next));
                            self.start_iteration(i)?;
                        }
                    }
                    ActiveLoop::ForArray { var, items, mut idx, .. } => {
                        if idx >= items.len() {
                            self.work.push(Task::LoopEnd(i));
                        } else {
                            let v = items[idx].clone();
                            idx += 1;
                            if let ActiveLoop::ForArray { idx: n, .. } = &mut self.loops[i] {
                                *n = idx;
                            }
                            self.frame().locals.insert(var, v);
                            self.start_iteration(i)?;
                        }
                    }
                }
            }

            Task::IterDone(i) => {
                self.work.push(Task::LoopTick(i));
            }

            Task::LoopEnd(i) => {
                if i < self.loops.len() {
                    self.loops.truncate(i);
                }
            }

            Task::Seq { b, mut idx, want_value } => {
                let len = self.blocks[b].len();
                if self.signal.is_some() {
                    // Live signal: stop this block; unwinding continues above.
                    return Ok(true);
                }
                if idx < len {
                    let s = self.blocks[b][idx].clone();
                    idx += 1;
                    self.work.push(Task::Seq { b, idx, want_value });
                    match s {
                        StmtOrExpr::Stmt(st) => self.exec_stmt(st)?,
                        // Bare expression-statement: record as the block
                        // tail candidate (same path as Stmt::Expr).
                        StmtOrExpr::Expr(e) => {
                            let eid = self.intern_expr(&e);
                            self.work.push(Task::Discard);
                            self.work.push(Task::RecordLast);
                            self.work.push(Task::Eval(eid));
                        }
                    }
                    return Ok(true);
                }
                // Block finished normally.
                if want_value {
                    self.work.push(Task::TakeLast);
                }
            }

            Task::EndFrame(exit) => match exit {
                FrameExit::Function => {
                    // Any live signal here can only be Return (run-loop skips
                    // make sure intermediate tasks never ran); consume it.
                    self.signal.take();
                    let tail = self.frame().last.clone();
                    self.frames.pop();
                    if self.frames.is_empty() {
                        self.result = Some(tail);
                        return Ok(false);
                    }
                    // Nested callee: hand the result to the caller's stack.
                    self.vals.push(tail);
                }
                FrameExit::Loop(i) => match self.signal.take() {
                    Some(Signal::Break) => { self.work.push(Task::LoopEnd(i)); }
                    Some(Signal::Continue) => { self.work.push(Task::IterDone(i)); }
                    // Return passes THROUGH: keep the signal live and simply
                    // drop this boundary -- unwinding continues to the next
                    // one. (Re-pushing here would loop forever.)
                    other => { self.signal = other; }
                },
            },

            Task::DoReturn => {
                let v = self.vals.pop().unwrap_or(CtfeValue::Unit);
                self.frame().last = v;
                self.signal = Some(Signal::Return);
            }
        }
        Ok(true)
    }

    /// Push the iteration machinery for loop #i whose condition/advance state
    /// is already valid. LIFO execution order: body, then the loop boundary
    /// (catches signals from inside the body), then the advance continuation.
    fn start_iteration(&mut self, i: usize) -> Result<(), CtfeError> {
        match &self.loops[i] {
            ActiveLoop::While { cond, .. } => {
                let cid = *cond;
                // LIFO: Eval computes the condition, WhileCheck consumes it.
                self.work.push(Task::WhileCheck(i));
                self.work.push(Task::Eval(cid));
            }
            ActiveLoop::ForRange { .. } | ActiveLoop::ForArray { .. } => {
                self.work.push(Task::IterDone(i));
            }
            ActiveLoop::RangeInit { .. } | ActiveLoop::ArrayInit { .. } => {
                return Err(self.internal("iteration started on deferred init"));
            }
        }
        let body = match &self.loops[i] {
            ActiveLoop::While { body, .. }
            | ActiveLoop::ForRange { body, .. }
            | ActiveLoop::ForArray { body, .. } => *body,
            _ => unreachable!(),
        };
        self.work.push(Task::EndFrame(FrameExit::Loop(i)));
        self.work.push(Task::Seq { b: body, idx: 0, want_value: false });
        Ok(())
    }

    fn exec_stmt(&mut self, s: Stmt) -> Result<(), CtfeError> {
        match s {
            Stmt::Let(id, _, e, _) | Stmt::Var(id, _, e, _) => {
                let eid = self.intern_expr(&e);
                self.work.push(Task::StoreLocal(id.name));
                self.work.push(Task::Eval(eid));
            }
            Stmt::Assign(lhs, rhs, _) => match lhs {
                Expr::Ident(id) => {
                    let rid = self.intern_expr(&rhs);
                    self.work.push(Task::AssignLocal(id.name));
                    self.work.push(Task::Eval(rid));
                }
                other => return Err(CtfeError::Unsupported(format!(
                    "assignment to non-variable target: {:?}", other))),
            },
            Stmt::Expr(e, _) => {
                let eid = self.intern_expr(&e);
                self.work.push(Task::Discard);
                self.work.push(Task::RecordLast);
                self.work.push(Task::Eval(eid));
            }
            Stmt::Return(Some(e), _) => {
                let eid = self.intern_expr(&e);
                self.work.push(Task::DoReturn);
                self.work.push(Task::Eval(eid));
            }
            Stmt::Return(None, _) => {
                self.frame().last = CtfeValue::Unit;
                self.signal = Some(Signal::Return);
            }
            Stmt::If(cond, then_b, elifs, else_b, _) => {
                let tb = self.intern(then_b.stmts.to_vec());
                let mut ei = Vec::with_capacity(elifs.len());
                for (ec, eb) in elifs {
                    let ecid = self.intern_expr(&ec);
                    let ebid = self.intern(eb.stmts.to_vec());
                    ei.push((ecid, ebid));
                }
                let ob = else_b.map(|b| self.intern(b.stmts.to_vec()));
                let cid = self.intern_expr(&cond);
                self.work.push(Task::Branch {
                    then_b: tb, elifs: Box::new(ei), else_b: ob, want_value: false,
                });
                self.work.push(Task::Eval(cid));
            }
            Stmt::While(cond, body, _, _, _) => {
                let cid = self.intern_expr(&cond);
                let bb = self.intern(body.stmts.to_vec());
                let i = self.loops.len();
                // LIFO execution: LoopStart registers, then Eval computes the
                // first condition value, then WhileCheck consumes it.
                self.work.push(Task::WhileCheck(i));
                self.work.push(Task::Eval(cid));
                self.work.push(Task::LoopStart(i, Box::new(ActiveLoop::While { cond: cid, body: bb })));
            }
            Stmt::For(var, iter, body, _, label) => {
                if label.is_some() {
                    return Err(CtfeError::Unsupported(
                        "labeled for loops are not supported in const evaluation".into()));
                }
                let bb = self.intern(body.stmts.to_vec());
                self.setup_for(var.name, &iter, bb)?;
            }
            Stmt::Match(scrutinee, arms, _) => {
                let specs: Vec<ArmSpec> = arms.iter()
                    .map(|a| ArmSpec {
                        pattern: a.pattern.clone(),
                        body: self.intern(match_body_stmts(&a.body)),
                    })
                    .collect();
                let sid = self.intern_expr(&scrutinee);
                self.work.push(Task::Arms { arms: Box::new(specs), want_value: false });
                self.work.push(Task::Eval(sid));
            }
            Stmt::Break(label, _) => {
                if label.is_some() {
                    return Err(CtfeError::Unsupported(
                        "labeled break is not supported in const evaluation".into()));
                }
                if self.loops.is_empty() {
                    return Err(CtfeError::Unsupported("break outside of a loop".into()));
                }
                self.signal = Some(Signal::Break);
            }
            Stmt::Continue(label, _) => {
                if label.is_some() {
                    return Err(CtfeError::Unsupported(
                        "labeled continue is not supported in const evaluation".into()));
                }
                if self.loops.is_empty() {
                    return Err(CtfeError::Unsupported("continue outside of a loop".into()));
                }
                self.signal = Some(Signal::Continue);
            }
            other => return Err(CtfeError::Unsupported(format!("statement: {:?}", other))),
        }
        Ok(())
    }

    /// Decode `for var in <iter>` forms. The parser desugars `a..b` /
    /// `a..=b` to range/range_inclusive calls; arrays appear as literals or
    /// identifiers bound to array values.
    fn setup_for(&mut self, var: String, iter: &Expr, body: BlockId) -> Result<(), CtfeError> {
        match iter {
            Expr::Call(f, args, _) if args.len() == 2 => {
                let (is_range, inclusive) = match f.as_ref() {
                    Expr::Ident(fid) => (
                        fid.name == "range" || fid.name == "range_inclusive",
                        fid.name == "range_inclusive",
                    ),
                    _ => (false, false),
                };
                if !is_range {
                    return Err(CtfeError::Unsupported(
                        "for-loop iterator must be a range or an array in const evaluation".into()));
                }
                let a = self.intern_expr(&args[0]);
                let b = self.intern_expr(&args[1]);
                let i = self.loops.len();
                self.loops.push(ActiveLoop::RangeInit { var, inclusive, body });
                // LIFO: Eval(a), Eval(b), then LoopTick pops (end, start).
                self.work.push(Task::LoopTick(i));
                self.work.push(Task::Eval(b));
                self.work.push(Task::Eval(a));
            }
            Expr::Array(items, _) => {
                let ids: Vec<ExprId> = items.iter().map(|it| self.intern_expr(it)).collect();
                let n = ids.len();
                let i = self.loops.len();
                self.loops.push(ActiveLoop::ArrayInit { var, count: n, body });
                self.work.push(Task::LoopTick(i));
                for id in ids.into_iter().rev() {
                    self.work.push(Task::Eval(id));
                }
            }
            Expr::Ident(_) => {
                // Pure map lookup -- safe to resolve synchronously.
                let v = match iter {
                    Expr::Ident(id) => self.lookup(&id.name)?
                        .ok_or_else(|| CtfeError::TypeError(format!(
                            "undefined for-loop iterable: {}", id.name)))?,
                    _ => unreachable!(),
                };
                match v {
                    CtfeValue::Array(items) => {
                        let i = self.loops.len();
                        self.loops.push(ActiveLoop::ForArray { var, items, idx: 0, body });
                        self.work.push(Task::LoopTick(i));
                    }
                    other => return Err(CtfeError::TypeError(format!(
                        "for-loop iterable must be an array, got {}", other.type_name()))),
                }
            }
            _ => return Err(CtfeError::Unsupported(
                "for-loop iterator must be a range or an array in const evaluation".into())),
        }
        Ok(())
    }

    fn invoke(&mut self, func: Box<Expr>, nargs: usize) -> Result<(), CtfeError> {
        let resolved = call_target_name(&func)?;

        if self.vals.len() < nargs {
            return Err(self.internal("call arguments"));
        }
        let start = self.vals.len() - nargs;
        let args: Vec<CtfeValue> = self.vals.drain(start..).collect();

        // Builtin fast-path (parity with the previous interpreter).
        if let Some(v) = eval_builtin(&resolved, &args) {
            self.vals.push(v);
            return Ok(());
        }

        // Parser-desugared range constructors produce Range VALUES (usable by
        // for-loops via identifiers? no -- for decodes range calls directly;
        // a Range appearing in a general expression has no meaning yet).
        if (resolved == "range" || resolved == "range_inclusive") && args.len() == 2 {
            let a = int_of(&args[0]).ok_or_else(|| CtfeError::TypeError(
                "range bounds must be Int".into()))?;
            let b = int_of(&args[1]).ok_or_else(|| CtfeError::TypeError(
                "range bounds must be Int".into()))?;
            self.vals.push(CtfeValue::Range(a, b, resolved == "range_inclusive"));
            return Ok(());
        }

        // User function.
        let (params, body) = self.funcs.get(&resolved)
            .cloned()
            .ok_or_else(|| CtfeError::UndefinedFunction(resolved.clone()))?;
        if params.len() != args.len() {
            return Err(CtfeError::TypeError(format!(
                "function {} expects {} args, got {}", resolved, params.len(), args.len())));
        }
        if self.frames.len() as u32 >= self.max_depth {
            return Err(CtfeError::RecursionLimit(self.max_depth));
        }
        let mut locals = HashMap::new();
        for (p, a) in params.iter().zip(args.iter()) {
            locals.insert(p.clone(), a.clone());
        }
        let bid = self.intern(body);
        self.frames.push(Frame { name: resolved, locals, last: CtfeValue::Unit });
        // LIFO: body sequence runs, then the boundary pops the callee frame
        // and hands frame.last back to the caller's value stack.
        self.work.push(Task::EndFrame(FrameExit::Function));
        self.work.push(Task::Seq { b: bid, idx: 0, want_value: false });
        Ok(())
    }

    fn lookup(&self, name: &str) -> Result<Option<CtfeValue>, CtfeError> {
        if let Some(v) = self.frames.last().and_then(|f| f.locals.get(name)) {
            return Ok(Some(v.clone()));
        }
        Ok(self.consts.get(name).cloned())
    }

    /// First-phase dispatch of an expression: compute directly (pure
    /// structural forms) or push continuation tasks.
    fn eval_dispatch(&mut self, e: Expr) -> Result<(), CtfeError> {
        match e {
            Expr::Int(n, _) => self.vals.push(CtfeValue::Int(n as i64)),
            Expr::Float(f, _) => self.vals.push(CtfeValue::Float(f)),
            Expr::Bool(b, _) => self.vals.push(CtfeValue::Bool(b)),
            Expr::Str(s, _) => self.vals.push(CtfeValue::Str(s)),
            Expr::Char(c, _) => self.vals.push(CtfeValue::Char(c)),
            Expr::Paren(inner, _) => {
                let id = self.intern_expr(&inner);
                self.work.push(Task::Eval(id));
            }
            Expr::Ident(id) => {
                match self.lookup(&id.name)? {
                    Some(v) => self.vals.push(v),
                    None => return Err(CtfeError::TypeError(format!(
                        "undefined variable: {}", id.name))),
                }
            }
            Expr::Unary(op, inner, _) => {
                let id = self.intern_expr(&inner);
                self.work.push(Task::UnOp(op));
                self.work.push(Task::Eval(id));
            }
            Expr::Binary(l, op, r, _) => match op {
                BinOp::And => {
                    let lid = self.intern_expr(&l);
                    let rid = self.intern_expr(&r);
                    self.work.push(Task::AndRhs(rid));
                    self.work.push(Task::Eval(lid));
                }
                BinOp::Or => {
                    let lid = self.intern_expr(&l);
                    let rid = self.intern_expr(&r);
                    self.work.push(Task::OrRhs(rid));
                    self.work.push(Task::Eval(lid));
                }
                _ => {
                    let lid = self.intern_expr(&l);
                    let rid = self.intern_expr(&r);
                    self.work.push(Task::BinOp(op));
                    self.work.push(Task::Eval(rid));
                    self.work.push(Task::Eval(lid));
                }
            },
            Expr::Some(inner, _) => {
                let id = self.intern_expr(&inner);
                self.work.push(Task::WrapVariant("Some"));
                self.work.push(Task::Eval(id));
            }
            Expr::None(_) => self.work.push(Task::WrapNone),
            Expr::Ok(inner, _) => {
                let id = self.intern_expr(&inner);
                self.work.push(Task::WrapVariant("Ok"));
                self.work.push(Task::Eval(id));
            }
            Expr::Err(inner, _) => {
                let id = self.intern_expr(&inner);
                self.work.push(Task::WrapVariant("Err"));
                self.work.push(Task::Eval(id));
            }
            Expr::Struct(name, fields, _, _) => {
                let sname = name.name;
                let names: Vec<String> = fields.iter().map(|(id, _)| id.name.clone()).collect();
                let count = fields.len();
                self.work.push(Task::BuildStruct { name: sname, names, count });
                for (_, fe) in fields.into_iter().rev() {
                    let id = self.intern_expr(&fe);
                    self.work.push(Task::Eval(id));
                }
            }
            Expr::Field(obj, fname, _) => {
                let oid = self.intern_expr(&obj);
                self.work.push(Task::FieldAccess(fname.name));
                self.work.push(Task::Eval(oid));
            }
            Expr::Index(arr, idx, _) => {
                let aid = self.intern_expr(&arr);
                let iid = self.intern_expr(&idx);
                // LIFO: array evaluates first, index second, so IndexOp pops
                // (index, array) from the top.
                self.work.push(Task::IndexOp);
                self.work.push(Task::Eval(iid));
                self.work.push(Task::Eval(aid));
            }
            Expr::Array(items, _) => {
                let n = items.len();
                let ids: Vec<ExprId> = items.iter().map(|it| self.intern_expr(it)).collect();
                self.work.push(Task::BuildArray(n));
                for id in ids.into_iter().rev() {
                    self.work.push(Task::Eval(id));
                }
            }
            Expr::If(cond, then_b, elifs, else_b, _) => {
                let tb = self.intern(then_b.stmts.to_vec());
                let mut ei = Vec::with_capacity(elifs.len());
                for (ec, eb) in elifs {
                    let ecid = self.intern_expr(&ec);
                    let ebid = self.intern(eb.stmts.to_vec());
                    ei.push((ecid, ebid));
                }
                let ob = else_b.map(|b| self.intern(b.stmts.to_vec()));
                let cid = self.intern_expr(&cond);
                self.work.push(Task::Branch {
                    then_b: tb, elifs: Box::new(ei), else_b: ob, want_value: true,
                });
                self.work.push(Task::Eval(cid));
            }
            Expr::Match(scrutinee, arms, _) => {
                let specs: Vec<ArmSpec> = arms.iter()
                    .map(|a| ArmSpec {
                        pattern: a.pattern.clone(),
                        body: self.intern(match_body_stmts(&a.body)),
                    })
                    .collect();
                let sid = self.intern_expr(&scrutinee);
                self.work.push(Task::Arms { arms: Box::new(specs), want_value: true });
                self.work.push(Task::Eval(sid));
            }
            Expr::BlockExpr(block, _) => {
                let b = self.intern(block.stmts.to_vec());
                self.work.push(Task::Seq { b, idx: 0, want_value: true });
            }
            Expr::ConstBlock(inner, _) => {
                let id = self.intern_expr(&inner);
                self.work.push(Task::Eval(id));
            }
            Expr::Call(func, args, _) => {
                let nargs = args.len();
                let ids: Vec<ExprId> = args.iter().map(|a| self.intern_expr(a)).collect();
                self.work.push(Task::Invoke { func, nargs });
                for id in ids.into_iter().rev() {
                    self.work.push(Task::Eval(id));
                }
            }
            other => {
                return Err(CtfeError::Unsupported(format!("expression: {:?}", other)));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers outside the machine (borrow-free)
// ---------------------------------------------------------------------------

fn call_target_name(func: &Expr) -> Result<String, CtfeError> {
    match func {
        Expr::Ident(id) => Ok(id.name.clone()),
        Expr::Field(base, method, _) => {
            if let Expr::Ident(id) = base.as_ref() {
                Ok(format!("{}.{}", id.name, method.name))
            } else {
                Err(CtfeError::Unsupported(format!(
                    "method call on complex receiver: .{}", method.name)))
            }
        }
        other => Err(CtfeError::Unsupported(format!("complex call target: {:?}", other))),
    }
}

fn int_of(v: &CtfeValue) -> Option<i64> {
    match v {
        CtfeValue::Int(n) => Some(*n),
        _ => None,
    }
}

fn match_body_stmts(body: &MatchBody) -> Vec<StmtOrExpr> {
    match body {
        MatchBody::Block(block) => block.stmts.to_vec(),
        MatchBody::Expr(e) => vec![StmtOrExpr::Expr(e.clone())],
    }
}

/// Checked binary ops. Const-evaluation overflow is a HARD ERROR (rustc
/// precedent) so one numeric story holds across checker/CTFE/runtime.
fn eval_binary_checked(l: CtfeValue, op: BinOp, r: CtfeValue) -> Result<CtfeValue, CtfeError> {
    match (&l, &r) {
        (CtfeValue::Int(a), CtfeValue::Int(b)) => {
            let (a, b) = (*a, *b);
            let v = match op {
                BinOp::Add => a.checked_add(b),
                BinOp::Sub => a.checked_sub(b),
                BinOp::Mul => a.checked_mul(b),
                BinOp::Div => {
                    if b == 0 { return Err(CtfeError::DivisionByZero); }
                    a.checked_div(b)
                }
                BinOp::Rem => {
                    if b == 0 { return Err(CtfeError::DivisionByZero); }
                    a.checked_rem(b)
                }
                BinOp::Shl => {
                    if !(0..64).contains(&b) { return Err(CtfeError::Overflow(format!("shift by {}", b))); }
                    a.checked_shl(b as u32)
                }
                BinOp::Shr => {
                    if !(0..64).contains(&b) { return Err(CtfeError::Overflow(format!("shift by {}", b))); }
                    a.checked_shr(b as u32)
                }
                BinOp::Eq => return Ok(CtfeValue::Bool(a == b)),
                BinOp::Neq => return Ok(CtfeValue::Bool(a != b)),
                BinOp::Lt => return Ok(CtfeValue::Bool(a < b)),
                BinOp::Gt => return Ok(CtfeValue::Bool(a > b)),
                BinOp::Le => return Ok(CtfeValue::Bool(a <= b)),
                BinOp::Ge => return Ok(CtfeValue::Bool(a >= b)),
                BinOp::BitAnd => Some(a & b),
                BinOp::BitOr => Some(a | b),
                BinOp::BitXor => Some(a ^ b),
                _ => return Err(CtfeError::Unsupported(format!("op {:?} on Int", op))),
            };
            v.map(CtfeValue::Int).ok_or_else(|| CtfeError::Overflow(format!(
                "{} {:?} {} overflows i64", a, op, b)))
        }
        (CtfeValue::Float(a), CtfeValue::Float(b)) => {
            let (a, b) = (*a, *b);
            match op {
                BinOp::Add => Ok(CtfeValue::Float(a + b)),
                BinOp::Sub => Ok(CtfeValue::Float(a - b)),
                BinOp::Mul => Ok(CtfeValue::Float(a * b)),
                BinOp::Div => Ok(CtfeValue::Float(a / b)),
                // EXACT IEEE comparison (was epsilon 1e-15 -- audited).
                BinOp::Eq => Ok(CtfeValue::Bool(a == b)),
                BinOp::Neq => Ok(CtfeValue::Bool(a != b)),
                BinOp::Lt => Ok(CtfeValue::Bool(a < b)),
                BinOp::Gt => Ok(CtfeValue::Bool(a > b)),
                BinOp::Le => Ok(CtfeValue::Bool(a <= b)),
                BinOp::Ge => Ok(CtfeValue::Bool(a >= b)),
                _ => Err(CtfeError::Unsupported(format!("op {:?} on Float", op))),
            }
        }
        (CtfeValue::Bool(a), CtfeValue::Bool(b)) => match op {
            BinOp::Eq => Ok(CtfeValue::Bool(a == b)),
            BinOp::Neq => Ok(CtfeValue::Bool(a != b)),
            _ => Err(CtfeError::Unsupported(format!("op {:?} on Bool", op))),
        },
        (CtfeValue::Str(a), CtfeValue::Str(b)) => match op {
            BinOp::Add => Ok(CtfeValue::Str(format!("{}{}", a, b))),
            BinOp::Eq => Ok(CtfeValue::Bool(a == b)),
            BinOp::Neq => Ok(CtfeValue::Bool(a != b)),
            _ => Err(CtfeError::Unsupported(format!("op {:?} on Str", op))),
        },
        _ => Err(CtfeError::TypeError(format!(
            "binary op {:?} on {} and {}", op, l.type_name(), r.type_name()))),
    }
}

fn eval_unary_checked(op: UnaryOp, v: CtfeValue) -> Result<CtfeValue, CtfeError> {
    match (op, &v) {
        (UnaryOp::Neg, CtfeValue::Int(n)) => n.checked_neg()
            .map(CtfeValue::Int)
            .ok_or_else(|| CtfeError::Overflow(format!("negation of {} overflows i64", n))),
        (UnaryOp::Neg, CtfeValue::Float(f)) => Ok(CtfeValue::Float(-f)),
        (UnaryOp::Not, CtfeValue::Bool(b)) => Ok(CtfeValue::Bool(!*b)),
        (UnaryOp::BitNot, CtfeValue::Int(n)) => Ok(CtfeValue::Int(!n)),
        _ => Err(CtfeError::Unsupported(format!("unary {:?} on {}", op, v.type_name()))),
    }
}

fn eval_builtin(name: &str, args: &[CtfeValue]) -> Option<CtfeValue> {
    match name {
        "str_len" | "xiom_str_len" => {
            if let CtfeValue::Str(s) = args.first()? {
                Some(CtfeValue::Int(s.len() as i64))
            } else { None }
        }
        "str_concat" | "xiom_str_concat" => {
            if let (CtfeValue::Str(a), CtfeValue::Str(b)) = (args.first()?, args.get(1)?) {
                Some(CtfeValue::Str(format!("{}{}", a, b)))
            } else { None }
        }
        "int_to_string" => {
            if let CtfeValue::Int(n) = args.first()? {
                Some(CtfeValue::Str(n.to_string()))
            } else { None }
        }
        _ => None,
    }
}

/// Pattern matching WITH binding collection into a scratch map (never
/// directly into the frame -- a failed later arm must not leave bindings).
fn pattern_matches_into(
    pattern: &Pattern,
    value: &CtfeValue,
    binds: &mut HashMap<String, CtfeValue>,
) -> Result<bool, CtfeError> {
    match pattern {
        Pattern::Wildcard(_) => Ok(true),
        Pattern::Ident(id) => {
            binds.insert(id.name.clone(), value.clone());
            Ok(true)
        }
        Pattern::Lit(lit) => Ok(match (lit, value) {
            (Literal::Bool(a, _), CtfeValue::Bool(b)) => a == b,
            (Literal::Int(a, _), CtfeValue::Int(b)) => *a as i64 == *b,
            // Exact float comparison (was epsilon -- audited inconsistency).
            (Literal::Float(a, _), CtfeValue::Float(b)) => a == b,
            (Literal::Str(a, _), CtfeValue::Str(b)) => a == b,
            (Literal::Char(a, _), CtfeValue::Char(b)) => a == b,
            _ => false,
        }),
        Pattern::Some(inner, _) => match value {
            CtfeValue::Variant(name, vals) if name == "Some" => match vals.first() {
                Some(v) => pattern_matches_into(inner, v, binds),
                None => Ok(false),
            },
            _ => Ok(false),
        },
        Pattern::None(_) => Ok(matches!(value, CtfeValue::Variant(n, vals) if n == "None" && vals.is_empty())),
        Pattern::Ok(inner, _) => match value {
            CtfeValue::Variant(name, vals) if name == "Ok" => match vals.first() {
                Some(v) => pattern_matches_into(inner, v, binds),
                None => Ok(false),
            },
            _ => Ok(false),
        },
        Pattern::Err(inner, _) => match value {
            CtfeValue::Variant(name, vals) if name == "Err" => match vals.first() {
                Some(v) => pattern_matches_into(inner, v, binds),
                None => Ok(false),
            },
            _ => Ok(false),
        },
        Pattern::Struct(pname, fields, _) => match value {
            CtfeValue::Struct(vname, vfields) if vname == &pname.name => {
                for (fname, fpat) in fields {
                    let fv = vfields.iter().find(|(n, _)| n == &fname.name).map(|(_, v)| v);
                    match fv {
                        Some(fv) => {
                            if !pattern_matches_into(fpat, fv, binds)? {
                                return Ok(false);
                            }
                        }
                        None => return Ok(false),
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        },
        Pattern::Tuple(elements, _) => match value {
            CtfeValue::Struct(_, vfields) => {
                if vfields.len() != elements.len() { return Ok(false); }
                for (i, elem) in elements.iter().enumerate() {
                    if !pattern_matches_into(elem, &vfields[i].1, binds)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        },
        _ => Ok(false),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- helpers -----------------------------------------------------------

    fn eng() -> CtfeEngine { CtfeEngine::new() }

    fn sp() -> Span { Span::new(0, 0) }

    fn i64e(n: i64) -> Expr { Expr::Int(n as u64, sp()) }
    fn fe(f: f64) -> Expr { Expr::Float(f, sp()) }
    fn be(b: bool) -> Expr { Expr::Bool(b, sp()) }
    fn se(s: &str) -> Expr { Expr::Str(s.to_string(), sp()) }
    fn idn(name: &str) -> Expr { Expr::Ident(Ident::new(name, sp())) }
    fn bin(l: Expr, op: BinOp, r: Expr) -> Expr {
        Expr::Binary(Box::new(l), op, Box::new(r), sp())
    }
    fn un(op: UnaryOp, e: Expr) -> Expr { Expr::Unary(op, Box::new(e), sp()) }
    fn blk(stmts: Vec<StmtOrExpr>) -> Block { Block { stmts, span: sp() } }
    fn soe_s(s: Stmt) -> StmtOrExpr { StmtOrExpr::Stmt(s) }
    fn soe_e(e: Expr) -> StmtOrExpr { StmtOrExpr::Expr(e) }
    fn let_(name: &str, e: Expr) -> Stmt {
        Stmt::Let(Ident::new(name, sp()), None, e, sp())
    }
    fn assign(name: &str, e: Expr) -> Stmt {
        Stmt::Assign(idn(name), e, sp())
    }
    fn ret(e: Option<Expr>) -> Stmt { Stmt::Return(e, sp()) }
    fn while_(cond: Expr, body: Vec<StmtOrExpr>) -> Stmt {
        Stmt::While(cond, blk(body), None, sp(), None)
    }
    fn for_(var: &str, iter: Expr, body: Vec<StmtOrExpr>) -> Stmt {
        Stmt::For(Ident::new(var, sp()), iter, blk(body), sp(), None)
    }
    fn if_(cond: Expr, then_stmts: Vec<StmtOrExpr>, else_stmts: Option<Vec<StmtOrExpr>>) -> Stmt {
        Stmt::If(cond, blk(then_stmts), vec![], else_stmts.map(blk), sp())
    }
    fn call(name: &str, args: Vec<Expr>) -> Expr {
        Expr::Call(Box::new(idn(name)), args, sp())
    }
    fn range(a: i64, b: i64, inclusive: bool) -> Expr {
        let name = if inclusive { "range_inclusive" } else { "range" };
        call(name, vec![i64e(a), i64e(b)])
    }

    fn register(e: &mut CtfeEngine, name: &str, params: &[&str], stmts: Vec<StmtOrExpr>) {
        e.register_function(
            name,
            params.iter().map(|p| p.to_string()).collect(),
            &stmts,
        );
    }

    fn run_fn(e: &mut CtfeEngine, name: &str, args: &[CtfeValue]) -> CtfeValue {
        e.eval_function(name, args, 0).expect("const eval failed")
    }

    // ---- CtfeValue coercions ------------------------------------------------

    #[test] fn test_value_coercions() {
        assert_eq!(CtfeValue::Int(42).as_int(), 42);
        assert_eq!(CtfeValue::Float(3.5).as_int(), 3);
        assert!(CtfeValue::Int(1).as_float() == 1.0);
        assert!(!CtfeValue::Int(1).as_bool()); // non-Bool is false
        assert!(CtfeValue::Bool(true).as_bool());
    }

    // ---- Arena ----------------------------------------------------------------

    #[test] fn test_arena_alloc_and_overflow() {
        let mut a = CtfeArena::new();
        let o = a.alloc(16).unwrap();
        assert_eq!(o, 0);
        assert_eq!(a.alloc(0).unwrap(), 16);
        // Overflow is an ERROR now (was panic -- audited).
        let mut big = CtfeArena::new();
        assert!(matches!(big.alloc(ARENA_SIZE + 1), Err(CtfeError::ArenaOverflow)));
    }

    // ---- Literals / lookup -------------------------------------------------

    #[test] fn test_literals() {
        assert_eq!(eng().eval_expr(&i64e(7)).unwrap(), CtfeValue::Int(7));
        assert_eq!(eng().eval_expr(&fe(1.25)).unwrap(), CtfeValue::Float(1.25));
        assert_eq!(eng().eval_expr(&be(false)).unwrap(), CtfeValue::Bool(false));
        assert_eq!(eng().eval_expr(&se("hi")).unwrap(), CtfeValue::Str("hi".into()));
    }

    #[test] fn test_constants_and_undefined() {
        let mut e = eng();
        e.constants.insert("PI".into(), CtfeValue::Float(3.5));
        assert_eq!(e.eval_expr(&idn("PI")).unwrap(), CtfeValue::Float(3.5));
        assert!(e.eval_expr(&idn("nope")).is_err());
    }

    // ---- Checked integer arithmetic ------------------------------------------

    #[test] fn test_int_ops() {
        let x = |l, op, r| eng().eval_expr(&bin(i64e(l), op, i64e(r))).unwrap();
        assert_eq!(x(2, BinOp::Add, 3), CtfeValue::Int(5));
        assert_eq!(x(10, BinOp::Sub, 3), CtfeValue::Int(7));
        assert_eq!(x(4, BinOp::Mul, 5), CtfeValue::Int(20));
        assert_eq!(x(10, BinOp::Div, 3), CtfeValue::Int(3));
        assert_eq!(x(10, BinOp::Rem, 3), CtfeValue::Int(1));
        assert_eq!(x(6, BinOp::BitAnd, 3), CtfeValue::Int(2));
        assert_eq!(x(6, BinOp::BitXor, 3), CtfeValue::Int(5));
        assert_eq!(x(1, BinOp::Shl, 4), CtfeValue::Int(16));
        assert_eq!(x(-1, BinOp::Shr, 62), CtfeValue::Int(-1)); // arithmetic shift
        assert_eq!(x(5, BinOp::Eq, 5), CtfeValue::Bool(true));
        assert_eq!(x(3, BinOp::Lt, 7), CtfeValue::Bool(true));
    }

    #[test] fn test_overflow_is_hard_error() {
        // Was silent wrapping (audited); const overflow is now a hard error.
        let r = eng().eval_expr(&bin(i64e(i64::MAX), BinOp::Add, i64e(1)));
        assert!(matches!(r, Err(CtfeError::Overflow(_))));
        let r = eng().eval_expr(&bin(i64e(i64::MIN), BinOp::Sub, i64e(1)));
        assert!(matches!(r, Err(CtfeError::Overflow(_))));
        let r = eng().eval_expr(&un(UnaryOp::Neg, i64e(i64::MIN)));
        assert!(matches!(r, Err(CtfeError::Overflow(_))));
        let r = eng().eval_expr(&bin(i64e(1), BinOp::Shl, i64e(70)));
        assert!(matches!(r, Err(CtfeError::Overflow(_))));
        // In a const FN too.
        let mut e = eng();
        register(&mut e, "big", &[], vec![soe_s(ret(Some(bin(
            i64e(i64::MAX), BinOp::Mul, i64e(2)))))]);
        assert!(matches!(e.eval_function("big", &[], 0), Err(CtfeError::Overflow(_))));
    }

    #[test] fn test_div_rem_by_zero() {
        assert!(eng().eval_expr(&bin(i64e(1), BinOp::Div, i64e(0))).is_err());
        assert!(eng().eval_expr(&bin(i64e(1), BinOp::Rem, i64e(0))).is_err());
    }

    // ---- Floats: exact IEEE comparison (was epsilon -- audited) ---------------

    #[test] fn test_float_exact_equality() {
        // These two differ by one ulp; epsilon folding said EQUAL, runtime
        // says NE. CTFE must agree with the runtime.
        let a = 1.0000000000000002_f64;
        assert_eq!(
            eng().eval_expr(&bin(fe(1.0), BinOp::Eq, fe(a))).unwrap(),
            CtfeValue::Bool(false));
        assert_eq!(
            eng().eval_expr(&bin(fe(1.0), BinOp::Neq, fe(a))).unwrap(),
            CtfeValue::Bool(true));
        assert_eq!(
            eng().eval_expr(&bin(fe(1.0), BinOp::Eq, fe(1.0))).unwrap(),
            CtfeValue::Bool(true));
    }

    #[test] fn test_float_arith() {
        assert_eq!(
            eng().eval_expr(&bin(fe(1.5), BinOp::Add, fe(2.5))).unwrap(),
            CtfeValue::Float(4.0));
    }

    // ---- Bool ops + short-circuit laziness -------------------------------------

    #[test] fn test_short_circuit_does_not_eval_rhs() {
        // RHS would divide by zero; short-circuit must skip it.
        let and_e = bin(be(false), BinOp::And, bin(i64e(1), BinOp::Div, i64e(0)));
        assert_eq!(eng().eval_expr(&and_e).unwrap(), CtfeValue::Bool(false));
        let or_e = bin(be(true), BinOp::Or, bin(i64e(1), BinOp::Div, i64e(0)));
        assert_eq!(eng().eval_expr(&or_e).unwrap(), CtfeValue::Bool(true));
    }

    #[test] fn test_str_ops_unary() {
        assert_eq!(
            eng().eval_expr(&bin(se("a"), BinOp::Add, se("b"))).unwrap(),
            CtfeValue::Str("ab".into()));
        assert_eq!(eng().eval_expr(&un(UnaryOp::Neg, i64e(5))).unwrap(), CtfeValue::Int(-5));
        assert_eq!(eng().eval_expr(&un(UnaryOp::Not, be(true))).unwrap(), CtfeValue::Bool(false));
        assert_eq!(eng().eval_expr(&un(UnaryOp::BitNot, i64e(0))).unwrap(), CtfeValue::Int(-1));
    }

    // ---- Structs / fields / variants ---------------------------------------------

    #[test] fn test_struct_literal_and_field() {
        let pt = Expr::Struct(
            Ident::new("Point", sp()),
            vec![(Ident::new("x", sp()), i64e(10)), (Ident::new("y", sp()), i64e(20))],
            None, sp());
        let f = Expr::Field(Box::new(pt), Ident::new("y", sp()), sp());
        assert_eq!(eng().eval_expr(&f).unwrap(), CtfeValue::Int(20));
    }

    #[test] fn test_field_not_found() {
        let pt = Expr::Struct(
            Ident::new("P", sp()), vec![(Ident::new("a", sp()), i64e(1))], None, sp());
        let f = Expr::Field(Box::new(pt), Ident::new("zz", sp()), sp());
        assert!(eng().eval_expr(&f).is_err());
    }

    #[test] fn test_variants() {
        assert!(eng().eval_expr(&Expr::None(sp())).unwrap()
            == CtfeValue::Variant("None".into(), vec![]));
        let some = Expr::Some(Box::new(i64e(9)), sp());
        assert_eq!(
            eng().eval_expr(&some).unwrap(),
            CtfeValue::Variant("Some".into(), vec![CtfeValue::Int(9)]));
    }

    // ---- If / match --------------------------------------------------------------

    #[test] fn test_if_expression() {
        let t = Expr::If(Box::new(be(true)), blk(vec![soe_e(i64e(1))]), vec![], None, sp());
        assert_eq!(eng().eval_expr(&t).unwrap(), CtfeValue::Int(1));
        let f = Expr::If(Box::new(be(false)), blk(vec![soe_e(i64e(1))]), vec![],
                         Some(blk(vec![soe_e(i64e(2))])), sp());
        assert_eq!(eng().eval_expr(&f).unwrap(), CtfeValue::Int(2));
    }

    #[test] fn test_block_tail_executes_statements_first() {
        // THE audited block bug: old eval_block_last evaluated `a * 2`
        // WITHOUT running `let a = 21` first.
        let b = blk(vec![
            soe_s(let_("a", i64e(21))),
            soe_e(bin(idn("a"), BinOp::Mul, i64e(2))),
        ]);
        let bexpr = Expr::BlockExpr(b, sp());
        assert_eq!(eng().eval_expr(&bexpr).unwrap(), CtfeValue::Int(42));

        // Same shape inside an if-arm.
        let iff = Expr::If(Box::new(be(true)),
            blk(vec![soe_s(let_("x", i64e(5))), soe_e(bin(idn("x"), BinOp::Add, i64e(1)))]),
            vec![], None, sp());
        assert_eq!(eng().eval_expr(&iff).unwrap(), CtfeValue::Int(6));
    }

    #[test] fn test_match_binds_and_selects() {
        let scrut = Expr::Some(Box::new(i64e(11)), sp());
        let arms = vec![
            MatchArm {
                pattern: Pattern::None(sp()),
                guard: None,
                body: MatchBody::Expr(i64e(0)),
                span: sp(),
            },
            MatchArm {
                pattern: Pattern::Some(Box::new(Pattern::Ident(Ident::new("v", sp()))), sp()),
                guard: None,
                body: MatchBody::Expr(idn("v")),
                span: sp(),
            },
        ];
        let m = Expr::Match(Box::new(scrut), arms, sp());
        assert_eq!(eng().eval_expr(&m).unwrap(), CtfeValue::Int(11));

        // A Some scrutinee with only a None arm must NOT match.
        let no_arm = vec![MatchArm {
            pattern: Pattern::None(sp()), guard: None,
            body: MatchBody::Expr(i64e(0)), span: sp(),
        }];
        let m2 = Expr::Match(Box::new(Expr::Some(Box::new(i64e(1)), sp())), no_arm, sp());
        assert!(eng().eval_expr(&m2).is_err()); // "match: no arm matched"
    }

    // ---- Functions: calls, recursion (THE ICE regression), limits ------------------

    #[test] fn test_function_call_basic() {
        let mut e = eng();
        register(&mut e, "add", &["a", "b"], vec![
            soe_s(ret(Some(bin(idn("a"), BinOp::Add, idn("b"))))),
        ]);
        assert_eq!(run_fn(&mut e, "add", &[CtfeValue::Int(3), CtfeValue::Int(4)]),
                   CtfeValue::Int(7));
    }

    #[test] fn test_function_tail_expression() {
        let mut e = eng();
        register(&mut e, "double", &["n"], vec![
            soe_s(let_("d", bin(idn("n"), BinOp::Mul, i64e(2)))),
            soe_e(idn("d")),
        ]);
        assert_eq!(run_fn(&mut e, "double", &[CtfeValue::Int(8)]), CtfeValue::Int(16));
    }

    #[test] fn test_recursion_no_native_stack_overflow() {
        // THE documented ICE: recursive consts crashed the COMPILER via Rust
        // stack overflow before the depth check fired. The machine makes
        // recursion flat -- factorial(20) must just WORK.
        let mut e = eng();
        register(&mut e, "fact", &["n"], vec![
            soe_s(if_(bin(idn("n"), BinOp::Le, i64e(1)),
                      vec![soe_s(ret(Some(i64e(1))))], None)),
            soe_e(bin(idn("n"), BinOp::Mul,
                      call("fact", vec![bin(idn("n"), BinOp::Sub, i64e(1))]))),
        ]);
        assert_eq!(run_fn(&mut e, "fact", &[CtfeValue::Int(10)]),
                   CtfeValue::Int(3628800));
        assert_eq!(run_fn(&mut e, "fact", &[CtfeValue::Int(20)]),
                   CtfeValue::Int(2432902008176640000));
    }

    #[test] fn test_recursion_limit_is_diagnostic_not_crash() {
        let mut e = eng();
        e.max_depth = 50;
        register(&mut e, "inf", &[], vec![
            soe_e(call("inf", vec![])),
        ]);
        let r = e.eval_function("inf", &[], 0);
        assert!(matches!(r, Err(CtfeError::RecursionLimit(50))));
    }

    #[test] fn test_session_fuel_shared_across_frames() {
        // Fuel is per-SESSION now (was per-call-frame, multiplied by depth).
        let mut e = eng();
        e.max_steps = 300;
        register(&mut e, "f", &["n"], vec![
            soe_s(if_(bin(idn("n"), BinOp::Le, i64e(0)),
                      vec![soe_s(ret(Some(i64e(0))))], None)),
            soe_e(call("f", vec![bin(idn("n"), BinOp::Sub, i64e(1))])),
        ]);
        let r = e.eval_function("f", &[CtfeValue::Int(1000)], 0);
        assert!(matches!(r, Err(CtfeError::Timeout(_))));
        // Fuel consumed is recorded back on the engine.
        assert!(e.steps > 0);
    }

    #[test] fn test_arity_mismatch_and_undefined_fn() {
        let mut e = eng();
        register(&mut e, "one", &["a"], vec![soe_e(idn("a"))]);
        assert!(e.eval_function("one", &[], 0).is_err());
        assert!(matches!(
            e.eval_function("missing", &[], 0),
            Err(CtfeError::UndefinedFunction(_))));
    }

    #[test] fn test_mutual_recursion_even_odd() {
        let mut e = eng();
        register(&mut e, "iseven", &["n"], vec![
            soe_s(if_(bin(idn("n"), BinOp::Eq, i64e(0)),
                      vec![soe_s(ret(Some(be(true))))], None)),
            soe_e(call("isodd", vec![bin(idn("n"), BinOp::Sub, i64e(1))])),
        ]);
        register(&mut e, "isodd", &["n"], vec![
            soe_s(if_(bin(idn("n"), BinOp::Eq, i64e(0)),
                      vec![soe_s(ret(Some(be(false))))], None)),
            soe_e(call("iseven", vec![bin(idn("n"), BinOp::Sub, i64e(1))])),
        ]);
        assert_eq!(run_fn(&mut e, "iseven", &[CtfeValue::Int(10)]), CtfeValue::Bool(true));
        assert_eq!(run_fn(&mut e, "iseven", &[CtfeValue::Int(7)]), CtfeValue::Bool(false));
    }

    // ---- While loops -----------------------------------------------------------------

    #[test] fn test_while_sum() {
        let mut e = eng();
        register(&mut e, "sum", &["n"], vec![
            soe_s(let_("acc", i64e(0))),
            soe_s(let_("i", i64e(1))),
            soe_s(while_(bin(idn("i"), BinOp::Le, idn("n")), vec![
                soe_s(assign("acc", bin(idn("acc"), BinOp::Add, idn("i")))),
                soe_s(assign("i", bin(idn("i"), BinOp::Add, i64e(1)))),
            ])),
            soe_e(idn("acc")),
        ]);
        assert_eq!(run_fn(&mut e, "sum", &[CtfeValue::Int(5)]), CtfeValue::Int(15));
    }

    #[test] fn test_break_continue() {
        let mut e = eng();
        // count to 3 with break
        register(&mut e, "count_to_three", &[], vec![
            soe_s(let_("i", i64e(0))),
            soe_s(Stmt::While(be(true), blk(vec![
                soe_s(assign("i", bin(idn("i"), BinOp::Add, i64e(1)))),
                soe_s(Stmt::If(bin(idn("i"), BinOp::Ge, i64e(3)),
                               blk(vec![soe_s(Stmt::Break(None, sp()))]), vec![], None, sp())),
            ]), None, sp(), None)),
            soe_e(idn("i")),
        ]);
        assert_eq!(run_fn(&mut e, "count_to_three", &[]), CtfeValue::Int(3));

        // sum odds below 6 using continue
        register(&mut e, "sum_odds", &[], vec![
            soe_s(let_("total", i64e(0))),
            soe_s(for_("k", range(0, 6, false), vec![
                soe_s(Stmt::If(bin(bin(idn("k"), BinOp::Rem, i64e(2)), BinOp::Eq, i64e(0)),
                               blk(vec![soe_s(Stmt::Continue(None, sp()))]), vec![], None, sp())),
                soe_s(assign("total", bin(idn("total"), BinOp::Add, idn("k")))),
            ])),
            soe_e(idn("total")),
        ]);
        assert_eq!(run_fn(&mut e, "sum_odds", &[]), CtfeValue::Int(9)); // 1+3+5
    }

    #[test] fn test_return_inside_loop() {
        let mut e = eng();
        // Return from inside a nested loop+if: signal crosses BOTH loop
        // boundaries and lands as the function result.
        register(&mut e, "find_first_gt", &["limit"], vec![
            soe_s(for_("v", range(0, 10, false), vec![
                soe_s(Stmt::If(bin(idn("v"), BinOp::Gt, idn("limit")),
                               blk(vec![soe_s(ret(Some(idn("v"))))]), vec![], None, sp())),
            ])),
            soe_s(ret(Some(bin(i64e(0), BinOp::Sub, i64e(1))))),
        ]);
        assert_eq!(run_fn(&mut e, "find_first_gt", &[CtfeValue::Int(4)]), CtfeValue::Int(5));
        assert_eq!(run_fn(&mut e, "find_first_gt", &[CtfeValue::Int(99)]), CtfeValue::Int(-1));
    }

    // ---- for loops (THE once-only regression) -----------------------------------------

    #[test] fn test_for_range_iterates_fully() {
        // Was: body evaluated ONCE with the loop var UNBOUND (audited).
        let mut e = eng();
        register(&mut e, "sumto", &["n"], vec![
            soe_s(let_("acc", i64e(0))),
            soe_s(for_("i", call("range", vec![i64e(1), idn("n")]), vec![
                soe_s(assign("acc", bin(idn("acc"), BinOp::Add, idn("i")))),
            ])),
            soe_e(idn("acc")),
        ]);
        assert_eq!(run_fn(&mut e, "sumto", &[CtfeValue::Int(6)]), CtfeValue::Int(15));
        assert_eq!(run_fn(&mut e, "sumto", &[CtfeValue::Int(0)]), CtfeValue::Int(0)); // empty
        // Inclusive bounds.
        register(&mut e, "sum_inc", &[], vec![
            soe_s(let_("acc", i64e(0))),
            soe_s(for_("i", range(1, 4, true), vec![
                soe_s(assign("acc", bin(idn("acc"), BinOp::Add, idn("i")))),
            ])),
            soe_e(idn("acc")),
        ]);
        assert_eq!(run_fn(&mut e, "sum_inc", &[]), CtfeValue::Int(10)); // 1+2+3+4
    }

    #[test] fn test_for_array_literal_and_value() {
        let mut e = eng();
        register(&mut e, "sum_lit", &[], vec![
            soe_s(let_("acc", i64e(0))),
            soe_s(for_("x", Expr::Array(vec![i64e(2), i64e(4), i64e(6)], sp()), vec![
                soe_s(assign("acc", bin(idn("acc"), BinOp::Add, idn("x")))),
            ])),
            soe_e(idn("acc")),
        ]);
        assert_eq!(run_fn(&mut e, "sum_lit", &[]), CtfeValue::Int(12));

        // Iterate an array held in a local.
        register(&mut e, "sum_local_arr", &[], vec![
            soe_s(let_("arr", Expr::Array(vec![i64e(1), i64e(2)], sp()))),
            soe_s(let_("acc", i64e(0))),
            soe_s(for_("x", idn("arr"), vec![
                soe_s(assign("acc", bin(idn("acc"), BinOp::Add, idn("x")))),
            ])),
            soe_e(idn("acc")),
        ]);
        assert_eq!(run_fn(&mut e, "sum_local_arr", &[]), CtfeValue::Int(3));
    }

    #[test] fn test_array_indexing_in_const() {
        let mut e = eng();
        register(&mut e, "pick", &[], vec![
            soe_s(let_("arr", Expr::Array(vec![i64e(7), i64e(8), i64e(9)], sp()))),
            soe_e(Expr::Index(Box::new(idn("arr")), Box::new(i64e(1)), sp())),
        ]);
        assert_eq!(run_fn(&mut e, "pick", &[]), CtfeValue::Int(8));
        // Out of bounds is an error, not garbage.
        register(&mut e, "oob", &[], vec![
            soe_s(let_("arr", Expr::Array(vec![i64e(1)], sp()))),
            soe_e(Expr::Index(Box::new(idn("arr")), Box::new(i64e(5)), sp())),
        ]);
        assert!(e.eval_function("oob", &[], 0).is_err());
    }

    // ---- Builtins --------------------------------------------------------------------

    #[test] fn test_builtins() {
        let mut e = eng();
        register(&mut e, "f", &[], vec![soe_e(call("str_len", vec![se("abcd")]))]);
        assert_eq!(run_fn(&mut e, "f", &[]), CtfeValue::Int(4));
        register(&mut e, "g", &[], vec![soe_e(call("int_to_string", vec![i64e(42)]))]);
        assert_eq!(run_fn(&mut e, "g", &[]), CtfeValue::Str("42".into()));
    }

    // ---- try_to_expr: aggregates are NEVER sentineled (the critical fix) ----------------

    #[test] fn test_try_to_expr_primitives_roundtrip() {
        let cases = vec![
            CtfeValue::Int(-3),
            CtfeValue::Float(1.5),
            CtfeValue::Bool(true),
            CtfeValue::Str("x".into()),
            CtfeValue::Char('q'),
        ];
        for v in cases {
            let e = CtfeEngine::try_to_expr(&v.clone()).expect("primitive must convert");
            let back = eng().eval_expr(&e).unwrap();
            assert_eq!(back, v);
        }
        // Unit keeps its historic Int(0) ABI slot.
        assert_eq!(CtfeEngine::try_to_expr(&CtfeValue::Unit).unwrap(), i64e(0));
    }

    #[test] fn test_try_to_expr_aggregates_not_sentinel() {
        // OLD behavior: Struct/Variant folded to Expr::Int(0) -- silent wrong
        // programs. They now reconstruct faithfully or refuse (Option).
        let s = CtfeValue::Struct("P".into(), vec![
            ("x".into(), CtfeValue::Int(1)),
            ("y".into(), CtfeValue::Int(2)),
        ]);
        match CtfeEngine::try_to_expr(&s) {
            Some(Expr::Struct(_, fields, _, _)) => assert_eq!(fields.len(), 2),
            other => panic!("struct must reconstruct, got {:?}", other.is_some()),
        }
        let some = CtfeValue::Variant("Some".into(), vec![CtfeValue::Int(5)]);
        assert!(matches!(CtfeEngine::try_to_expr(&some), Some(Expr::Some(_, _))));
        let none_v = CtfeValue::Variant("None".into(), vec![]);
        assert!(matches!(CtfeEngine::try_to_expr(&none_v), Some(Expr::None(_))));
        let arr = CtfeValue::Array(vec![CtfeValue::Int(1)]);
        assert!(matches!(CtfeEngine::try_to_expr(&arr), Some(Expr::Array(_, _))));
        // No faithful form -> None (caller falls back to UNEVALUATED).
        assert!(CtfeEngine::try_to_expr(&CtfeValue::Range(0, 5, false)).is_none());
        assert!(CtfeEngine::try_to_expr(&CtfeValue::Null).is_none());
        assert!(CtfeEngine::try_to_expr(&CtfeValue::Ptr(0)).is_none());
        // Custom enum variant names cannot be reconstructed safely either.
        let custom = CtfeValue::Variant("Color".into(), vec![CtfeValue::Int(1)]);
        assert!(CtfeEngine::try_to_expr(&custom).is_none());
    }

    #[test] fn test_const_struct_through_full_pipeline() {
        let mut e = eng();
        register(&mut e, "mk_point", &["a"], vec![
            soe_e(Expr::Struct(
                Ident::new("Point", sp()),
                vec![(Ident::new("x", sp()), idn("a")), (Ident::new("y", sp()), i64e(9))],
                None, sp())),
        ]);
        let v = run_fn(&mut e, "mk_point", &[CtfeValue::Int(4)]);
        match &v {
            CtfeValue::Struct(name, fields) => {
                assert_eq!(name, "Point");
                assert_eq!(fields[0].1, CtfeValue::Int(4));
                assert_eq!(fields[1].1, CtfeValue::Int(9));
            }
            other => panic!("expected struct value, got {:?}", other),
        }
        // And the codegen-facing conversion preserves it (was Int(0)).
        assert!(matches!(
            CtfeEngine::try_to_expr(&v),
            Some(Expr::Struct(_, ref f, _, _)) if f.len() == 2));
    }

    // ---- Misc unsupported forms error cleanly (never mis-evaluate) -----------------------

    #[test] fn test_labeled_loops_rejected_cleanly() {
        let mut e = eng();
        register(&mut e, "lbl", &[], vec![
            soe_s(for_("i", range(0, 3, false), vec![
                soe_s(Stmt::Break(Some(Ident::new("outer", sp())), sp())),
            ])),
            soe_e(i64e(0)),
        ]);
        assert!(matches!(
            e.eval_function("lbl", &[], 0),
            Err(CtfeError::Unsupported(_))));
    }

    #[test] fn test_break_outside_loop_rejected() {
        let mut e = eng();
        register(&mut e, "bad", &[], vec![soe_s(Stmt::Break(None, sp())), soe_e(i64e(0))]);
        assert!(matches!(e.eval_function("bad", &[], 0), Err(CtfeError::Unsupported(_))));
    }
}

