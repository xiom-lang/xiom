// m256 (REL-20261010-1803-selfhost): tuple (Int, Vec[Int]) returned from a
// match arm whose Vec payload binding is an M19-erased i64 handle. Pre-fix
// the tuple element was named Int and clang rejected
// `ret %struct.Tuple__Int__Vec` on a %struct.Tuple__Int__Int value
// (%tmp36 defined with type Tuple__Int__Int ... but expected Tuple__Int__Vec).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m256_vec_arm_binding_tuple

use m256_ast.Node;
use m256_ast.NodeKind;
use m256_psp.Parser;
use m256_cgp;

fn main() -> Int {
  var nodes = Vec[Node].new();
  nodes.push(Node{ kind: NodeKind.NkStmtMatch(7, Vec[Int].new()) });
  let p = Parser{ nodes: nodes };
  return m256_cgp.use_it(&p, 0) - 7;
}
