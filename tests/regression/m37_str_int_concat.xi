// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

module m37_str_int_concat
// Regression: `Str + Int` (and `Str + Char`/UInt) must FORMAT the integer
// via @xiom_int_to_string, not inttoptr it into a garbage pointer (which
// crashed with AV at runtime: "y = " + 42 -> deref of 0x2A).

use xiom.io;

fn _add(a: Int, b: Int) -> Int { return a + b; }

fn main() -> Int {
  var z: Int = 0;
  io.println("z = " + z);
  var n = _add(20, 22);
  io.println("n = " + n);
  var u: UInt64 = 12345;
  io.println("u = " + u);
  var g = 7;
  io.println("g = " + g);
  if ("v=" + 42) != "v=42" { return 1; }
  return 0;
}
