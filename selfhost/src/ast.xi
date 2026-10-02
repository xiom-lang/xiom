// XIOM -- Selfhost AST model (Phase 2, arena form)
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Arena AST for the parser port. The Rust parser builds recursive value
// trees (Box<Expr> children, Vec<StmtOrExpr>, ...). Direct recursive enums
// are NOT reliable in the current compiler: a recursive payload is boxed as
// a pointer, and nesting one boxed payload inside another mis-lowers (probes
// tmp/sprintc/phase2_parser/probe_rec_nested.xi, probe_rec_locals.xi crash
// with 0xC000001D while two-level construction from leaves happens to work;
// COMPILER_BUGS 2026-10-02 (e)).
//
// The selfhost therefore stores nodes in a flat arena (`Vec[Node]`) and uses
// integer INDICES for every child (-1 = absent). The representation is not
// part of the parity contract: the canonical dump (ast_dump.xi) resolves
// indices and emits the same lines as crates/xiom/src/main.rs::dump_ast.
// NodeKind variants are `Nk`-prefixed (the Phase 1 `Tk` convention) so they
// never collide with builtins or stdlib variants.
//
// Field order inside every variant mirrors the Rust AST variant order
// (crates/xiom-ast/src/lib.rs) 1:1. Optional children use -1; lists use an
// empty Vec. `is_*` booleans are Int 0/1.

module selfhost_ast

// ============================================================================
// Spans and nodes
// ============================================================================

pub type Span = {
  line: Int;
  col: Int;
  byte_start: Int;
  byte_end: Int;
}

pub type Node = {
  kind: NodeKind;
  span: Span;
}

/// Absent child index.
pub fn none_idx() -> Int {
  return -1;
}

pub fn span_new(line: Int, col: Int, byte_start: Int, byte_end: Int) -> Span {
  return Span{ line: line, col: col, byte_start: byte_start, byte_end: byte_end };
}

pub fn span_zero() -> Span {
  return Span{ line: 0, col: 0, byte_start: 0, byte_end: 0 };
}

pub fn node_new(kind: NodeKind, span: Span) -> Node {
  return Node{ kind: kind, span: span };
}

/// Interned-style Ident node (`Ident { name, span }`).
pub fn ident_node(name: Str, span: Span) -> Node {
  return Node{ kind: NodeKind.NkIdent(name), span: span };
}

// ============================================================================
// Operator / derive / trait codes
// ============================================================================

pub const UOP_NEG: Int = 0;
pub const UOP_NOT: Int = 1;
pub const UOP_REF: Int = 2;
pub const UOP_MUTREF: Int = 3;
pub const UOP_BITNOT: Int = 4;
pub const UOP_DEREF: Int = 5;

pub const OP_ADD: Int = 0;
pub const OP_SUB: Int = 1;
pub const OP_MUL: Int = 2;
pub const OP_DIV: Int = 3;
pub const OP_REM: Int = 4;
pub const OP_EQ: Int = 5;
pub const OP_NEQ: Int = 6;
pub const OP_LT: Int = 7;
pub const OP_GT: Int = 8;
pub const OP_LE: Int = 9;
pub const OP_GE: Int = 10;
pub const OP_SHL: Int = 11;
pub const OP_SHR: Int = 12;
pub const OP_AND: Int = 13;
pub const OP_OR: Int = 14;
pub const OP_ASSIGN: Int = 15;
pub const OP_BITXOR: Int = 16;
pub const OP_BITAND: Int = 17;
pub const OP_BITOR: Int = 18;

pub const DRV_EQ: Int = 0;
pub const DRV_CLONE: Int = 1;
pub const DRV_DISPLAY: Int = 2;
pub const DRV_HASH: Int = 3;
pub const DRV_ORD: Int = 4;
pub const DRV_DEBUG: Int = 5;

// ============================================================================
// Node kinds (mirror of xiom_ast variants, arena-shaped)
// ============================================================================

pub type NodeKind = enum {
  // --- program / top-level declarations ---
  NkProgram(items: Vec[Int]),
  NkModule(name: Int, path: Vec[Int], items: Vec[Int], file_level: Int, has_source: Int, source: Str),
  NkUse(path: Vec[Int], glob: Int, alias: Int),
  NkTypeDecl(is_pub: Int, name: Int, generics: Vec[Int], fields: Vec[Int], derived: Vec[Int], invariants: Vec[Int], derives: Vec[Int], alias: Int),
  NkDerivedField(name: Int, ty: Int, expr: Int),
  NkEnumDecl(is_pub: Int, name: Int, generics: Vec[Int], variants: Vec[Int], derives: Vec[Int]),
  NkVariant(name: Int, fields: Vec[Int]),
  NkInterface(is_pub: Int, name: Int, generics: Vec[Int], parent: Int, members: Vec[Int]),
  NkImpl(trait_name: Int, trait_args: Vec[Int], type_name: Int, members: Vec[Int]),
  NkFn(is_pub: Int, is_async: Int, recv: Int, name: Int, generics: Vec[Int], params: Vec[Int], ret: Int, contracts: Vec[Int], body: Int, attrs: Vec[Int]),
  NkConst(is_pub: Int, is_mut: Int, name: Int, ty: Int, value: Int),
  NkExtern(linkage: Str, fns: Vec[Int]),
  NkSpawnTop(move_: Int, block: Int),
  NkField(name: Int, ty: Int),
  NkIdent(name: Str),
  NkParam(name: Int, ty: Int, mutself: Int, refself: Int),
  NkGeneric(name: Int, bounds: Vec[Int], is_const: Int, const_ty: Int),
  NkAttr(name: Int, args: Vec[Int]),
  NkAttrArg(key: Str, value: Str),
  NkRequires(expr: Int),
  NkEnsures(expr: Int),

  // --- types ---
  NkTyNamed(name: Int, args: Vec[Int]),
  NkTyRef(inner: Int),
  NkTyMutRef(inner: Int),
  NkTyOption(inner: Int),
  NkTyResult(ok: Int, err: Int),
  NkTyVec(inner: Int),
  NkTySlice(inner: Int),
  NkTyMap(key: Int, value: Int),
  NkTySet(inner: Int),
  NkTyTuple(items: Vec[Int]),
  NkTyPtr(inner: Int),
  NkTyArray(size_expr: Int, inner: Int),
  NkTyFn(params: Vec[Int], ret: Int),
  NkTyImplTrait(bounds: Vec[Int]),
  NkTyAnonStruct(fields: Vec[Int]),
  NkTyNever,

  // --- literals ---
  NkLitInt(value: UInt),
  NkLitBigInt(hi: UInt, lo: UInt),
  NkLitFloat(lex: Str),
  NkLitStr(data: Vec[UInt8]),
  NkLitChar(cp: Int),
  NkLitBool(value: Int),

  // --- expressions ---
  NkExprIdent(name: Int),
  NkExprParen(inner: Int),
  NkExprUnary(op: Int, inner: Int),
  NkExprBinary(l: Int, op: Int, r: Int),
  NkExprTry(inner: Int),
  NkExprImply(l: Int, r: Int),
  NkExprIs(inner: Int, pattern: Int),
  NkExprField(inner: Int, name: Int),
  NkExprCall(callee: Int, args: Vec[Int]),
  NkExprGenericCall(callee: Int, types: Vec[Int], args: Vec[Int]),
  NkExprIndex(base: Int, index: Int),
  NkExprAtPre(inner: Int),
  NkExprRef(inner: Int),
  NkExprMutRef(inner: Int),
  NkExprSome(inner: Int),
  NkExprNone,
  NkExprOk(inner: Int),
  NkExprErr(inner: Int),
  NkExprStruct(name: Int, fields: Vec[Int], base: Int),
  NkFieldInit(name: Int, value: Int),
  NkExprArray(items: Vec[Int]),
  NkExprBlock(block: Int),
  NkExprConstBlock(inner: Int),
  NkExprClosure(params: Vec[Int], ret: Int, body: Int),
  NkExprPipeClosure(names: Vec[Int], body: Int),
  NkExprAwait(inner: Int),
  NkExprComptime(inner: Int),
  NkExprAs(inner: Int, ty: Int),
  NkExprTuple(items: Vec[Int]),
  NkExprIf(cond: Int, then: Int, elifs: Vec[Int], els: Int),
  NkElif(cond: Int, block: Int),
  NkExprMatch(scrut: Int, arms: Vec[Int]),
  NkExprUnsafe(block: Int),
  NkExprError,

  // --- statements ---
  NkLet(name: Int, ty: Int, value: Int),
  NkVar(name: Int, ty: Int, value: Int),
  NkAssign(l: Int, r: Int),
  NkReturn(value: Int),
  NkExprStmt(expr: Int),
  NkStmtIf(cond: Int, then: Int, elifs: Vec[Int], els: Int),
  NkStmtMatch(scrut: Int, arms: Vec[Int]),
  NkWhile(cond: Int, body: Int, inv: Int, label: Int),
  NkFor(name: Int, iter: Int, body: Int, label: Int),
  NkStmtSpawn(move_: Int, block: Int),
  NkDestructure(names: Vec[Int], value: Int),
  NkBreak(label: Int),
  NkContinue(label: Int),
  NkAsm(template: Str, outputs: Vec[Int], inputs: Vec[Int], clobbers: Vec[Str]),
  NkAsmOut(constraint: Str, name: Int),
  NkAsmIn(constraint: Str, expr: Int),
  NkDefer(block: Int),
  NkAssert(cond: Int, msg: Int),
  NkDebugger,

  // --- blocks / arms / stmt-or-expr wrappers ---
  NkBlock(stmts: Vec[Int]),
  NkStmtW(stmt: Int),
  NkTailW(expr: Int),
  NkMatchArm(pattern: Int, guard: Int, body: Int, body_is_block: Int),

  // --- patterns ---
  NkPatWildcard,
  NkPatIdent(name: Int),
  NkPatVariant(name: Int, fields: Vec[Int]),
  NkPatStruct(name: Int, fields: Vec[Int]),
  NkPatField(name: Int, pattern: Int),
  NkPatTuple(items: Vec[Int]),
  NkPatSome(inner: Int),
  NkPatNone,
  NkPatOk(inner: Int),
  NkPatErr(inner: Int),
  NkPatOr(items: Vec[Int]),
}
