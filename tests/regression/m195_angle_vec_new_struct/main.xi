// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m195: `Vec<T>.new()` written with angle brackets must keep T so element
// slots are sized correctly. Pre-fix the parser discarded <T>, .new()
// allocated 16 x 8-byte slots and pushes of 32-byte elements overflowed the
// buffer (heap corruption).
type Big = {
  a: Int;
  b: Int;
  c: Int;
  d: Int;
}

fn main() -> Int {
  var v = Vec<Big>.new();
  var i: Int = 0;
  while i < 10 {
    v.push(Big{a: i, b: i + 1, c: i + 2, d: i + 3});
    i = i + 1;
  }
  if v.len() != 10 { return 1; }
  var j: Int = 0;
  while j < 10 {
    if v[j].a != j { return 2; }
    if v[j].b != j + 1 { return 3; }
    if v[j].c != j + 2 { return 4; }
    if v[j].d != j + 3 { return 5; }
    j = j + 1;
  }
  return 0;
}
