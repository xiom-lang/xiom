// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m81 root: two modules with the same leaf fn (`is_even`) and the same
// higher-order helper (`apply`). Each module passes its OWN fn by value;
// R25 makes the fn-REFERENCE pick scope-first + deterministic, so
// alpha.check() == true and beta.check() == false regardless of registry
// iteration order.
module gateway

use alpha;
use beta;

fn main() -> Int {
  if !alpha.check() { return 1; }
  if beta.check() { return 2; }
  return 0;
}
