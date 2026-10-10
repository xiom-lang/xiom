// m256 companion: Parser with the arena field, Node imported selectively
// exactly like the real selfhost parser_state.xi.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m256_psp

use m256_ast.Node;

pub type Parser = {
  nodes: Vec[Node];
}
