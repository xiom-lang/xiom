// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_timer_literal;

fn main() -> Int {
  var t = make_timer(42);
  var r = read_fields(&t);
  if r != 0 { return r; }
  return 0;
}
