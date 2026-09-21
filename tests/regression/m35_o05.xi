// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-O05: Option[struct] -- struct wrapped in enum (workaround for Option-struct field access)
type Data = { a: Int; b: Int; }
enum MaybeData { Present(d: Data), Absent }
fn main() -> Int {
  var d1 = Data{ a: 1; b: 2; };
  var a = MaybeData.Present(d1);
  match a { MaybeData.Present(v) => { if v.a != 1 { return 1; } if v.b != 2 { return 2; } } MaybeData.Absent => { return 3; } }
  var b: MaybeData = MaybeData.Absent;
  match b { MaybeData.Present(_) => { return 4; } MaybeData.Absent => {} }
  var c = MaybeData.Present(Data{ a: 3; b: 4; });
  match c { MaybeData.Present(v) => { if v.a != 3 { return 5; } if v.b != 4 { return 6; } } MaybeData.Absent => { return 7; } }
  var d: Option[Int] = Some(42);
  match d { Some(v) => { if v != 42 { return 8; } } None => { return 9; } }
  return 0;
}
