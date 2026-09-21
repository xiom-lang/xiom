// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T13: String in struct -- struct with Str field
type Person = { name: Str; age: Int; }
fn main() -> Int {
  var p = Person{ name: "Alice"; age: 30; };
  if p.name == "Alice" && p.age == 30 { return 0; }
  return 1;
}
