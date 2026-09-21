// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-E13: Mixed case identifiers -- camelCase, PascalCase, snake_case
fn myFunc() -> Int { return 10; }
type MyType = { MyField: Int; }
fn main() -> Int {
  var my_var = myFunc();
  var mt = MyType{ MyField: my_var };
  if mt.MyField != 10 { return 1; }
  return 0;
}
