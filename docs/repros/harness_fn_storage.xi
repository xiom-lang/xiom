// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use repro_fn_storage;

fn plus_100(x: Int) -> Int { return x + 100; }

fn main() -> Int {
  var r1 = call_f(5);       // default _id: returns 5
  if r1 != 5 { return 1; }
  set_f(plus_100);
  var r2 = call_f(5);       // now returns 105
  if r2 != 105 { return 2; }
  return 0;
}
