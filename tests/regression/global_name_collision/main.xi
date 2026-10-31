// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

use xiom.io;
use xiom.string;
use xiom.coll.a;
use xiom.coll.b;

fn main() -> Int {
  println("a=" + int_to_string(a.get_a()));
  println("b=" + float_to_fixed_str(b.get_b(), 2));
  if a.get_a() != 1 || b.get_b() != 0.5 {
    println("global_name_collision: FAIL");
    return 1;
  }
  println("global_name_collision: PASS");
  return 0;
}
