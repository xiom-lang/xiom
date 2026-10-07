// m204 (C-PULSE-07): cross-module callee for the module-scope init lock.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module xiom.rate

pub fn rate_keyed_new(capacity: Int, refill: Int) -> Int {
  return capacity + refill;
}
