// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m227 (C-PULSE-09): consumer -- first cross-module access to the module-level
// Vec used to trap (the LEN field was called as a function pointer).
module m227_module_global_vec_len

use m227_bridge;

fn main() -> Int {
  m227_bridge.session_reset();
  if m227_bridge.session_count() != 0 { return 1; }
  m227_bridge.session_incr();
  if m227_bridge.session_count() != 1 { return 2; }
  return 0;
}
