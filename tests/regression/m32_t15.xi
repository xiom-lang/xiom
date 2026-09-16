// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T15: String parameter/return -- function with Str param and return
fn greet(name: Str) -> Str {
  return "Hello, " + name;
}
fn main() -> Int {
  var result: Str = greet("World");
  if result == "Hello, World" { return 0; }
  return 1;
}
