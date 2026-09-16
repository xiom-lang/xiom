// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-S11: Join strings -- concatenate with delimiter between
fn join(a: Str, b: Str, delim: Str) -> Str {
  if a.len() == 0 { return b; }
  if b.len() == 0 { return a; }
  return a + delim + b;
}
fn main() -> Int {
  if join("hello", "world", " ") == "hello world" && join("a", "b", "-") == "a-b" && join("", "c", ",") == "c" { return 0; }
  return 1;
}
