// m185 lock (packages relay): `var r: Row;` (no initializer) + assignments
// inside match arms + `return r` crashed on the official v0.62.3 archive
// (0xC000001D): the zero placeholder was coerced to a struct via
// `inttoptr i64 0` + load (a NULL dereference). The zero value must be
// materialized safely.
// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0
module m185_uninit_local_struct

type Row = { name: Str; count: Int; }

fn pick(flag: Int) -> Row {
  var r: Row;
  match flag {
    0 => { r = Row{ name: "zero"; count: 0 }; }
    1 => { r = Row{ name: "one"; count: 1 }; }
    _ => { r = Row{ name: "other"; count: 9 }; }
  }
  return r;
}

fn main() -> Int {
  var a = pick(1);
  if a.count != 1 { return 1; }
  if a.name != "one" { return 2; }
  var b = pick(0);
  if b.count != 0 { return 3; }
  if b.name != "zero" { return 4; }
  var c = pick(7);
  if c.count != 9 { return 5; }
  return 0;
}
