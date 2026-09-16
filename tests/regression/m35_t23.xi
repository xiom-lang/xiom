// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T23: Str exhaustive -- every context: var, param, return, struct, enum, array, cmp, len
fn str_eq(a: Str, b: Str) -> Bool { return a == b; }
fn str_pass(s: Str) -> Str { return s; }
fn check_prefix(s: Str) -> Bool { if s == "hello" { return true; } return false; }
type StrBox = { val: Str; }
enum StrKind { Short(s: Str), Long(s: Str), Empty }
fn classify_str(s: Str) -> StrKind {
  var l: Int = s.len() as Int;
  if l == 0 { return StrKind.Empty; }
  if l < 5 { return StrKind.Short(s); }
  return StrKind.Long(s);
}
fn str_match(s: Str) -> Int { if s == "a" { return 1; } if s == "bb" { return 2; } if s == "ccc" { return 3; } return 0; }
fn main() -> Int {
  var s1: Str = "hello"; var s2: Str = "world";
  if !str_eq(s1, "hello") { return 1; }
  if str_eq(s1, s2) { return 2; }
  if str_pass("ok") != "ok" { return 3; }
  if !check_prefix("hello") { return 4; }
  if check_prefix("world") { return 5; }
  var box: StrBox = StrBox{ val: "test" };
  if box.val != "test" { return 6; }
  var c1: StrKind = classify_str("hi");
  match c1 { StrKind.Short(s) => if s != "hi" { return 7; } _ => return 8; }
  var c2: StrKind = classify_str("longer");
  match c2 { StrKind.Long(s) => if s != "longer" { return 9; } _ => return 10; }
  var c3: StrKind = classify_str("");
  match c3 { StrKind.Empty => { } _ => return 11; }
  if str_match("a") != 1 { return 12; }
  if str_match("bb") != 2 { return 13; }
  if str_match("ccc") != 3 { return 14; }
  if str_match("dddd") != 0 { return 15; }
  var i: Int = 0; var c: Int = 0;
  while i < 3 { if i < 3 { c = c + 1; } i = i + 1; }
  if c != 3 { return 16; }
  var len: Int = s1.len() as Int;
  if len != 5 { return 17; }
  return 0;
}

