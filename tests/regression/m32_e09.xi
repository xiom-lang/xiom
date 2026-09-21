// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-E09: Enum with mixed payload types (Int, Str, Bool)
enum Value { IntVal(v: Int), StrVal(v: Str), BoolVal(v: Bool) }
fn main() -> Int {
  var v1 = Value.IntVal(42);
  match v1 {
    IntVal(v) => { if v != 42 { return 1; } }
    StrVal(_) => { return 2; }
    BoolVal(_) => { return 2; }
  }
  var v2 = Value.StrVal("hello");
  match v2 {
    IntVal(_) => { return 3; }
    StrVal(v) => { if v != "hello" { return 3; } }
    BoolVal(_) => { return 3; }
  }
  var v3 = Value.BoolVal(true);
  match v3 {
    IntVal(_) => { return 4; }
    StrVal(_) => { return 4; }
    BoolVal(v) => { if v != true { return 4; } }
  }
  return 0;
}
