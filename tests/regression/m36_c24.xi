// Copyright (c) 2026 Eleftherios Notas and XIOM Foundation
// SPDX-License-Identifier: MIT OR Apache-2.0

// M36-C24: Every string pattern -- string literal, concatenation, len, byte_at/slice-like, comparison, return, param, store
use stdlib.xiom.string;
fn string_literal() -> Str { return "hello"; }
fn string_concat(a: Str, b: Str) -> Str { return a + b; }
fn string_len(s: Str) -> Int { return s.len(); }
fn string_first_char(s: Str) -> UInt8 { return string.byte_at(s, 0); }
fn string_cmp(a: Str, b: Str) -> Int {
  if a == b { return 1; }
  if a != b { return 2; }
  return 0;
}
fn main() -> Int {
  var s1: Str = "hello";
  if s1 != "hello" { return 1; }
  var s2: Str = "";
  if s2 != "" { return 2; }
  if string_literal() != "hello" { return 3; }
  if string_concat("ab", "cd") != "abcd" { return 4; }
  if string_concat("", "a") != "a" { return 5; }
  if string_concat("a", "") != "a" { return 6; }
  if string_concat("", "") != "" { return 7; }
  if string_len("test") != 4 { return 8; }
  if string_len("") != 0 { return 9; }
  if string_len("abc") != 3 { return 10; }
  if string_cmp("x", "x") != 1 { return 11; }
  if string_cmp("x", "y") != 2 { return 12; }
  var b = string_first_char("XYZ");
  if b != 88 { return 13; }
  var b2 = string_first_char("abc");
  if b2 != 97 { return 14; }
  var s3: Str = string_concat("foo", "bar");
  if s3 != "foobar" { return 15; }
  var s4: Str = s3;
  if s4 != "foobar" { return 16; }
  return 0;
}
