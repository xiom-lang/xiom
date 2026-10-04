// m189 lock (packages relay): a struct TYPE and an enum VARIANT sharing the
// same name (`Field`) made `Field{...}` resolve to the variant inside a
// `module` scope -- the bare-key checks missed the module-qualified struct
// registration, so the literal was built as the enum and emitted invalid IR
// (gep index past the struct end + a bogus struct load). Must compile and
// read the payload correctly.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m189_enum_struct_payload

type Field = { name: Str; value: Int; }

enum Selection {
  Field(sel: Field),
  Other
}

fn pick(s: Selection) -> Str {
  match s {
    Field(sel) => { return sel.name; }
    Other => { return "?"; }
  }
}

fn main() -> Int {
  var f = Field{ name: "x"; value: 1 };
  if f.name != "x" { return 1; }
  var s = Selection.Field(f);
  if pick(s) != "x" { return 2; }
  var o = Selection.Other;
  if pick(o) != "?" { return 3; }
  return 0;
}
