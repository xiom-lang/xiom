// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S03: Parser AST node construction -- tree node types and factory functions
type AstKind = { kind: Int; line: Int; col: Int; }
type AstExpr = { kind: Int; left: Int; right: Int; value: Int; }
type AstNode = { tag: Int; data: Int; children_count: Int; }
fn make_kind(k: Int, l: Int, c: Int) -> AstKind {
  return AstKind{ kind: k; line: l; col: c; };
}
fn make_expr(k: Int, l: Int, r: Int, v: Int) -> AstExpr {
  return AstExpr{ kind: k; left: l; right: r; value: v; };
}
fn make_node(tag: Int, data: Int, count: Int) -> AstNode {
  return AstNode{ tag: tag; data: data; children_count: count; };
}
fn kind_tag(k: AstKind) -> Int { return k.kind * 1000 + k.line * 10 + k.col; }
fn expr_tag(e: AstExpr) -> Int { return e.kind + e.left + e.right + e.value; }
fn node_tag(n: AstNode) -> Int { return n.tag + n.data + n.children_count; }
fn main() -> Int {
  var k = make_kind(1, 5, 3);
  if k.kind != 1 { return 1; }
  if k.line != 5 { return 2; }
  if k.col != 3 { return 3; }
  var e = make_expr(10, 1, 2, 42);
  if expr_tag(e) != 55 { return 4; }
  var n = make_node(100, 200, 3);
  if node_tag(n) != 303 { return 5; }
  if kind_tag(k) != 1053 { return 6; }
  return 0;
}
