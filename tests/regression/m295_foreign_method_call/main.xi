// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m295 lock (REL-20261010-1548-stdlib-6 leg b): a method-style call of a
// free fn on a BY-VALUE struct/enum receiver (`v.json_get_path(&path)` on a
// `JsonValue` local) emitted the receiver-qualified alias
// `JsonValue.json_get_path` with no definition -> C001 unresolved symbol,
// while the qualified `sjson.json_get_path(v, &path)` worked. Codegen's UFCS
// candidate search only accepted pointer-like receivers, so the registered
// free fn (param 0 typed `%struct.JsonValue` BY VALUE) never matched. The
// runtime payload check below ("42") proves the real callee ran, not a stub.
module m295_foreign_method_call.main

use xiom.serialize.json as sjson;

fn main() -> Int {
  let parsed = sjson.json_parse("{\"a\": {\"b\": 42}}");
  if parsed.is_err { return 2; }
  let v = parsed.unwrap();
  let path = Vec[Str].new();
  path.push("a");
  path.push("b");
  let g = v.json_get_path(&path);
  if g.is_none { return 3; }
  let inner = g.unwrap();
  let s = inner.json_stringify();
  if s != "42" { return 4; }
  let nv = sjson.json_null();
  let empty = Vec[Str].new();
  let ng = nv.json_get_path(&empty);
  if ng.is_none { return 5; }
  let q = sjson.json_get_path(sjson.json_null(), &empty);
  if q.is_none { return 6; }
  return 0;
}
