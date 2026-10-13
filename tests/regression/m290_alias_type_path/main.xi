// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m290 lock (REL-20261010-1548-stdlib-6 leg a): with `use xiom.serialize as
// ser;` an ALIAS-qualified struct literal `ser.SerializeError{..}` reported
// T001 "unknown type 'ser.SerializeError'" -- `get_type` resolved the bare
// name and the current-module prefix but never expanded the alias segment
// (`xiom.serialize.SerializeError`). The bare name compiled all along.
module m290_alias_type_path.main

use xiom.serialize as ser;

fn main() -> Int {
  let e = ser.SerializeError{ kind: 0; message: ""; path: ""; line: 0; col: 0; };
  let s = e.format_error();
  if s.len() < 15 { return 1; }
  return 0;
}
