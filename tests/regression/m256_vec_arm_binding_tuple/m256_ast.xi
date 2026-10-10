// m256 (REL-20261010-1803-selfhost): a match-arm Vec payload binding whose
// field name REPEATS across enum variants (`arms` in NkStmtMatch and
// NkExprMatch) is M19-erased to an i64 handle slot; returning it inside a
// tuple made the literal build Tuple__Int__Int against the declared
// Tuple__Int__Vec and clang rejected the ret. Leading module: the ast
// module with Node + the repeated-field enum.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m256_ast

pub type Node = {
  kind: NodeKind;
}

pub type NodeKind = enum {
  NkStmtMatch(scrut: Int, arms: Vec[Int]),
  NkExprMatch(scrut: Int, arms: Vec[Int]),
  NkOther,
}
