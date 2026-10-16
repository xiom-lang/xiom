// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m338 lock (REL-20261016-1128-engine class 1) -- ENTRY-module form: the
// entry module itself declares an enum with a variant named `Str`;
// `Str::from_utf8` in that same module must still resolve to the builtin
// Str assoc table (pre-fix: T001 "cannot call 'from_utf8' on this
// expression"; post-fix: bytes exact at runtime, "AB").
module m338_entry_variant_str_shadow;

use xiom.io;

pub type JLValue = enum {
  Null,
  Str(value: Str),
}

fn main() -> Int {
  var buf = Vec[UInt8].new();
  buf.push(65); // 'A'
  buf.push(66); // 'B'
  var s = Str::from_utf8(buf);
  if s != "AB" {
    io.println("m338 entry: [CORRUPT] from_utf8 bytes = " + s);
    return 1;
  }
  io.println("m338_entry_variant_str_shadow: ok");
  return 0;
}
