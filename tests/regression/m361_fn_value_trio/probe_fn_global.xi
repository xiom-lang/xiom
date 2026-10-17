// Probe (REL-20261017-1227-packages item 1, xiom.pulse 0.2 hook seam):
// a module-global fn-typed variable is accepted by the type checker but
// fails codegen at 0.64.6 (undefined @__fnwrap / malformed top-level IR).
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module probe_fn_global

fn inc(x: Int) -> Int { return x + 1; }

var hook: fn(Int) -> Int = inc;

pub fn main() -> Int {
  return hook(1);
}
