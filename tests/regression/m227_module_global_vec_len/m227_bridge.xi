// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m227 (C-PULSE-09): bridge module -- cross-module access to A's store.
module m227_bridge

use m227_store;

pub fn session_count() -> Int {
  return m227_store.sb_count();
}

pub fn session_reset() {
  m227_store.sb_reset();
}

pub fn session_incr() {
  m227_store.sb_incr();
}
