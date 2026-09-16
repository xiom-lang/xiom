// XIOM -- AST
// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

//! XIOM Abstract Syntax Tree -- every construct from the EBNF grammar.
//! This is the single source of truth for what the parser produces
//! and what every downstream pass consumes.

pub mod structural;

// ============================================================================
// Applicability -- suggestion confidence contract (5c-R, rustc lesson)
// ============================================================================

/// How confident the compiler is that a suggested fix is correct.
/// Tools (LSP, xiom-fix) auto-apply only `MachineApplicable` suggestions.
/// Direct transplant from rustc's `Applicability` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    /// The suggestion is definitely correct -- safe to auto-apply.
    MachineApplicable,
    /// The suggestion may be correct but the compiler cannot guarantee it.
    MaybeIncorrect,
    /// The suggestion contains placeholder types that need resolution first.
    HasPlaceholders,
    /// The suggestion is untested / ad-hoc -- never auto-apply.
    Unspecified,
}

use std::fmt;

// ============================================================================
// Source location
// ============================================================================

/// A position in source code: line/column (1-based) plus BYTE OFFSETS.
///
/// AUDIT FIX (readiness Stage 2, finding: "line/col pairs without offsets or
/// end positions"): Spans now carry a half-open byte range [byte_start,
/// byte_end) into the tokenized source (post-BOM), which unlocks precise
/// diagnostic ranges, LSP UTF-16 math, incremental change detection and
/// DWARF line tables without touching any existing consumer.
///
/// COMPATIBILITY: `new(line, col)` still constructs a span WITHOUT offsets
/// (byte range 0..0 -- used by synthetic/dummy sites until they are
/// migrated). Equality and hashing deliberately consider ONLY (line, col),
/// preserving every pre-existing span-comparison semantic; use
/// `same_range()` for offset-aware comparison.
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    pub line: u32,
    pub col: u32,
    pub byte_start: u32,
    pub byte_end: u32,
}

impl PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.col == other.col
    }
}
impl Eq for Span {}
impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.line.hash(state);
        self.col.hash(state);
    }
}

impl Span {
    pub const fn new(line: u32, col: u32) -> Self {
        Self { line, col, byte_start: 0, byte_end: 0 }
    }

    /// Full-fidelity constructor: position PLUS byte range.
    pub const fn range(line: u32, col: u32, byte_start: u32, byte_end: u32) -> Self {
        Self { line, col, byte_start, byte_end }
    }

    /// The byte range, or None for synthetic spans constructed via
    /// `new()` (no offset information).
    pub fn byte_range(&self) -> Option<(u32, u32)> {
        if self.byte_start == 0 && self.byte_end == 0 {
            None
        } else {
            Some((self.byte_start, self.byte_end))
        }
    }

    /// True when both spans carry offsets and their byte ranges match
    /// exactly (stronger than PartialEq).
    pub fn same_range(&self, other: &Span) -> bool {
        match (self.byte_range(), other.byte_range()) {
            (Some(a), Some(b)) => a == b && self.line == other.line,
            _ => false,
        }
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
    /// `impl Trait` -- opaque return type (existential)
    ImplTrait(Vec<Ident>),
    /// Anonymous struct type `{ field: Type; ... }` -- used in generic
    /// function signatures where the struct has no standalone name.
    AnonStruct(Vec<FieldDecl>),
    /// v0.55: `!` -- the Never type (bottom type). Functions returning `!`
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
    /// D1 (2026-08-08): integer literal beyond u64 (fits u128/i128) -- used for
    /// native Int128/UInt128 literals. Produced by the lexer's BigInt token.
    BigInt(u128, Span),
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
    /// `expr?` -- error propagation
    Try(Box<Expr>, Span),
    /// `expr => expr` -- implication (in contracts)
    Imply(Box<Expr>, Box<Expr>, Span),
    /// `expr is Pattern` -- type test
    Is(Box<Expr>, Pattern, Span),
    /// `expr.field` -- field access
    Field(Box<Expr>, Ident, Span),
    /// `expr(args)` -- function call
    Call(Box<Expr>, Vec<Expr>, Span),
    /// v0.56/P2-6: `expr::<Type1, Type2>(args)` -- turbofish call (multiple generic type args).
    /// Stores the parsed types for CTFE builtins and generic specialization.
    GenericCall(Box<Expr>, Vec<Type>, Vec<Expr>, Span),
    /// `expr[index]` -- index
    Index(Box<Expr>, Box<Expr>, Span),
    /// `expr@pre` -- pre-state (in contracts)
    AtPre(Box<Expr>, Span),
    /// `&expr` -- reference
    Ref(Box<Expr>, Span),
    /// `&mut expr` -- mutable reference
    MutRef(Box<Expr>, Span),
    /// `Some(expr)`
    Some(Box<Expr>, Span),
    /// `None`
    None(Span),
    /// `Ok(expr)`
    Ok(Box<Expr>, Span),
    /// `Err(expr)`
    Err(Box<Expr>, Span),
    /// `Type{ field: val, ... }` -- struct literal
    Struct(Ident, Vec<(Ident, Expr)>, Option<Box<Expr>>, Span),
    /// `[expr, ...]` -- array literal
    Array(Vec<Expr>, Span),
    /// `{ stmt; ... }` -- bare block expression
    BlockExpr(Block, Span),
    /// v0.54: `const { expr }` -- compile-time constant block expression.
    /// Evaluated by CTFE Phase A; the result replaces the node before codegen.
    ConstBlock(Box<Expr>, Span),
    /// `fn(params) -> RetType { ... }` -- closure
    Closure(Vec<Param>, Option<Box<Type>>, Block, Span),
    /// `|x, y| expr` -- pipe closure
    PipeClosure(Vec<Ident>, Box<Expr>, Span),
    /// `await expr`
    Await(Box<Expr>, Span),
    /// `comptime expr`
    Comptime(Box<Expr>, Span),
    /// `expr as Type` -- type cast
    As(Box<Expr>, Type, Span),
    /// `(a, b, ...)` -- tuple expression
    Tuple(Vec<Expr>, Span),
    /// `if cond { then } else { else }` -- if-expression
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
/// Cannot be serialized -- panics on encode (prevents caching errors).
/// (Direct transplant from rustc's `ErrorGuaranteed` -- see docs/rust/05-diagnostics.md)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorGuaranteed {
    _private: (),
}

impl ErrorGuaranteed {
    /// Create a new error guarantee.
    ///
    /// AUDIT FIX (readiness Stage 2b): this was pub(crate) + allow(dead_code)
    /// -- the parser and checker could never construct it, so the
    /// error-poisoning design was aspirational. It is now PUBLIC; every
    /// ParseError/CheckError carries one as the proof token that a
    /// diagnostic has definitively been emitted, and downstream passes
    /// skip error-poisoned work on the strength of it.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident(i) => i.span,
            Expr::Int(_, s) | Expr::BigInt(_, s) | Expr::Float(_, s) | Expr::Str(_, s) | Expr::Char(_, s) | Expr::Bool(_, s) => *s,
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
    /// Downstream passes should skip silently -- the diagnostic was already emitted.
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
    /// `_` -- wildcard
    Wildcard(Span),
    /// `ident` -- variable binding
    Ident(Ident),
    /// `Variant(field1, field2)` -- enum variant pattern
    Variant(Ident, Vec<Ident>, Span),
    /// v0.56/P1-1: `TypeName { field1, field2: pat2 }` -- struct destructure pattern
    Struct(Ident, Vec<(Ident, Pattern)>, Span),
    /// v0.56/P1-2: `(a, b, c)` -- tuple destructure pattern
    Tuple(Vec<Pattern>, Span),
    /// Literal pattern (Int, Float, Bool, Str, Char)
    Lit(Literal),
    /// `Some(pattern)`
    Some(Box<Pattern>, Span),
    /// `None`
    None(Span),
    /// `Ok(pattern)`
    Ok(Box<Pattern>, Span),
    /// `Err(pattern)`
    Err(Box<Pattern>, Span),
    /// `A | B | C` -- or-pattern (matches if any alternative matches)
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
    /// `expr;` -- expression statement
    Expr(Expr, Span),
    /// `if expr block {elif expr block} [else block]`
    If(Expr, Block, Vec<(Expr, Block)>, Option<Block>, Span),
    /// `match expr { arms }`
    Match(Expr, Vec<MatchArm>, Span),
    /// `while expr [invariant: expr] block` -- optional loop label for break/continue targeting
    While(Expr, Block, Option<Expr>, Span, Option<Ident>),
    /// `for ident in expr block` -- optional loop label for break/continue targeting
    For(Ident, Expr, Block, Span, Option<Ident>),
    /// `spawn [move] { block }`
    Spawn(Block, Span, /* move */ bool),
    /// `var (a, b) = expr;` / `let (a, b) = expr;`
    Destructure(Vec<Ident>, Expr, Span),
    /// `break;` or `break 'label;`
    Break(Option<Ident>, Span),
    /// `continue;` or `continue 'label;`
    Continue(Option<Ident>, Span),
    /// v0.55: `asm("...")` -- inline assembly statement
    Asm(AsmBlock),
    /// v0.55: `defer { expr }` -- guaranteed scope-exit execution
    Defer(Block, Span),
    /// BUG 27: `assert(cond)` / `assert(cond, "msg")` -- runtime-checked
    /// invariant (panics cleanly on violation via xiom_panic).
    Assert(Expr, Option<Expr>, Span),
    /// BUG 27: `debugger;` -- break into the attached debugger (no-op
    /// without one). Used with the xiom-dbg DAP server.
    Debugger(Span),
}

/// v0.55: Inline assembly block -- `asm("template" : outputs : inputs : clobbers)`
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
    /// v0.56: Parent interface for inheritance (DerefMut: Deref)
    pub parent: Option<Ident>,
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
    /// `true` when declared as `pub const` -- enables cross-module visibility.
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
// ImplDecl -- impl Trait for Type { ... }
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ImplDecl {
    pub trait_name: Ident,
    /// Generic args applied to the trait (e.g. `impl Num[Int]` -> [Int]).
    /// Empty for the `impl Trait for Type` form.
    pub trait_args: Vec<Type>,
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

/// D1 (2026-08-08): extract the type name from a Type AST node
/// (e.g. `impl Num[Int]` -> "Int"). Free function so nested helpers can use it.
pub fn type_name_from_ast(ty: &Type) -> String {
    match ty {
        Type::Named(id, _) => id.name.clone(),
        Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) => type_name_from_ast(inner),
        Type::Vec(inner) => format!("Vec[{}]", type_name_from_ast(inner)),
        Type::Option(inner) => format!("Option[{}]", type_name_from_ast(inner)),
        Type::Slice(inner) => format!("Slice[{}]", type_name_from_ast(inner)),
        _ => format!("{ty:?}"),
    }
}

impl Program {
    pub fn new(items: Vec<TopDecl>, span: Span) -> Self {
        Self { items, source_files: Vec::new(), root_dir: None, span }
    }

}
