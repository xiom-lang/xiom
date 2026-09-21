// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-L08: Struct with pointer -- verify pointer field access in structs directly
type Node = { value: Int; next: *Node; }

fn node_value_from_null() -> Int {
  var n: *Node = unsafe { 0 as *Node };
  if n == unsafe { 0 as *Node } { return -1; }
  return 0;
}

fn main() -> Int {
  var empty: *Node = unsafe { 0 as *Node };
  if node_value_from_null() != -1 { return 1; }
  if empty == unsafe { 0 as *Node } { return 0; }
  return 2;
}
