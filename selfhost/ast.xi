// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M20-B1: XIOM AST Data Structures for Self-Hosting Compiler
// Simplified -- uses only supported XIOM features.

// ============================================================================
// Tokens
// ============================================================================
pub enum TokenKind {
  Fn, Return, If, Else, While, For, Let, Var, Match,
  Type, Enum, Interface, Use, Module, Pub, Extern, Unsafe, Const,
  True, False, IntT, FloatT, BoolT, StrT, CharT, VoidT,
  Ident(name: Str),
  IntLit(val: Int),
  FloatLit(val: Float64),
  StringLit(val: Str),
  Plus, Minus, Star, Slash, Eq, EqEq, NotEq, Lt, Gt,
  Arrow, FatArrow, Dot, Colon, Semi, Comma,
  LParen, RParen, LBrace, RBrace, LBracket, RBracket,
  Eof, Error(message: Str),
}

// ============================================================================
// Binary and Unary Operators
// ============================================================================
pub enum BinOp { Add, Sub, Mul, Div, Eq, Neq, Lt, Gt, Lte, Gte, And, Or }
pub enum UnaryOp { Neg, Not, Deref }

// ============================================================================
// Types
// ============================================================================
pub enum TypeNode {
  Named(name: Str),
  Ptr(inner: Str),       // inner type name as string
  Array(inner: Str, size: Int),
}

// ============================================================================
// Expressions
// ============================================================================
pub enum ExprNode {
  Ident(name: Str),
  IntLit(val: Int),
  FloatLit(val: Float64),
  BoolLit(val: Bool),
  StringLit(val: Str),
  Binary(left: Str, op: BinOp, right: Str),  // use string IDs for tree refs
  Call(func: Str, args: Int),                // func name + arg count
  Field(obj: Str, field: Str),
  Block(stmt_count: Int),
  If(cond: Str, then_count: Int, else_count: Int),
  StructLit(name: Str, field_count: Int),
  PipeClosure(param_count: Int, body: Str),
  Some(inner: Str),
  None,
  Ok(inner: Str),
  Err(inner: Str),
}

// ============================================================================
// Statements
// ============================================================================
pub enum StmtNode {
  Expr(expr_id: Int),
  Let(name: Str, type_name: Str, value_id: Int),
  Var(name: Str, type_name: Str),
  IfStmt(cond_id: Int, then_count: Int, else_count: Int),
  While(cond_id: Int, body_count: Int),
  For(loop_var: Str, iter_id: Int, body_count: Int),
  Return(value_id: Int),        // -1 = no value (return void)
  Match(scrutinee_id: Int, arm_count: Int),
}

// ============================================================================
// Patterns
// ============================================================================
pub enum PatternNode {
  Wildcard,
  Ident(name: Str),
  IntLit(val: Int),
  BoolLit(val: Bool),
  Variant(name: Str, field_count: Int),
  Or(alt_count: Int),
}

// ============================================================================
// Top-level declarations (flat, not recursive)
// ============================================================================
pub enum TopDeclKind {
  FnDecl, TypeDecl, EnumDecl, InterfaceDecl, UseDecl, ExternDecl, ConstDecl,
}

// Function declaration data
pub type FnData = {
  name: Str;
  param_count: Int;
  ret_type_name: Str;     // "" = void
  is_pub: Bool;
}

// Type (struct) declaration data
pub type TypeData = {
  name: Str;
  field_count: Int;
  is_pub: Bool;
}

// Enum declaration data
pub type EnumData = {
  name: Str;
  variant_count: Int;
  is_pub: Bool;
}

// A parameter
pub type ParamInfo = { name: Str; type_name: Str; }

// A struct field
pub type FieldInfo = { name: Str; type_name: Str; }

// An enum variant
pub type VariantInfo = { name: Str; field_count: Int; }

// A variant field
pub type VariantFieldInfo = { name: Str; type_name: Str; }

// ============================================================================
// Self-test
// ============================================================================
fn main() -> Int {
  // Verify enum construction
  let _t = TokenKind.Fn;
  let _t2 = TokenKind.Ident("hello");
  let _b = BinOp.Add;
  let _u = UnaryOp.Neg;
  let _ty = TypeNode.Named("Int");
  let _e = ExprNode.IntLit(42);
  let _s = StmtNode.Return(0);
  let _p = PatternNode.Wildcard;
  return 0;
}
