// Copyright (c) 2026 Eleftherios Notas and The XIOM Authors
// SPDX-License-Identifier: MIT OR Apache-2.0

// m286 lock (REL-20261010-1549-stdlib-7): relational operators on Str must
// be CONTENT ordering (strcmp), not a first-byte compare -- the pointer
// auto-deref path loaded `i8` from each `i8*` and "1" vs "10" compared
// equal, which made Ord[Str].compare return 0 and sort_by_key mis-sort.
// Multi-character keys and prefix cases pin the lexicographic order.
module m286_str_lexicographic_compare.main

fn main() -> Int {
  let a = "1";
  let b = "10";
  let c = "2";
  if !(a < b) { return 41; }
  if !(b < c) { return 42; }
  if a > b { return 43; }
  if b > c { return 44; }
  if a >= b { return 45; }
  if !(a <= a) { return 46; }
  if !("abc" < "abd") { return 47; }
  if !("ab" < "abc") { return 48; }
  if "abd" <= "abc" { return 49; }
  return 0;
}
