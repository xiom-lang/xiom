// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-D12: Doubly linked list -- forward/backward traversal verification (null only)
type DNode = { value: Int; prev: *DNode; next: *DNode; }

fn dll_forward_len(head: *DNode) -> Int {
  var len: Int = 0;
  var cur: *DNode = head;
  while cur != unsafe { 0 as *DNode } { len = len + 1; unsafe { cur = (*cur).next; } }
  return len;
}

fn dll_backward_len(tail: *DNode) -> Int {
  var len: Int = 0;
  var cur: *DNode = tail;
  while cur != unsafe { 0 as *DNode } { len = len + 1; unsafe { cur = (*cur).prev; } }
  return len;
}

fn main() -> Int {
  var empty: *DNode = unsafe { 0 as *DNode };
  if dll_forward_len(empty) != 0 { return 1; }
  if dll_backward_len(empty) != 0 { return 2; }
  return 0;
}
