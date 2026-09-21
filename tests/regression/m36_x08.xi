// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X08: Circular type references -- mutually referencing type structures
type NodeRef = { id: Int; next_id: Int; color: ColorRef; }
type ColorRef = { r: Int; g: Int; b: Int; owner_id: Int; }
fn build_node(id: Int, next: Int) -> NodeRef {
  var c = ColorRef{ r: id; g: next; b: 0; owner_id: id; };
  return NodeRef{ id: id; next_id: next; color: c; };
}
fn get_color(n: NodeRef) -> ColorRef { return n.color; }
fn get_node_id(n: NodeRef) -> Int { return n.id; }
fn linkage(a: Int, b: Int) -> NodeRef {
  return build_node(a, b);
}
fn main() -> Int {
  var n1 = build_node(1, 2);
  var n2 = build_node(2, 3);
  var n3 = build_node(3, 0);
  if n1.id != 1 { return 1; }
  if n1.next_id != 2 { return 2; }
  if n1.color.r != 1 { return 3; }
  if get_color(n2).g != 3 { return 4; }
  if get_node_id(n3) != 3 { return 5; }
  var chained = linkage(4, 5);
  if chained.id != 4 { return 6; }
  if chained.color.owner_id != 4 { return 7; }
  return 0;
}
