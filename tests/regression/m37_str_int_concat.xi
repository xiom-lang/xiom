// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_str_int_concat
// Regression: integer-to-Str formatting uses the EXPLICIT conversion
// (`convert.int_to_string`). The old `Str + Int` spelling is now rejected by
// the checker (SPEC: no implicit conversions; REL-20261010-1554-playground-2).
// The codegen concat gate still formats the converted Str values.

use xiom.io;
use xiom.convert;

fn _add(a: Int, b: Int) -> Int { return a + b; }

fn main() -> Int {
  var z: Int = 0;
  io.println("z = " + convert.int_to_string(z));
  var n = _add(20, 22);
  io.println("n = " + convert.int_to_string(n));
  var u: UInt64 = 12345;
  io.println("u = " + convert.int_to_string(u as Int));
  var g = 7;
  io.println("g = " + convert.int_to_string(g));
  if ("v=" + convert.int_to_string(42)) != "v=42" { return 1; }
  return 0;
}
