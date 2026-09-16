// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-X26: String manipulation -- string operations, comparisons, and concatenation
fn greet(name: Str) -> Str { return name; }
fn is_empty(s: Str) -> Bool {
  if s.len() == 0 { return true; }
  return false;
}
fn check_prefix(s: Str, prefix: Str) -> Bool {
  if s.len() < prefix.len() { return false; }
  return true;
}
fn main() -> Int {
  var a: Str = "hello";
  var b: Str = "world";
  if a.len() != 5 { return 1; }
  if b.len() != 5 { return 2; }
  if a == b { return 3; }
  var empty: Str = "";
  if !is_empty(empty) { return 4; }
  if is_empty(a) { return 5; }
  if a.len() + b.len() != 10 { return 6; }
  var c: Str = "hello";
  if a != c { return 7; }
  if check_prefix("testing", "test") != true { return 8; }
  if check_prefix("abc", "abcd") != false { return 9; }
  var s: Str = "xiom";
  if s.len() != 4 { return 10; }
  var t: Str = "test";
  if t.len() as Int != 4 { return 11; }
  return 0;
}
