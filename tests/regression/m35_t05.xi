// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T05: Str in every context -- var, param, return, struct, enum, array, if, match, len
type StrBox = { val: Str; }
enum StrOption { Some(s: Str), None }
fn str_pass(s: Str) -> Str { return s; }
fn str_if(s: Str) -> Int { if s == "ok" { return 1; } return 0; }
fn str_match(s: Str) -> Int { if s == "hello" { return 10; } if s == "world" { return 20; } return 0; }
fn generic_str[T](x: T) -> T { return x; }
fn main() -> Int {
  var s1: Str = "hello"; var s2: Str = "world";
  if s1 != "hello" { return 1; }
  if s2 != "world" { return 2; }
  if str_pass("ok") != "ok" { return 3; }
  if str_if("ok") != 1 { return 4; }
  if str_if("bad") != 0 { return 5; }
  if str_match("hello") != 10 { return 6; }
  if str_match("world") != 20 { return 7; }
  if str_match("z") != 0 { return 8; }
  var box: StrBox = StrBox{ val: "test" }; if box.val != "test" { return 9; }
  var e: StrOption = StrOption.Some("value"); match e { StrOption.Some(s) => if s != "value" { return 10; } StrOption.None => return 11; }
  var arr: Vec[Str] = ["a", "b", "c"]; if arr[1] != "b" { return 12; }
  var g: Str = generic_str("abc"); if g != "abc" { return 13; }
  var len: Int = s1.len() as Int; if len != 5 { return 14; }
  return 0;
}

