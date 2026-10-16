// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m349 lock (REL-20261016-2004-packages / XARROW-ARRAYNAME): a bare `Array`
// receiver resolving to a REGISTERED user type must keep its own method
// surface -- the legacy builtin fallback silently typed `a.len()` as the
// builtin Int (pre-fix `--check` PASSED) even though this struct has no
// `len`. Fixed arrays spell `Array[<elem>]` and never reach the bare name.
module m349_array_builtin_gate.main;

type Array = { n: Int; m: Int; }

fn main() -> Int {
  let a = Array { n: 1, m: 2 };
  return a.len();
}
