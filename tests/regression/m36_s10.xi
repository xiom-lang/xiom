// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-S10: Symbol table -- scope push/pop and nested resolution
type Scope = { level: Int; parent: Int; entries: Int; }
fn make_scope(level: Int, parent: Int, entries: Int) -> Scope {
  return Scope{ level: level; parent: parent; entries: entries; };
}
fn push_scope(current: Scope) -> Scope {
  return make_scope(current.level + 1, current.level, 0);
}
fn pop_scope(current: Scope, parent: Scope) -> Scope {
  if current.parent == parent.level { return parent; }
  return make_scope(parent.level, parent.parent, parent.entries);
}
fn resolve_in_scope(s: Scope, expected_entries: Int) -> Int {
  if s.entries >= expected_entries { return s.level; }
  return -1;
}
fn scope_depth(s: Scope) -> Int {
  return s.level;
}
fn is_global(s: Scope) -> Bool {
  return s.level == 0;
}
fn main() -> Int {
  var global = make_scope(0, -1, 100);
  if !is_global(global) { return 1; }
  if scope_depth(global) != 0 { return 2; }
  var inner = push_scope(global);
  if scope_depth(inner) != 1 { return 3; }
  if inner.parent != 0 { return 4; }
  var inner2 = push_scope(inner);
  if scope_depth(inner2) != 2 { return 5; }
  var popped = pop_scope(inner2, inner);
  if scope_depth(popped) != 1 { return 6; }
  var back = pop_scope(popped, global);
  if scope_depth(back) != 0 { return 7; }
  var r = resolve_in_scope(inner, 0);
  if r != 1 { return 8; }
  if resolve_in_scope(inner, 1) != -1 { return 9; }
  return 0;
}
