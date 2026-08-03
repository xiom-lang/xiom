// XIOM — AST
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! XIOM Abstract Syntax Tree — every construct from the EBNF grammar.
//! This is the single source of truth for what the parser produces
//! and what every downstream pass consumes.

// ============================================================================
// Applicability — suggestion confidence contract (5c-R, rustc lesson)
// ============================================================================

/// How confident the compiler is that a suggested fix is correct.
/// Tools (LSP, xiom-fix) auto-apply only `MachineApplicable` suggestions.
/// Direct transplant from rustc's `Applicability` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    /// The suggestion is definitely correct — safe to auto-apply.
    MachineApplicable,
    /// The suggestion may be correct but the compiler cannot guarantee it.
    MaybeIncorrect,
    /// The suggestion contains placeholder types that need resolution first.
    HasPlaceholders,
    /// The suggestion is untested / ad-hoc — never auto-apply.
    Unspecified,
}

use std::fmt;

// ============================================================================
// Source location
// ============================================================================

/// A position in source code: line and column, both 1-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub line: u32,
    pub col: u32,
}

impl Span {
    pub const fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

// ============================================================================
// Identifiers
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self { name: name.into(), span }
    }
}

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A named type: `Int`, `Str`, `Vec`, etc.
    Named(Ident, Vec<Type>),
    /// `&T`
    Ref(Box<Type>),
    /// `&mut T`
    MutRef(Box<Type>),
    /// `Option[T]`
    Option(Box<Type>),
    /// `Result[T, E]`
    Result(Box<Type>, Box<Type>),
    /// `Vec[T]`
    Vec(Box<Type>),
    /// `Slice[T]`
    Slice(Box<Type>),
    /// `Map[K, V]`
    Map(Box<Type>, Box<Type>),
    /// `Set[T]`
    Set(Box<Type>),
    /// `(T, U, ...)`
    Tuple(Vec<Type>),
    /// `*T` (raw pointer)
    Ptr(Box<Type>),
    /// `[N]T` (fixed array)
    Array(Box<Expr>, Box<Type>),
    /// `fn(T, U) -> V` (function pointer)
    Fn(Vec<Type>, Box<Type>),
    /// `impl Trait` — opaque return type (existential)
    ImplTrait(Vec<Ident>),
    /// Anonymous struct type `{ field: Type; ... }` — used in generic
    /// function signatures where the struct has no standalone name.
    AnonStruct(Vec<FieldDecl>),
    /// v0.55: `!` — the Never type (bottom type). Functions returning `!`
    /// never return (infinite loop, exit, panic). Enables exhaustiveness
    /// proofs in match expressions.
    Never,
}

// ============================================================================
// Literals
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(u64, Span),
    Float(f64, Span),
    Str(String, Span),
    Char(char, Span),
    Bool(bool, Span),
}

// ============================================================================
// Expressions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// `ident`
    Ident(Ident),
    /// Integer literal
    Int(u64, Span),
    /// Float literal
    Float(f64, Span),
    /// String literal
    Str(String, Span),
    /// Char literal
    Char(char, Span),
    /// `true` / `false`
    Bool(bool, Span),
    /// `(expr)`
    Paren(Box<Expr>, Span),
    /// `-expr` or `!expr`
    Unary(UnaryOp, Box<Expr>, Span),
    /// `left op right`
    Binary(Box<Expr>, BinOp, Box<Expr>, Span),
    /// `expr?` — error propagation
    Try(Box<Expr>, Span),
    /// `expr => expr` — implication (in contracts)
    Imply(Box<Expr>, Box<Expr>, Span),
    /// `expr is Pattern` — type test
    Is(Box<Expr>, Pattern, Span),
    /// `expr.field` — field access
    Field(Box<Expr>, Ident, Span),
    /// `expr(args)` — function call
    Call(Box<Expr>, Vec<Expr>, Span),
    /// v0.54: `expr::<Type>(args)` — turbofish call (generic type arguments).
    /// Stores the parsed type for CTFE builtins: align_of, type_id, field_offset.
    GenericCall(Box<Expr>, Type, Vec<Expr>, Span),
    /// `expr[index]` — index
    Index(Box<Expr>, Box<Expr>, Span),
    /// `expr@pre` — pre-state (in contracts)
    AtPre(Box<Expr>, Span),
    /// `&expr` — reference
    Ref(Box<Expr>, Span),
    /// `&mut expr` — mutable reference
    MutRef(Box<Expr>, Span),
    /// `Some(expr)`
    Some(Box<Expr>, Span),
    /// `None`
    None(Span),
    /// `Ok(expr)`
    Ok(Box<Expr>, Span),
    /// `Err(expr)`
    Err(Box<Expr>, Span),
    /// `Type{ field: val, ... }` — struct literal
    Struct(Ident, Vec<(Ident, Expr)>, Option<Box<Expr>>, Span),
    /// `[expr, ...]` — array literal
    Array(Vec<Expr>, Span),
    /// `{ stmt; ... }` — bare block expression
    BlockExpr(Block, Span),
    /// v0.54: `const { expr }` — compile-time constant block expression.
    /// Evaluated by CTFE Phase A; the result replaces the node before codegen.
    ConstBlock(Box<Expr>, Span),
    /// `fn(params) -> RetType { ... }` — closure
    Closure(Vec<Param>, Option<Box<Type>>, Block, Span),
    /// `|x, y| expr` — pipe closure
    PipeClosure(Vec<Ident>, Box<Expr>, Span),
    /// `await expr`
    Await(Box<Expr>, Span),
    /// `comptime expr`
    Comptime(Box<Expr>, Span),
    /// `expr as Type` — type cast
    As(Box<Expr>, Type, Span),
    /// `(a, b, ...)` — tuple expression
    Tuple(Vec<Expr>, Span),
    /// `if cond { then } else { else }` — if-expression
    If(Box<Expr>, Block, Vec<(Expr, Block)>, Option<Block>, Span),
    /// `match expr { arms }` as an expression
    Match(Box<Expr>, Vec<MatchArm>, Span),
    /// `unsafe { ... }` block
    Unsafe(Block, Span),
    /// Error-poisoned node: downstream passes skip silently.
    /// Carries an `ErrorGuaranteed` proof that a diagnostic WAS emitted.
    /// (rustc lesson: kills cascading errors across the entire pipeline.)
    Error(ErrorGuaranteed, Span),
}

/// Zero-sized proof that a diagnostic has been emitted for this error.
/// Cannot be constructed outside this crate; downstream passes check
/// `expr.is_error()` before processing to avoid cascading diagnostics.
/// Cannot be serialized — panics on encode (prevents caching errors).
/// (Direct transplant from rustc's `ErrorGuaranteed` — see docs/rust/05-diagnostics.md)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorGuaranteed {
    _private: (),
}

impl ErrorGuaranteed {
    /// Create a new error guarantee. Only constructible within the AST crate
    /// so only the parser/checker can issue guarantees.
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident(i) => i.span,
            Expr::Int(_, s) | Expr::Float(_, s) | Expr::Str(_, s) | Expr::Char(_, s) | Expr::Bool(_, s) => *s,
            Expr::Paren(_, s) | Expr::Unary(_, _, s) | Expr::Binary(_, _, _, s) | Expr::Try(_, s) => *s,
            Expr::Imply(_, _, s) | Expr::Is(_, _, s) | Expr::Field(_, _, s) | Expr::Call(_, _, s) | Expr::GenericCall(_, _, _, s) => *s,
            Expr::Index(_, _, s) | Expr::AtPre(_, s) | Expr::Ref(_, s) | Expr::MutRef(_, s) => *s,
            Expr::Some(_, s) | Expr::None(s) | Expr::Ok(_, s) | Expr::Err(_, s) => *s,
            Expr::Struct(_, _, _, s) | Expr::Array(_, s) | Expr::BlockExpr(_, s) | Expr::ConstBlock(_, s) | Expr::Closure(_, _, _, s) | Expr::PipeClosure(_, _, s) => *s,
            Expr::Await(_, s) | Expr::Comptime(_, s) | Expr::As(_, _, s) | Expr::Tuple(_, s) | Expr::If(_, _, _, _, s) | Expr::Unsafe(_, s) => *s,
            Expr::Match(_, _, s) => *s,
            Expr::Error(_, s) => *s,
        }
    }

    /// True when this node was error-poisoned and carries no meaningful value.
    /// Downstream passes should skip silently — the diagnostic was already emitted.
    pub fn is_error(&self) -> bool {
        matches!(self, Expr::Error(..))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Ref,
    MutRef,
    BitNot,
    Deref,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Neq, Lt, Gt, Le, Ge,
    Shl, Shr,
    And, Or,
    Assign,
    BitXor,
    BitAnd,
    BitOr,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Eq => "==",
            BinOp::Neq => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Le => "<=",
            BinOp::Ge => ">=",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::Assign => "=",
            BinOp::BitXor => "^",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
        };
        write!(f, "{s}")
    }
}

// ============================================================================
// Patterns
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// `_` — wildcard
    Wildcard(Span),
    /// `ident` — variable binding
    Ident(Ident),
    /// `Variant(field1, field2)` — enum variant pattern
    Variant(Ident, Vec<Ident>, Span),
    /// Literal pattern
    Lit(Literal),
    /// `Some(pattern)`
    Some(Box<Pattern>, Span),
    /// `None`
    None(Span),
    /// `Ok(pattern)`
    Ok(Box<Pattern>, Span),
    /// `Err(pattern)`
    Err(Box<Pattern>, Span),
    /// `A | B | C` — or-pattern (matches if any alternative matches)
    Or(Vec<Pattern>, Span),
}

// ============================================================================
// Statements
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `let ident [: type] = expr;`
    Let(Ident, Option<Box<Type>>, Expr, Span),
    /// `var ident [: type] = expr;`
    Var(Ident, Option<Box<Type>>, Expr, Span),
    /// `place = expr;`
    Assign(Expr, Expr, Span),
    /// `return [expr];`
    Return(Option<Expr>, Span),
    /// `expr;` — expression statement
    Expr(Expr, Span),
    /// `if expr block {elif expr block} [else block]`
    If(Expr, Block, Vec<(Expr, Block)>, Option<Block>, Span),
    /// `match expr { arms }`
    Match(Expr, Vec<MatchArm>, Span),
    /// `while expr [invariant: expr] block`
    While(Expr, Block, Option<Expr>, Span),
    /// `for ident in expr block`
    For(Ident, Expr, Block, Span),
    /// `spawn [move] { block }`
    Spawn(Block, Span, /* move */ bool),
    /// `var (a, b) = expr;` / `let (a, b) = expr;`
    Destructure(Vec<Ident>, Expr, Span),
    /// `break;` or `break 'label;`
    Break(Option<Ident>, Span),
    /// `continue;` or `continue 'label;`
    Continue(Option<Ident>, Span),
    /// v0.55: `asm("...")` — inline assembly statement
    Asm(AsmBlock),
    /// v0.55: `defer { expr }` — guaranteed scope-exit execution
    Defer(Block, Span),
}

/// v0.55: Inline assembly block — `asm("template" : outputs : inputs : clobbers)`
#[derive(Debug, Clone, PartialEq)]
pub struct AsmBlock {
    /// Assembly template string (Intel syntax)
    pub template: String,
    /// Output constraints: ("=r", var_name)
    pub outputs: Vec<(String, Ident)>,
    /// Input constraints: ("r", expression)
    pub inputs: Vec<(String, Expr)>,
    /// Clobbered registers: ["rax", "rcx", "memory"]
    pub clobbers: Vec<String>,
    /// Span for diagnostics
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtOrExpr {
    Stmt(Stmt),
    Expr(Expr),
}

/// A match arm: `pattern => (block | expr,)`
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: MatchBody,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchBody {
    Block(Block),
    Expr(Expr),
}

/// A block `{ stmts... [expr] }`
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<StmtOrExpr>,
    pub span: Span,
}

// ============================================================================
// Parameters
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
    /// `true` when this param was declared as `&mut self` (mutable receiver).
    pub is_mut_self: bool,
    /// `true` when this param was declared as `&self` (ref receiver).
    pub is_ref_self: bool,
}

// ============================================================================
// Contracts
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ContractClause {
    Requires(Expr, Span),
    Ensures(Expr, Span),
}

// ============================================================================
// Attributes
// ============================================================================

/// A compiler-recognized attribute attached to declarations.
/// e.g. `#[safety_audit(justification: "required for FFI")]`
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Ident,
    /// Key-value arguments: `[(justification, "required for FFI")]`
    pub args: Vec<(String, String)>,
    pub span: Span,
}

// ============================================================================
// Functions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: Ident,
    pub bounds: Vec<Ident>, // interface names
    pub is_const: bool,
    pub const_ty: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub attributes: Vec<Attribute>,
    pub is_async: bool,
    pub is_pub: bool,
    pub receiver: Option<Ident>, // TypeName for methods
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub contracts: Vec<ContractClause>,
    pub body: Option<Block>,
    pub span: Span,
}

impl FnDecl {
    pub fn is_method(&self) -> bool {
        self.receiver.is_some()
    }
}

// ============================================================================
// Type Declarations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum DeriveTrait {
    Eq, Clone, Display, Hash, Ord, Debug,
}

impl DeriveTrait {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Eq" => Some(Self::Eq),
            "Clone" => Some(Self::Clone),
            "Display" => Some(Self::Display),
            "Hash" => Some(Self::Hash),
            "Ord" => Some(Self::Ord),
            "Debug" => Some(Self::Debug),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: Ident,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub is_pub: bool,
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<FieldDecl>,
    pub derived_fields: Vec<(Ident, Type, Expr)>,
    pub invariants: Vec<Expr>,
    pub derives: Vec<DeriveTrait>,
    pub alias: Option<Box<Type>>,
    pub span: Span,
}

// ============================================================================
// Enum Declarations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: Ident,
    pub fields: Vec<FieldDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub is_pub: bool,
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub derives: Vec<DeriveTrait>,
    pub span: Span,
}

// ============================================================================
// Interface Declarations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDecl {
    pub is_pub: bool,
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub members: Vec<InterfaceMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceMember {
    Field(FieldDecl),
    FnSignature(FnDecl),
}

// ============================================================================
// Module / Const / Use
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub name: Ident,
    pub ty: Type,
    pub value: Expr,
    /// `true` for a mutable module-level `var` (emitted as a real LLVM global
    /// read via `load` / written via `store`); `false` for an immutable `const`
    /// (compile-time value substituted at each read site).
    pub is_mut: bool,
    /// `true` when declared as `pub const` — enables cross-module visibility.
    pub is_pub: bool,
    pub span: Span,
}

/// `extern "C" { fn foo(...) -> ...; fn bar(...) -> ...; }`
#[derive(Debug, Clone, PartialEq)]
pub struct ExternBlock {
    pub linkage: String,
    pub functions: Vec<FnDecl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseDecl {
    pub path: Vec<Ident>,
    pub alias: Option<Ident>,
    pub glob: bool, // use math.vector.*
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDecl {
    pub name: Ident,
    pub path: Vec<Ident>,
    pub items: Vec<TopDecl>,
    pub is_file_level: bool,
    pub source_file: Option<String>,
    pub span: Span,
}

// ============================================================================
// Top-level declarations
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum TopDecl {
    Module(ModuleDecl),
    Use(UseDecl),
    Type(TypeDecl),
    Enum(EnumDecl),
    Interface(InterfaceDecl),
    Fn(FnDecl),
    Const(ConstDecl),
    Extern(ExternBlock),
    Impl(ImplDecl),
    /// Module-level `spawn [move] { ... }` block (M21 async statement at top level).
    Spawn(Block, Span, /* move */ bool),
}

// ============================================================================
// ImplDecl — impl Trait for Type { ... }
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ImplDecl {
    pub trait_name: Ident,
    pub type_name: Ident,
    pub members: Vec<ImplItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImplItem {
    Fn(FnDecl),
    Const(ConstDecl),
}

// ============================================================================
// Program
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<TopDecl>,
    pub source_files: Vec<String>,
    pub root_dir: Option<String>,
    pub span: Span,
}

impl Program {
    pub fn new(items: Vec<TopDecl>, span: Span) -> Self {
        Self { items, source_files: Vec::new(), root_dir: None, span }
    }

    /// M20: Expand impl blocks into freestanding functions.
    /// `impl Trait for Type { fn m() { body } }` becomes `fn Type.m() { body }`.
    /// M22: Recurse into modules so impl blocks inside `module { ... }` are expanded.
    pub fn expand_impl_blocks(&self) -> Program {
        // Helper: rewrite bare method calls (e.g. `value()`) to `self.value()`
        // in interface default bodies. This ensures the expanded inherent method
        // uses proper self.method() syntax for calls to other interface methods.
        fn rewrite_bare_calls(block: &mut Block, iface_methods: &std::collections::HashSet<String>, span: Span) {
            for se in &mut block.stmts {
                match se {
                    StmtOrExpr::Stmt(s) => rewrite_stmt(s, iface_methods, span),
                    StmtOrExpr::Expr(e) => rewrite_expr(e, iface_methods, span),
                }
            }
        }
        fn rewrite_stmt(stmt: &mut Stmt, iface_methods: &std::collections::HashSet<String>, span: Span) {
            match stmt {
                Stmt::Expr(e, _) | Stmt::Return(Some(e), _) => rewrite_expr(e, iface_methods, span),
                Stmt::Var(_, _, e, _) | Stmt::Let(_, _, e, _) => rewrite_expr(e, iface_methods, span),
                Stmt::Assign(_, e, _) => rewrite_expr(e, iface_methods, span),
                Stmt::While(cond, body, _, _) => { rewrite_expr(cond, iface_methods, span); rewrite_bare_calls(body, iface_methods, span); }
                Stmt::If(cond, tb, elifs, eb, _) => {
                    rewrite_expr(cond, iface_methods, span);
                    rewrite_bare_calls(tb, iface_methods, span);
                    for (ec, eb2) in elifs { rewrite_expr(ec, iface_methods, span); rewrite_bare_calls(eb2, iface_methods, span); }
                    if let Some(eb3) = eb { rewrite_bare_calls(eb3, iface_methods, span); }
                }
                Stmt::Match(sc, arms, _) => {
                    rewrite_expr(sc, iface_methods, span);
                    for arm in arms {
                        if let Some(ref mut g) = arm.guard { rewrite_expr(g, iface_methods, span); }
                        match &mut arm.body {
                            MatchBody::Block(b) => rewrite_bare_calls(b, iface_methods, span),
                            MatchBody::Expr(e) => rewrite_expr(e, iface_methods, span),
                        }
                    }
                }
                Stmt::For(_, iter, body, _) => { rewrite_expr(iter, iface_methods, span); rewrite_bare_calls(body, iface_methods, span); }
                Stmt::Spawn(body, _, _move) => rewrite_bare_calls(body, iface_methods, span),
                _ => {}
            }
        }
        fn rewrite_expr(expr: &mut Expr, iface_methods: &std::collections::HashSet<String>, span: Span) {
            match expr {
                Expr::Call(func, args, _) => {
                    // Rewrite bare `method()` to `self.method()`
                    if let Expr::Ident(id) = func.as_ref() {
                        if iface_methods.contains(&id.name) && id.name != "self" {
                            *func = Box::new(Expr::Field(
                                Box::new(Expr::Ident(Ident { name: "self".to_string(), span })),
                                Ident { name: id.name.clone(), span }, span,
                            ));
                        }
                    }
                    rewrite_expr(func, iface_methods, span);
                    for arg in args { rewrite_expr(arg, iface_methods, span); }
                }
                Expr::Binary(a, _, b, _) | Expr::Imply(a, b, _) => {
                    rewrite_expr(a, iface_methods, span); rewrite_expr(b, iface_methods, span);
                }
                Expr::Unary(_, e, _) | Expr::Ref(e, _) | Expr::MutRef(e, _)
                | Expr::Paren(e, _) | Expr::Try(e, _) | Expr::AtPre(e, _)
                | Expr::Some(e, _) | Expr::Ok(e, _) | Expr::Err(e, _)
                | Expr::As(e, _, _) => rewrite_expr(e, iface_methods, span),
                Expr::Field(obj, _, _) => rewrite_expr(obj, iface_methods, span),
                Expr::Index(arr, idx, _) => { rewrite_expr(arr, iface_methods, span); rewrite_expr(idx, iface_methods, span); }
                Expr::Struct(_, fields, base, _) => {
                    for (_, v) in fields { rewrite_expr(v, iface_methods, span); }
                    if let Some(b) = base { rewrite_expr(b, iface_methods, span); }
                }
                Expr::If(c, t, elifs, els, _) => {
                    rewrite_expr(c, iface_methods, span);
                    rewrite_bare_calls(t, iface_methods, span);
                    for (ec, eb) in elifs { rewrite_expr(ec, iface_methods, span); rewrite_bare_calls(eb, iface_methods, span); }
                    if let Some(eb) = els { rewrite_bare_calls(eb, iface_methods, span); }
                }
                Expr::Match(sc, arms, _) => {
                    rewrite_expr(sc, iface_methods, span);
                    for arm in arms {
                        if let Some(ref mut g) = arm.guard { rewrite_expr(g, iface_methods, span); }
                        match &mut arm.body {
                            MatchBody::Block(b) => rewrite_bare_calls(b, iface_methods, span),
                            MatchBody::Expr(e) => rewrite_expr(e, iface_methods, span),
                        }
                    }
                }
                Expr::Is(e, _, _) => rewrite_expr(e, iface_methods, span),
                Expr::Array(items, _) => for e in items { rewrite_expr(e, iface_methods, span); },
                Expr::Closure(_, _, _body, _) => {
                    // Don't recurse into closures — they have their own scope
                }
                _ => {}
            }
        }
        // Helper: collect interface defaults recursively (including inside modules).
        fn collect_interface_defaults(items: &[TopDecl]) -> std::collections::HashMap<String, Vec<(String, FnDecl)>> {
            let mut defaults: std::collections::HashMap<String, Vec<(String, FnDecl)>> = std::collections::HashMap::new();
            for item in items {
                match item {
                    TopDecl::Interface(id) => {
                        let mut methods = Vec::new();
                        for member in &id.members {
                            if let InterfaceMember::FnSignature(fd) = member {
                                if fd.body.is_some() {
                                    methods.push((fd.name.name.clone(), fd.clone()));
                                }
                            }
                        }
                        if !methods.is_empty() {
                            defaults.insert(id.name.name.clone(), methods);
                        }
                    }
                    TopDecl::Module(md) => {
                        let nested = collect_interface_defaults(&md.items);
                        for (k, v) in nested {
                            if !defaults.contains_key(&k) { defaults.insert(k, v); }
                        }
                    }
                    _ => {}
                }
            }
            defaults
        }

        // Helper: collect interface REQUIRED methods (those WITHOUT body).
        fn collect_interface_required(items: &[TopDecl]) -> std::collections::HashMap<String, Vec<String>> {
            let mut required: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
            for item in items {
                match item {
                    TopDecl::Interface(id) => {
                        let methods: Vec<String> = id.members.iter()
                            .filter_map(|m| if let InterfaceMember::FnSignature(fd) = m {
                                if fd.body.is_none() { Some(fd.name.name.clone()) } else { None }
                            } else { None })
                            .collect();
                        if !methods.is_empty() {
                            required.insert(id.name.name.clone(), methods);
                        }
                    }
                    TopDecl::Module(md) => {
                        let nested = collect_interface_required(&md.items);
                        for (k, v) in nested {
                            if !required.contains_key(&k) { required.insert(k, v); }
                        }
                    }
                    _ => {}
                }
            }
            required
        }

        // Helper: collect inherent methods per type from fn items with receivers.
        // `fn TypeName.method(...)` has receiver TypeName and name `TypeName.method`
        // or just `method`. Extract the bare method name (after any dot).
        fn collect_inherent_methods(items: &[TopDecl]) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
            let mut methods: std::collections::HashMap<String, std::collections::HashSet<String>> = std::collections::HashMap::new();
            for item in items {
                match item {
                    TopDecl::Fn(fd) => {
                        if let Some(ref receiver) = fd.receiver {
                            let type_name = receiver.name.clone();
                            let method_name = fd.name.name.rsplit('.').next().unwrap_or(&fd.name.name).to_string();
                            methods.entry(type_name).or_default().insert(method_name);
                        }
                    }
                    TopDecl::Module(md) => {
                        let nested = collect_inherent_methods(&md.items);
                        for (k, v) in nested {
                            methods.entry(k).or_default().extend(v);
                        }
                    }
                    _ => {}
                }
            }
            methods
        }

        // Collect ALL declared type names (TypeDecl + EnumDecl), recursing into
        // modules, so the auto-detect pass knows about types with zero inherent
        // methods (e.g. `type Cat = {}` that should still receive interface defaults).
        fn collect_declared_types(items: &[TopDecl]) -> Vec<String> {
            let mut names = Vec::new();
            for item in items {
                match item {
                    TopDecl::Type(td) => { names.push(td.name.name.clone()); }
                    TopDecl::Enum(ed) => { names.push(ed.name.name.clone()); }
                    TopDecl::Module(md) => {
                        names.extend(collect_declared_types(&md.items));
                    }
                    _ => {}
                }
            }
            names
        }

        let interface_defaults = collect_interface_defaults(&self.items);
        let interface_required = collect_interface_required(&self.items);
        let mut inherent_methods = collect_inherent_methods(&self.items);
        // Merge bare type declarations (types with NO inherent methods) into
        // the map so auto-detect considers them for interface defaults.
        for type_name in collect_declared_types(&self.items) {
            inherent_methods.entry(type_name).or_default();
        }

        // Helper: expand impl blocks in a list of items, recursing into modules.
        // Also auto-detects types that satisfy interfaces through inherent methods.
        fn expand_items(
            items: &[TopDecl],
            interface_defaults: &std::collections::HashMap<String, Vec<(String, FnDecl)>>,
            interface_required: &std::collections::HashMap<String, Vec<String>>,
            inherent_methods: &std::collections::HashMap<String, std::collections::HashSet<String>>,
        ) -> Vec<TopDecl> {
            let mut out = Vec::new();
            let mut seen_impls: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
            for item in items {
                match item {
                    TopDecl::Impl(impl_decl) => {
                        let type_name = impl_decl.type_name.name.clone();
                        let iface_name = impl_decl.trait_name.name.clone();
                        seen_impls.insert((type_name.clone(), iface_name.clone()));
                        let mut provided_methods: std::collections::HashSet<String> = std::collections::HashSet::new();
                        for member in &impl_decl.members {
                            if let ImplItem::Fn(fn_decl) = member {
                                let mut new_fn = fn_decl.clone();
                                new_fn.name = Ident { name: format!("{}.{}", type_name, fn_decl.name.name), span: fn_decl.name.span };
                                new_fn.receiver = Some(Ident { name: type_name.clone(), span: impl_decl.type_name.span });
                                provided_methods.insert(fn_decl.name.name.clone());
                                out.push(TopDecl::Fn(new_fn));
                            }
                        }
                        // M19: Fill in default methods from the interface that weren't provided
                        if let Some(defaults) = interface_defaults.get(&iface_name) {
                            for (method_name, default_fd) in defaults {
                                if provided_methods.contains(method_name) { continue; }
                                let mut new_fn = default_fd.clone();
                                new_fn.name = Ident { name: format!("{}.{}", type_name, method_name), span: impl_decl.span };
                                new_fn.receiver = Some(Ident { name: type_name.clone(), span: impl_decl.type_name.span });
                                out.push(TopDecl::Fn(new_fn));
                            }
                        }
                    }
                    TopDecl::Module(md) => {
                        // M22: Recurse into module to expand nested impl blocks
                        let expanded_items = expand_items(&md.items, interface_defaults, interface_required, inherent_methods);
                        out.push(TopDecl::Module(ModuleDecl {
                            name: md.name.clone(), path: md.path.clone(),
                            items: expanded_items, is_file_level: md.is_file_level,
                            source_file: md.source_file.clone(), span: md.span,
                        }));
                    }
                    other => {
                        out.push(other.clone());
                    }
                }
            }
            // Auto-detect: for each interface, find types with inherent methods
            // matching all required methods, and expand default methods.
            // Collect already-emitted function names (recursing into modules)
            // to prevent duplicates when expand_impl_blocks is called multiple times.
            fn collect_fn_names(items: &[TopDecl]) -> std::collections::HashSet<String> {
                let mut names = std::collections::HashSet::new();
                for item in items {
                    match item {
                        TopDecl::Fn(fd) => { names.insert(fd.name.name.clone()); }
                        TopDecl::Module(md) => { names.extend(collect_fn_names(&md.items)); }
                        _ => {}
                    }
                }
                names
            }
            let already_emitted = collect_fn_names(&out);
            // Auto-detect: iterate over all interfaces that have defaults.
            // Interfaces with only default methods (no required) still need
            // expansion for every type (e.g. `interface Greeter { fn greet() -> Str { return "hello"; } }`).
            for (iface_name, _defaults) in interface_defaults.iter() {
                let required_methods = interface_required.get(iface_name).cloned().unwrap_or_default();
                for (type_name, type_methods) in inherent_methods {
                    // Skip if explicit impl already exists
                    if seen_impls.contains(&(type_name.clone(), iface_name.clone())) { continue; }
                    // Check if type has ALL required methods as inherent methods
                    let all_required_present = required_methods.iter()
                        .all(|rm| type_methods.contains(rm));
                    if all_required_present {
                        // Auto-expand default methods for this type+interface pair
                        if let Some(defaults) = interface_defaults.get(iface_name) {
                            for (method_name, default_fd) in defaults {
                                // Skip if type already has this method
                                if type_methods.contains(method_name) { continue; }
                                let fn_name = format!("{}.{}", type_name, method_name);
                                // Skip if already emitted (e.g. from previous expand_impl_blocks call)
                                if already_emitted.contains(&fn_name) { continue; }
                                let dummy_span = Span::new(0, 0);
                                let mut new_fn = default_fd.clone();
                                // Use the type-qualified name (matching explicit impl expansion).
                                // fn_key strips the receiver prefix to avoid doubling.
                                new_fn.name = Ident { name: format!("{}.{}", type_name, method_name), span: dummy_span };
                                new_fn.receiver = Some(Ident { name: type_name.clone(), span: dummy_span });
                                // Rewrite bare method calls to self.method() in the default body
                                // Build set of all interface method names for rewriting
                                let mut all_iface_methods: std::collections::HashSet<String> = std::collections::HashSet::new();
                                if let Some(req) = interface_required.get(iface_name) {
                                    for m in req { all_iface_methods.insert(m.clone()); }
                                }
                                if let Some(defs) = interface_defaults.get(iface_name) {
                                    for (m, _) in defs { all_iface_methods.insert(m.clone()); }
                                }
                                if let Some(ref mut body) = new_fn.body {
                                    rewrite_bare_calls(body, &all_iface_methods, dummy_span);
                                }
                                out.push(TopDecl::Fn(new_fn));
                            }
                        }
                    }
                }
            }
            out
        }

        let items = expand_items(&self.items, &interface_defaults, &interface_required, &inherent_methods);
        Program { items, source_files: self.source_files.clone(), root_dir: self.root_dir.clone(), span: self.span }
    }
}
