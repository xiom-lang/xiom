// AXIOM — AST
// Copyright (c) 2026 Eleftherios Notas
// Licensed under the MIT or Apache-2.0 license, at your option.

//! AXIOM Abstract Syntax Tree — every construct from the EBNF grammar.
//! This is the single source of truth for what the parser produces
//! and what every downstream pass consumes.

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
    Struct(Ident, Vec<(Ident, Expr)>, Span),
    /// `[expr, ...]` — array literal
    Array(Vec<Expr>, Span),
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
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident(i) => i.span,
            Expr::Int(_, s) | Expr::Float(_, s) | Expr::Str(_, s) | Expr::Char(_, s) | Expr::Bool(_, s) => *s,
            Expr::Paren(_, s) | Expr::Unary(_, _, s) | Expr::Binary(_, _, _, s) | Expr::Try(_, s) => *s,
            Expr::Imply(_, _, s) | Expr::Is(_, _, s) | Expr::Field(_, _, s) | Expr::Call(_, _, s) => *s,
            Expr::Index(_, _, s) | Expr::AtPre(_, s) | Expr::Ref(_, s) | Expr::MutRef(_, s) => *s,
            Expr::Some(_, s) | Expr::None(s) | Expr::Ok(_, s) | Expr::Err(_, s) => *s,
            Expr::Struct(_, _, s) | Expr::Array(_, s) | Expr::Closure(_, _, _, s) | Expr::PipeClosure(_, _, s) => *s,
            Expr::Await(_, s) | Expr::Comptime(_, s) | Expr::As(_, _, s) => *s,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Ref,
    MutRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    Eq, Neq, Lt, Gt, Le, Ge,
    And, Or,
    Assign,
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
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::Assign => "=",
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
    /// `while expr block`
    While(Expr, Block, Span),
    /// `for ident in expr block`
    For(Ident, Expr, Block, Span),
    /// `spawn block`
    Spawn(Block, Span),
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
// Functions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub name: Ident,
    pub bounds: Vec<Ident>, // interface names
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
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
    Eq, Clone, Display, Hash, Ord,
}

impl DeriveTrait {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Eq" => Some(Self::Eq),
            "Clone" => Some(Self::Clone),
            "Display" => Some(Self::Display),
            "Hash" => Some(Self::Hash),
            "Ord" => Some(Self::Ord),
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
}
