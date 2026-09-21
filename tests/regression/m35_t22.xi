// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// M35-T22: Char exhaustive -- every context: var, param, return, struct, enum, array, cmp
fn char_eq(a: Char, b: Char) -> Bool { return a == b; }
fn char_ne(a: Char, b: Char) -> Bool { return a != b; }
fn char_pass(c: Char) -> Char { return c; }
fn char_to_int(c: Char) -> Int { if c == 'A' { return 1; } if c == 'B' { return 2; } return 0; }
type CharBox = { val: Char; }
enum CharKind { Letter(c: Char), Digit(c: Char), Other }
fn classify(c: Char) -> CharKind {
  if c == 'A' || c == 'Z' { return CharKind.Letter(c); }
  if c == '1' { return CharKind.Digit(c); }
  return CharKind.Other;
}
fn char_match(c: Char) -> Int { if c == 'X' { return 0; } if c == 'Y' { return 1; } if c == 'Z' { return 2; } return -1; }
fn main() -> Int {
  var a: Char = 'A'; var z: Char = 'Z';
  if !char_eq(a, 'A') { return 1; }
  if char_eq(a, z) { return 2; }
  if !char_ne(a, z) { return 3; }
  if char_pass('Q') != 'Q' { return 4; }
  if char_to_int('A') != 1 { return 5; }
  if char_to_int('B') != 2 { return 6; }
  var box: CharBox = CharBox{ val: 'K' };
  if box.val != 'K' { return 7; }
  var ck: CharKind = classify('A');
  match ck { CharKind.Letter(c) => if c != 'A' { return 8; } _ => return 9; }
  var i: Int = 0; var s: Int = 0;
  while i < 3 { if 'a' == 'a' { s = s + 1; } i = i + 1; }
  if s != 3 { return 10; }
  if char_match('X') != 0 { return 11; }
  if char_match('Y') != 1 { return 12; }
  if char_match('Z') != 2 { return 13; }
  if char_match('?') != -1 { return 14; }
  return 0;
}

