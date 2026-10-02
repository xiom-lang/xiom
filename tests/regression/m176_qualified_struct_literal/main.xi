// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m176 lock (packages relay, gcp): module-qualified struct literals and
// annotations (`qlib.LabelParts{...}`, `var c: qlib.LabelParts`, by-value
// `x: qlib.LabelParts` params) must resolve to the registered struct. The
// codegen i64 unknown-name fallback used to build/read them as scalars, so
// the literal zeroed and the field reads loaded integers.
module m176_qualified_struct_literal

use qlib;

fn keylen(x: qlib.LabelParts) -> Int {
  return x.key.len();
}

fn main() -> Int {
  // Unannotated qualified literal + reads.
  var a = qlib.LabelParts{ key: "k2"; value: "v2"; };
  if a.key != "k2" { return 11; }
  if a.value != "v2" { return 12; }

  // Annotated qualified binding + qualified param.
  var c: qlib.LabelParts = qlib.LabelParts{ key: "kk"; value: "vv"; };
  if c.key != "kk" { return 13; }
  if c.value != "vv" { return 14; }
  if keylen(c) != 2 { return 15; }

  // Sibling-module function control.
  var d = qlib.mk("m", "n");
  if d.key != "m" { return 16; }
  return 0;
}
