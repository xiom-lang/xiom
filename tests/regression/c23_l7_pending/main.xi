// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// C23 lock, l7-39 shape (playground tools/compiler-repros/c23): Vec[struct]
// get/set/unwrap + Bool field reads inside loops. On hosts with LLVM 18 the
// driver's old double pipeline (opt -O1+ then clang -O2) counted 3 pending
// instead of 2. Returns 11 when the count is wrong.

use xiom.io;

type TodoItem = {
  id: Int;
  text: Str;
  done: Bool;
}

type TodoManager = {
  items: Vec[TodoItem];
  next_id: Int;
}

fn add_todo(manager: &mut TodoManager, text: Str) {
  manager.items.push(TodoItem{ id: manager.next_id, text: text, done: false });
  manager.next_id += 1;
}

fn mark_done(manager: &mut TodoManager, id: Int) -> Bool {
  var i = 0;
  while i < manager.items.len() {
    let item = manager.items.get(i).unwrap();
    if item.id == id {
      manager.items.set(i, TodoItem{ id: item.id, text: item.text, done: true });
      return true;
    };
    i += 1;
  };
  false
}

fn count_pending(manager: &TodoManager) -> Int {
  var count = 0;
  var i = 0;
  while i < manager.items.len() {
    if !manager.items.get(i).unwrap().done { count += 1; };
    i += 1;
  };
  count
}

fn main() -> Int {
  var manager = TodoManager{ items: Vec[TodoItem].new(), next_id: 1 };
  add_todo(&mut manager, "Buy groceries");
  add_todo(&mut manager, "Finish report");
  add_todo(&mut manager, "Call dentist");
  mark_done(&mut manager, 1);
  let pending = count_pending(&manager);
  io.println(pending.to_str());
  if pending != 2 { return 11; }
  return 0;
}
