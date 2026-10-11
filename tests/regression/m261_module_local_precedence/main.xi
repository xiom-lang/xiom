// m261 (REL-20261011-1124, C-ORBIT-07): a bare call inside a module must bind
// the CALLER module's own definition -- including module-PRIVATE fns -- ahead
// of the keep-first bare slot. The consumer's same-named arm(Str,Str,Str) must
// never receive mod_svc.ping()'s zero-arg call. Pre-fix this returned 1
// (consumer arm ran, value 0); post-fix mod_svc.arm returns 1 and main exits 0.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m261_consumer

use mod_svc;

fn arm(a: Str, b: Str, c: Str) -> Int {
  return 0;
}

fn main() -> Int {
  if mod_svc.ping() == 1 {
    return 0;
  }
  return 1;
}
