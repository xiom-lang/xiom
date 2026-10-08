// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m227 (C-PULSE-09): module A owns a module-level `Vec[SessionStore]`; the
// bridge module B calls into A. `stores.len()` inside sb_ensure used to fall
// to the fn-field call path (BUG 29): the module-global receiver was invisible
// to Vec builtin detection, so the LEN integer was loaded and `inttoptr`+called
// (0xC000001D / 0xC0000005 at the first access on Windows).

module m227_store

pub type SessionStore = {
  count: Int;
  cap: Int;
}

pub var stores: Vec[SessionStore] = Vec[SessionStore].new();

pub fn sb_ensure() {
  if stores.len() == 0 {
    var s = SessionStore{ count: 0, cap: 4 };
    stores.push(s);
  }
}

pub fn sb_reset() {
  stores = Vec[SessionStore].new();
  sb_ensure();
}

pub fn sb_incr() {
  sb_ensure();
  stores[0].count = stores[0].count + 1;
}

pub fn sb_count_at(s: &SessionStore) -> Int {
  return s.count;
}

pub fn sb_count() -> Int {
  sb_ensure();
  return sb_count_at(&stores[0]);
}
