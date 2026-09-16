// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_bare_prelude;

fn main() -> Int {
  var s = to_str(42);
  if s != "42" { return 1; }
  return 0;
}
