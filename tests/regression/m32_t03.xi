// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M32-T03: String concat -- + operator
fn main() -> Int {
  var s: Str = "hello";
  var t: Str = " world";
  var u: Str = s + t;
  if u == "hello world" { return 0; }
  return 1;
}
